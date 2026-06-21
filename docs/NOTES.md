# Bitácora de Nexus

Registro de fases cerradas, decisiones técnicas y trabajo en curso. Ante bloqueos o dudas de alcance, anotar aquí en lugar de dejar `TODO` en el código (`RULES.md`).

## Hitos cerrados

| Fase | Contenido | Puerta de salida |
|------|-----------|------------------|
| 0 | Runtime C, libgc, estructura del repo | Runtime compila y asigna con GC |
| 1 | Lexer (bootstrap Rust, luego Nexus) | `make test` — suite lexer |
| 2 | Parser y AST | `make test` — suite parser |
| 3 | Análisis semántico | `make test` — suite sema |
| 4 | Codegen C y runtime completo | `make test` — todas las suites; `make example NAME=hello` |
| 5 | Compilador escrito en Nexus (`compiler/src/`) | `make stage2` produce `nxc-stage2` |
| 6 | Auto-hospedaje verificado | `make verify-bootstrap` — stage2 y stage3 idénticos byte a byte |
| LSP I (A–G) | JSON, servidor LSP, extensión VS Code | `make test` (incl. `test-lsp`), `make verify-bootstrap`, `nexus-lsp` operativo |

Las fases 1–4 se implementaron primero en `bootstrap/` (Rust). El compilador de referencia hoy es `compiler/src/` (Nexus); Rust solo reconstruye stage0 tras `make clean` o clone limpio.

## Decisiones técnicas vigentes

**Codegen determinista.** Misma entrada produce los mismos bytes de C. El codegen ordena campos y variantes alfabéticamente antes de emitir. No se incrustan timestamps, rutas absolutas ni direcciones de memoria. Requisito para `make verify-bootstrap`.

**Nombres en runtime.** Funciones emitidas siguen el esquema `nx_method_TypeName_methodName`, constructores `nx_mk_TypeName`, helpers `nx_int_to_string`, etc. El prefijo `nx_` y el mangling se derivan de un solo sitio en codegen; no re-escribir el esquema inline.

**Imports.** Rutas con punto (`import std.json`, `import compiler.ast`, `import lsp.server`) resuelven a `nx/std/`, `compiler/src/` y `nx/lsp/`. El último segmento es el alias del módulo.

**LSP y stdout.** El parser acumula errores en `Program.parse_errors` en lugar de escribir en stdout, para no corromper el canal LSP stdio.

**Inferencia en codegen.** Tabla `var_types` rastrea tipos de variables locales para emitir C correcto. Llamadas calificadas `mod.fn()` delegan el tipo de retorno en `moduleFnReturnType` (compartido con sema); ver `docs/CODEGEN-QUALIFIED-CALL-FIX.md`.

## Plan maestro (fases 0–2)

**Fase 0.** CI en `.github/workflows/ci.yml` (`make test`, `make verify-bootstrap`). Tests en cuarentena en `tests/quarantine/` documentan
bugs que las fases 1–2 cierran.

**Fase 1.** Dispatch nullable sobre campos: `unwrapNullableTy` en sema/codegen para
receptores `T?`; `?:` con ternario tipado (no `||`).

**Fase 2.** Keyword `val` (`TK_VAL = 74`); `Scope.final_flags` rechaza reasignación de
`val`/`final`.

### Nexus Core (frontera stage0)

Tokens: `TK_*` 0–74 (`TK_VAL` último keyword). Construcciones: `var`/`val`/`final`,
`class`/`data`/`value`, `import`, `if`/`while`/`for`/`switch`/`try`, operadores
`?.` `?:` `!`, tipos primitivos y `List`/`Map`.

### Auditoría de primitivos

| Tipo | Aritmética codegen | toString |
|------|-------------------|----------|
| Int/Long | Sí (`NxInt`) | `nx_int_to_string` |
| Bool/Char | Parcial | `nx_bool_to_string`, `nx_char_to_string` |
| String | Concat `+` | nativo |
| Float/Double | **No** (emite `NxInt`) | **No** |
| Void/Null | N/A | N/A |

Gaps Float/Double: fase 3 del plan maestro.

## En curso y pendiente

**TLS en runtime y std.network.** Se añadió TLS de cliente vía OpenSSL en el runtime (`nx_tls_*`) y en `std.network` (`connectTlsSocket`, `httpGetOverTls`). OpenSSL 3.x es dependencia de enlace (`SSL_INCLUDE`/`SSL_LIB`, igual que `GC_INCLUDE`/`GC_LIB`). Limitación conocida: la verificación de certificado del servidor está desactivada por defecto; solo se activa si se define `NX_TLS_CA_BUNDLE` apuntando a un bundle de CAs. Endurecer la verificación por defecto (trust store del SO) queda como follow-up.

**API `fetch` en std.network.** Módulo `url.nx` (`URLSearchParams`, `parseUrl`/`ParsedUrl`), módulo `fetch.nx` (`HttpResponse` con `ok`/`status`/`json()`, función `fetch(url)`), apoyada en `connectTlsSocket` de 001. El ejemplo `zenserp_search` usa esta API. Limitaciones: `fetch` solo hace GET y asume `Content-Length` (sin `Transfer-Encoding: chunked`); la codificación de query solo cubre ASCII imprimible (caracteres no-ASCII se descartan), igual que el prototipo original.

**LSP Parte II — stdlib** (`LSP-STDLIB-PLAN.md`). Tier A cerrado (`nx/std/core/`, `nx/std/collections/`). Tier B cerrado (`nx/std/fs/`, `system/`, `network/`, `datetime/`, `regex/` + primitivas runtime). Tier C parcial: `try`/`catch`/`throw` en compilador; `nx/std/reflection/type_info.nx` (registro manual); `nx/std/concurrency/thread.nx` (identidad sin OS threads). Pendiente Tier C: RTTI generado por codegen, threads OS (pthreads/Win32), `Executor`/`Future`.

**Sintaxis parse-only.** `module`, `interface`, `annotation`, `extends`, `implements` y parámetros de tipo `<T>` están en parser y gramática pero aún sin sema ni codegen. Ver tabla en `SPEC.md`.

## LSP Parte I — detalle

- **A:** primitivas IO en runtime (`nx_read_header_line`, `nx_read_stdin_n`, `nx_write_stdout_n`, `nx_map_keys`, TCP); tests en `test_runtime.c`.
- **B:** librería JSON en `nx/std/json/`; suite `tests/json/`.
- **C:** spans `line`/`col` en AST; `SemaError` con posición en sema.
- **D–G:** servidor LSP en `nx/lsp/`; transporte stdio/TCP; diagnostics, completion, hover, definition, symbols; target `nexus-lsp`; cliente VS Code con `vscode-languageclient`.

## Histórico bootstrap (Rust)

El compilador semilla en `bootstrap/` usó una sintaxis intermedia (`struct`, `enum`, `impl`, `use`, `match`) durante las fases 2–4. Esa sintaxis ya no es Nexus Core; el subconjunto actual (`class`, `data`, `value`, `import`, ...) está definido en `SPEC.md` y `GRAMMAR.md`. El bootstrap Rust se conserva solo para `make bootstrap` → stage0.