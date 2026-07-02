use std::fs;
use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::ast::*;

pub fn run_lex_file(path: &str) -> Result<String, String> {
    let source = fs::read_to_string(path)
        .map_err(|e| format!("no se pudo leer {}: {}", path, e))?;
    let mut lexer = Lexer::new(&source);
    let tokens = lexer.tokenize().map_err(|e| e.to_string())?;
    let mut out = String::new();
    for tok in &tokens {
        out.push_str(&format!("{}\n", tok.kind));
    }
    Ok(out)
}

pub fn generate_tokens_file(nx_path: &str) -> Result<(), String> {
    let output = run_lex_file(nx_path)?;
    let tokens_path = nx_path.replace(".nx", ".tokens");
    fs::write(&tokens_path, &output)
        .map_err(|e| format!("no se pudo escribir {}: {}", tokens_path, e))?;
    println!("generado: {}", tokens_path);
    Ok(())
}

pub fn run_lexer_tests(test_dir: &str) -> bool {
    let entries = match fs::read_dir(test_dir) {
        Ok(e) => e,
        Err(_) => {
            eprintln!("no se pudo abrir el directorio de tests: {}", test_dir);
            return false;
        }
    };
    let mut all_pass = true;
    let mut nx_files: Vec<_> = entries
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map(|x| x == "nx").unwrap_or(false))
        .collect();
    nx_files.sort_by_key(|e| e.path());
    for entry in nx_files {
        let nx_path = entry.path();
        let tokens_path = nx_path.with_extension("tokens");
        if !tokens_path.exists() {
            eprintln!("falta archivo esperado: {}", tokens_path.display());
            all_pass = false;
            continue;
        }
        let expected = match fs::read_to_string(&tokens_path) {
            Ok(s) => s,
            Err(e) => { eprintln!("error leyendo {}: {}", tokens_path.display(), e); all_pass = false; continue; }
        };
        match run_lex_file(nx_path.to_str().unwrap()) {
            Ok(got) => {
                if got == expected {
                    println!("PASS lexer: {}", nx_path.display());
                } else {
                    eprintln!("FAIL lexer: {}", nx_path.display());
                    eprintln!("esperado:\n{}", expected);
                    eprintln!("obtenido:\n{}", got);
                    all_pass = false;
                }
            }
            Err(e) => {
                eprintln!("FAIL lexer: {} => {}", nx_path.display(), e);
                all_pass = false;
            }
        }
    }
    all_pass
}

pub fn parse_file(path: &str) -> Result<Program, String> {
    let source = fs::read_to_string(path)
        .map_err(|e| format!("no se pudo leer {}: {}", path, e))?;
    let mut lexer = Lexer::new(&source);
    let tokens = lexer.tokenize().map_err(|e| e.to_string())?;
    let mut parser = Parser::new(tokens);
    parser.parse_program().map_err(|errs| {
        errs.iter().map(|e| e.to_string()).collect::<Vec<_>>().join("\n")
    })
}

pub fn dump_ast(program: &Program) -> String {
    let mut out = String::new();
    dump_program(program, &mut out);
    out
}

fn dump_program(program: &Program, out: &mut String) {
    out.push_str("Program\n");
    for item in &program.items {
        dump_item(item, out, 1);
    }
}

fn indent(depth: usize) -> String {
    "  ".repeat(depth)
}

