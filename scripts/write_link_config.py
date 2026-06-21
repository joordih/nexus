import os
import subprocess
import sys


def path_for_c(path):
    return path.replace("\\", "/")


def file_exists(path):
    return os.path.isfile(path)


def dir_exists(path):
    return os.path.isdir(path)


def find_vcpkg_root():
    for candidate in [
        os.environ.get("VCPKG_ROOT", ""),
        os.path.expanduser("~/Documents/vcpkg"),
        os.path.expanduser("~/vcpkg"),
    ]:
        if candidate and dir_exists(candidate):
            return os.path.normpath(candidate)
    return None


def find_vcpkg_triplet(root, triplet):
    return os.path.join(root, "installed", triplet)


def find_openssl_vcpkg(root):
    for triplet in ("x64-windows-static", "x64-windows", "x86-windows-static"):
        base = find_vcpkg_triplet(root, triplet)
        inc_dir = os.path.join(base, "include")
        inc = os.path.join(inc_dir, "openssl", "ssl.h")
        lib_dir = os.path.join(base, "lib")
        if not file_exists(inc):
            continue
        for lib_name in ("libssl.lib", "ssl.lib", "libssl.a", "libssl.so"):
            if file_exists(os.path.join(lib_dir, lib_name)):
                return inc_dir, lib_dir
    return None, None


def find_gc_vcpkg(root):
    include_triplets = ("x64-windows", "x64-windows-static")
    lib_triplets = ("x64-windows-static", "x64-windows")
    gc_inc = None
    for triplet in include_triplets:
        inc = os.path.join(find_vcpkg_triplet(root, triplet), "include", "gc.h")
        if file_exists(inc):
            gc_inc = os.path.dirname(inc)
            break
    gc_lib = None
    for triplet in lib_triplets:
        lib_dir = os.path.join(find_vcpkg_triplet(root, triplet), "lib")
        if file_exists(os.path.join(lib_dir, "gc.lib")):
            gc_lib = lib_dir
            break
    if gc_inc and gc_lib:
        return gc_inc, gc_lib
    return None, None


def find_openssl_pkg_config():
    try:
        subprocess.run(
            ["pkg-config", "--exists", "openssl"],
            check=True,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
        )
    except (subprocess.CalledProcessError, FileNotFoundError):
        return None, None
    try:
        inc_out = subprocess.check_output(
            ["pkg-config", "--variable=includedir", "openssl"],
            text=True,
        ).strip()
        lib_out = subprocess.check_output(
            ["pkg-config", "--variable=libdir", "openssl"],
            text=True,
        ).strip()
        if dir_exists(inc_out) and dir_exists(lib_out):
            return inc_out, lib_out
    except (subprocess.CalledProcessError, FileNotFoundError):
        pass
    return None, None


def find_openssl_brew():
    try:
        prefix = subprocess.check_output(
            ["brew", "--prefix", "openssl"],
            text=True,
        ).strip()
    except (subprocess.CalledProcessError, FileNotFoundError):
        return None, None
    inc = os.path.join(prefix, "include")
    lib = os.path.join(prefix, "lib")
    if file_exists(os.path.join(inc, "openssl", "ssl.h")) and dir_exists(lib):
        return inc, lib
    return None, None


def find_openssl_linux():
    for inc, lib in [
        ("/usr/include", "/usr/lib/x86_64-linux-gnu"),
        ("/usr/include", "/usr/lib64"),
        ("/usr/include", "/usr/lib"),
    ]:
        if file_exists(os.path.join(inc, "openssl", "ssl.h")) and dir_exists(lib):
            return inc, lib
    return None, None


def find_gc_linux():
    for inc, lib in [
        ("/usr/include", "/usr/lib/x86_64-linux-gnu"),
        ("/usr/include", "/usr/lib64"),
        ("/usr/include", "/usr/lib"),
    ]:
        if file_exists(os.path.join(inc, "gc.h")) and dir_exists(lib):
            return inc, lib
    return None, None


