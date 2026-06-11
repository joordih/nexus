# Nexus

Nexus es un lenguaje de programación de propósito general que combina tipado estático con inferencia (Java, TypeScript), seguridad de memoria por propiedad y `match` exhaustivo (Rust), concurrencia ligera con canales (Go) y una biblioteca estándar amplia con sintaxis limpia (Python). Este repositorio contiene el lenguaje y su compilador autoalojado: `nxc` está escrito en Nexus y se compila a sí mismo.

Este documento es el plan de construcción. No describe el lenguaje terminado: describe el camino verificable desde cero hasta un compilador que se reproduce a sí mismo, y desde ahí hasta el conjunto completo de características.

## Objetivo y alcance

El Nexus completo descrito en la especificación es enorme: recolección de basura, propiedad y borrow checker, goroutines, canales, async/await, genéricos, traits, macros y decoradores. Intentar implementar todo eso antes de tener un compilador que funcione es la forma garantizada de no terminar nunca.

La estrategia es la que usaron Rust (compilador inicial en OCaml) y Go (compilador inicial en C): se define un subconjunto mínimo, **Nexus Core**, suficiente para escribir un compilador en él. Primero se construye un compilador de Core en un lenguaje anfitrión. Después se reescribe ese mismo compilador en Nexus Core y se compila con el anfitrión. A partir del punto en que el compilador se reproduce a sí mismo de forma idéntica, el anfitrión se retira y cada característica nueva se añade tanto al lenguaje como al compilador, que ya está escrito en Nexus.

El resultado de este plan no es “Nexus terminado”. El resultado es un compilador autoalojado y una ruta de fases en la que cada incremento deja el sistema en estado funcional y verificado.

## Decisiones de arquitectura

Estas decisiones están tomadas. Si se cambian, hay que cambiarlas aquí antes de empezar, no a mitad de una fase.

**Anfitrión de la Etapa 0: Rust.** El compilador inicial se escribe en Rust. La razón no es el rendimiento sino el modelo mental: la semántica de propiedad de Nexus se piensa mejor desde un lenguaje que ya la impone, y el borrow checker posterior será más fácil de diseñar habiéndolo vivido. La Etapa 0 es desechable: se retira en cuanto el compilador se autoaloja, así que su anfitrión no condiciona el producto final.

**Backend: transpilación a C.** El compilador no genera ensamblador ni código máquina directamente. Emite C portable y lo entrega a `clang` o `gcc`. Esto resuelve de un golpe varios problemas del arranque: el recolector de basura se obtiene enlazando la librería de Boehm-Demers-Weiser (`libgc`), la FFI con el sistema es trivial, la salida es depurable y el binario es portable. La generación de código del compilador autoalojado se reduce a generar cadenas de texto, lo que hace el auto-hospedaje muchísimo más tratable que escribir un backend nativo en Nexus Core.

**Backend LLVM: posterior y opcional.** Una vez logrado el auto-hospedaje, se puede añadir un backend que emita LLVM IR en texto (`.ll`) y lo pase a `llc`. Ese backend se escribe en Nexus, no en Rust, y sirve como demostración de la potencia del lenguaje. No es requisito para el auto-hospedaje.

**Memoria en el arranque: GC para todo.** El subconjunto Core no impone propiedad ni préstamos en tiempo de ejecución; todo se gestiona con el recolector. El borrow checker es una fase de análisis estático posterior que no cambia el modelo de ejecución, solo rechaza programas en tiempo de compilación. Separar el modelo de ejecución (GC primero) del análisis estático (borrow checker después) es lo que mantiene el plan abordable.

**Sin Docker.** Toda la infraestructura es local: toolchain de Rust, un compilador de C, `libgc` y un sistema de construcción. No se introduce contenedores para el desarrollo ni para las pruebas.

