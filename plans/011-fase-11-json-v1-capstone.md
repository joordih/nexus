# Plan 011: Fase 11 — JSON v1 capstone (Definition of Done)

> **Executor instructions**: Capstone del roadmap. Requiere fases 1, 6, 8, 10. D7: azúcar `.` solo en `JsonValue`.

## Status

- **Priority**: P0 | **Effort**: L | **Risk**: HIGH | **Depends on**: 001, 006, 008, 010
- **Planned at**: `3229263`, 2026-06-21

## Why this matters

Objetivo declarado del plan maestro:

```nexus
val query = data["query"]?.string("q") ?: "N/A"
val q2 = data.query?.q ?: "N/A"
```

Sin `!` redundantes. Cuatro ejemplos README deben compilar con JSON null-safe.

## Current state

- `nx/std/json/access.nx` — funciones `jsonGetField`, `jsonGetString` (no extensiones tipadas)
- `tests/json/roundtrip.nx` — round-trip parser/writer (arreglar harness newline en `.expected`)
- No existe `tests/json/json_safe_access.nx`

## Scope

**In scope:** `nx/std/json/access.nx`, `compiler/src/sema.nx` (D7 desazúcar), extensiones vía fase 6, `tests/json/json_safe_access.nx`, ejemplos README, `docs/SPEC.md`

**Out of scope:** `match` sobre JSON (fase 17), tipo `dynamic`

## Steps

### Step 1: Extensiones JsonValue

En `access.nx` (o módulos hijos): `fun JsonValue.string(key: String): String?`, `number`, `bool`, `array`, `object` — como extensiones fase 6.

### Step 2: Azúcar punto D7 en sema

`data.query` sobre `JsonValue` → `data["query"]` tipo `JsonValue?`. Miembro de `JsonValue?` sin `?.`/`!` → error.

### Step 3: Tests + ejemplos

- `tests/json/json_safe_access.nx` + `.expected` — ambos patrones arriba
- Reescribir 4 ejemplos README con acceso null-safe
- Arreglar `roundtrip.expected` newline para `make test-json`

**Verify:** `make test-json`, `make test-e2e`, `make test` → exit 0

### Step 4: Bootstrap

`make verify-bootstrap` → exit 0.

## Done criteria

- [ ] `json_safe_access.nx` PASS
- [ ] Cuatro ejemplos README compilan y ejecutan
- [ ] `make test` completo verde
- [ ] `make verify-bootstrap` idéntico
- [ ] D7 limitado estrictamente a `JsonValue`

## STOP conditions

- Azúcar punto se generaliza a otros tipos (agujero de tipos)
- Cadenas `?.` largas no cortocircuitan en primer null