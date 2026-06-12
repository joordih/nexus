.PHONY: all bootstrap stage1 stage2 stage3 verify-bootstrap test clean example
.PHONY: test-lexer test-parser test-sema test-codegen test-e2e test-runtime

NXC = build\nxc-stage0.exe
CC  ?= clang
GC_INCLUDE ?=
GC_LIB     ?=

all: bootstrap

bootstrap:
	cd bootstrap && cargo build --release
	mkdir -p build
	cp bootstrap/target/release/nxc-stage0.exe build/nxc-stage0.exe

stage1: bootstrap
	@echo "Construyendo stage1..."

stage2: stage1
	@echo "Construyendo stage2..."

stage3: stage2
	@echo "Construyendo stage3..."

verify-bootstrap: stage2 stage3
	cmp build/nxc-stage2 build/nxc-stage3
	@echo "Bootstrap verificado."

test: bootstrap test-runtime test-lexer test-parser test-sema test-codegen test-e2e
	@echo "Todas las suites de pruebas pasaron."

test-runtime:
	mkdir -p build
	$(CC) -I runtime $(if $(GC_INCLUDE),-I $(GC_INCLUDE)) -o build/test_runtime runtime/nexus_runtime.c runtime/test_runtime.c $(if $(GC_LIB),-L $(GC_LIB)) -lgc -Wno-deprecated-declarations
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

example: bootstrap
	$(NXC) compile examples/$(NAME).nx build/$(NAME)
	build/$(NAME)

clean:
	cd bootstrap && cargo clean
	rm -rf build