**Normalización de sintaxis.** La especificación original mezcla `class` al estilo Java con `&self` y `&mut self` al estilo Rust, lo cual es incoherente. Core estandariza en `struct` más bloques `impl` con métodos que reciben `self`, `&self` o `&mut self`. El azúcar `class` se puede añadir más tarde como capa sobre `struct`/`impl` o descartarse; no entra en el arranque.

## Nexus Core: el subconjunto de arranque

Core es exactamente lo necesario para escribir un compilador, y nada más. Cualquier característica fuera de esta lista pertenece a una fase posterior y no debe usarse en el código del compilador hasta que esa fase esté cerrada.

Dentro de Core:

- Tipos primitivos: `int`, `bool`, `char`, `string`. `float` se pospone si no es imprescindible para el compilador.
- Funciones libres con `fn`, sin genéricos definidos por el usuario.
- `struct` (tipos producto) y `enum` con cargas útiles (tipos suma etiquetados).
- Bloques `impl` con métodos sobre `struct`.
- `match` exhaustivo sobre enums y literales.
- `let` con mutabilidad explícita mediante `mut`, asignación.
- Control de flujo: `if`/`else`, `while`, `return`, `break`, `continue`.
- Operadores con precedencia, parseados con un analizador de Pratt.
- `use` mínimo para organización en ficheros.
- `List<T>` y `Map<K, V>` provistos por el runtime como tipos integrados.

Fuera de Core, en fases posteriores: genéricos y traits definidos por el usuario, borrow checker, goroutines y canales, async/await, cierres, macros y decoradores.

**La tensión de `List` y `Map` sin genéricos.** Para escribir un compilador hacen falta listas y mapas, pero los genéricos del usuario no están en Core. Se resuelve así: el runtime en C provee `List` y `Map` con borrado de tipos (`void*` internamente), y el compilador les da un tratamiento especial en el sistema de tipos. El programador los usa como si fueran genéricos, pero el compilador no necesita implementar genericidad general para soportarlos. Cuando lleguen los genéricos del usuario (Fase 7), el propio compilador puede refactorizarse para usarlos, pero eso es opcional y posterior.

Gramática compacta de Core, en EBNF. La versión autoritativa y completa se mantiene en `GRAMMAR.md`:

```
program     = { item } ;
item        = use_decl | struct_decl | enum_decl | impl_block | function ;
use_decl    = "use" path ";" ;
struct_decl = "struct" IDENT "{" { field "," } "}" ;
field       = IDENT ":" type ;
enum_decl   = "enum" IDENT "{" { variant "," } "}" ;
variant     = IDENT [ "(" type { "," type } ")" ] ;
impl_block  = "impl" IDENT "{" { function } "}" ;
function    = [ "pub" ] "fn" IDENT "(" [ params ] ")" [ "->" type ] block ;
params      = param { "," param } ;
param       = ( "self" | "&" "self" | "&" "mut" "self" ) | ( IDENT ":" type ) ;
block       = "{" { stmt } "}" ;
stmt        = let_stmt | return_stmt | if_stmt | while_stmt
            | "break" ";" | "continue" ";" | expr ";" ;
let_stmt    = "let" [ "mut" ] IDENT [ ":" type ] "=" expr ";" ;
expr        = (* analizador de Pratt: literales, llamadas, acceso a campos, *)
              (* operadores binarios y unarios, match, construcción de structs *) ;
```

Programa objetivo de referencia para el extremo a extremo (debe compilar y ejecutar correctamente al cerrar la Fase 4):

```nexus
use std::io;

struct Punto {
    x: int,
    y: int,
}

impl Punto {
    fn distancia_origen_cuadrado(&self) -> int {
        return self.x * self.x + self.y * self.y;
    }
}

enum Figura {
    Circulo(int),
    Rectangulo(int, int),
}

fn area_aproximada(figura: Figura) -> int {
    match figura {
        Figura::Circulo(radio) => return 3 * radio * radio,
        Figura::Rectangulo(ancho, alto) => return ancho * alto,
    }
}

fn main() {
    let p = Punto { x: 3, y: 4 };
    io::println(p.distancia_origen_cuadrado());
    io::println(area_aproximada(Figura::Circulo(5)));
}
```

