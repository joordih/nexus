# Plan 014: Fase 14 — `extends` / herencia + companion/factory

> **Executor instructions**: D9 — layout padre primero (alfabético), luego hijo. Detectar ciclos de herencia.

## Status

- **Priority**: P3 | **Effort**: L | **Risk**: HIGH | **Depends on**: 012, 013
- **Planned at**: `3229263`, 2026-06-21

## Scope

**In scope:** `sema.nx`, `codegen.nx`, tests e2e/sema (`inheritance.nx`, `companion.nx`)

## Steps

### Step 1: Sema herencia

Registrar relación `extends`; overrides compatibles; `Tipo.metodo` estático → companion.

### Step 2: Codegen layout + vtable

Campos padre + hijo ordenados. Override en vtable del constructor. `super.method()` → llamada directa padre. `static of(...)` → `nx_fn_Tipo_of(...)`.

**Verify:** `inheritance.nx`, `companion.nx` (`Optional.of(x)`)

### Step 3: Sema negativo + bootstrap

Override incompatible → error. `make verify-bootstrap` → exit 0.

## Done criteria

- [ ] Herencia + super + companion ejecutan
- [ ] Sin ciclos de herencia silenciosos
- [ ] `make verify-bootstrap` idéntico