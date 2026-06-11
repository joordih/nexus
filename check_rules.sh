#!/bin/sh
set -e

FAIL=0

check_comments() {
    find bootstrap/src compiler/src runtime -name "*.rs" -o -name "*.nx" -o -name "*.c" -o -name "*.h" 2>/dev/null | while read f; do
        if grep -nP '(?<!:)\/\/' "$f" | grep -vP '^\s*#' | grep -q .; then
            echo "FAIL comentario de linea en: $f"
            FAIL=1
        fi
        if grep -nP '/\*' "$f" | grep -vP '^\s*#' | grep -q .; then
            echo "FAIL comentario de bloque en: $f"
            FAIL=1
        fi
    done
}

check_emojis() {
    find . -name "*.rs" -o -name "*.nx" -o -name "*.c" -o -name "*.h" -o -name "*.md" 2>/dev/null | grep -v "^./build" | while read f; do
        if grep -qP '[\x{1F300}-\x{1F9FF}]' "$f" 2>/dev/null; then
            echo "FAIL emoji en: $f"
            FAIL=1
        fi
    done
}

check_comments
check_emojis

if [ "$FAIL" = "0" ]; then
    echo "Verificacion de reglas OK"
else
    echo "Verificacion de reglas FALLO"
    exit 1
fi