## Estructura del repositorio

```
nexus/
  PLAN.md              este plan
  RULES.md             reglas operativas vinculantes del proyecto
  SPEC.md              especificacion del lenguaje, crece por fases
  GRAMMAR.md           gramatica formal autoritativa, crece por fases
  NOTES.md             bitacora de bloqueos y decisiones
  Makefile             objetivos de construccion y verificacion
  bootstrap/           compilador de Etapa 0 en Rust
    Cargo.toml
    src/
      lexer.rs
      ast.rs
      parser.rs
      sema.rs
      codegen_c.rs
      driver.rs
  compiler/            compilador de Nexus escrito en Nexus
    src/
      lexer.nx
      ast.nx
      parser.nx
      sema.nx
      codegen.nx
      driver.nx
  runtime/             runtime en C
    nexus_runtime.h
    nexus_runtime.c
  std/                 biblioteca estandar en Nexus
  tests/
    lexer/             entradas .nx y salidas .tokens esperadas
    parser/            entradas .nx y volcados de AST esperados
    sema/              programas validos e invalidos con diagnosticos esperados
    codegen/           snapshots de C generado
    e2e/               programas con su salida estandar esperada
  examples/
  build/               artefactos, ignorado por git
```

## Requisitos del entorno

- Toolchain de Rust estable para la Etapa 0.
- Un compilador de C: `clang` preferido, `gcc` aceptable.
- La librería de recolección de basura de Boehm-Demers-Weiser (`bdwgc` / `libgc`) con sus cabeceras de desarrollo.
- `make`.
- Posterior y opcional, solo para el backend LLVM: `llc` y `clang` de una instalación de LLVM.

No se usa Docker. Si una dependencia del sistema no está disponible, se documenta el bloqueo en `NOTES.md` y se detiene en la puerta de la fase correspondiente.

## Protocolo de desarrollo

1. Leer `PLAN.md` y `RULES.md` completos antes de escribir una sola línea.
1. Trabajar estrictamente por fases. No se empieza la Fase N+1 hasta que la puerta de salida de la Fase N está verde y confirmada en un commit. Una fase no está cerrada porque “parezca” terminada: está cerrada cuando sus pruebas pasan.
1. Cada fase termina con tres cosas: las pruebas de esa fase en verde, `SPEC.md` y `GRAMMAR.md` actualizados si el lenguaje cambió, y un commit con un mensaje claro que nombre la fase.
1. Dentro de una fase, avanzar en pasos pequeños y verificables. Es preferible diez commits pequeños que pasan a uno grande que hay que depurar.
1. Si aparece un bloqueo que impide cerrar la puerta, detenerse en la puerta. Escribir el bloqueo en `NOTES.md` con detalle suficiente para retomarlo. No improvisar para pasar la puerta a la fuerza.
1. Mantener `SPEC.md`, `GRAMMAR.md` y `NOTES.md` como documentos vivos. El compilador y la especificación no pueden divergir.

### Reglas duras de código

- **Sin comentarios.** Ni en Rust, ni en Nexus, ni en C, ni en ningún sitio. Los nombres cargan el significado. Si un fragmento necesita un comentario para entenderse, hay que reescribirlo o extraerlo a una función con nombre.
- **Sin emojis.** En código, documentación, mensajes de commit ni logs.
- **Generación de código determinista.** El código emitido debe ser idéntico para la misma entrada en ejecuciones distintas. Sin orden de iteración no determinista, sin timestamps incrustados, sin rutas absolutas en la salida. El determinismo no es opcional: es lo que hace posible la verificación del auto-hospedaje.
- **Las pruebas son la puerta.** Ninguna fase se cierra sin pruebas que demuestren su criterio de salida.

## Plan por fases

