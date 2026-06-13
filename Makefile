.PHONY: all bootstrap stage1 stage2 stage3 verify-bootstrap test clean example
.PHONY: test-lexer test-parser test-sema test-codegen test-e2e test-json test-std test-runtime test-lsp
.PHONY: nexus-lsp stdlib vscode-nexus build-all example-stdlib check-examples

NXC = build/nxc-stage0.exe
CC ?= clang
GC_INCLUDE ?=
GC_LIB ?=
export GC_INCLUDE
export GC_LIB

all: bootstrap

bootstrap:
	cd bootstrap && cargo build --release
	-if not exist build mkdir build
	copy /Y bootstrap\target\release\nxc-stage0.exe build\nxc-stage0.exe

stage1: bootstrap
	@echo "Construyendo stage1..."
	-if not exist build mkdir build
	$(NXC) compile-dir compiler/src build/nxc-stage1

stage2: stage1
	@echo "Construyendo stage2..."
	build/nxc-stage1 compile-dir compiler/src build/nxc-stage2

stage3: stage2
	@echo "Construyendo stage3..."
	build/nxc-stage2 compile-dir compiler/src build/nxc-stage3

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
	-if not exist build mkdir build
	"$(CC)" -I runtime $(if $(GC_INCLUDE),-I $(GC_INCLUDE)) -o build/test_runtime runtime/nexus_runtime.c runtime/test_runtime.c $(if $(GC_LIB),-L $(GC_LIB)) -lgc -Wno-deprecated-declarations
	./build/test_runtime

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
	build/nxc-stage2 test-std tests/std

stdlib: test-std

nexus-lsp: stage2
	build/nxc-stage2 compile-lsp build/nexus-lsp

vscode-nexus: nexus-lsp
	-if not exist vscode-nexus\bin mkdir vscode-nexus\bin
	copy /Y build\nexus-lsp.exe vscode-nexus\bin\nexus-lsp.exe
	cd vscode-nexus && npm run package

build-all: verify-bootstrap vscode-nexus test

test-lsp: nexus-lsp
	python tests/lsp/run_lsp_test.py
	python tests/lsp/test_dir_diagnostics.py
	python tests/lsp/test_import_completion.py

example: bootstrap
	$(NXC) compile examples/$(NAME).nx build/$(NAME)
	build/$(NAME)

example-stdlib: stage2
	build/nxc-stage2 compile examples/stdlib_showcase.nx build/stdlib_showcase
	build/stdlib_showcase

check-examples: stage2
	-if not exist examples\build mkdir examples\build
	build/nxc-stage2 compile examples/hello.nx examples/build/hello
	build/nxc-stage2 compile examples/stdlib_showcase.nx examples/build/stdlib_showcase

clean:
	cd bootstrap && cargo clean
	rm -rf build