fn dump_item(item: &Item, out: &mut String, depth: usize) {
    match item {
        Item::ModuleDecl(m) => {
            out.push_str(&format!("{}ModuleDecl({})\n", indent(depth), m.path.join(".")));
        }
        Item::ImportDecl(i) => {
            out.push_str(&format!("{}ImportDecl({})\n", indent(depth), i.path.join(".")));
        }
        Item::DataDecl(d) => {
            out.push_str(&format!("{}DataDecl({})\n", indent(depth), d.name));
            for f in &d.fields {
                out.push_str(&format!("{}  Field({}: {})\n", indent(depth), f.name, dump_type(&f.ty)));
            }
        }
        Item::ValueDecl(v) => {
            out.push_str(&format!("{}ValueDecl({})\n", indent(depth), v.name));
            for f in &v.fields {
                out.push_str(&format!("{}  Field({}: {})\n", indent(depth), f.name, dump_type(&f.ty)));
            }
        }
        Item::ClassDecl(c) => {
            let ann_str: Vec<_> = c.annotations.iter().map(|a| format!("@{}", a.name)).collect();
            let ann_prefix = if ann_str.is_empty() { String::new() } else { format!("{} ", ann_str.join(" ")) };
            out.push_str(&format!("{}{}ClassDecl({})\n", indent(depth), ann_prefix, c.name));
            for p in &c.primary_params {
                out.push_str(&format!("{}  PrimaryParam({}: {})\n", indent(depth), p.name, dump_type(&p.ty)));
            }
            for m in &c.members {
                match m {
                    ClassMember::Field(f) => {
                        out.push_str(&format!("{}  Field({}: {})\n", indent(depth), f.name, dump_type(&f.ty)));
                    }
                    ClassMember::Method(f) => {
                        dump_function(f, out, depth + 1);
                    }
                }
            }
        }
        Item::InterfaceDecl(i) => {
            out.push_str(&format!("{}InterfaceDecl({})\n", indent(depth), i.name));
            for m in &i.members {
                match m {
                    InterfaceMember::AbstractMethod(s) => {
                        let ret_str = s.return_type.as_ref().map(|t| format!(": {}", dump_type(t))).unwrap_or_default();
                        let params: Vec<_> = s.params.iter().map(|p| format!("{}: {}", p.name, dump_type(&p.ty))).collect();
                        out.push_str(&format!("{}  Abstract {}({}){}\n", indent(depth), s.name, params.join(", "), ret_str));
                    }
                    InterfaceMember::DefaultMethod(f) => {
                        dump_function(f, out, depth + 1);
                    }
                }
            }
        }
        Item::AnnotationDecl(a) => {
            out.push_str(&format!("{}AnnotationDecl({})\n", indent(depth), a.name));
            for f in &a.fields {
                out.push_str(&format!("{}  Field({}: {})\n", indent(depth), f.name, dump_type(&f.ty)));
            }
        }
        Item::Function(f) => {
            dump_function(f, out, depth);
        }
        Item::GlobalConst(g) => {
            let kw = if g.is_final { "final" } else { "var" };
            let ty_str = g.ty.as_ref().map(|t| format!(": {}", dump_type(t))).unwrap_or_default();
            out.push_str(&format!("{}GlobalConst({} {}{})\n", indent(depth), kw, g.name, ty_str));
        }
    }
}

fn dump_type(ty: &Type) -> String {
    match ty {
        Type::Named { path, args } => {
            if args.is_empty() {
                path.join(".")
            } else {
                let args_str: Vec<_> = args.iter().map(dump_type).collect();
                format!("{}<{}>", path.join("."), args_str.join(", "))
            }
        }
        Type::Nullable(inner) => format!("{}?", dump_type(inner)),
    }
}

fn dump_function(f: &Function, out: &mut String, depth: usize) {
    let ann_str: Vec<_> = f.annotations.iter().map(|a| format!("@{}", a.name)).collect();
    let ann_prefix = if ann_str.is_empty() { String::new() } else { format!("{} ", ann_str.join(" ")) };
    let ret_str = f.return_type.as_ref().map(|t| format!(": {}", dump_type(t))).unwrap_or_default();
    let params: Vec<_> = f.params.iter().map(|p| format!("{}: {}", p.name, dump_type(&p.ty))).collect();
    out.push_str(&format!("{}{}fn {}({}){}\n", indent(depth), ann_prefix, f.name, params.join(", "), ret_str));
    match &f.body {
        FunctionBody::Block(b) => dump_block(b, out, depth + 1),
        FunctionBody::Expr(e) => {
            out.push_str(&format!("{}ExprBody\n", indent(depth + 1)));
            dump_expr(e, out, depth + 2);
        }
    }
}

fn dump_block(block: &Block, out: &mut String, depth: usize) {
    out.push_str(&format!("{}Block\n", indent(depth)));
    for stmt in &block.stmts {
        dump_stmt(stmt, out, depth + 1);
    }
}

