# Nexus

[![CodSpeed](https://img.shields.io/endpoint?url=https://codspeed.io/badge.json)](https://app.codspeed.io/joordih/nexus?utm_source=badge)

Nexus is a statically typed, compiled, general-purpose programming language with null safety built into the type system and automatic memory management. It compiles to native code through a C backend — there is no virtual machine. The syntax targets developers familiar with Java and Kotlin, without the boilerplate.

The compiler is **self-hosted**: it is written in Nexus and compiles itself, with the bootstrap chain verified down to byte-identical binaries.

The language is under active development. Issues and pull requests are welcome:

* [Issues](https://github.com/joordih/nexus/issues)
* [Pull requests](https://github.com/joordih/nexus/pulls)

## Documentation quick links

* [Overview](#overview)
* [Example](#example)
* [Current status](#current-status)
* [Standard library](#standard-library)
* [Tooling](#tooling)
* [Building from source](#building-from-source)
* [Using the compiler](#using-the-compiler)
* [Repository structure](#repository-structure)
* [Documentation](#documentation)

## Overview

### Null safety

Nullability is part of the type system. A non-nullable type can never hold `null`; the compiler rejects it at compile time. Nullable types are declared with `T?`, and the language provides safe access (`?.`), the Elvis operator (`?:`), and the non-null assertion (`!`).

### Native compilation, no VM

The compiler emits C, which is compiled to a native binary with `clang` (configurable via `CC`). The runtime is a small C library providing `List`, `Map`, string operations, and I/O, with garbage collection based on the [Boehm GC](https://www.hboehm.info/gc/).

### Verified self-hosting

The bootstrap chain has four stages:

| Stage | Compiler | Written in | Built by |
|-------|----------|------------|----------|
| 0 | `nxc-stage0` | Rust (`bootstrap/`) | `cargo` |
| 1 | `nxc-stage1` | Nexus (`compiler/src/`) | stage 0 |
| 2 | `nxc-stage2` | Nexus (`compiler/src/`) | stage 1 |
| 3 | `nxc-stage3` | Nexus (`compiler/src/`) | stage 2 |

`make verify-bootstrap` checks that the stage 2 and stage 3 binaries are byte-identical: the compiler written in Nexus compiles its own source code with deterministic output.

### Tooling

The repository includes a language server written in Nexus itself (`nx/lsp/`) and a VS Code extension that uses it. See [Tooling](#tooling-1).

## Example

This program compiles and runs with the current self-hosted compiler:

<table>
<tr>
<th>Nexus</th>
<th>Output</th>
</tr>
<tr>
<td>

```typescript
import std.io

data User {
    name: String
    email: String?
}

class UserService {
    describe(user: User): String {
        if (user.email != null) {
            return user.name + " <" + user.email! + ">"
        }
        return user.name + " (no email on file)"
    }
}

main(): Void {
    var users: List<User> = List()
    users.add(User(name: "Ada", email: "ada@example.com"))
    users.add(User(name: "Grace", email: null))

    var service = UserService()
    for user in users {
        io.println(service.describe(user))
    }
}
```

</td>
<td>

```
Ada <ada@example.com>
Grace (no email on file)
```

</td>
</tr>
</table>

## Current status

Self-hosting is complete (phases 0–6 of the bootstrap plan are closed). The subset of the language the compiler is written in is called **Nexus Core**.

Implemented end to end (semantic analysis and code generation):

* `import`, free functions, `data`, `value`, `class`, local and global variables
* `List<T>`, `Map<K,V>`, null safety (`T?`, `?.`, `?:`, `!`)
* `if`/`else`, `while`, `for...in`, `switch`, `try`/`catch`/`throw`, lambdas, named constructor arguments

Parsed but not yet processed by semantic analysis or codegen (reserved syntax for later phases):

* `module`, `interface`, `annotation`, `extends`, `implements`, type parameters `<T>` on user declarations

Primitive types: `Int` (64-bit signed), `Long`, `Float` (32-bit), `Double` (64-bit), `Bool`, `Char`, `String`, `Void`.

Planned extensions (generics, borrow checker, concurrency, async) are tracked in `docs/PLAN.md`.

## Standard library

The standard library lives in `nx/std/` and is written in Nexus. Current modules:

| Package | Modules |
|---------|---------|
| `std.core` | `math`, `random`, `boolean`, `integer`, `strings`, `string_builder`, `uuid` |
| `std.collections` | `array_list`, `hash_set`, `linked_hash_map`, `linked_hash_set`, `tree_map`, `tree_set`, `queue`, `stack`, `optional` |
| `std.json` | `parser`, `writer`, `value`, `access` (`json.parse`, `json.stringify`) |
| `std.fs` | `file`, `files`, `path`, `directory_stream`, `file_input_stream`, `file_output_stream` |
| `std.network` | `socket`, `server_socket`, `http_client` |
| `std.datetime` | `local_date`, `local_time`, `local_date_time`, `duration`, `period` |
| `std.regex` | `pattern`, `matcher` |
| `std.system` | `environment`, `runtime_info` |
| `std.concurrency` | `thread` |
| `std.reflection` | `type_info` |

A program exercising the standard library is in `examples/stdlib_showcase.nx` (`make example-stdlib`).

## Tooling

### Language server

`nx/lsp/` contains an LSP server written in Nexus and compiled with the self-hosted compiler (`make nexus-lsp`). Implemented features: diagnostics, completion, hover, go-to-definition, and document symbols.

### VS Code extension

`vscode-nexus/` provides syntax highlighting for `.nx` files and a client for the language server, with diagnostics on save and configurable compiler/server paths (`make vscode-nexus` builds the `.vsix` package).

## Building from source

Requirements:

* Rust (stable) — for the stage 0 compiler
* `clang` — or any C compiler, via the `CC` variable
* [Boehm GC](https://www.hboehm.info/gc/) (`libgc`)
* `make`
* Python — only for `make verify-bootstrap` and the LSP test suite
* Node.js / `npm` — only for packaging the VS Code extension

Environment variables for linking:

| Variable | Meaning |
|----------|---------|
| `CC` | C compiler (default `clang`) |
| `GC_INCLUDE` | Boehm GC include directory |
| `GC_LIB` | Boehm GC library directory |
| `GC_STATIC` | `1` for static GC linkage (default on Windows) |

Main targets:

```
make bootstrap         # build the stage 0 compiler (Rust) -> build/nxc-stage0
make stage1            # stage 0 compiles the Nexus compiler -> build/nxc-stage1
make stage2            # stage 1 compiles itself            -> build/nxc-stage2
make stage3            # stage 2 compiles itself            -> build/nxc-stage3
make verify-bootstrap  # assert stage2 == stage3 byte for byte
make test              # all suites: runtime, lexer, parser, sema, codegen,
                       #             e2e, json, stdlib, lsp
make nexus-lsp         # build the language server -> build/nexus-lsp
make vscode-nexus      # package the VS Code extension
make build-all         # verify-bootstrap + vscode-nexus + test
```

## Using the compiler

Compile a single file to a native binary:

```
build/nxc-stage2 compile examples/hello.nx build/hello
build/hello
```

Or via the Makefile shortcut:

```
make example NAME=hello
```

## Repository structure

```
bootstrap/      Stage 0 compiler in Rust (nxc-stage0)
compiler/       Self-hosted compiler in Nexus (nxc-stage1+)
nx/             Nexus source outside the compiler: std/ (stdlib), lsp/ (language server)
runtime/        C runtime (GC, List, Map, io)
tests/          Test suites with expected snapshots
examples/       Example programs
docs/           Spec, grammar, roadmap, notes
vscode-nexus/   VS Code extension
```

## Documentation

All language documentation is currently written in Spanish.

* `docs/SPEC.md` — language specification of Nexus Core; documents only what the compiler implements today
* `docs/GRAMMAR.md` — the authoritative formal grammar
* `docs/PLAN.md` — architecture and roadmap
* `docs/NOTES.md` — development log of the bootstrap phases
