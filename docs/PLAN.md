# Nexus — arquitectura y hoja de ruta

Nexus es un lenguaje tipado estáticamente que compila a C nativo con recolección de basura (Boehm GC). Este repositorio contiene el compilador autoalojado, el runtime, la stdlib en crecimiento, el servidor LSP y la extensión de VS Code.

La especificación del lenguaje implementado hoy está en `SPEC.md` y `GRAMMAR.md`. Las reglas de trabajo del repositorio están en `RULES.md`. El historial de fases cerradas y decisiones técnicas está en `NOTES.md`.

## Estado actual

| Componente | Ubicación | Estado |
|------------|-----------|--------|
| Compilador etapa 0 (semilla) | `bootstrap/` (Rust) | Operativo; solo necesario para reconstruir desde cero |
| Compilador autoalojado | `compiler/src/` (Nexus) | Operativo; `nxc-stage2` es el compilador de referencia |
| Punto fijo bootstrap | `make verify-bootstrap` | Verde: stage2 y stage3 son binarios idénticos |
| Runtime C | `runtime/` | Operativo: GC, List, Map, IO, primitivas LSP/TCP |
| Stdlib y LSP | `nx/std/`, `nx/lsp/` | JSON, servidor LSP, features IDE |
| Extensión VS Code | `vscode-nexus/` | Operativa con `nexus-lsp` empaquetado |
| Stdlib amplia (Tier A/B) | `nx/std/` | Pendiente — ver `LSP-STDLIB-PLAN.md` |

Fases 0–6 del compilador (fundaciones, lexer, parser, sema, codegen, self-hosting) están cerradas. Detalle en `NOTES.md`.

## Cadena de compilación

```
bootstrap/ (Rust)  →  nxc-stage0
       ↓
compiler/src/      →  nxc-stage1  (stage0 compila Nexus)
       ↓
compiler/src/      →  nxc-stage2  (stage1 compila Nexus)
       ↓
compiler/src/      →  nxc-stage3  (stage2 compila Nexus; debe ser idéntico a stage2)
```

En el día a día se usa `build/nxc-stage2.exe`. Rust solo hace falta al reconstruir desde un clone limpio o tras `make clean` sin binarios en `build/`.

Targets adicionales:

- `make nexus-lsp` — compila `build/nexus-lsp` (lexer/parser/sema de `compiler/` + `nx/lsp/` + JSON)
- `make test-json` — round-trip de la librería JSON
- `make test-lsp` — pruebas del servidor LSP por stdio

## Decisiones de arquitectura

**Etapa 0 en Rust.** Compilador semilla desechable. Se conserva en el repositorio para el bootstrap, no condiciona el producto final.

**Backend C.** El compilador emite C portable; `clang` enlaza con `runtime/nexus_runtime.c` y `libgc`. La generación de código es determinista (misma entrada → mismos bytes), requisito del auto-hospedaje.

**Backend LLVM.** Opcional y posterior. Emitiría `.ll` y pasaría por `llc`. No es requisito del bootstrap.

**Memoria.** GC en runtime desde el arranque. Un borrow checker sería análisis estático posterior, sin cambiar el modelo de ejecución.

**Sin Docker.** Toolchain local: Rust (bootstrap), clang, libgc, make.

**Imports.** Rutas con punto (`import std.json`, `import compiler.ast`, `import lsp.server`) se resuelven a ficheros bajo `nx/std/`, `compiler/src/` y `nx/lsp/`. Ver `SPEC.md`.

## Estructura del repositorio

```
nexus/
  README.md
  docs/              SPEC, GRAMMAR, RULES, NOTES, planes
  bootstrap/         compilador Rust (stage0)
  compiler/src/      compilador Nexus (self-hosted)
  nx/
    std/             biblioteca estándar
    lsp/             servidor LSP y features IDE
  runtime/           runtime C
  tests/             suites lexer, parser, sema, codegen, e2e, json, lsp
  examples/
  vscode-nexus/      extensión VS Code
  build/             artefactos (gitignored)
```

## Requisitos del entorno

- Rust estable (solo bootstrap)
- clang (o gcc) y cabeceras de Boehm GC (`libgc`)
- make
- Node.js (empaquetar `vscode-nexus`)
- Opcional: LLVM (`llc`) para un futuro backend IR

## Hoja de ruta (pendiente)

Trabajo inmediato documentado en `LSP-STDLIB-PLAN.md` (Parte II: stdlib Tier A/B en `nx/std/`).

Ampliaciones del lenguaje previstas después del auto-hospedaje estable:

| Área | Contenido |
|------|-----------|
| Genéricos y traits | Monomorfización, despacho estático |
| Borrow checker | Análisis de propiedad sin cambiar el GC |
| Concurrencia | Goroutines, canales |
| Async/await | Lowering a máquinas de estado |
| Backend LLVM | `codegen_llvm.nx`, paridad e2e con backend C |
| Herramientas | Formateador, empaquetado de módulos |

Cada ampliación debe mantener `make verify-bootstrap` verde y actualizar `SPEC.md` / `GRAMMAR.md` en el mismo commit que cambie el lenguaje.

## Comandos habituales

```
make bootstrap          stage0 (Rust) → build/nxc-stage0
make stage2             cadena hasta build/nxc-stage2
make verify-bootstrap   stage2 == stage3 byte a byte
make test               todas las suites
make nexus-lsp          build/nexus-lsp
make test-lsp           pruebas LSP
make test-json          pruebas JSON
make example NAME=hello compila y ejecuta examples/hello.nx
make clean              borra build/
```

## Riesgos conocidos

**Determinismo del codegen.** Cualquier orden no estable al emitir C, rutas absolutas o marcas de tiempo rompe `verify-bootstrap`. El codegen ordena estructuras y evita estado ambiental en la salida.

**Borrow checker y concurrencia.** Son las extensiones de análisis y runtime más costosas; van después del punto fijo del compilador.

**Bootstrap desde cero.** Requiere Rust, clang, libgc y que `make stage2` complete sin errores antes de poder usar solo Nexus.