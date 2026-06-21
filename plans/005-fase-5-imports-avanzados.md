# Plan 005: Fase 5 — Imports alias, agrupados y wildcard

> **Executor instructions**: D3 — wildcard importa módulos hijos del directorio, no miembros. Orden determinista en `nx_list_dir`.

## Status

- **Priority**: P1 | **Effort**: M | **Risk**: MED | **Depends on**: 003
- **Planned at**: `3229263`, 2026-06-21

## Why this matters

Hoy `parseImportDecl` solo acepta `import dot.path`. Alias, grupos y `import pkg.*` reducen verbosidad y habilitan fase 15 (`module`).

## Current state

- `compiler/src/parser.nx` — `parseImportDecl` path plano
- `compiler/src/sema.nx` — `expandProgramImports`, `importPathToFile`
- `compiler/src/ast.nx` — `Item.import_path` sin alias/wildcard

## Scope

**In scope:** `parser.nx`, `ast.nx`, `sema.nx`, tests parser/sema/e2e, `docs/GRAMMAR.md`

**Out of scope:** `bootstrap/`, cambiar resolución de `compiler/src/` o `nx/lsp/` (mantener rutas actuales)

## Steps

### Step 1: AST + parser

Añadir a `Item`: `import_alias: String`, `import_wildcard: Bool`. Reescribir `parseImportDecl`:
- `import x.y as z`
- `import std.{io, json}` → expandir a múltiples `ITEM_IMPORT`
- `import std.core.*` → wildcard flag

**Verify:** `make test-parser` → `import_alias.ast`, `import_group.ast`, `import_wildcard.ast`

### Step 2: Sema

Usar alias como nombre de módulo. Wildcard: `nx_list_dir` del directorio, ordenar nombres, expandir `.nx`. Colisión de dos wildcards → error.

**Verify:** `make test-sema` → `import_collision.nx` error; `import_alias_usage.nx` PASS

### Step 3: E2E + bootstrap

Programa con import agrupado + alias ejecuta. `make verify-bootstrap` → exit 0.

## Done criteria

- [ ] Tres formas de import funcionan
- [ ] Wildcard determinista y colisión detectada
- [ ] `make test` suites parser/sema/e2e verdes
- [ ] `make verify-bootstrap` idéntico

## STOP conditions

- Wildcard rompe imports existentes en `compiler/src/` sin migración
- `verify-bootstrap` difiere