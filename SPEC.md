# Especificación de Nexus Core

Este documento describe el subconjunto de arranque, Nexus Core, tal como lo implementa el compilador de Etapa 0.

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
import std.collections.List
```

### Funciones libres

```
suma(a: Int, b: Int): Int {
    return a + b
}

cuadrado(x: Int): Int => x * x
```

La palabra clave `fn` no existe. El cuerpo puede ser un bloque `{ ... }` o una expresión con `=>`.

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

### Interfaces

```
interface Figura {
    area(): Double
    perimetro(): Double
}
```

### Anotaciones

```
annotation Deprecated {
    razon: String
}
```

## Variables locales

```
var x: Int = 42
var nombre = "Nexus"    // inferencia de tipo
final pi: Double = 3.14159
```

`var` declara una variable mutable. `final` declara una constante local.

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
var longitud = nombre?.length ?: 0
```

### Lambdas

```
var doble = (x: Int) => x * 2
var activos = usuarios.filter(u => u.activo)
```

## Tipos integrados

- `List<T>`: lista dinámica con orden de inserción.
- `Map<K, V>`: mapa asociativo.

## Semántica de null

`null` es un valor del tipo `Void` y de cualquier tipo nullable `T?`. Un tipo no nullable no puede contener `null`; el compilador lo rechaza en tiempo de compilación.