fn dump_stmt(stmt: &Stmt, out: &mut String, depth: usize) {
    match stmt {
        Stmt::Var(v) => {
            let kw = if v.is_final { "final" } else { "var" };
            let ty_str = v.ty.as_ref().map(|t| format!(": {}", dump_type(t))).unwrap_or_default();
            out.push_str(&format!("{}Var({} {}{})\n", indent(depth), kw, v.name, ty_str));
            dump_expr(&v.value, out, depth + 1);
        }
        Stmt::Return(r) => {
            out.push_str(&format!("{}Return\n", indent(depth)));
            if let Some(v) = &r.value {
                dump_expr(v, out, depth + 1);
            }
        }
        Stmt::Break => { out.push_str(&format!("{}Break\n", indent(depth))); }
        Stmt::Continue => { out.push_str(&format!("{}Continue\n", indent(depth))); }
        Stmt::If(i) => {
            out.push_str(&format!("{}If\n", indent(depth)));
            dump_expr(&i.cond, out, depth + 1);
            dump_block(&i.then_block, out, depth + 1);
            if let Some(else_b) = &i.else_branch {
                match else_b {
                    ElseBranch::Block(b) => {
                        out.push_str(&format!("{}Else\n", indent(depth)));
                        dump_block(b, out, depth + 1);
                    }
                    ElseBranch::If(i2) => {
                        out.push_str(&format!("{}ElseIf\n", indent(depth)));
                        dump_stmt(&Stmt::If(*i2.clone()), out, depth + 1);
                    }
                }
            }
        }
        Stmt::While(w) => {
            out.push_str(&format!("{}While\n", indent(depth)));
            dump_expr(&w.cond, out, depth + 1);
            dump_block(&w.body, out, depth + 1);
        }
        Stmt::For(f) => {
            out.push_str(&format!("{}For({})\n", indent(depth), f.var));
            dump_expr(&f.iterable, out, depth + 1);
            dump_block(&f.body, out, depth + 1);
        }
        Stmt::Switch(s) => {
            dump_switch(s, out, depth);
        }
        Stmt::Expr(e) => {
            if let Some(assign) = &e.assign {
                out.push_str(&format!("{}Assign\n", indent(depth)));
                dump_expr(&e.expr, out, depth + 1);
                dump_expr(assign, out, depth + 1);
            } else {
                out.push_str(&format!("{}ExprStmt\n", indent(depth)));
                dump_expr(&e.expr, out, depth + 1);
            }
        }
    }
}

fn dump_switch(s: &SwitchStmt, out: &mut String, depth: usize) {
    out.push_str(&format!("{}Switch\n", indent(depth)));
    dump_expr(&s.subject, out, depth + 1);
    for arm in &s.arms {
        match &arm.pattern {
            SwitchPattern::Case(e) => {
                out.push_str(&format!("{}Case\n", indent(depth + 1)));
                dump_expr(e, out, depth + 2);
            }
            SwitchPattern::Default => {
                out.push_str(&format!("{}Default\n", indent(depth + 1)));
            }
        }
        match &arm.body {
            SwitchBody::Expr(e) => dump_expr(e, out, depth + 2),
            SwitchBody::Block(b) => dump_block(b, out, depth + 2),
        }
    }
}

