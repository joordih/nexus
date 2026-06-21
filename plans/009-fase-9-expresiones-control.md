# Plan 009: Fase 9 — `if` / `switch` / `try` como expresiones

> **Executor instructions**: Repara hack `EXPR_SWITCH` (Stmt empaquetado en `expr.args`). Mantener switch-sentencia hasta que switch-expr pase tests.

## Status

- **Priority**: P2 | **Effort**: L | **Risk**: HIGH | **Depends on**: 003
- **Planned at**: `3229263`, 2026-06-21

## Why this matters

`val x = if (c) a else b` y switch-valor requieren asignación por rama al temporal común. Hoy `EXPR_SWITCH` emite `void*` sin asignar (~codegen línea 1015).

## Scope

**In scope:** `parser.nx`, `ast.nx` (`EXPR_IF=50`, `EXPR_TRY=51`), `sema.nx`, `codegen.nx`, tests e2e/sema/codegen

**Out of scope:** `match` (fase 16), `bootstrap/`

## Steps

### Step 1: AST refactor switch

`EXPR_SWITCH` con `switch_subject` y `switch_arms` dedicados (eliminar hack).

### Step 2: Parser expresiones

En `parsePrimary`: `if`/`try` como expresión. `if` valor exige `else`. D5 para tipos de `try` expr.

### Step 3: Sema

Tipo común de ramas; switch-valor exige `default`; errores si no exhaustivo.

### Step 4: Codegen

Temporal tipado; cada rama asigna. Switch: asignación por brazo (no `void*` huérfano).

**Verify:** `if_expr.nx`, `switch_expr.nx`, `try_expr.nx` en `make test-e2e`

### Step 5: Eliminar hack antiguo + bootstrap

Borrar parse legacy solo tras tests verdes. `make verify-bootstrap` → exit 0.

## Done criteria

- [ ] Tres construcciones como expresión con valor correcto
- [ ] `if_expr_no_else.nx` y switch no exhaustivo → error sema
- [ ] Snapshot codegen muestra asignación por brazo
- [ ] `make verify-bootstrap` idéntico

## STOP conditions

- Romper `switch` sentencia usado en `compiler/src/` antes de tener equivalente
- `verify-bootstrap` difiere