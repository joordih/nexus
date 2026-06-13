import json
import os
import subprocess
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

def find_lsp_binary():
    for name in ("nexus-lsp.exe", "nexus-lsp"):
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

def main():
    lsp = find_lsp_binary()
    if lsp is None:
        print("FAIL lsp: binary not found")
        return 1

    env = os.environ.copy()
    proc = subprocess.Popen(
        [lsp],
        cwd=ROOT,
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        env=env,
    )

    send_message(proc, {
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "processId": None,
            "rootUri": None,
            "capabilities": {},
        },
    })
    init_resp = read_message(proc)
    if init_resp is None or "result" not in init_resp:
        print("FAIL lsp: initialize response")
        proc.kill()
        return 1
    caps = init_resp["result"].get("capabilities", {})
    if "completionProvider" not in caps:
        print("FAIL lsp: missing completionProvider capability")
        proc.kill()
        return 1

    send_message(proc, {"jsonrpc": "2.0", "method": "initialized", "params": {}})

    sample = "import std.io\n\nmain(): Void {\n    var x: Int = 1\n    io.\n}\n"
    send_message(proc, {
        "jsonrpc": "2.0",
        "method": "textDocument/didOpen",
        "params": {
            "textDocument": {
                "uri": "file:///test.nx",
                "languageId": "nexus",
                "version": 1,
                "text": sample,
            }
        },
    })

    diag = read_message(proc)
    if diag is None or diag.get("method") != "textDocument/publishDiagnostics":
        print("FAIL lsp: expected publishDiagnostics notification")
        proc.kill()
        return 1

    send_message(proc, {
        "jsonrpc": "2.0",
        "id": 2,
        "method": "textDocument/completion",
        "params": {
            "textDocument": {"uri": "file:///test.nx"},
            "position": {"line": 4, "character": 7},
        },
    })
    comp_resp = read_message(proc)
    if comp_resp is None or "result" not in comp_resp:
        print("FAIL lsp: completion response")
        proc.kill()
        return 1
    items = comp_resp["result"].get("items", [])
    labels = {item.get("label") for item in items}
    if "println" not in labels:
        print("FAIL lsp: completion missing println, got", labels)
        proc.kill()
        return 1

    send_message(proc, {"jsonrpc": "2.0", "id": 3, "method": "shutdown", "params": {}})
    shutdown_resp = read_message(proc)
    if shutdown_resp is None or "result" not in shutdown_resp:
        print("FAIL lsp: shutdown response")
        proc.kill()
        return 1

    send_message(proc, {"jsonrpc": "2.0", "method": "exit", "params": {}})
    proc.wait(timeout=5)
    print("PASS lsp: initialize didOpen completion shutdown")
    return 0

if __name__ == "__main__":
    sys.exit(main())