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

def check_file(lsp, rel_path):
    full = os.path.join(ROOT, rel_path.replace("/", os.sep))
    with open(full, encoding="utf-8") as f:
        source = f.read()
    uri = path_to_uri(full)
    send_message(lsp, {
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
    diag = read_message(lsp)
    if diag is None or diag.get("method") != "textDocument/publishDiagnostics":
        print(f"FAIL {rel_path}: no publishDiagnostics")
        return False
    messages = [d.get("message", "") for d in diag.get("params", {}).get("diagnostics", [])]
    unresolved = [m for m in messages if m.startswith("nombre no resuelto:") or m.startswith("función no resuelta:")]
    if unresolved:
        print(f"FAIL {rel_path}: {len(unresolved)} unresolved names, e.g. {unresolved[0]}")
        return False
    print(f"PASS {rel_path}: no unresolved-name/unresolved-function diagnostics")
    return True

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

    ok = True
    for rel in (
        "compiler/src/main.nx",
        "compiler/src/codegen.nx",
        "nx/std/json/access.nx",
        "nx/lsp/documents.nx",
        "nx/lsp/server.nx",
        "nx/lsp/protocol.nx",
        "nx/lsp/program_build.nx",
        "nx/lsp/features/definition.nx",
        "nx/lsp/features/hover.nx",
        "nx/lsp/features/symbols.nx",
        "nx/lsp/features/completion.nx",
        "nx/lsp/features/diagnostics.nx",
    ):
        if not check_file(proc, rel):
            ok = False

    send_message(proc, {"jsonrpc": "2.0", "id": 2, "method": "shutdown", "params": {}})
    read_message(proc)
    send_message(proc, {"jsonrpc": "2.0", "method": "exit", "params": {}})
    proc.wait(timeout=5)
    return 0 if ok else 1

if __name__ == "__main__":
    sys.exit(main())