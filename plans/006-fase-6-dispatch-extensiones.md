# Plan 006: Fase 6 — Dispatch unificado, extensiones y `toString`

> **Executor instructions**: Migrar dispatch sin cambiar C emitido para el compilador self-host. Comparar stage2 antes/después.

## Status

- **Priority**: P1 | **Effort**: L | **Risk**: HIGH | **Depends on**: 003
- **Planned at**: `3229263`, 2026-06-21

## Why this matters

Dispatch de métodos hoy es if/else hardcodeado en sema/codegen. Tabla central + `fun Tipo.metodo()` habilita extensiones JSON (fase 11) y `toString` unificado (D4).

## Current state

- `compiler/src/sema.nx` — `inferCallExpr` hardcodeado; sin `DispatchEntry`
- `compiler/src/codegen.nx` — cadenas if/else por tipo receptor
- `compiler/src/parser.nx` — no parsea `fun String.foo()`

## Scope

**In scope:** `sema.nx`, `codegen.nx`, `parser.nx`, `ast.nx`, `runtime/nexus_runtime.c` (helpers faltantes), tests e2e/codegen/sema

**Out of scope:** JSON accessors (fase 11), interpolación (fase 7)

## Steps

### Step 1: Tabla de dispatch en sema

Introducir `DispatchEntry { recv_type, method_name, return_type, lowering_fn }`. Poblar builtins: String, List, Map, Int, Long, Float, Double, Bool, Char. Añadir `String.endsWith` si falta en sema.

### Step 2: Parser extensiones

`fun TipoNombre.metodo(params): Ret { ... }` → `ITEM_EXTENSION` con `receiver_type` en `Function`.

### Step 3: Codegen consulta tabla

Reemplazar if/else por lookup. Extensiones: `nx_ext_String_shout(NxString this, ...)`.

**Verify:** `make test-e2e` → `primitive_methods.nx`, `extension_string_shout.nx`

### Step 4: Snapshot codegen + bootstrap

`tests/codegen/dispatch_table.c.expected`. `make verify-bootstrap` → exit 0 (diff stage2 vacío).

## Done criteria

- [ ] Tabla reemplaza hardcode sin cambiar C del compilador
- [ ] Extensiones concretas funcionan
- [ ] Paridad sema/codegen verificada en tests
- [ ] `make verify-bootstrap` idéntico

## STOP conditions

- `verify-bootstrap` difiere → revertir y ajustar lowering para preservar bytes
- Extensiones visibles sin import (ambigüedad) → STOP y acotar visibilidad