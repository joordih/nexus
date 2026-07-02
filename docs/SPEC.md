# Especificación de Nexus Core

Este documento describe Nexus Core, el subconjunto con el que está escrito el compilador autoalojado. La referencia de implementación es `build/nxc-stage2` (`compiler/src/` en Nexus). El compilador semilla en Rust (`bootstrap/`) implementa el mismo subconjunto para reconstruir la cadena desde cero.

La gramática formal está en `GRAMMAR.md`. La hoja de ruta de ampliaciones futuras está en `PLAN.md`.

## Qué implementa el compilador hoy

| Área | Estado |
|------|--------|
| `import`, funciones, `data`, `value`, `class`, variables locales y globales | Sema y codegen completos |
| `List<T>`, `Map<K,V>`, null safety (`T?`, `?.`, `?:`, `!`) | Sema y codegen completos |
| `if`/`else`, `while`, `for...in`, `switch`, lambdas, argumentos nombrados | Sema y codegen completos |
| Strings raw `r"..."` y multilínea `"""..."""` (dedent) | Lexer completo: producen literales `String` normales |
| Imports con alias (`as`), agrupados (`{a, b}`) y wildcard (`.*`) | Sema y codegen completos |
| Extensiones `fun Tipo.metodo(...)` y `toString` en todos los primitivos | Sema y codegen completos; dispatch por tabla central |
| Smart casts: tras `if (x != null)`, `x == null` con return, `&&` y `while`, las variables locales nullable se usan sin `!`; la reasignación invalida el estrechado | Solo sema; el acceso a un local nullable sin estrechar es error |
| `module`, `interface`, `annotation`, `extends`, `implements`, parámetros de tipo `<T>` | Solo parser: la sintaxis es válida pero sema y codegen aún no las procesan |

Solo se documentan aquí las características de la primera tabla. El resto aparece en `GRAMMAR.md` como sintaxis reservada para fases posteriores.

## Tipos primitivos

- `Int`: entero con signo de 64 bits.
- `Long`: entero con signo de 64 bits (alias explícito).
- `Float`: coma flotante de 32 bits.
- `Double`: coma flotante de 64 bits.
- `Bool`: booleano (`true`, `false`).
- `Char`: carácter Unicode.
- `String`: cadena de texto.
- `Void`: ausencia de valor de retorno.

Los nombres de tipo comienzan en mayúscula.

## Declaraciones de nivel superior

### Módulos e importaciones

```
module com.ejemplo.miapp

import std.io
import std.json
import compiler.ast
import lsp.documents
```

`module` declara el nombre lógico del fichero; hoy solo se parsea, sin efecto en sema ni codegen.

`import std.io` registra el módulo builtin `io` (`io.println`, `io.readFile`, ...).

`import std.json` carga `nx/std/json.nx` y expone `json.parse` y `json.stringify`.

Rutas con punto se resuelven a ficheros `.nx`:

- `std.json.value` -> `nx/std/json/value.nx`
- `compiler.ast` -> `compiler/src/ast.nx`

Formas adicionales de import:

- `import compiler.ast as arbol` registra el módulo con el alias `arbol`.
- `import std.core.{math, strings}` equivale a un import por miembro; cada miembro admite su propio `as`.
- `import std.core.*` importa cada fichero `.nx` del directorio en orden alfabético. Si un wildcard aporta un nombre de módulo ya ligado a otra ruta distinta, sema emite error.
- `lsp.features.hover` -> `nx/lsp/features/hover.nx`

Código Nexus fuera del compilador vive en `nx/`: `nx/std/` (stdlib) y `nx/lsp/` (servidor LSP).
El compilador autoalojado permanece en `compiler/src/`.

El último segmento del path es el alias del módulo (`json`, `io`, `ast`, ...).
Las llamadas `alias.funcion(...)` resuelven a funciones de nivel superior del módulo importado.

### Funciones libres

```
suma(a: Int, b: Int): Int {
    return a + b
}

cuadrado(x: Int): Int => x * x
```

La palabra clave `fn` no existe. El cuerpo puede ser un bloque `{ ... }` o una expresión con `=>`.

### Constantes y variables globales

```
final MAX_INTENTOS: Int = 3
var contadorGlobal: Int = 0
```

`final` declara una constante de módulo. `var` declara una variable global mutable.

### Tipos de datos

`data` define un tipo producto inmutable (equivalente a un record):

```
data Punto {
    x: Int
    y: Int
}
```

`value` define un tipo valor con semántica de igualdad estructural:

```
value Color {
    r: Int
    g: Int
    b: Int
}
```

### Clases

`class` define un tipo con estado y comportamiento. Admite un constructor primario en la cabecera:

```
class Circulo(radio: Int) {
    area(): Double => 3.14159 * radio * radio
}
```

Los métodos reciben `this` como receptor implícito; no se declara explícitamente como parámetro.

## Variables locales

```
var x: Int = 42
var nombre = "Nexus"
final pi: Double = 3.14159
```

`var` declara una variable mutable. `val` y `final` declaran constantes locales inmutables.

## Control de flujo

```
if condicion {
    ...
} else {
    ...
}

while condicion {
    ...
}

for elemento in lista {
    ...
}

switch valor {
    case 1 => accion()
    case 2 => otraAccion()
    default => porDefecto()
}

return expresion
break
continue

try {
    ...
} catch (err) {
    ...
}

throw
```

## Expresiones

### Llamadas con argumentos nombrados

```
var p = Punto(x: 3, y: 4)
```

### Acceso a campos y llamadas a métodos

```
p.x
svc.calcular(p)
```

### Operadores de nulabilidad

- `Tipo?`: tipo que admite `null`.
- `?.`: acceso seguro (devuelve `null` si el receptor es `null`).
- `?:`: operador Elvis, valor por defecto cuando la izquierda es `null`.
- `!`: aserción de no nulo.

```
var nombre: String? = null
var longitud = nombre?.length() ?: 0
```

### Lambdas

```
var doble = (x: Int) => x * 2
```

## Tipos integrados

- `List<T>`: lista dinámica con orden de inserción.
- `Map<K, V>`: mapa asociativo.

## Semántica de null

`null` es un valor del tipo `Void` y de cualquier tipo nullable `T?`. Un tipo no nullable no puede contener `null`; el compilador lo rechaza en tiempo de compilación.