# Plan 001: Fase 1 — Reparar dispatch nullable (`?.`, `?:`, `!`)

> **Executor instructions**: Fase **cerrada**. Verificar con drift check antes de asumir estado.

## Status

- **Priority**: P0 | **Effort**: M | **Risk**: MED | **Depends on**: 000
- **Planned at**: `3229263`, 2026-06-21 | **Estado**: DONE

## Scope (referencia)

- `compiler/src/sema.nx` — tipo real del campo en `?.`/`?:`
- `compiler/src/codegen.nx` — ternario tipado para `?:`, no `||`; safe call real, no `nx_method_void_*`

## Done criteria

- [ ] `make test-e2e` incluye PASS: `elvis_field`, `safecall_field`, `safecall_chain`, `null_safety`
- [ ] `make verify-bootstrap` → exit 0 (compilador no usa operadores sobre campos en `compiler/src/`)
- [ ] Repro `u.email ?: "sin email"` imprime `sin email`, no `true`

## STOP conditions

- Cualquier test e2e de nullable vuelve a `tests/quarantine/` sin plan de regresión explícito.