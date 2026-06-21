# Plan 013: Fase 13 — Genéricos `<T>` con erasure

> **Executor instructions**: D8 — erasure a `void*` + boxing; sin monomorfización (preservar determinismo codegen).

## Status

- **Priority**: P3 | **Effort**: L | **Risk**: HIGH | **Depends on**: 012
- **Planned at**: `3229263`, 2026-06-21

## Scope

**In scope:** `sema.nx`, `codegen.nx`, tests e2e/sema (`generics.nx`)

**Out of scope:** RTTI generado, cambiar layout de `NxList`/`NxMap` en runtime

## Steps

### Step 1: Sema tipos paramétricos

Kind para type params; inferir `T` de contexto/args; error si incompatible.

### Step 2: Codegen erasure

Todo `T` → `void*`; boxing/unboxing con `voidCastIn`/`voidCastOut` existentes.

**Verify:** `tests/e2e/generics.nx` — `List<String>`, `Map<String, Int>`

### Step 3: Bootstrap

`make verify-bootstrap` → exit 0 (no nuevas funciones que alteren orden de emisión).

## Done criteria

- [ ] Colecciones genéricas tipan y ejecutan
- [ ] Uso incompatible → error sema
- [ ] `make verify-bootstrap` idéntico