//! CodSpeed benchmarks for the Nexus Stage 0 bootstrap compiler.
//!
//! These benchmarks exercise the main stages of the compilation pipeline
//! (lexing, parsing, semantic analysis and C code generation) over both a
//! real example program and synthetic programs of growing size.

use nxc_stage0::codegen_c::generate_c;
use nxc_stage0::lexer::Lexer;
use nxc_stage0::parser::Parser;
use nxc_stage0::sema::check_program;

fn main() {
    divan::main();
}

/// Number of generated functions used to scale the synthetic workloads.
const SIZES: &[usize] = &[16, 64, 256];

const EXAMPLE: &str = include_str!("fixtures/stage0_example.nx");

/// Build a syntactically and semantically valid Nexus program with a
/// configurable number of top-level functions, so the pipeline can be
/// measured at different input sizes.
fn make_source(num_funcs: usize) -> String {
    let mut s = String::new();
    s.push_str("data Punto {\n    x: Int\n    y: Int\n}\n\n");
    for i in 0..num_funcs {
        s.push_str(&format!(
            "compute{i}(x: Int): Int {{\n    return x * x + {i}\n}}\n\n"
        ));
    }
    s.push_str("main(): Void {\n}\n");
    s
}

fn tokenize(source: &str) -> Vec<nxc_stage0::lexer::Token> {
    Lexer::new(source)
        .tokenize()
        .unwrap_or_else(|_| panic!("source should tokenize"))
}

fn parse(source: &str) -> nxc_stage0::ast::Program {
    let tokens = tokenize(source);
    Parser::new(tokens)
        .parse_program()
        .unwrap_or_else(|_| panic!("source should parse"))
}

// --- Lexing -----------------------------------------------------------------

#[divan::bench]
fn lex_example(bencher: divan::Bencher) {
    bencher
        .with_inputs(|| EXAMPLE)
        .bench_values(|source| divan::black_box(tokenize(source)));
}

#[divan::bench(args = SIZES)]
fn lex(bencher: divan::Bencher, size: usize) {
    let source = make_source(size);
    bencher
        .with_inputs(|| source.clone())
        .bench_values(|source| divan::black_box(tokenize(&source)));
}

// --- Parsing ----------------------------------------------------------------

#[divan::bench]
fn parse_example(bencher: divan::Bencher) {
    let tokens = tokenize(EXAMPLE);
    bencher
        .with_inputs(|| tokens.clone())
        .bench_values(|tokens| divan::black_box(Parser::new(tokens).parse_program()));
}

#[divan::bench(args = SIZES)]
fn parse_bench(bencher: divan::Bencher, size: usize) {
    let tokens = tokenize(&make_source(size));
    bencher
        .with_inputs(|| tokens.clone())
        .bench_values(|tokens| divan::black_box(Parser::new(tokens).parse_program()));
}

// --- Semantic analysis ------------------------------------------------------

#[divan::bench(args = SIZES)]
fn sema(bencher: divan::Bencher, size: usize) {
    let program = parse(&make_source(size));
    bencher.bench(|| divan::black_box(check_program(divan::black_box(&program))));
}

// --- Code generation --------------------------------------------------------

#[divan::bench(args = SIZES)]
fn codegen(bencher: divan::Bencher, size: usize) {
    let program = parse(&make_source(size));
    bencher
        .with_inputs(|| {
            check_program(&program).unwrap_or_else(|_| panic!("program should pass sema"))
        })
        .bench_values(|ctx| divan::black_box(generate_c(&program, ctx)));
}

// --- Full pipeline ----------------------------------------------------------

#[divan::bench(args = SIZES)]
fn full_pipeline(bencher: divan::Bencher, size: usize) {
    let source = make_source(size);
    bencher
        .with_inputs(|| source.clone())
        .bench_values(|source| {
            let tokens = Lexer::new(&source)
                .tokenize()
                .unwrap_or_else(|_| panic!("tokenize"));
            let program = Parser::new(tokens)
                .parse_program()
                .unwrap_or_else(|_| panic!("parse"));
            let ctx = check_program(&program).unwrap_or_else(|_| panic!("sema"));
            divan::black_box(generate_c(&program, ctx))
        });
}