Cada fase tiene un objetivo, sus entregables y una puerta de salida con condiciones concretas y verificables. La puerta es lo único que determina si la fase está cerrada.

### Fase 0 — Fundaciones y toolchain

Objetivo: dejar el repositorio listo para construir, con el subconjunto Core definido y el arnés de pruebas operativo.

Entregables: estructura de directorios, `Cargo.toml` de la Etapa 0 que compila un binario vacío, `Makefile` con los objetivos esqueleto, `SPEC.md` y `GRAMMAR.md` con la definición inicial de Core, `RULES.md`, y el runtime en C compilando y enlazando contra `libgc` en un binario de prueba mínimo.

Puerta de salida:

- `make bootstrap` produce `build/nxc-stage0`, aunque por ahora no haga nada útil.
- El runtime en C compila, enlaza `libgc` y un binario de prueba que asigna memoria gestionada se ejecuta sin fallos.
- `make test` ejecuta el arnés, aunque no haya casos todavía.
- `GRAMMAR.md` contiene la gramática completa de Core.

### Fase 1 — Lexer (Etapa 0)

Objetivo: tokenizar Nexus Core en Rust.

Entregables: `bootstrap/src/lexer.rs` con el tipo de token, posiciones de línea y columna, reconocimiento de identificadores, palabras clave, números, cadenas, caracteres y operadores. Casos de error con diagnósticos definidos para caracteres ilegales y cadenas sin cerrar.

Puerta de salida:

- La Etapa 0 tokeniza todos los ficheros de `tests/lexer/` y la salida coincide byte a byte con los `.tokens` esperados.
- Los casos de error producen el diagnóstico definido, con línea y columna correctas.
- `make test` pasa la suite de lexer.

### Fase 2 — Parser y AST (Etapa 0)

Objetivo: construir el árbol de sintaxis abstracta de Core mediante descenso recursivo, con analizador de Pratt para expresiones.

Entregables: `bootstrap/src/ast.rs` con los nodos, `bootstrap/src/parser.rs` con el parser. Soporte para `use`, `struct`, `enum`, `impl`, `fn`, sentencias, expresiones, `match` y construcción de structs. Diagnósticos definidos para tokens inesperados con recuperación suficiente para reportar más de un error por fichero.

Puerta de salida:

- La Etapa 0 parsea todos los ficheros de `tests/parser/` y el volcado del AST coincide con el esperado.
- Los programas malformados producen los diagnósticos definidos en la posición correcta.
- `make test` pasa la suite de parser.

### Fase 3 — Análisis semántico y tipos (Etapa 0)

Objetivo: resolución de nombres y verificación de tipos para Core, sin borrow checker.

Entregables: `bootstrap/src/sema.rs` con tabla de símbolos, resolución de tipos, comprobación de tipos en expresiones y sentencias, comprobación de exhaustividad de `match`, y verificación de que las funciones con tipo de retorno siempre retornan. Tratamiento especial de `List` y `Map` integrados.

Puerta de salida:

- Todos los programas válidos de `tests/sema/` pasan la comprobación.
- Todos los programas inválidos son rechazados con el diagnóstico esperado: tipos incompatibles, nombres no resueltos, `match` no exhaustivo, retorno faltante.
- `make test` pasa la suite de semántica.

### Fase 4 — Backend a C y runtime (Etapa 0)

Objetivo: generar C a partir del AST tipado, enlazar el runtime y producir un binario nativo. Cierre del flujo `.nx` a ejecutable.

Entregables: `bootstrap/src/codegen_c.rs`, runtime en C completo para Core con `List`, `Map`, formateo de cadenas y `io::println`, y el driver que orquesta lexer, parser, sema, codegen, invocación de `clang`/`gcc` y enlazado con `libgc`.

Puerta de salida:

