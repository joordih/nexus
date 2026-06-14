import json
import os
import subprocess
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

def find_lsp_binary():
    for name in ("nexus-lsp.exe", "nexus-lsp", "nexus-lsp-test.exe", "nexus-lsp-test"):
        path = os.path.join(ROOT, "build", name)
        if os.path.isfile(path):
            return path
    return None

def send_message(proc, obj):
    body = json.dumps(obj, separators=(",", ":"))
    header = f"Content-Length: {len(body)}\r\n\r\n"
    proc.stdin.write(header.encode("utf-8"))
    proc.stdin.write(body.encode("utf-8"))
    proc.stdin.flush()

def read_message(proc):
    header = b""
    while b"\r\n\r\n" not in header:
        ch = proc.stdout.read(1)
        if not ch:
            return None
        header += ch
    lines = header.decode("utf-8").split("\r\n")
    content_length = 0
    for line in lines:
        if line.startswith("Content-Length:"):
            content_length = int(line.split(":", 1)[1].strip())
    body = proc.stdout.read(content_length)
    return json.loads(body.decode("utf-8"))

def path_to_uri(path):
    path = path.replace("\\", "/")
    if len(path) >= 2 and path[1] == ":":
        return "file:///" + path[0] + "%3A" + path[2:]
    return "file://" + path

def request_completion(proc, uri, source, line, character):
    send_message(proc, {
        "jsonrpc": "2.0",
        "method": "textDocument/didOpen",
        "params": {
            "textDocument": {
                "uri": uri,
                "languageId": "nexus",
                "version": 1,
                "text": source,
            }
        },
    })
    read_message(proc)
    send_message(proc, {
        "jsonrpc": "2.0",
        "id": 10,
        "method": "textDocument/completion",
        "params": {
            "textDocument": {"uri": uri},
            "position": {"line": line, "character": character},
        },
    })
    resp = read_message(proc)
    if resp is None or "result" not in resp:
        return None
    return resp["result"]

def main():
    lsp_path = find_lsp_binary()
    if lsp_path is None:
        print("FAIL: lsp binary not found")
        return 1

    proc = subprocess.Popen(
        [lsp_path],
        cwd=ROOT,
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )

    send_message(proc, {
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {"processId": None, "rootUri": None, "capabilities": {}},
    })
    if read_message(proc) is None:
        print("FAIL: initialize")
        proc.kill()
        return 1
    send_message(proc, {"jsonrpc": "2.0", "method": "initialized", "params": {}})

    uri = path_to_uri(os.path.join(ROOT, "examples", "hello.nx"))
    source = "import \n\nmain(): Void {\n}\n"
    result = request_completion(proc, uri, source, 0, 7)
    if result is None:
        print("FAIL: no completion result after import ")
        proc.kill()
        return 1
    labels = {item.get("label") for item in result.get("items", [])}
    for expected in ("std.io", "std.json", "compiler.ast", "lsp.server"):
        if expected not in labels:
            print(f"FAIL: missing import suggestion {expected}, got sample {sorted(labels)[:8]}")
            proc.kill()
            return 1

    source2 = "import std.\n\nmain(): Void {\n}\n"
    result2 = request_completion(proc, uri, source2, 0, 10)
    if result2 is None:
        print("FAIL: no completion result after import std.")
        proc.kill()
        return 1
    labels2 = {item.get("label") for item in result2.get("items", [])}
    for expected2 in ("std.io", "std.json", "std.json.value"):
        if expected2 not in labels2:
            print(f"FAIL: missing std.* suggestion {expected2}, got {sorted(labels2)}")
            proc.kill()
            return 1
    if not result2.get("isIncomplete", False):
        print("FAIL: expected isIncomplete after import std.")
        proc.kill()
        return 1

    source3 = "import std.network.http_client\n\nmain(): Void {\n    var r = http_client.\n}\n"
    member_line = 3
    member_character = len("    var r = http_client.")
    result3 = request_completion(proc, uri, source3, member_line, member_character)
    if result3 is None:
        print("FAIL: no completion result after http_client.")
        proc.kill()
        return 1
    labels3 = {item.get("label") for item in result3.get("items", [])}
    for expected3 in ("buildHttpGetRequest", "parseHttpStatusLine"):
        if expected3 not in labels3:
            print(f"FAIL: missing http_client member {expected3}, got {sorted(labels3)}")
            proc.kill()
            return 1

    send_message(proc, {"jsonrpc": "2.0", "id": 2, "method": "shutdown", "params": {}})
    read_message(proc)
    send_message(proc, {"jsonrpc": "2.0", "method": "exit", "params": {}})
    proc.wait(timeout=5)
    print("PASS import completion: std.io, std.json, compiler.ast, lsp.server")
    print("PASS member completion: http_client.buildHttpGetRequest, http_client.parseHttpStatusLine")
    return 0

if __name__ == "__main__":
    sys.exit(main())