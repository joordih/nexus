# Reglas del proyecto Nexus

Este documento es vinculante para cualquier desarrollador que trabaje en este repositorio. No son recomendaciones. `PLAN.md` describe qué se construye y en qué orden; este fichero describe cómo se trabaja. Ante cualquier duda sobre si una acción está permitida, la respuesta por defecto es no actuar y registrar la duda en `NOTES.md`.

## Disciplina de fases

El trabajo avanza por las fases definidas en el README, en orden, sin solaparse.

No se empieza una fase hasta que la anterior está cerrada. Una fase no está cerrada porque el código parezca terminado ni porque compile: está cerrada cuando su puerta de salida está verde y confirmada en un commit. La puerta de salida de cada fase está definida en el README y es la única autoridad sobre si la fase termina.

Dentro de una fase se avanza en pasos pequeños y verificables. Es preferible una sucesión de commits pequeños que dejan el repositorio en verde a un único commit grande que luego hay que depurar.

No se adelanta trabajo de fases futuras. Si durante una fase resulta tentador implementar algo que pertenece a otra posterior, no se hace; se anota la idea en `NOTES.md` y se retoma cuando llegue su fase.

## Definición de fase cerrada

Una fase está cerrada cuando se cumplen las tres condiciones a la vez:

- Las pruebas correspondientes a esa fase pasan, ejecutadas con el objetivo de `make` que las cubre.
- `SPEC.md` y `GRAMMAR.md` están actualizados si la fase cambió el lenguaje. El compilador y los documentos del lenguaje nunca deben divergir.
- Existe un commit que deja el repositorio en estado verde y cuyo mensaje nombra la fase.

A partir de la Fase 6, una condición adicional aplica a toda fase que toque el compilador o el backend: el punto fijo del arranque debe seguir cumpliéndose, es decir, `make verify-bootstrap` debe confirmar que la Etapa 2 y la Etapa 3 son binarios idénticos.

## Reglas de código

Estas reglas son absolutas y aplican a todo el código del repositorio, sea Rust, Nexus o C.

### Sin comentarios

No se escriben comentarios. Esto incluye `//`, `/* */`, `///` y `//!`. El significado lo cargan los nombres. Si un fragmento necesita un comentario para entenderse, se reescribe o se extrae a una función con nombre descriptivo.

No se usan marcadores como `TODO`, `FIXME`, `XXX` o `HACK`. El trabajo pendiente y los bloqueos van a `NOTES.md`, no al código.

Conviene precisar qué no es un comentario y por tanto está permitido, porque son código necesario, no texto explicativo: los atributos de Rust como `#[derive(...)]` y `#![...]`, las directivas de preprocesador de C como `#include`, `#define` y las guardas de cabecera `#ifndef`, `#define`, `#endif`, las líneas shebang y los pragmas. La prohibición es sobre la sintaxis de comentario, no sobre todo lo que empiece por `#`.

### Sin emojis

No se usan emojis en ningún sitio: ni en código, ni en `SPEC.md`, `GRAMMAR.md`, `README.md` o `NOTES.md`, ni en mensajes de commit, ni en la salida de los programas o de las herramientas.

### Nombres

Los nombres son la documentación. Una función, un tipo o una variable debe poder entenderse por su nombre y su firma sin más contexto. Se prefiere extraer una función con nombre claro a dejar una expresión densa que pediría una explicación.

### Generación de código determinista

La generación de código debe producir bytes idénticos para la misma entrada en ejecuciones distintas. Esto no es una preferencia: es lo que hace posible verificar el auto-hospedaje en la Fase 6, y debe respetarse desde la Fase 4.

En concreto:

- No se itera sobre mapas u otras estructuras sin orden garantizado al emitir código. Si hace falta recorrer claves para generar salida, se ordenan antes de emitir o se usa una estructura con orden estable.
- No se incrustan en la salida marcas de tiempo, rutas absolutas, números aleatorios ni direcciones de memoria.
- La misma entrada, compilada dos veces, produce el mismo fichero byte a byte.

## Las pruebas son la puerta

Ninguna fase se cierra sin pruebas que demuestren su criterio de salida. Las pruebas no son un añadido posterior: forman parte del entregable de la fase y son lo que autoriza a pasar a la siguiente.

