# Plan 016: Fase 16 — `match` / pattern matching

> **Executor instructions**: D10 — exhaustividad obligatoria como expresión. Keyword `match` en siguiente hueco libre de token.

## Status

- **Priority**: P3 | **Effort**: L | **Risk**: HIGH | **Depends on**: 009
- **Planned at**: `3229263`, 2026-06-21

## Scope

**In scope:** `lexer.nx`, `parser.nx`, `ast.nx` (`EXPR_MATCH`), `sema.nx`, `codegen.nx`, tests e2e/sema

**Out of scope:** patrones JSON (fase 17)

## Steps

### Step 1: Lexer + parser

`match` keyword. Brazos: literales, `val x`, desestructuración data/value, guard `if`, `_`.

### Step 2: Sema

Exhaustividad en expresión; tipar bindings por patrón.

### Step 3: Codegen

Lowering a cascada de checks con temporal común (reutilizar patrón switch fase 9).

**Verify:** `match_basic.nx`; `match_non_exhaustive.nx` → error

### Step 4: Bootstrap

`make verify-bootstrap` → exit 0.

## Done criteria

- [ ] `match` v1 ejecuta (literales, binding, guard, `_`)
- [ ] Exhaustividad comprobada
- [ ] `make verify-bootstrap` idéntico