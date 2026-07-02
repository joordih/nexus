import hashlib
import os
import shutil
import subprocess
import sys
import tempfile


def resolve_bin(path):
    if os.path.exists(path + ".exe"):
        return path + ".exe"
    return path


def read_bytes(path):
    with open(resolve_bin(path), "rb") as f:
        return f.read()


def first_diff(a, b):
    limit = min(len(a), len(b))
    for i in range(limit):
        if a[i] != b[i]:
            return i
    if len(a) != len(b):
        return limit
    return -1


def strip_elf_notes(path):
    objcopy = shutil.which("objcopy")
    if objcopy is None:
        return read_bytes(path)
    src = resolve_bin(path)
    fd, tmp = tempfile.mkstemp(suffix=".elf")
    os.close(fd)
    try:
        shutil.copy2(src, tmp)
        subprocess.run(
            [
                objcopy,
                "--remove-section=.note.gnu.build-id",
                "--remove-section=.comment",
                tmp,
            ],
            check=False,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
        )
        return read_bytes(tmp)
    finally:
        os.remove(tmp)


def main():
    c2 = read_bytes("build/nxc-stage2.c")
    c3 = read_bytes("build/nxc-stage3.c")
    if c2 != c3:
        idx = first_diff(c2, c3)
        print("verify-bootstrap: nxc-stage2.c y nxc-stage3.c difieren", file=sys.stderr)
        if idx >= 0:
            start = max(0, idx - 60)
            end = min(len(c2), idx + 60)
            print(f"  primer byte distinto en offset {idx}", file=sys.stderr)
            print(f"  stage2: {c2[start:end]!r}", file=sys.stderr)
            print(f"  stage3: {c3[start:end]!r}", file=sys.stderr)
        print(
            f"  sha256 stage2.c {hashlib.sha256(c2).hexdigest()}",
            file=sys.stderr,
        )
        print(
            f"  sha256 stage3.c {hashlib.sha256(c3).hexdigest()}",
            file=sys.stderr,
        )
        return 1

    b2 = read_bytes("build/nxc-stage2")
    b3 = read_bytes("build/nxc-stage3")
    if b2 != b3:
        b2 = strip_elf_notes("build/nxc-stage2")
        b3 = strip_elf_notes("build/nxc-stage3")

    if b2 != b3:
        idx = first_diff(b2, b3)
        print(
            "verify-bootstrap: C identico pero los binarios difieren",
            file=sys.stderr,
        )
        if idx >= 0:
            print(f"  primer byte distinto en offset {idx}", file=sys.stderr)
        print(f"  tam stage2 {len(b2)} tam stage3 {len(b3)}", file=sys.stderr)
        return 1

    print("Bootstrap verificado.")
    return 0


if __name__ == "__main__":
    sys.exit(main())