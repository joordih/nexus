# Plan 000: Fase 0 — Cimientos CI, guardarraíles y baseline

> **Executor instructions**: Esta fase está **cerrada**. Usar este plan solo para verificar cierre o re-abrir si algo regresó. Si los drift checks fallan, STOP y reportar.

## Status

- **Priority**: P0
- **Effort**: S
- **Risk**: LOW
- **Depends on**: none
- **Category**: dx
- **Planned at**: commit `3229263`, 2026-06-21
- **Estado repo (2026-06-21)**: DONE (residual menor en `test-json` harness)

## Why this matters

Congela la línea base verificable: CI, lint de comentarios, tests en cuarentena documentando bugs de fases 1–2, y auditoría en `docs/NOTES.md`.

## Done criteria (verificación de cierre)

- [ ] `.github/workflows/ci.yml` existe y corre `make test` + `make verify-bootstrap`
- [ ] `make lint` → exit 0
- [ ] `make verify-bootstrap` → exit 0
- [ ] `tests/quarantine/` contiene repros de nullable (histórico)
- [ ] `docs/NOTES.md` documenta frontera Nexus Core y gaps Float/Double (actualizar si fase 3 cerró gaps)

## Residual opcional

Añadir newline final a `tests/json/roundtrip.expected` para que `make test-json` pase en `make test` agregado.

## STOP conditions

- `make verify-bootstrap` deja de ser byte-idéntico sin cambio documentado en el plan que lo causó.