fn dump_expr(expr: &Expr, out: &mut String, depth: usize) {
    match expr {
        Expr::Int(n) => { out.push_str(&format!("{}Int({})\n", indent(depth), n)); }
        Expr::Float(n) => { out.push_str(&format!("{}Float({})\n", indent(depth), n)); }
        Expr::StringLit(s) => { out.push_str(&format!("{}String({:?})\n", indent(depth), s)); }
        Expr::CharLit(c) => { out.push_str(&format!("{}Char({:?})\n", indent(depth), c)); }
        Expr::Bool(b) => { out.push_str(&format!("{}Bool({})\n", indent(depth), b)); }
        Expr::Null => { out.push_str(&format!("{}Null\n", indent(depth))); }
        Expr::This => { out.push_str(&format!("{}This\n", indent(depth))); }
        Expr::Path(p) => { out.push_str(&format!("{}Path({})\n", indent(depth), p.join("."))); }
        Expr::Unary { op, expr } => {
            out.push_str(&format!("{}Unary({:?})\n", indent(depth), op));
            dump_expr(expr, out, depth + 1);
        }
        Expr::Binary { op, left, right } => {
            out.push_str(&format!("{}Binary({:?})\n", indent(depth), op));
            dump_expr(left, out, depth + 1);
            dump_expr(right, out, depth + 1);
        }
        Expr::Call { callee, args } => {
            out.push_str(&format!("{}Call\n", indent(depth)));
            dump_expr(callee, out, depth + 1);
            for a in args { dump_call_arg(a, out, depth + 1); }
        }
        Expr::MethodCall { receiver, method, args } => {
            out.push_str(&format!("{}MethodCall({})\n", indent(depth), method));
            dump_expr(receiver, out, depth + 1);
            for a in args { dump_call_arg(a, out, depth + 1); }
        }
        Expr::SafeCall { receiver, method, args } => {
            out.push_str(&format!("{}SafeCall({})\n", indent(depth), method));
            dump_expr(receiver, out, depth + 1);
            for a in args { dump_call_arg(a, out, depth + 1); }
        }
        Expr::FieldAccess { receiver, field } => {
            out.push_str(&format!("{}FieldAccess({})\n", indent(depth), field));
            dump_expr(receiver, out, depth + 1);
        }
        Expr::SafeFieldAccess { receiver, field } => {
            out.push_str(&format!("{}SafeFieldAccess({})\n", indent(depth), field));
            dump_expr(receiver, out, depth + 1);
        }
        Expr::Index { base, index } => {
            out.push_str(&format!("{}Index\n", indent(depth)));
            dump_expr(base, out, depth + 1);
            dump_expr(index, out, depth + 1);
        }
        Expr::NullAssert(e) => {
            out.push_str(&format!("{}NullAssert\n", indent(depth)));
            dump_expr(e, out, depth + 1);
        }
        Expr::Elvis { left, right } => {
            out.push_str(&format!("{}Elvis\n", indent(depth)));
            dump_expr(left, out, depth + 1);
            dump_expr(right, out, depth + 1);
        }
        Expr::Lambda { params, body } => {
            out.push_str(&format!("{}Lambda({})\n", indent(depth), params.join(", ")));
            dump_expr(body, out, depth + 1);
        }
        Expr::Switch(s) => {
            dump_switch(s, out, depth);
        }
        Expr::ListLit(elems) => {
            out.push_str(&format!("{}ListLit\n", indent(depth)));
            for e in elems { dump_expr(e, out, depth + 1); }
        }
    }
}

fn dump_call_arg(arg: &CallArg, out: &mut String, depth: usize) {
    match arg {
        CallArg::Positional(e) => dump_expr(e, out, depth),
        CallArg::Named { name, value } => {
            out.push_str(&format!("{}NamedArg({})\n", indent(depth), name));
            dump_expr(value, out, depth + 1);
        }
    }
}


pub fn generate_ast_file(nx_path: &str) -> Result<(), String> {
    let program = parse_file(nx_path)?;
    let ast_dump = dump_ast(&program);
    let ast_path = nx_path.replace(".nx", ".ast");
    fs::write(&ast_path, &ast_dump)
        .map_err(|e| format!("no se pudo escribir {}: {}", ast_path, e))?;
    println!("generado: {}", ast_path);
    Ok(())
}

pub fn run_parser_tests(test_dir: &str) -> bool {
    let entries = match fs::read_dir(test_dir) {
        Ok(e) => e,
        Err(_) => {
            eprintln!("no se pudo abrir el directorio de tests: {}", test_dir);
            return false;
        }
    };
    let mut all_pass = true;
    let mut nx_files: Vec<_> = entries
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map(|x| x == "nx").unwrap_or(false))
        .collect();
    nx_files.sort_by_key(|e| e.path());
    for entry in nx_files {
        let nx_path = entry.path();
        let ast_path = nx_path.with_extension("ast");
        if !ast_path.exists() {
            eprintln!("falta archivo esperado: {}", ast_path.display());
            all_pass = false;
            continue;
        }
        let expected = match fs::read_to_string(&ast_path) {
            Ok(s) => s,
            Err(e) => { eprintln!("error leyendo {}: {}", ast_path.display(), e); all_pass = false; continue; }
        };
        match parse_file(nx_path.to_str().unwrap()) {
            Ok(program) => {
                let got = dump_ast(&program);
                if got == expected {
                    println!("PASS parser: {}", nx_path.display());
                } else {
                    eprintln!("FAIL parser: {}", nx_path.display());
                    eprintln!("esperado:\n{}", expected);
                    eprintln!("obtenido:\n{}", got);
                    all_pass = false;
                }
            }
            Err(e) => {
                eprintln!("FAIL parser: {} => {}", nx_path.display(), e);
                all_pass = false;
            }
        }
    }
    all_pass
}

