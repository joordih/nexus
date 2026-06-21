# Plan 003: Fase 3 — Aritmética y `toString` Float/Double

> **Executor instructions**: Fase **cerrada**. Verificar cierre.

## Status

- **Priority**: P0 | **Effort**: M | **Risk**: MED | **Depends on**: 002
- **Planned at**: `3229263`, 2026-06-21 | **Estado**: DONE

## Scope de referencia

- `compiler/src/codegen.nx` — ops binarias promueven a `double`
- `compiler/src/sema.nx` — tipos Float/Double en inferencia
- `runtime/nexus_runtime.c` — `nx_float_to_string`, `nx_double_to_string`, `nx_println_double`
- Tests: `tests/e2e/float_arith.nx`, `tests/codegen/float_arith.nx`, `runtime/test_runtime.c`

## Done criteria

- [ ] `make test-e2e` → `float_arith.nx` imprime `1.5` para `3.0 / 2.0`
- [ ] `make test-codegen` → snapshot float
- [ ] `make test-runtime` → exit 0
- [ ] `make verify-bootstrap` → exit 0
- [ ] Actualizar tabla Float/Double en `docs/NOTES.md` si aún dice "No"

## STOP conditions

- Codegen vuelve a emitir `NxInt` para literales/op flotantes.