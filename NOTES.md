# Bitácora de Nexus

## Fase 0 - Fundaciones
Creada la estructura de directorios completa. Instalados Rust 1.75, Clang 18, libgc-dev y Make.
El runtime en C compila y enlaza correctamente con libgc. El binario de prueba asigna memoria gestionada sin fallos.
Puerta de salida: verde.

## Fase 1 - Lexer
Implementado el lexer completo en bootstrap/src/lexer.rs. Reconoce todos los tokens de Nexus Core: identificadores, palabras clave, literales enteros, cadenas, caracteres, operadores con munch maximo, y signos de puntuacion. Los casos de error producen diagnosticos con linea y columna.
Puerta de salida: make test pasa la suite de lexer con todos los archivos de tests/lexer/.

## Fase 2 - Parser y AST
Implementado el AST completo en bootstrap/src/ast.rs y el parser de descenso recursivo con Pratt para expresiones en bootstrap/src/parser.rs. Soporta todos los constructos de Core: use, struct, enum, impl, fn, sentencias, expresiones, match y literales de struct.
Puerta de salida: make test pasa la suite de parser con todos los archivos de tests/parser/.

## Fase 3 - Analisis semantico
Implementado el analizador semantico en bootstrap/src/sema.rs con tabla de simbolos, resolucion de tipos, comprobacion de tipos en expresiones y sentencias, y verificacion de retorno. Tratamiento especial de List y Map integrados.
Puerta de salida: make test pasa la suite de sema, incluyendo programas invalidos que son rechazados correctamente.

## Fase 4 - Backend a C y runtime
Implementado el generador de codigo C en bootstrap/src/codegen_c.rs. El runtime en C es completo con List, Map, formateo de cadenas e io::println. El driver orquesta lexer, parser, sema, codegen e invocacion de clang con enlazado de libgc.
El programa de referencia compila a binario y produce la salida esperada: 25 y 75.
Puerta de salida: make test pasa todas las suites (lexer, parser, sema, codegen, e2e). make example NAME=hello compila y ejecuta correctamente.

## Decisiones tomadas
- El codegen ordena los campos de struct y variantes de enum alfabeticamente para garantizar determinismo en la salida.
- Los constructores de variantes de enum se generan como funciones estaticas con nombre nx_mk_EnumName_VariantName.
- Los metodos de impl se generan como funciones con nombre nx_method_TypeName_methodName.
- El main() de C se genera automaticamente cuando el programa Nexus tiene una funcion main().
- La inferencia de tipos en el codegen usa una tabla de variables locales (var_types) para rastrear los tipos correctamente.