- El programa objetivo de referencia compila a binario y su salida estándar es la esperada.
- Todos los programas de `tests/e2e/` compilan, se ejecutan y su salida coincide con la esperada.
- El C generado para `tests/codegen/` coincide con los snapshots, y es determinista entre ejecuciones.
- `make example NAME=hello` compila y ejecuta un ejemplo.

A partir de aquí la Etapa 0 puede compilar cualquier programa de Nexus Core a código nativo. Empieza el auto-hospedaje.

### Fase 5 — Reescritura del compilador en Nexus Core (Etapa 1)

Objetivo: reescribir lexer, parser, sema y codegen en Nexus Core, dentro de `compiler/`, compilable por la Etapa 0.

Entregables: `compiler/src/*.nx` reproduciendo la funcionalidad de la Etapa 0, escrito exclusivamente con características de Core. Sin comentarios.

Puerta de salida:

- La Etapa 0 compila `compiler/` y produce `build/nxc-stage1`.
- `build/nxc-stage1` pasa las suites de lexer, parser, sema y e2e con los mismos resultados que la Etapa 0.
- `make stage1` ejecuta el proceso completo.

### Fase 6 — Verificación de auto-hospedaje

Objetivo: demostrar que el compilador se reproduce a sí mismo de forma idéntica. Este es el hito central del proyecto.

El criterio es el arranque en tres etapas que usa GCC. No basta con que la Etapa 1 funcione: hay que probar el punto fijo.

- Etapa 1 es producida por la Etapa 0, escrita en Rust. Su binario puede diferir por detalles incidentales del backend de Rust.
- Etapa 2 es `nxc-stage1` compilando `compiler/`.
- Etapa 3 es `nxc-stage2` compilando `compiler/`.
- Como Etapa 2 y Etapa 3 las produce el mismo compilador de Nexus a partir del mismo fuente, si la generación de código es determinista deben ser binarios idénticos byte a byte.

No se comparan Etapa 1 y Etapa 2, porque las generan compiladores distintos. Se comparan Etapa 2 y Etapa 3.

Puerta de salida:

- `make stage2` y `make stage3` producen `build/nxc-stage2` y `build/nxc-stage3`.
- `make verify-bootstrap` confirma que `nxc-stage2` y `nxc-stage3` son idénticos byte a byte mediante `cmp`.
- `nxc-stage2` pasa la suite completa de pruebas con resultados idénticos a la Etapa 0.
- A partir de este punto la Etapa 0 sale de la ruta crítica. Se conserva en el repositorio pero el compilador de referencia pasa a ser el escrito en Nexus.

### Fase 7 — Genéricos y traits

Objetivo: añadir genéricos definidos por el usuario y traits al lenguaje y al compilador.

Entregables: gramática y sema extendidos, monomorfización en el backend, traits con despacho estático. `SPEC.md` y `GRAMMAR.md` actualizados. Pruebas de genéricos y traits.

Puerta de salida:

- El compilador autoalojado compila programas con genéricos y traits, con sus pruebas en verde.
- El punto fijo del arranque se mantiene: `make verify-bootstrap` sigue confirmando Etapa 2 igual a Etapa 3.

### Fase 8 — Borrow checker

Objetivo: análisis estático de propiedad y préstamos. No cambia el modelo de ejecución, que sigue apoyado en el GC; rechaza programas en tiempo de compilación.

Entregables: paso de análisis de préstamos en el compilador, con diagnósticos definidos para usos tras movimiento y préstamos en conflicto. Pruebas con programas válidos e inválidos.

Puerta de salida:

- Los programas con violaciones de préstamo son rechazados con el diagnóstico esperado.
- Los programas válidos previos siguen compilando.
- El punto fijo del arranque se mantiene.

### Fase 9 — Concurrencia: goroutines y canales

Objetivo: concurrencia ligera con canales tipados.

Entregables: planificador en el runtime, ya sea sobre hilos del sistema con un planificador M:N o hilos verdes mediante `ucontext`. Primitiva `go`, canales tipados, sincronización. Pruebas de concurrencia deterministas en su resultado observable.