Las pruebas de comparación, como los volcados de tokens, los volcados de AST o los snapshots de C generado, se comparan byte a byte contra el resultado esperado. Cuando un cambio legítimo altera la salida esperada, se actualiza el fichero esperado de forma consciente y se revisa la diferencia, nunca se relaja la comparación para que pase.

## Documentos vivos

`SPEC.md` y `GRAMMAR.md` describen el lenguaje tal como el compilador lo implementa, no tal como se aspira a que sea. Nunca deben describir una característica que el compilador no soporta todavía. Cada fase que cambia el lenguaje actualiza ambos en el mismo commit que cierra la fase.

`NOTES.md` es la bitácora. Recoge los bloqueos encontrados y las decisiones tomadas durante la ejecución, con el detalle suficiente para que cualquier desarrollador pueda retomar el trabajo sin reconstruir el contexto desde cero.

## Control de versiones

Cada commit deja el repositorio en estado verde: compila y pasa las pruebas existentes. No se confirma trabajo a medias que rompa la construcción.

El mensaje de commit nombra la fase y describe el cambio de forma concreta, por ejemplo `fase-4: backend a C y enlazado con libgc`. Sin emojis en el mensaje.

Un commit que cierra una fase deja explícito que la puerta de salida de esa fase está verde.

## Gestión de bloqueos

Si aparece un bloqueo que impide cerrar la puerta de una fase, el trabajo se detiene en la puerta. No se improvisa una solución que esquive el criterio de salida ni se avanza a la fase siguiente dejando la actual a medias.

El bloqueo se documenta en `NOTES.md`: qué se intentaba, qué falló, qué se descartó y qué haría falta para desbloquearlo. Una dependencia del sistema ausente, por ejemplo `libgc` no instalada, es un bloqueo de este tipo: se anota y se detiene, no se rodea.

## Entorno

El desarrollo y las pruebas son locales. No se introduce Docker ni contenedores para construir ni para probar el proyecto.

El toolchain necesario está en el README: Rust estable para la Etapa 0, un compilador de C, la librería de recolección de basura de Boehm con sus cabeceras, y `make`. El backend LLVM, posterior y opcional, añade `llc` y `clang` de LLVM.

## Disciplina del subconjunto Core

Hasta que el compilador se autoaloja y se cierran las fases que amplían el lenguaje, el código del compilador escrito en Nexus usa exclusivamente Nexus Core, el subconjunto definido en el README y en `GRAMMAR.md`. No se usa una característica del lenguaje antes de que su fase la haya implementado y cerrado.

Esta es la regla que más fácil es romper por descuido durante las primeras fases. La tentación de usar genéricos del usuario, un cierre o un bucle `for` antes de tiempo aparecerá. La respuesta es la misma siempre: esa característica entra en su fase, con su puerta, y solo entonces el compilador puede apoyarse en ella.

## Verificación automatizada

Las reglas que pueden comprobarse con una máquina deben comprobarse con una máquina, no confiarse a la revisión manual. El repositorio debe disponer de una verificación que, como mínimo:

- Rechace la presencia de sintaxis de comentario `//` y `/* */` en los ficheros de código.
- Rechace la presencia de emojis en código y documentación.
- Ejecute la suite de pruebas.
- A partir de la Fase 6, ejecute `make verify-bootstrap` y falle si la Etapa 2 y la Etapa 3 no son idénticas.

La comprobación de comentarios apunta a la sintaxis de comentario de forma precisa, de modo que no marque por error las directivas de preprocesador de C ni los atributos de Rust, que son código permitido.

## Resumen

Qué hacer: avanzar fase a fase cerrando cada puerta antes de la siguiente, escribir código sin comentarios cuyos nombres expliquen su intención, mantener la generación de código determinista, escribir las pruebas que cierran cada fase, mantener `SPEC.md` y `GRAMMAR.md` al día, y registrar los bloqueos en `NOTES.md`.

Qué no hacer: empezar una fase con la anterior abierta, escribir comentarios o emojis, usar características del lenguaje antes de su fase, introducir no determinismo en la salida, relajar una prueba para que pase, rodear un bloqueo en lugar de detenerse, o introducir Docker.