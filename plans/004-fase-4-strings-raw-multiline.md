# Plan 004: Fase 4 — Strings raw (`r"..."`) y multilínea (`"""..."""`)

> **Executor instructions**: Sigue paso a paso. Verifica cada gate. D11: no tocar `bootstrap/`.

## Status

- **Priority**: P1 | **Effort**: S | **Risk**: LOW | **Depends on**: 003
- **Planned at**: `3229263`, 2026-06-21

## Why this matters

Lexer puro que desbloquea literales sin escapes y bloques multilínea (D2 dedent). No toca sema/codegen: ambos emiten `TK_STRING` normal.

## Current state

- `compiler/src/lexer.nx` — `scanString` solo `"..."` con escapes; no `r"`, no `"""`
- `docs/GRAMMAR.md` — sin raw/triple-quote

## Commands

| Propósito | Comando | Éxito |
|-----------|---------|-------|
| Lexer | `make test-lexer` | exit 0 |
| E2E | `make test-e2e` | exit 0 |
| Bootstrap | `make verify-bootstrap` | exit 0 |

Windows: `set CC=C:\Program Files\LLVM\bin\clang.exe` antes de `make stage2`.

## Scope

**In scope:** `compiler/src/lexer.nx`, `tests/lexer/raw_string.tokens`, `tests/lexer/multiline_string.tokens`, `tests/e2e/raw_and_multiline.nx`, `docs/GRAMMAR.md`

**Out of scope:** parser, sema, codegen, interpolación (fase 7), `bootstrap/`

## Steps

### Step 1: `scanRawString` para `r"..."`

En `lexer.nx`, antes de `scanIdent`: si `r` + `"`, leer hasta `"` sin procesar `\`. Emitir `TK_STRING`.

**Verify:** `make test-lexer` con `tests/lexer/raw_string.nx` → `r"a\tb"` tokeniza como `a\tb` literal.

### Step 2: Modo multilínea `"""`

En `scanString`: si tras `"` hay dos `"` más, modo multilínea hasta `"""`. Dedent por indentación mínima común (D2). Emitir `TK_STRING`.

**Verify:** `tests/lexer/multiline_string.tokens` snapshot PASS.

### Step 3: E2E y docs

Crear `tests/e2e/raw_and_multiline.nx` + `.expected`. Actualizar `docs/GRAMMAR.md`.

**Verify:** `make test-e2e` + `make verify-bootstrap` → exit 0.

## Done criteria

- [ ] Raw y multilínea en lexer con snapshots
- [ ] E2E compila y ejecuta
- [ ] `make verify-bootstrap` byte-idéntico
- [ ] Sin comentarios en código nuevo (`make lint`)

## STOP conditions

- `verify-bootstrap` difiere → STOP (compilador no debe usar nuevas formas de string aún)
- Conflicto `r` como identificador vs raw prefix