# Plan 017: Fase 17 — JSON v2 con patrones `match` (cierre roadmap)

> **Executor instructions**: Última fase del plan maestro. Requiere 011 + 016. Actualizar `docs/SPEC.md` y `docs/GRAMMAR.md` al cerrar.

## Status

- **Priority**: P0 | **Effort**: L | **Risk**: HIGH | **Depends on**: 011, 016
- **Planned at**: `3229263`, 2026-06-21

## Why this matters

Cierre del roadmap — JSON declarativo:

```nexus
match data {
  case { query: { q: String } } -> handle(q)
  default -> fallback()
}
```

## Scope

**In scope:** `parser.nx`, `sema.nx`, `codegen.nx`, `tests/json/match_patterns.nx`, `docs/SPEC.md`, `docs/GRAMMAR.md`

## Steps

### Step 1: Patrones objeto/array en parser

Patrones JSON en brazos `match` sobre `JsonValue`.

### Step 2: Sema

Tipar patrón contra `JsonValue`; bindings con tipo extraído; nullable salvo garantía del patrón.

### Step 3: Codegen

Lowering a checks tipo + subscript en cascada; reutilizar `nx/std/json/access.nx`.

**Verify:** `tests/json/match_patterns.nx` — objeto anidado, array, default

### Step 4: Cierre global

- `make test` → exit 0 (todas las suites)
- `make verify-bootstrap` → exit 0
- `docs/SPEC.md` + `docs/GRAMMAR.md` reflejan todas las features del roadmap

## Done criteria

- [ ] JSON declarativo con `match` ejecuta
- [ ] Patrón que no casa cae en `default` sin panic
- [ ] Documentación viva al día
- [ ] `make test` + `make verify-bootstrap` verdes

## STOP conditions

- Bindings JSON incorrectamente non-null sin garantía de patrón
- Alguna suite de `make test` roja sin plan de regresión