pub fn run_sema_tests(test_dir: &str) -> bool {
    let entries = match fs::read_dir(test_dir) {
        Ok(e) => e,
        Err(_) => {
            eprintln!("no se pudo abrir el directorio de tests: {}", test_dir);
            return false;
        }
    };
    let mut all_pass = true;
    let mut nx_files: Vec<_> = entries
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map(|x| x == "nx").unwrap_or(false))
        .collect();
    nx_files.sort_by_key(|e| e.path());
    for entry in nx_files {
        let nx_path = entry.path();
        let is_invalid = nx_path.file_name()
            .and_then(|n| n.to_str())
            .map(|n| n.starts_with("invalid_"))
            .unwrap_or(false);
        match parse_file(nx_path.to_str().unwrap()) {
            Ok(program) => {
                match crate::sema::check_program(&program) {
                    Ok(_) => {
                        if is_invalid {
                            eprintln!("FAIL sema (invalid, esperaba error): {}", nx_path.display());
                            all_pass = false;
                        } else {
                            println!("PASS sema: {}", nx_path.display());
                        }
                    }
                    Err(errs) => {
                        if is_invalid {
                            println!("PASS sema (invalid): {}", nx_path.display());
                        } else {
                            eprintln!("FAIL sema: {}", nx_path.display());
                            for e in &errs {
                                eprintln!("  {}", e);
                            }
                            all_pass = false;
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("FAIL sema (parse error): {} => {}", nx_path.display(), e);
                all_pass = false;
            }
        }
    }
    all_pass
}

pub fn generate_c_file(nx_path: &str) -> Result<(), String> {
    let program = parse_file(nx_path)?;
    let ctx = crate::sema::check_program(&program)
        .map_err(|errs| errs.iter().map(|e| e.to_string()).collect::<Vec<_>>().join("\n"))?;
    let c_code = crate::codegen_c::generate_c(&program, ctx);
    let c_path = nx_path.replace(".nx", ".c");
    fs::write(&c_path, &c_code)
        .map_err(|e| format!("no se pudo escribir {}: {}", c_path, e))?;
    println!("generado: {}", c_path);
    Ok(())
}

pub fn parse_dir(dir: &str) -> Result<Program, String> {
    let entries = fs::read_dir(dir)
        .map_err(|e| format!("no se pudo abrir {}: {}", dir, e))?;
    let mut nx_files: Vec<_> = entries
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map(|x| x == "nx").unwrap_or(false))
        .collect();
    nx_files.sort_by_key(|e| e.path());
    let mut all_items = Vec::new();
    for entry in nx_files {
        let path = entry.path();
        let p = parse_file(path.to_str().unwrap())?;
        all_items.extend(p.items);
    }
    Ok(Program { items: all_items })
}

pub fn compile_dir(dir: &str, output: &str) -> Result<(), String> {
    let program = parse_dir(dir)?;
    let ctx = crate::sema::check_program(&program)
        .map_err(|errs| errs.iter().map(|e| e.to_string()).collect::<Vec<_>>().join("\n"))?;
    let c_code = crate::codegen_c::generate_c(&program, ctx);

    let c_path = format!("{}.c", output);
    fs::write(&c_path, &c_code)
        .map_err(|e| format!("no se pudo escribir {}: {}", c_path, e))?;

    let runtime_c = find_runtime_path();
    let runtime_dir = find_runtime_dir();
    let clang = find_clang();
    let mut cmd = std::process::Command::new(&clang);
    cmd.args(&[&c_path, &runtime_c, "-I", &runtime_dir, "-o", output, "-Wno-return-type", "-Wno-deprecated-declarations"]);
    if let Ok(inc) = std::env::var("GC_INCLUDE") {
        cmd.args(&["-I", &inc]);
    }
    if let Ok(lib) = std::env::var("GC_LIB") {
        cmd.args(&["-L", &lib]);
    }
    if let Ok(inc) = std::env::var("SSL_INCLUDE") {
        cmd.args(&["-I", &inc]);
    }
    if let Ok(lib) = std::env::var("SSL_LIB") {
        cmd.args(&["-L", &lib]);
    }
    cmd.arg("-lgc");
    #[cfg(target_os = "windows")]
    {
        cmd.arg("-llibssl");
        cmd.arg("-llibcrypto");
        cmd.arg("-lcrypt32");
        cmd.arg("-ladvapi32");
        cmd.arg("-luser32");
        cmd.arg("-lws2_32");
    }
    #[cfg(not(target_os = "windows"))]
    {
        cmd.arg("-lssl");
        cmd.arg("-lcrypto");
    }
    #[cfg(target_os = "windows")]
    cmd.args(&["-Xlinker", "/subsystem:console", "-Wl,/Brepro"]);
    #[cfg(not(target_os = "windows"))]
    cmd.args(&[
        "-Wl,--build-id=none",
        "-Wl,--hash-style=gnu",
        "-frandom-seed=nexus",
        "-fno-record-gcc-switches",
        "-fno-ident",
    ]);
    let status = cmd.status()
        .map_err(|e| format!("no se pudo ejecutar {}: {}", clang, e))?;

    if !status.success() {
        return Err(format!("clang falló al compilar {}", c_path));
    }
    Ok(())
}

pub fn compile_file(nx_path: &str, output: &str) -> Result<(), String> {
    let program = parse_file(nx_path)?;
    let ctx = crate::sema::check_program(&program)
        .map_err(|errs| errs.iter().map(|e| e.to_string()).collect::<Vec<_>>().join("\n"))?;
    let c_code = crate::codegen_c::generate_c(&program, ctx);

    let c_path = format!("{}.c", output);
    fs::write(&c_path, &c_code)
        .map_err(|e| format!("no se pudo escribir {}: {}", c_path, e))?;

    let runtime_c = find_runtime_path();
    let runtime_dir = find_runtime_dir();
    let clang = find_clang();
    let mut cmd = std::process::Command::new(&clang);
    cmd.args(&[&c_path, &runtime_c, "-I", &runtime_dir, "-o", output, "-Wno-return-type", "-Wno-deprecated-declarations"]);
    if let Ok(inc) = std::env::var("GC_INCLUDE") {
        cmd.args(&["-I", &inc]);
    }
    if let Ok(lib) = std::env::var("GC_LIB") {
        cmd.args(&["-L", &lib]);
    }
    if let Ok(inc) = std::env::var("SSL_INCLUDE") {
        cmd.args(&["-I", &inc]);
    }
    if let Ok(lib) = std::env::var("SSL_LIB") {
        cmd.args(&["-L", &lib]);
    }
    cmd.arg("-lgc");
    #[cfg(target_os = "windows")]
    {
        cmd.arg("-llibssl");
        cmd.arg("-llibcrypto");
        cmd.arg("-lcrypt32");
        cmd.arg("-ladvapi32");
        cmd.arg("-luser32");
        cmd.arg("-lws2_32");
    }
    #[cfg(not(target_os = "windows"))]
    {
        cmd.arg("-lssl");
        cmd.arg("-lcrypto");
    }
    #[cfg(target_os = "windows")]
    cmd.args(&["-Xlinker", "/subsystem:console", "-Wl,/Brepro"]);
    #[cfg(not(target_os = "windows"))]
    cmd.args(&[
        "-Wl,--build-id=none",
        "-Wl,--hash-style=gnu",
        "-frandom-seed=nexus",
        "-fno-record-gcc-switches",
        "-fno-ident",
    ]);
    let status = cmd.status()
        .map_err(|e| format!("no se pudo ejecutar {}: {}", clang, e))?;

    if !status.success() {
        return Err(format!("clang falló al compilar {}", c_path));
    }
    Ok(())
}

fn find_clang() -> String {
    std::env::var("CC").unwrap_or_else(|_| "clang".to_string())
}

fn find_runtime_dir() -> String {
    let candidates = ["runtime", "../runtime"];
    for c in &candidates {
        if std::path::Path::new(&format!("{}/nexus_runtime.h", c)).exists() {
            return c.to_string();
        }
    }
    "runtime".to_string()
}

fn find_runtime_path() -> String {
    let candidates = ["runtime/nexus_runtime.c", "../runtime/nexus_runtime.c"];
    for c in &candidates {
        if std::path::Path::new(c).exists() {
            return c.to_string();
        }
    }
    "runtime/nexus_runtime.c".to_string()
}

fn resolve_e2e_executable(bin_path: &str) -> std::path::PathBuf {
    let path = std::path::PathBuf::from(bin_path);
    if path.exists() {
        return path;
    }
    #[cfg(target_os = "windows")]
    {
        let with_exe = format!("{}.exe", bin_path);
        let exe_path = std::path::PathBuf::from(&with_exe);
        if exe_path.exists() {
            return exe_path;
        }
    }
    path
}

pub fn run_e2e_tests(test_dir: &str) -> bool {
    let entries = match fs::read_dir(test_dir) {
        Ok(e) => e,
        Err(_) => {
            eprintln!("no se pudo abrir el directorio de tests: {}", test_dir);
            return false;
        }
    };
    let mut all_pass = true;
    let mut nx_files: Vec<_> = entries
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map(|x| x == "nx").unwrap_or(false))
        .collect();
    nx_files.sort_by_key(|e| e.path());
    for entry in nx_files {
        let nx_path = entry.path();
        let expected_path = nx_path.with_extension("expected");
        if !expected_path.exists() {
            eprintln!("falta archivo esperado: {}", expected_path.display());
            all_pass = false;
            continue;
        }
        let expected = match fs::read_to_string(&expected_path) {
            Ok(s) => s,
            Err(e) => { eprintln!("error leyendo {}: {}", expected_path.display(), e); all_pass = false; continue; }
        };
        let base = nx_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("test");
        let bin_path = format!("{}/{}.out", test_dir, base);
        match compile_file(nx_path.to_str().unwrap(), &bin_path) {
            Ok(_) => {
                let run_path = resolve_e2e_executable(&bin_path);
                let output = std::process::Command::new(&run_path)
                    .output()
                    .map_err(|e| e.to_string());
                match output {
                    Ok(out) => {
                        let got = String::from_utf8_lossy(&out.stdout).to_string();
                        if got == expected {
                            println!("PASS e2e: {}", nx_path.display());
                        } else {
                            eprintln!("FAIL e2e: {}", nx_path.display());
                            eprintln!("esperado:\n{}", expected);
                            eprintln!("obtenido:\n{}", got);
                            all_pass = false;
                        }
                    }
                    Err(e) => {
                        eprintln!("FAIL e2e (ejecución): {} => {}", nx_path.display(), e);
                        all_pass = false;
                    }
                }
            }
            Err(e) => {
                eprintln!("FAIL e2e (compilación): {} => {}", nx_path.display(), e);
                all_pass = false;
            }
        }
    }
    all_pass
}

