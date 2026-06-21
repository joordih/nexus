# Plan 010: Fase 10 — Indexación segura `?[]` y asignación subscript

> **Executor instructions**: `?[` es token distinto de `?.` y `?`. Asignación `m[k]=v` no es reasignación de variable (fase 2 no bloquea).

## Status

- **Priority**: P2 | **Effort**: M | **Risk**: MED | **Depends on**: 001, 009
- **Planned at**: `3229263`, 2026-06-21

## Scope

**In scope:** `lexer.nx` (`TK_QUESTION_BRACKET`), `parser.nx`, `ast.nx` (`is_safe` en `EXPR_INDEX`), `sema.nx`, `codegen.nx`, tests e2e/sema

## Steps

### Step 1: Lexer + parser

`?[` en postfix. LHS `EXPR_INDEX` en asignación.

### Step 2: Sema

`?[]` → tipo nullable. `m[k]=v` valida tipos clave/valor.

### Step 3: Codegen

`?[]` → temporal + null check (patrón fase 1). Subscript assign → insert Map/List.

**Verify:** `safe_index.nx`, `subscript_assign.nx` en `make test-e2e`

### Step 4: Sema negativo + bootstrap

Acceso inseguro a resultado `?[]` → error. `make verify-bootstrap` → exit 0.

## Done criteria

- [ ] `?[]` y subscript assign ejecutan
- [ ] Test sema de acceso inseguro falla compilación
- [ ] `make verify-bootstrap` idéntico