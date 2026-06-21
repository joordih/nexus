# Plan 002: Fase 2 — Inmutabilidad `val` y enforcement `final`/`val`

> **Executor instructions**: Fase **cerrada**. Verificar cierre con comandos abajo.

## Status

- **Priority**: P0 | **Effort**: M | **Risk**: MED | **Depends on**: 001
- **Planned at**: `3229263`, 2026-06-21 | **Estado**: DONE

## Cambios de referencia

- `compiler/src/lexer.nx`: `TK_VAL = 74`
- `compiler/src/parser.nx`: `TK_VAL` → `is_final`
- `compiler/src/sema.nx`: `Scope` + `final_flags`, error en reasignación
- **Colisión `val`:** no usar `var val =` ni parámetro nombrado `val:` en `.nx` (conflicto con keyword). Renombrar a `parsed`, `expr`, `value`, etc.

## Done criteria

- [ ] `make test-lexer` → `tests/lexer/val_token.nx`
- [ ] `make test-parser` → `tests/parser/val_local.ast`
- [ ] `make test-sema` → `invalid_val_reassign.nx` y `invalid_final_reassign.nx` rechazados
- [ ] `make test-e2e` → `val_basic.nx`
- [ ] `make verify-bootstrap` → exit 0

## STOP conditions

- Reintroducir identificador `val` como nombre de variable/parámetro en `compiler/src/` o `nx/std/`.