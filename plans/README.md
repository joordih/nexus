# Planes de implementación — Plan maestro Nexus

Generado por el skill **improve** el 2026-06-21. Fuente: [`plan-maestro.md`](../plan-maestro.md) (commit base `3229263`).

Cada archivo `NNN-*.md` es un plan **autocontenido** para un ejecutor sin contexto previo. Ejecutar en orden salvo que las dependencias indiquen paralelismo.

## Orden de ejecución y estado

| Plan | Título | Prioridad | Esfuerzo | Depende de | Estado |
|------|--------|-----------|----------|------------|--------|
| 000 | Fase 0 — Cimientos CI y baseline | P0 | S | — | DONE |
| 001 | Fase 1 — Nullable `?.` `?:` `!` | P0 | M | 000 | DONE |
| 002 | Fase 2 — `val` e inmutabilidad | P0 | M | 001 | DONE |
| 003 | Fase 3 — Float/Double aritmética | P0 | M | 002 | DONE |
| 004 | Fase 4 — Strings raw y multilínea | P1 | S | 003 | DONE |
| 005 | Fase 5 — Imports alias/grupo/wildcard | P1 | M | 003 | DONE |
| 006 | Fase 6 — Dispatch y extensiones | P1 | L | 003 | DONE |
| 007 | Fase 7 — Interpolación de strings | P2 | L | 004, 006 | TODO |
| 008 | Fase 8 — Smart casts / flow typing | P2 | M | 001 | DONE |
| 009 | Fase 9 — `if`/`switch`/`try` expresión | P2 | L | 003 | TODO |
| 010 | Fase 10 — `?[]` y subscript assign | P2 | M | 001, 009 | TODO |
| 011 | Fase 11 — JSON v1 capstone | P0 | L | 001, 006, 008, 010 | TODO |
| 012 | Fase 12 — Defaults ctor + interface | P2 | L | 006 | TODO |
| 013 | Fase 13 — Genéricos `<T>` erasure | P3 | L | 012 | TODO |
| 014 | Fase 14 — `extends` y companion | P3 | L | 012, 013 | TODO |
| 015 | Fase 15 — `annotation` y `module` | P3 | M | 005 | TODO |
| 016 | Fase 16 — `match` pattern matching | P3 | L | 009 | TODO |
| 017 | Fase 17 — JSON v2 con `match` | P0 | L | 011, 016 | TODO |

Estados: `TODO` | `IN PROGRESS` | `DONE` | `BLOCKED` | `REJECTED`

## Grafo de dependencias (resumen)

```
000 → 001 → 002 → 003
                  ├→ 004 ─┐
                  ├→ 005 ─┼→ 015
                  ├→ 006 ─┼→ 007, 011, 012
                  ├→ 008 ─┘       │
                  └→ 009 → 010 ───┤
                            016 ──┴→ 017
                  012 → 013 → 014
```

**Paralelizable tras 003:** 004, 005, 006, 008, 009 (ramas independientes).

**Capstone del roadmap:** 011 (JSON v1) y 017 (JSON v2 + `match`).

## Comandos de verificación (repo Nexus)

| Propósito | Comando | Éxito |
|-----------|---------|-------|
| Lint sin comentarios | `make lint` | exit 0 |
| Bootstrap | `make verify-bootstrap` | exit 0, mensaje "Bootstrap verificado." |
| Suite completa | `make test` | exit 0 (requiere `CC` con clang en PATH) |
| E2E | `make test-e2e` | exit 0 |
| JSON | `make test-json` | exit 0 |
| LSP | `make test-lsp` | exit 0 |

En Windows, si `clang` no está en PATH:

```cmd
set CC=C:\Program Files\LLVM\bin\clang.exe
```

**Regla D11 (no negociable):** ninguna feature del plan maestro se porta a `bootstrap/` (stage0 Rust). Solo `compiler/src/`, `runtime/`, `nx/std/`, tests y docs.

## Residual conocido (no bloquea fases 4+)

- `make test-json` falla por newline final en `tests/json/roundtrip.expected` (harness); el binario ejecuta bien. Arreglar en cualquier momento antes de cerrar 011/017.

## Hallazgos descartados

- Portar features a stage0 Rust: explícitamente prohibido (D11).
- Monomorfización de genéricos: descartada (D8 erasure).

## Histórico

Planes anteriores (TLS, fetch, etc.) movidos a `plans/.old/`. Este índice cubre solo el plan maestro del compilador (fases 0–17).