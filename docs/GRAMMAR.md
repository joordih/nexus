# Gramática de Nexus Core

Este documento es la gramática autoritativa de Nexus Core: la sintaxis completa del subconjunto con el que se escribe el compilador autoalojado. El auto-hospedaje (Fases 0–6) está cerrado; la gramática sigue siendo la referencia del lexer y el parser en `compiler/src/`.

No todo lo que la gramática acepta está implementado en sema y codegen. La tabla de cobertura del compilador está en `SPEC.md`. Las ampliaciones del lenguaje previstas (genéricos, borrow checker, concurrencia, async) están en `PLAN.md`.

## Convenciones de notación

La gramática usa una variante de EBNF.

- `=` define una regla.
- `|` separa alternativas.
- `{ X }` indica cero o más repeticiones de `X`.
- `[ X ]` indica que `X` es opcional.
- `"texto"` es un terminal literal.
- Los nombres en MAYÚSCULAS son tokens léxicos producidos por el lexer.
- `EOF` es el final de la entrada.

## Estructura léxica

### Espacios

Espacio, tabulador, retorno de carro y salto de línea son descartables. Los bloques se delimitan con llaves; la indentación no tiene valor sintáctico.

### Identificadores y palabras clave

```
letter = "a" .. "z" | "A" .. "Z" | "_" ;
digit  = "0" .. "9" ;
IDENT  = letter { letter | digit } ;
```

Palabras reservadas:

```
var final class data value interface annotation
import module extends implements
if else while for in switch case default
return break continue try catch throw
true false null this
```

Los nombres de tipos primitivos (`Int`, `Bool`, `String`, `Void`, etc.) son identificadores predeclarados, no palabras clave.

### Literales

```
INT    = digit { digit } ;
FLOAT  = digit { digit } "." digit { digit } ;
STRING = '"' { string_char } '"' ;
CHAR   = "'" ( char_char | escape ) "'" ;

string_char = caracter_excepto_comilla_y_barra | escape ;
char_char   = caracter_excepto_comilla_simple_y_barra ;
escape      = "\" ( '"' | "'" | "\" | "n" | "t" | "r" | "0" ) ;
```

### Operadores y signos de puntuación

```
+   -   *   /   %
==  !=  <   <=  >   >=
&&  ||  !
=   =>  ->
.   ,   :   ;
(   )   {   }   [   ]
@   ?   ?.  ?:
```

El lexer aplica munch máximo. `?.` y `?:` se reconocen antes que `?` solo.

## Gramática sintáctica

### Programa y elementos

```
program = { item } EOF ;

item    = module_decl
        | import_decl
        | global_decl
        | class_decl
        | data_decl
        | value_decl
        | interface_decl
        | annotation_decl
        | function ;
```

### Módulo e importaciones

```
module_decl = "module" dot_path ;
import_decl = "import" dot_path ;
dot_path    = IDENT { "." IDENT } ;
```

### Constantes y variables globales

```
global_decl = ( "var" | "final" ) IDENT [ ":" type ] "=" expr [ ";" ] ;
```

### Anotaciones aplicadas

```
annotation_use = "@" IDENT [ "(" named_arg_list ")" ] ;
```

### Declaración de datos

```
data_decl  = { annotation_use } "data" IDENT "{" { field_decl } "}" ;
value_decl = { annotation_use } "value" IDENT "{" { field_decl } "}" ;
field_decl = IDENT ":" type ;
```

Los campos se separan por saltos de línea o punto y coma opcionales.

### Declaración de clase

```
class_decl     = { annotation_use } "class" IDENT [ type_params ]
                 [ "(" [ field_list ] ")" ]
                 [ "extends" type ]
                 [ "implements" type { "," type } ]
                 "{" { class_member } "}" ;

type_params    = "<" IDENT { "," IDENT } ">" ;
field_list     = field_decl { "," field_decl } ;
class_member   = { annotation_use } function ;
```

`type_params`, `extends` e `implements` son sintaxis válida; sema y codegen aún no los procesan.

### Declaración de interfaz

```
interface_decl = { annotation_use } "interface" IDENT [ type_params ]
                 [ "extends" type { "," type } ]
                 "{" { method_sig } "}" ;

method_sig     = IDENT "(" [ param_list ] ")" ":" type ;
```

Reconocida por el parser; sin sema ni codegen todavía.

### Declaración de anotación

```
annotation_decl = "annotation" IDENT "{" { field_decl } "}" ;
```

Reconocida por el parser; sin sema ni codegen todavía.

### Funciones

```
function   = IDENT "(" [ param_list ] ")" ":" type function_body ;
param_list = param { "," param } ;
param      = IDENT ":" type ;
function_body = block | "=>" expr ;
```

