# Plan 015: Fase 15 — `annotation` y `module` (sema + codegen)

> **Executor instructions**: Anotaciones sin efecto runtime v1. `module` prefija nombres emitidos de forma determinista; no romper `nx_*` runtime.

## Status

- **Priority**: P3 | **Effort**: M | **Risk**: MED | **Depends on**: 005
- **Planned at**: `3229263`, 2026-06-21

## Scope

**In scope:** `sema.nx`, `codegen.nx`, tests sema/e2e

**Out of scope:** procesadores de anotaciones en runtime, cambiar resolución de builtins `nx_*`

## Steps

### Step 1: Sema annotation

Registrar metadato adjunto; validar que anotación existe. Sin codegen.

### Step 2: Sema module

Espacio de nombres; afecta resolución de imports.

### Step 3: Codegen module

Prefijo determinista en símbolos emitidos (no en helpers runtime).

**Verify:** `annotation_unknown.nx` error; `module_scope.nx` PASS; e2e con module+annotation ejecuta

### Step 4: Bootstrap

`make verify-bootstrap` → exit 0.

## Done criteria

- [ ] Tests sema/e2e verdes
- [ ] `make verify-bootstrap` idéntico