Puerta de salida:

- Los programas concurrentes producen resultados correctos y deterministas en su salida.
- El punto fijo del arranque se mantiene.

### Fase 10 — Async y await

Objetivo: funciones asíncronas con `await`.

Entregables: lowering de funciones async a máquinas de estado, al estilo de Rust y C#, integradas con el runtime de concurrencia. Pruebas de async.

Puerta de salida:

- Los programas con async y await compilan y se ejecutan correctamente.
- El punto fijo del arranque se mantiene.

### Fase 11 — Backend LLVM IR

Objetivo: un backend alternativo que emita LLVM IR en texto, escrito en Nexus.

Entregables: `compiler/src/codegen_llvm.nx` que emite `.ll` y lo entrega a `llc`. Selección de backend por opción del driver. Paridad de la suite e2e con el backend de C.

Puerta de salida:

- La suite e2e pasa con el backend LLVM con la misma salida que con el backend de C.
- El backend de C sigue funcionando; LLVM es alternativa, no reemplazo.
- El punto fijo del arranque se mantiene para ambos backends.

### Fase 12 — Biblioteca estándar y herramientas

Objetivo: ampliar `std/` y añadir herramientas de desarrollo.

Entregables: módulos de `std/` en Nexus, un formateador de código, y la organización de paquetes documentada. Pruebas de la biblioteca estándar.

Puerta de salida:

- Los módulos de `std/` compilan y pasan sus pruebas.
- El formateador es idempotente: formatear dos veces produce el mismo resultado.
- El punto fijo del arranque se mantiene.

## Documentos vivos

- `SPEC.md`: la especificación del lenguaje. Crece en cada fase que añade características. Nunca debe describir algo que el compilador no implementa.
- `GRAMMAR.md`: la gramática formal autoritativa. La de este README es una versión compacta de Core; la completa vive aquí.
- `RULES.md`: las reglas operativas y de código del proyecto.
- `NOTES.md`: bitácora de bloqueos y decisiones tomadas durante la ejecución, con detalle suficiente para retomar el trabajo.

## Comandos de referencia

```
make bootstrap          construye la Etapa 0 en Rust
make stage1             la Etapa 0 compila compiler/ a nxc-stage1
make stage2             nxc-stage1 compila compiler/ a nxc-stage2
make stage3             nxc-stage2 compila compiler/ a nxc-stage3
make verify-bootstrap   confirma que stage2 y stage3 son identicos y corre la suite
make test               ejecuta todas las pruebas
make example NAME=foo   compila y ejecuta examples/foo.nx
make clean              limpia build/
```

## Riesgos y dónde puede atascarse

Conviene ser honesto sobre los puntos duros, porque es donde la ejecución se detiene.

El auto-hospedaje de la Fase 6 falla en la práctica casi siempre por no determinismo en la generación de código: orden de recorrido de mapas, rutas absolutas en la salida, cualquier diferencia incidental rompe la igualdad byte a byte entre Etapa 2 y Etapa 3. La disciplina de determinismo de las reglas duras existe precisamente para esto y debe respetarse desde la Fase 4, no parchearse en la Fase 6.

El borrow checker de la Fase 8 es conceptualmente la parte más difícil del análisis estático. Está colocado después del auto-hospedaje a propósito: no bloquea el hito central, y para entonces el compilador ya es una base estable sobre la que iterar.

La concurrencia y el async de las Fases 9 y 10 introducen un planificador en el runtime, que es la parte del sistema más propensa a fallos sutiles. Por eso van al final, sobre un compilador ya autoalojado y verificado, y por eso sus pruebas deben tener salida observable determinista aunque la ejecución interna no lo sea.

El subconjunto Core está deliberadamente recortado para que el primer compilador funcional llegue pronto. La tentación durante las primeras fases será usar una característica que aún no existe en Core. No se hace. Cada característica entra en su fase, con su puerta, y solo entonces el compilador puede usarla.