No existe palabra clave `fn`. La función se identifica porque su nombre va seguido de `(`.

### Tipos

```
type       = named_type [ "?" ] ;
named_type = dot_path [ "<" type_args ">" ] ;
type_args  = type { "," type } ;
```

Un tipo seguido de `?` es nullable: `String?`, `Punto?`.

### Bloques y sentencias

```
block       = "{" { stmt } "}" ;

stmt        = var_stmt
            | return_stmt
            | break_stmt
            | continue_stmt
            | if_stmt
            | while_stmt
            | for_stmt
            | switch_stmt
            | try_stmt
            | throw_stmt
            | expr_stmt ;

var_stmt    = ( "var" | "final" ) IDENT [ ":" type ] "=" expr [ ";" ] ;
return_stmt = "return" [ expr ] [ ";" ] ;
break_stmt  = "break" [ ";" ] ;
continue_stmt = "continue" [ ";" ] ;
if_stmt     = "if" expr block [ "else" ( if_stmt | block ) ] ;
while_stmt  = "while" expr block ;
for_stmt    = "for" IDENT "in" expr block ;
switch_stmt = "switch" expr "{" { switch_arm } "}" ;
switch_arm  = ( "case" expr | "default" ) "=>" ( block | expr [ ";" ] ) ;
try_stmt    = "try" block "catch" "(" IDENT ")" block ;
throw_stmt  = "throw" [ ";" ] ;
expr_stmt   = expr [ ";" ] ;
```

Los puntos y coma son opcionales. La ausencia de `;` al final de una línea es válida.

### Expresiones

```
expr         = elvis_expr ;
elvis_expr   = or_expr [ "?:" or_expr ] ;
or_expr      = and_expr { "||" and_expr } ;
and_expr     = equality { "&&" equality } ;
equality     = comparison { ( "==" | "!=" ) comparison } ;
comparison   = term { ( "<" | "<=" | ">" | ">=" ) term } ;
term         = factor { ( "+" | "-" ) factor } ;
factor       = unary { ( "*" | "/" | "%" ) unary } ;
unary        = ( "!" | "-" ) unary | postfix ;
postfix      = primary { postfix_op } ;
postfix_op   = "." IDENT [ call_args ]
             | "?." IDENT [ call_args ]
             | call_args
             | "[" expr "]"
             | "!" ;
call_args    = "(" [ arg_list ] ")" ;
arg_list     = call_arg { "," call_arg } ;
call_arg     = IDENT ":" expr | expr ;
```

Un sufijo `"." IDENT` seguido de `call_args` es una llamada a método; sin `call_args` es acceso a campo. `"?." IDENT` es acceso seguro (devuelve `null` si el receptor es `null`). Un `"!"` postfijo es aserción de no nulo.

Los argumentos nombrados (`IDENT ":" expr`) y posicionales pueden mezclarse en una llamada.

```
primary     = INT
            | FLOAT
            | STRING
            | CHAR
            | "true"
            | "false"
            | "null"
            | "this"
            | lambda_expr
            | list_literal
            | IDENT
            | "(" expr ")" ;
```

Un `IDENT` en posición primaria es siempre un identificador simple. Las cadenas `a.b.c` se construyen en la fase de postfix como accesos a campo consecutivos.

### Lambda

```
lambda_expr = lambda_params "=>" expr ;
lambda_params = IDENT
              | "(" [ IDENT { "," IDENT } ] ")" ;
```

### Literal de lista

```
list_literal = "[" [ arg_list_expr ] "]" ;
arg_list_expr = expr { "," expr } ;
```

## Precedencia y asociatividad

De menor a mayor ligadura:

```
1   ?:                      Elvis, asociativa por la derecha
2   ||                      disyunción, asociativa por la izquierda
3   &&                      conjunción, asociativa por la izquierda
4   ==  !=                  igualdad, asociativa por la izquierda
5   <   <=  >   >=          relacional, asociativa por la izquierda
6   +   -                   aditiva, asociativa por la izquierda
7   *   /   %               multiplicativa, asociativa por la izquierda
8   !   -                   prefija unaria
9   ()  .x  ?.x  []  !      sufija, máxima ligadura
```

## Fuera del alcance actual

Sintaxis o características no incluidas en Core hoy:

- Asignación compuesta (`+=`, `-=`, `*=`, `/=`).
- Comprensiones de lista.
- Macros.

Ampliaciones previstas del lenguaje (genéricos con monomorfización, borrow checker, goroutines/canales, async/await, backend LLVM): ver `PLAN.md`.