.PHONY: all bootstrap stage1 stage2 stage3 verify-bootstrap test clean example
.PHONY: test-lexer test-parser test-sema test-codegen test-e2e test-json test-std test-runtime test-lsp
.PHONY: nexus-lsp stdlib vscode-nexus intellij-nexus build-all example-stdlib check-examples install uninstall

ifeq ($(OS),Windows_NT)
    EXE_EXT         := .exe
    NXC_INSTALL_DIR ?= $(USERPROFILE)/bin
    GRADLEW         := gradlew.bat
else
    EXE_EXT         :=
    NXC_INSTALL_DIR ?= /usr/local/bin
    GRADLEW         := ./gradlew
endif

NXC = build/nxc-stage0$(EXE_EXT)
CC ?= clang
GC_INCLUDE ?=
GC_LIB ?=
export GC_INCLUDE
export GC_LIB

all: bootstrap

bootstrap:
	cd bootstrap && cargo build --release
	python -c "import os; os.makedirs('build', exist_ok=True)"
	python -c "import shutil; shutil.copy('bootstrap/target/release/nxc-stage0$(EXE_EXT)', 'build/nxc-stage0$(EXE_EXT)')"

stage1: bootstrap
	@echo "Construyendo stage1..."
	$(NXC) compile-dir compiler/src build/nxc-stage1

stage2: stage1
	@echo "Construyendo stage2..."
	build/nxc-stage1 compile-dir compiler/src build/nxc-stage2

stage3: stage2
	@echo "Construyendo stage3..."
	build/nxc-stage2$(EXE_EXT) compile-dir compiler/src build/nxc-stage3

test-stage1: stage1
	build/nxc-stage1 test-lexer tests/lexer
	build/nxc-stage1 test-parser tests/parser
	build/nxc-stage1 test-sema tests/sema
	build/nxc-stage1 test-e2e tests/e2e
	@echo "Todos los tests de stage1 pasaron."

verify-bootstrap: stage2 stage3
	python -c "import sys,os; p=lambda n:n+'.exe' if os.path.exists(n+'.exe') else n; a=open(p('build/nxc-stage2'),'rb').read(); b=open(p('build/nxc-stage3'),'rb').read(); sys.exit(0 if a==b else 1)"
	@echo "Bootstrap verificado."

test: bootstrap test-runtime test-lexer test-parser test-sema test-codegen test-e2e test-json test-std test-lsp
	@echo "Todas las suites de pruebas pasaron."

test-runtime:
	python -c "import os; os.makedirs('build', exist_ok=True)"
	"$(CC)" -I runtime $(if $(GC_INCLUDE),-I $(GC_INCLUDE)) -o build/test_runtime runtime/nexus_runtime.c runtime/test_runtime.c $(if $(GC_LIB),-L $(GC_LIB)) -lgc -Wno-deprecated-declarations
	build/test_runtime

test-lexer: bootstrap
	$(NXC) test-lexer tests/lexer

test-parser: bootstrap
	$(NXC) test-parser tests/parser

test-sema: bootstrap
	$(NXC) test-sema tests/sema

test-codegen: bootstrap
	$(NXC) test-codegen tests/codegen

test-e2e: bootstrap
	$(NXC) test-e2e tests/e2e

test-json: stage1
	build/nxc-stage1 test-json tests/json

test-std: stage2
	build/nxc-stage2$(EXE_EXT) test-std tests/std

stdlib: test-std

nexus-lsp: stage2
	build/nxc-stage2$(EXE_EXT) compile-lsp build/nexus-lsp

vscode-nexus: nexus-lsp
	python -c "import os; os.makedirs('vscode-nexus/bin', exist_ok=True)"
	python -c "import shutil; shutil.copy('build/nexus-lsp$(EXE_EXT)', 'vscode-nexus/bin/nexus-lsp$(EXE_EXT)')"
	cd vscode-nexus && npm run package

intellij-nexus:
	cd editors/intellij && $(GRADLEW) buildPlugin verifyPlugin

build-all: verify-bootstrap vscode-nexus test

test-lsp: nexus-lsp
	python tests/lsp/run_lsp_test.py
	python tests/lsp/test_dir_diagnostics.py
	python tests/lsp/test_import_completion.py

example: bootstrap
	$(NXC) compile examples/$(NAME).nx build/$(NAME)
	build/$(NAME)$(EXE_EXT)

example-stdlib: stage2
	build/nxc-stage2$(EXE_EXT) compile examples/stdlib_showcase.nx build/stdlib_showcase
	build/stdlib_showcase$(EXE_EXT)

check-examples: stage2
	python -c "import os; os.makedirs('examples/build', exist_ok=True)"
	build/nxc-stage2$(EXE_EXT) compile examples/hello.nx examples/build/hello
	build/nxc-stage2$(EXE_EXT) compile examples/stdlib_showcase.nx examples/build/stdlib_showcase

install: stage2
	python -c "import shutil,os; d=r'$(NXC_INSTALL_DIR)'; dst=os.path.join(d,'nxc$(EXE_EXT)'); os.makedirs(d,exist_ok=True); shutil.copy2('build/nxc-stage2$(EXE_EXT)',dst); print('Instalado:',dst)"

uninstall:
	python -c "import os; p=os.path.join(r'$(NXC_INSTALL_DIR)','nxc$(EXE_EXT)'); os.remove(p); print('Desinstalado:',p)"

clean:
	cd bootstrap && cargo clean
	python -c "import shutil; shutil.rmtree('build', ignore_errors=True)"
