# Nexus

Nexus is a statically typed, compiled, general-purpose programming language with automatic memory management and native performance. It targets developers familiar with Java and Kotlin, offering a clean syntax without unnecessary boilerplate.

## Design goals

- Static typing with inference.
- Null safety built into the type system (`T?`, `?.`, `?:`).
- Object-oriented with `class`, `data`, `value`, and `interface`.
- Named constructor arguments.
- Lambda expressions.
- Compiles to native code via a C backend.
- No runtime overhead from a virtual machine.

## Current state

The bootstrap compiler (`nxc`, Stage 0) is written in Rust and compiles Nexus Core programs to C, which is then passed to `clang`. The bootstrap covers:

- Lexer, parser, semantic analysis, and C code generation.
- `data`, `class`, named arguments, `for...in`, `switch/case`, null-safety operators.
- A garbage-collected runtime based on the Boehm GC.

Test suites for lexer, parser, sema, and codegen pass. The next milestone is the self-hosted compiler (Stage 1), written in Nexus itself.

## Example

```nexus
import std.io

data Punto {
    x: Int
    y: Int
}

class PuntoService {
    distanciaOrigenCuadrado(p: Punto): Int {
        return p.x * p.x + p.y * p.y
    }
}

areaAproximada(radio: Int): Int {
    return 3 * radio * radio
}

main(): Void {
    var p = Punto(x: 3, y: 4)
    var svc = PuntoService()
    io.println(svc.distanciaOrigenCuadrado(p))
    io.println(areaAproximada(5))
}
```

## Building

Requirements: Rust stable, clang, Boehm GC (`libgc`), make.

Optional environment variables for linking tests and compiled programs:

- `CC`: C compiler (default `clang`)
- `GC_INCLUDE`: Boehm GC include directory
- `GC_LIB`: Boehm GC library directory
- `GC_STATIC`: set to `1` for static GC linkage (default on Windows)

```
make bootstrap      # build Stage 0 compiler -> build/nxc-stage0
make test           # run all test suites
make test-lexer     # lexer suite only
make test-parser    # parser suite only
make test-sema      # semantic analysis suite only
make test-codegen   # code generation suite only
make test-e2e       # end-to-end suite (requires clang + libgc)
```

## Repository structure

```
bootstrap/      Stage 0 compiler in Rust (nxc-stage0)
compiler/       Self-hosted compiler in Nexus (nxc-stage1+)
nx/             Nexus source outside the compiler (stdlib, LSP)
runtime/        C runtime (GC, List, Map, io)
tests/          Test suites with expected snapshots
examples/       Example programs
docs/           Spec, grammar, architecture, rules, notes
vscode-nexus/   VS Code extension
```

## Language specification

The full language specification is in `docs/SPEC.md`. The formal grammar is in `docs/GRAMMAR.md`. Both documents describe only what the compiler currently implements; they grow with each phase.

Other documentation: `docs/PLAN.md` (architecture and roadmap), `docs/RULES.md`, `docs/NOTES.md`, `docs/LSP-STDLIB-PLAN.md`.
