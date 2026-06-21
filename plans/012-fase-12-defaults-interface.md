# Plan 012: Fase 12 — Defaults en constructores + `interface`/`implements`

> **Executor instructions**: D6 — defaults evaluados en call site (no una vez). Vtable determinista (métodos alfabéticos).

## Status

- **Priority**: P2 | **Effort**: L | **Risk**: HIGH | **Depends on**: 006
- **Planned at**: `3229263`, 2026-06-21

## Scope

**In scope:** `parser.nx`, `ast.nx` (`Param.default_val`), `sema.nx`, `codegen.nx`, tests e2e/sema

**Out of scope:** companion/factory (fase 14), herencia (fase 14)

## Steps

### Step 1: Parser defaults

`parseParam`: tras tipo, `=` + expr default.

### Step 2: Sema call site

Rellenar args omitidos con default. `interface` sin campos; clase `implements I` debe definir métodos de `I`.

### Step 3: Codegen

Emitir default en call site. Interface → struct de punteros a función; constructor rellena vtable.

**Verify:** `ctor_defaults.nx`, `interface_dispatch.nx`; sema: clase incompleta → error

### Step 4: Bootstrap

`make verify-bootstrap` → exit 0.

## Done criteria

- [ ] `URLSearchParams()` sin `order: List(), entries: Map()` verbose
- [ ] Dispatch interface funciona
- [ ] Tests sema/e2e verdes
- [ ] `make verify-bootstrap` idéntico