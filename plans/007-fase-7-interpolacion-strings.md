# Plan 007: Fase 7 — Interpolación de strings (`$ident`, `${expr}`)

> **Executor instructions**: D1 — sin interpolación en `r"..."`; con interpolación en `"""..."""`. Lexer stateful: probar tokens antes del parser.

## Status

- **Priority**: P2 | **Effort**: L | **Risk**: HIGH | **Depends on**: 004, 006
- **Planned at**: `3229263`, 2026-06-21

## Why this matters

Elimina concatenación manual (`"GET " + url`). Depende de tabla `toString` (fase 6) y strings avanzados (fase 4).

## Scope

**In scope:** `lexer.nx`, `parser.nx`, `ast.nx` (`EXPR_INTERPOLATION = 49`), `sema.nx`, `codegen.nx`, tests lexer/e2e/codegen

**Out of scope:** `bootstrap/`

## Steps

### Step 1: Tokens lexer

`TK_STRING_START`, `TK_STRING_PART`, `TK_STRING_END`, `TK_INTERP_START`. En `scanString`, segmentar `$ident` y `${...}`.

**Verify:** `tests/lexer/interpolation.tokens` PASS

### Step 2: Parser + AST

`parsePrimary` construye `EXPR_INTERPOLATION` con lista de partes literal/expr.

### Step 3: Sema + codegen

Tipo `String`. Slots no-String → `toString` vía tabla fase 6. Codegen: cadena de `nx_string_concat` en orden de aparición; `null` → `"null"`.

**Verify:** `tests/e2e/interpolation.nx` → `"x=${2+2}"` imprime `x=4`

### Step 4: Bootstrap

`make verify-bootstrap` → exit 0.

## Done criteria

- [ ] `${expr}` y `$ident` ejecutan
- [ ] Snapshots lexer y codegen deterministas
- [ ] `make test` relevante verde

## STOP conditions

- Interpolación anidada rompe lexer sin tests de conteo de llaves
- `verify-bootstrap` difiere