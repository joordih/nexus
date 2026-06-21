# Plan 008: Fase 8 — Smart casts / flow typing

> **Executor instructions**: Solo sema; codegen no cambia. Limitar a variables locales y checks sintácticos directos.

## Status

- **Priority**: P2 | **Effort**: M | **Risk**: MED | **Depends on**: 001
- **Planned at**: `3229263`, 2026-06-21

## Why this matters

Tras `if (x != null)`, usar `x.metodo()` sin `!`. Reduce ruido antes del capstone JSON.

## Scope

**In scope:** `compiler/src/sema.nx`, tests `tests/sema/smart_cast_*.nx`

**Out of scope:** codegen, análisis interprocedural, campos mutables vía llamadas

## Steps

### Step 1: FlowEnv

`FlowEnv` por scope: `variable -> tipo estrechado`. `if (x != null)` → then: no-nullable; `x == null` → else. También tras `&&` y early `return`.

### Step 2: Scope.narrow

Apilar override; descartar al salir del bloque. Revertir si variable se reasigna.

### Step 3: Tests

- `smart_cast_if.nx` — compila sin `!`
- `smart_cast_early_return.nx` — compila
- `smart_cast_fail.nx` — error sin estrechar

**Verify:** `make test-sema` + `make verify-bootstrap` → exit 0

## Done criteria

- [ ] Tres tests sema PASS/FAIL según diseño
- [ ] `make verify-bootstrap` idéntico (sin cambio codegen)

## STOP conditions

- Smart cast atraviesa asignación en el mismo bloque sin invalidar