mod lexer;
mod ast;
mod parser;
mod sema;
mod codegen_c;
mod driver;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() >= 2 {
        match args[1].as_str() {
            "test-lexer" => {
                let dir = if args.len() >= 3 { &args[2] } else { "tests/lexer" };
                if driver::run_lexer_tests(dir) {
                    println!("Todos los tests de lexer pasaron.");
                } else {
                    eprintln!("Algunos tests de lexer fallaron.");
                    std::process::exit(1);
                }
                return;
            }
            "gen-tokens" => {
                if args.len() < 3 { eprintln!("uso: nxc gen-tokens <archivo.nx>"); std::process::exit(1); }
                if let Err(e) = driver::generate_tokens_file(&args[2]) {
                    eprintln!("{}", e);
                    std::process::exit(1);
                }
                return;
            }
            "test-parser" => {
                let dir = if args.len() >= 3 { &args[2] } else { "tests/parser" };
                if driver::run_parser_tests(dir) {
                    println!("Todos los tests de parser pasaron.");
                } else {
                    eprintln!("Algunos tests de parser fallaron.");
                    std::process::exit(1);
                }
                return;
            }
            "gen-ast" => {
                if args.len() < 3 { eprintln!("uso: nxc gen-ast <archivo.nx>"); std::process::exit(1); }
                if let Err(e) = driver::generate_ast_file(&args[2]) {
                    eprintln!("{}", e);
                    std::process::exit(1);
                }
                return;
            }
            "test-sema" => {
                let dir = if args.len() >= 3 { &args[2] } else { "tests/sema" };
                if driver::run_sema_tests(dir) {
                    println!("Todos los tests de sema pasaron.");
                } else {
                    eprintln!("Algunos tests de sema fallaron.");
                    std::process::exit(1);
                }
                return;
            }
            "compile" => {
                if args.len() < 3 { eprintln!("uso: nxc compile <archivo.nx> [salida]"); std::process::exit(1); }
                let output = if args.len() >= 4 { &args[3] } else { "a.out" };
                if let Err(e) = driver::compile_file(&args[2], output) {
                    eprintln!("{}", e);
                    std::process::exit(1);
                }
                return;
            }
            "test-e2e" => {
                let dir = if args.len() >= 3 { &args[2] } else { "tests/e2e" };
                if driver::run_e2e_tests(dir) {
                    println!("Todos los tests e2e pasaron.");
                } else {
                    eprintln!("Algunos tests e2e fallaron.");
                    std::process::exit(1);
                }
                return;
            }
            "test-codegen" => {
                let dir = if args.len() >= 3 { &args[2] } else { "tests/codegen" };
                if driver::run_codegen_tests(dir) {
                    println!("Todos los tests de codegen pasaron.");
                } else {
                    eprintln!("Algunos tests de codegen fallaron.");
                    std::process::exit(1);
                }
                return;
            }
            "gen-c" => {
                if args.len() < 3 { eprintln!("uso: nxc gen-c <archivo.nx>"); std::process::exit(1); }
                if let Err(e) = driver::generate_c_file(&args[2]) {
                    eprintln!("{}", e);
                    std::process::exit(1);
                }
                return;
            }
            _ => {}
        }
    }
    println!("Nexus Stage 0 Bootstrap Compiler");
}