pub fn run_codegen_tests(test_dir: &str) -> bool {
    let entries = match fs::read_dir(test_dir) {
        Ok(e) => e,
        Err(_) => {
            eprintln!("no se pudo abrir el directorio de tests: {}", test_dir);
            return false;
        }
    };
    let mut all_pass = true;
    let mut nx_files: Vec<_> = entries
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map(|x| x == "nx").unwrap_or(false))
        .collect();
    nx_files.sort_by_key(|e| e.path());
    for entry in nx_files {
        let nx_path = entry.path();
        let c_snapshot_path = nx_path.with_extension("c.expected");
        if !c_snapshot_path.exists() {
            eprintln!("falta snapshot esperado: {}", c_snapshot_path.display());
            all_pass = false;
            continue;
        }
        let expected = match fs::read_to_string(&c_snapshot_path) {
            Ok(s) => s,
            Err(e) => { eprintln!("error leyendo {}: {}", c_snapshot_path.display(), e); all_pass = false; continue; }
        };
        match parse_file(nx_path.to_str().unwrap()) {
            Ok(program) => {
                match crate::sema::check_program(&program) {
                    Ok(ctx) => {
                        let got = crate::codegen_c::generate_c(&program, ctx);
                        if got == expected {
                            println!("PASS codegen: {}", nx_path.display());
                        } else {
                            eprintln!("FAIL codegen: {}", nx_path.display());
                            all_pass = false;
                        }
                    }
                    Err(errs) => {
                        eprintln!("FAIL codegen (sema): {}", nx_path.display());
                        for e in &errs { eprintln!("  {}", e); }
                        all_pass = false;
                    }
                }
            }
            Err(e) => {
                eprintln!("FAIL codegen (parse): {} => {}", nx_path.display(), e);
                all_pass = false;
            }
        }
    }
    all_pass
}