def detect_paths():
    gc_include = os.environ.get("GC_INCLUDE", "")
    gc_lib = os.environ.get("GC_LIB", "")
    ssl_include = os.environ.get("SSL_INCLUDE", "")
    ssl_lib = os.environ.get("SSL_LIB", "")

    if sys.platform == "win32":
        vcpkg = find_vcpkg_root()
        if vcpkg:
            if not ssl_include or not ssl_lib:
                ssl_inc, ssl_lib_dir = find_openssl_vcpkg(vcpkg)
                if ssl_inc and ssl_lib_dir:
                    ssl_include = ssl_include or ssl_inc
                    ssl_lib = ssl_lib or ssl_lib_dir
            if not gc_include or not gc_lib:
                gc_inc, gc_lib_dir = find_gc_vcpkg(vcpkg)
                if gc_inc and gc_lib_dir:
                    gc_include = gc_include or gc_inc
                    gc_lib = gc_lib or gc_lib_dir
    else:
        if not ssl_include or not ssl_lib:
            ssl_inc, ssl_lib_dir = find_openssl_pkg_config()
            if not ssl_inc:
                ssl_inc, ssl_lib_dir = find_openssl_brew()
            if not ssl_inc:
                ssl_inc, ssl_lib_dir = find_openssl_linux()
            if ssl_inc and ssl_lib_dir:
                ssl_include = ssl_include or ssl_inc
                ssl_lib = ssl_lib or ssl_lib_dir
        if not gc_include or not gc_lib:
            gc_inc, gc_lib_dir = find_gc_linux()
            if gc_inc and gc_lib_dir:
                gc_include = gc_include or gc_inc
                gc_lib = gc_lib or gc_lib_dir

    return gc_include, gc_lib, ssl_include, ssl_lib


def write_header(path, gc_include, gc_lib, ssl_include, ssl_lib):
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "w", encoding="utf-8", newline="\n") as f:
        f.write("#ifndef NEXUS_LINK_CONFIG_H\n")
        f.write("#define NEXUS_LINK_CONFIG_H\n")
        f.write('#define NX_DEFAULT_GC_INCLUDE "' + path_for_c(gc_include) + '"\n')
        f.write('#define NX_DEFAULT_GC_LIB "' + path_for_c(gc_lib) + '"\n')
        f.write('#define NX_DEFAULT_SSL_INCLUDE "' + path_for_c(ssl_include) + '"\n')
        f.write('#define NX_DEFAULT_SSL_LIB "' + path_for_c(ssl_lib) + '"\n')
        f.write("#endif\n")


def write_makefile_fragment(path, gc_include, gc_lib, ssl_include, ssl_lib):
    os.makedirs(os.path.dirname(path), exist_ok=True)

    def mk_val(value):
        if not value:
            return ""
        escaped = value.replace("\\", "/").replace(" ", "\\ ")
        return escaped

    with open(path, "w", encoding="utf-8", newline="\n") as f:
        if gc_include:
            f.write("GC_INCLUDE ?= " + mk_val(gc_include) + "\n")
        if gc_lib:
            f.write("GC_LIB ?= " + mk_val(gc_lib) + "\n")
        if ssl_include:
            f.write("SSL_INCLUDE ?= " + mk_val(ssl_include) + "\n")
        if ssl_lib:
            f.write("SSL_LIB ?= " + mk_val(ssl_lib) + "\n")


def main():
    repo_root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    gc_include, gc_lib, ssl_include, ssl_lib = detect_paths()
    write_header(
        os.path.join(repo_root, "runtime", "nexus_link_config.h"),
        gc_include,
        gc_lib,
        ssl_include,
        ssl_lib,
    )
    write_makefile_fragment(
        os.path.join(repo_root, "build", "link_paths.mk"),
        gc_include,
        gc_lib,
        ssl_include,
        ssl_lib,
    )
    if ssl_include and ssl_lib:
        print("OpenSSL: " + ssl_include)
    else:
        print("OpenSSL: no detectado (define SSL_INCLUDE y SSL_LIB)", file=sys.stderr)
    return 0


if __name__ == "__main__":
    sys.exit(main())