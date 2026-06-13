use crate::ast::*;
use crate::sema::{SemaContext, NxType};
use std::collections::HashMap;

pub struct CGen {
    output: String,
    indent: usize,
    tmp_counter: usize,
    ctx: SemaContext,
    current_class: Option<String>,
    var_types: HashMap<String, NxType>,
}

impl CGen {
    pub fn new(ctx: SemaContext) -> Self {
        CGen {
            output: String::new(),
            indent: 0,
            tmp_counter: 0,
            ctx,
            current_class: None,
            var_types: HashMap::new(),
        }
    }

    fn emit_line(&mut self, s: &str) {
        let ind = "    ".repeat(self.indent);
        self.output.push_str(&format!("{}{}\n", ind, s));
    }

    fn fresh_tmp(&mut self) -> String {
        let n = self.tmp_counter;
        self.tmp_counter += 1;
        format!("_nx_tmp_{}", n)
    }

    pub fn generate(mut self, program: &Program) -> String {
        self.emit_line("#include \"nexus_runtime.h\"");
        self.output.push('\n');

        let mut sorted_structs: Vec<_> = self.ctx.structs.keys().cloned().collect();
        sorted_structs.sort();
        for name in &sorted_structs {
            self.emit_line(&format!("typedef struct NxStruct_{} NxStruct_{};", name, name));
        }
        if !sorted_structs.is_empty() { self.output.push('\n'); }

        for name in &sorted_structs {
            let si = self.ctx.structs[name].clone();
            self.emit_line(&format!("struct NxStruct_{} {{", name));
            let mut sorted_fields = si.fields.clone();
            sorted_fields.sort_by_key(|(n, _)| n.clone());
            for (fname, fty) in &sorted_fields {
                let cty = self.nx_type_to_c(fty);
                self.emit_line(&format!("    {} {};", cty, fname));
            }
            self.emit_line("};");
            self.output.push('\n');
        }

        let items: Vec<_> = program.items.clone();
        for item in &items {
            if let Item::GlobalConst(g) = item {
                let ty = g.ty.as_ref()
                    .map(|t| self.ctx.resolve_type(t))
                    .unwrap_or_else(|| self.infer_expr_type(&g.value));
                let cty = self.nx_type_to_c(&ty);
                let val = self.gen_expr(&g.value, Some(&ty));
                let qualifier = if g.is_final { "static const" } else { "static" };
                self.emit_line(&format!("{} {} {} = {};", qualifier, cty, g.name, val));
            }
        }
        let has_globals = items.iter().any(|i| matches!(i, Item::GlobalConst(_)));
        if has_globals { self.output.push('\n'); }

        let mut forward_decls = Vec::new();
        for item in &items {
            match item {
                Item::Function(f) => {
                    forward_decls.push(self.fn_signature(f, None));
                }
                Item::ClassDecl(c) => {
                    for m in &c.members {
                        if let ClassMember::Method(f) = m {
                            forward_decls.push(self.fn_signature(f, Some(&c.name)));
                        }
                    }
                }
                _ => {}
            }
        }
        for decl in &forward_decls {
            self.emit_line(&format!("{};", decl));
        }
        if !forward_decls.is_empty() { self.output.push('\n'); }

        let has_main = items.iter().any(|i| matches!(i, Item::Function(f) if f.name == "main"));
        for item in &items {
            self.gen_item(item);
        }
        if has_main {
            self.emit_line("int main(int argc, char** argv) {");
            self.emit_line("    nexus_init();");
            self.emit_line("    nexus_set_args(argc, argv);");
            self.emit_line("    nx_fn_main();");
            self.emit_line("    return 0;");
            self.emit_line("}");
            self.output.push('\n');
        }

        self.output
    }

    fn fn_signature(&self, f: &Function, class_name: Option<&str>) -> String {
        let ret = f.return_type.as_ref()
            .map(|t| self.nx_type_to_c(&self.ctx.resolve_type(t)))
            .unwrap_or_else(|| "void".to_string());
        let name = match class_name {
            Some(t) => format!("nx_method_{}_{}", t, f.name),
            None => format!("nx_fn_{}", f.name),
        };
        let mut params = Vec::new();
        if let Some(t) = class_name {
            params.push(format!("NxStruct_{}* this", t));
        }
        for p in &f.params {
            let cty = self.nx_type_to_c(&self.ctx.resolve_type(&p.ty));
            params.push(format!("{} {}", cty, p.name));
        }
        format!("{} {}({})", ret, name, params.join(", "))
    }

    fn gen_item(&mut self, item: &Item) {
        match item {
            Item::Function(f) => {
                let sig = self.fn_signature(f, None);
                self.emit_line(&format!("{} {{", sig));
                self.indent += 1;
                self.var_types.clear();
                for p in &f.params {
                    self.var_types.insert(p.name.clone(), self.ctx.resolve_type(&p.ty));
                }
                self.gen_fn_body(&f.body);
                self.indent -= 1;
                self.emit_line("}");
                self.output.push('\n');
            }
            Item::ClassDecl(c) => {
                let class_name = c.name.clone();
                self.current_class = Some(class_name.clone());
                for m in &c.members.clone() {
                    if let ClassMember::Method(f) = m {
                        let sig = self.fn_signature(f, Some(&class_name));
                        self.emit_line(&format!("{} {{", sig));
                        self.indent += 1;
                        self.var_types.clear();
                        self.var_types.insert("this".to_string(), NxType::Named(class_name.clone()));
                        for p in &c.primary_params {
                            self.var_types.insert(p.name.clone(), self.ctx.resolve_type(&p.ty));
                        }
                        for p in &f.params {
                            self.var_types.insert(p.name.clone(), self.ctx.resolve_type(&p.ty));
                        }
                        self.gen_fn_body(&f.body);
                        self.indent -= 1;
                        self.emit_line("}");
                        self.output.push('\n');
                    }
                }
                self.current_class = None;
            }
            _ => {}
        }
    }

    fn gen_fn_body(&mut self, body: &FunctionBody) {
        match body {
            FunctionBody::Block(b) => self.gen_block(b),
            FunctionBody::Expr(e) => {
                let val = self.gen_expr(e, None);
                self.emit_line(&format!("return {};", val));
            }
        }
    }

    fn gen_block(&mut self, block: &Block) {
        for stmt in &block.stmts {
            self.gen_stmt(stmt);
        }
    }

    fn gen_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Var(v) => {
                let expr_type = self.infer_expr_type(&v.value);
                let declared_type = v.ty.as_ref()
                    .map(|t| self.ctx.resolve_type(t))
                    .unwrap_or_else(|| expr_type.clone());
                let cty = self.nx_type_to_c(&declared_type);
                let val = self.gen_expr(&v.value, Some(&declared_type));
                self.var_types.insert(v.name.clone(), declared_type);
                self.emit_line(&format!("{} {} = {};", cty, v.name, val));
            }
            Stmt::Return(r) => {
                if let Some(v) = &r.value {
                    let val = self.gen_expr(v, None);
                    self.emit_line(&format!("return {};", val));
                } else {
                    self.emit_line("return;");
                }
            }
            Stmt::Break => { self.emit_line("break;"); }
            Stmt::Continue => { self.emit_line("continue;"); }
            Stmt::If(i) => {
                let cond = self.gen_expr(&i.cond, Some(&NxType::Bool));
                self.emit_line(&format!("if ({}) {{", cond));
                self.indent += 1;
                let then_b = i.then_block.clone();
                self.gen_block(&then_b);
                self.indent -= 1;
                match &i.else_branch {
                    Some(ElseBranch::Block(b)) => {
                        self.emit_line("} else {");
                        self.indent += 1;
                        let b = b.clone();
                        self.gen_block(&b);
                        self.indent -= 1;
                        self.emit_line("}");
                    }
                    Some(ElseBranch::If(i2)) => {
                        self.emit_line("} else {");
                        self.indent += 1;
                        let i2 = *i2.clone();
                        self.gen_stmt(&Stmt::If(i2));
                        self.indent -= 1;
                        self.emit_line("}");
                    }
                    None => { self.emit_line("}"); }
                }
            }
            Stmt::While(w) => {
                let cond = self.gen_expr(&w.cond, Some(&NxType::Bool));
                self.emit_line(&format!("while ({}) {{", cond));
                self.indent += 1;
                let body = w.body.clone();
                self.gen_block(&body);
                self.indent -= 1;
                self.emit_line("}");
            }
            Stmt::For(f) => {
                let iter = self.gen_expr(&f.iterable, None);
                let iter_tmp = self.fresh_tmp();
                let idx_tmp = self.fresh_tmp();
                self.emit_line(&format!("NxList* {} = {};", iter_tmp, iter));
                self.emit_line(&format!("for (NxInt {} = 0; {} < nx_list_len({}); {}++) {{", idx_tmp, idx_tmp, iter_tmp, idx_tmp));
                self.indent += 1;
                let iter_ty = self.infer_expr_type(&f.iterable);
                let elem_ty = if let NxType::List(e) = iter_ty { *e } else { NxType::Unknown };
                let elem_c = self.nx_type_to_c(&elem_ty);
                self.emit_line(&format!("{} {} = {});", elem_c, f.var, self.void_cast_out(&elem_ty, &format!("nx_list_get({}, {})", iter_tmp, idx_tmp))));
                self.var_types.insert(f.var.clone(), elem_ty);
                let body = f.body.clone();
                self.gen_block(&body);
                self.indent -= 1;
                self.emit_line("}");
            }
            Stmt::Switch(s) => {
                self.gen_switch(s);
            }
            Stmt::Expr(e) => {
                if let Some(assign_val) = &e.assign {
                    let lhs = self.gen_lvalue(&e.expr);
                    let rhs = self.gen_expr(assign_val, None);
                    self.emit_line(&format!("{} = {};", lhs, rhs));
                } else {
                    let val = self.gen_expr(&e.expr, None);
                    self.emit_line(&format!("{};", val));
                }
            }
        }
    }

    fn gen_switch(&mut self, s: &SwitchStmt) {
        let subj = self.gen_expr(&s.subject, None);
        let subj_tmp = self.fresh_tmp();
        let subj_ty = self.infer_expr_type(&s.subject);
        let subj_cty = self.nx_type_to_c(&subj_ty);
        self.emit_line(&format!("{} {} = {};", subj_cty, subj_tmp, subj));

        let mut first = true;
        for arm in &s.arms.clone() {
            let cond = match &arm.pattern {
                SwitchPattern::Default => "1".to_string(),
                SwitchPattern::Case(e) => {
                    let case_val = self.gen_expr(e, None);
                    // string comparison needs strcmp
                    if matches!(subj_ty, NxType::StringType) {
                        format!("nx_string_equals({}, {})", subj_tmp, case_val)
                    } else {
                        format!("{} == {}", subj_tmp, case_val)
                    }
                }
            };
            if first {
                self.emit_line(&format!("if ({}) {{", cond));
                first = false;
            } else {
                self.emit_line(&format!("}} else if ({}) {{", cond));
            }
            self.indent += 1;
            match &arm.body.clone() {
                SwitchBody::Expr(e) => {
                    let val = self.gen_expr(e, None);
                    self.emit_line(&format!("{};", val));
                }
                SwitchBody::Block(b) => {
                    let b = b.clone();
                    self.gen_block(&b);
                }
            }
            self.indent -= 1;
        }
        if !first {
            self.emit_line("}");
        }
    }

    fn gen_lvalue(&self, expr: &Expr) -> String {
        match expr {
            Expr::Path(p) => p.join("_"),
            Expr::This => "this".to_string(),
            Expr::FieldAccess { receiver, field } => {
                let recv = self.gen_lvalue(receiver);
                format!("{}->{}", recv, field)
            }
            Expr::Index { base, index } => {
                // lvalue for list[i] = v handled specially via gen_stmt
                let base_str = self.gen_lvalue(base);
                let _ = index;
                base_str
            }
            _ => "_nx_invalid_lvalue".to_string(),
        }
    }

    // cast a void* out to the concrete C type
    fn void_cast_out(&self, ty: &NxType, expr: &str) -> String {
        match ty {
            NxType::Int | NxType::Long => format!("(NxInt)(intptr_t)({})", expr),
            NxType::Bool => format!("(NxBool)(intptr_t)({})", expr),
            NxType::Char => format!("(NxChar)(intptr_t)({})", expr),
            _ => {
                let cty = self.nx_type_to_c(ty);
                format!("({})({})", cty, expr)
            }
        }
    }

    // cast a value into void* for list/map storage
    fn void_cast_in(&self, ty: &NxType, val: &str) -> String {
        match ty {
            NxType::Int | NxType::Long | NxType::Bool | NxType::Char =>
                format!("(void*)(intptr_t)({})", val),
            _ => format!("(void*)({})", val),
        }
    }

    fn gen_expr(&mut self, expr: &Expr, _hint: Option<&NxType>) -> String {
        match expr {
            Expr::Int(n) => format!("((NxInt){}LL)", n),
            Expr::Float(n) => format!("((double){})", n),
            Expr::Bool(b) => if *b { "NX_TRUE".to_string() } else { "NX_FALSE".to_string() },
            Expr::CharLit(c) => {
                let esc = match c {
                    '\0' => "\\0".to_string(),
                    '\n' => "\\n".to_string(),
                    '\t' => "\\t".to_string(),
                    '\r' => "\\r".to_string(),
                    '\\' => "\\\\".to_string(),
                    '\'' => "\\'".to_string(),
                    _ => c.to_string(),
                };
                format!("((NxChar)'{}')", esc)
            }
            Expr::StringLit(s) => format!("{:?}", s),
            Expr::Null => "NULL".to_string(),
            Expr::This => "this".to_string(),
            Expr::Path(p) => {
                if p.len() == 1 { p[0].clone() } else { p.join("_") }
            }
            Expr::Unary { op, expr } => {
                let inner = self.gen_expr(expr, None);
                match op {
                    UnaryOp::Neg => format!("(-{})", inner),
                    UnaryOp::Not => format!("(!{})", inner),
                }
            }
            Expr::Binary { op, left, right } => {
                // String concatenation and comparison
                if matches!(op, BinaryOp::Add) {
                    let l_ty = self.infer_expr_type(left);
                    let r_ty = self.infer_expr_type(right);
                    if matches!(l_ty, NxType::StringType) || matches!(r_ty, NxType::StringType) {
                        let l = self.coerce_to_string(left, &l_ty);
                        let r = self.coerce_to_string(right, &r_ty);
                        return format!("nx_string_concat({}, {})", l, r);
                    }
                }
                if matches!(op, BinaryOp::Eq | BinaryOp::Ne) {
                    let l_ty = self.infer_expr_type(left);
                    if matches!(l_ty, NxType::StringType) {
                        let l = self.gen_expr(left, None);
                        let r = self.gen_expr(right, None);
                        return if matches!(op, BinaryOp::Eq) {
                            format!("nx_string_equals({}, {})", l, r)
                        } else {
                            format!("(!nx_string_equals({}, {}))", l, r)
                        };
                    }
                }
                let l = self.gen_expr(left, None);
                let r = self.gen_expr(right, None);
                let op_str = match op {
                    BinaryOp::Add => "+",
                    BinaryOp::Sub => "-",
                    BinaryOp::Mul => "*",
                    BinaryOp::Div => "/",
                    BinaryOp::Mod => "%",
                    BinaryOp::Eq => "==",
                    BinaryOp::Ne => "!=",
                    BinaryOp::Lt => "<",
                    BinaryOp::Le => "<=",
                    BinaryOp::Gt => ">",
                    BinaryOp::Ge => ">=",
                    BinaryOp::And => "&&",
                    BinaryOp::Or => "||",
                };
                format!("({} {} {})", l, op_str, r)
            }
            Expr::Call { callee, args } => {
                self.gen_call(callee, args)
            }
            Expr::MethodCall { receiver, method, args } => {
                self.gen_method_call(receiver, method, args, false)
            }
            Expr::SafeCall { receiver, method, args } => {
                self.gen_method_call(receiver, method, args, true)
            }
            Expr::FieldAccess { receiver, field } => {
                // String.length as field (no parens)
                let recv_ty = self.infer_expr_type(receiver);
                if matches!(recv_ty, NxType::StringType) && field == "length" {
                    let recv = self.gen_expr(receiver, None);
                    return format!("nx_string_len({})", recv);
                }
                let recv = self.gen_expr(receiver, None);
                format!("{}->{}", recv, field)
            }
            Expr::SafeFieldAccess { receiver, field } => {
                let recv = self.gen_expr(receiver, None);
                format!("(({}) != NULL ? ({})->{} : NULL)", recv, recv, field)
            }
            Expr::Index { base, index } => {
                let base_str = self.gen_expr(base, None);
                let idx = self.gen_expr(index, Some(&NxType::Int));
                let elem_ty = if let NxType::List(e) = self.infer_expr_type(base) { *e } else { NxType::Unknown };
                self.void_cast_out(&elem_ty, &format!("nx_list_get({}, {})", base_str, idx))
            }
            Expr::NullAssert(e) => {
                let inner = self.gen_expr(e, None);
                let cty = self.nx_type_to_c(&self.infer_expr_type(e));
                format!("(({}) != NULL ? ({}) : (nx_panic(\"null assertion failed\"), ({})NULL))", inner, inner, cty)
            }
            Expr::Elvis { left, right } => {
                let l = self.gen_expr(left, None);
                let r = self.gen_expr(right, None);
                format!("(({}) != NULL ? ({}) : ({}))", l, l, r)
            }
            Expr::Lambda { .. } => "NULL".to_string(),
            Expr::Switch(s) => {
                let tmp = self.fresh_tmp();
                self.emit_line(&format!("void* {};", tmp));
                self.gen_switch(s);
                tmp
            }
            Expr::ListLit(elems) => {
                let tmp = self.fresh_tmp();
                self.emit_line(&format!("NxList* {} = nx_list_new();", tmp));
                for e in elems {
                    let ty = self.infer_expr_type(e);
                    let val = self.gen_expr(e, None);
                    let cast = self.void_cast_in(&ty, &val);
                    self.emit_line(&format!("nx_list_push({}, {});", tmp, cast));
                }
                tmp
            }
        }
    }

    fn coerce_to_string(&mut self, expr: &Expr, ty: &NxType) -> String {
        let val = self.gen_expr(expr, None);
        match ty {
            NxType::StringType => val,
            NxType::Int | NxType::Long => format!("nx_int_to_string({})", val),
            NxType::Bool => format!("nx_bool_to_string({})", val),
            NxType::Char => format!("nx_char_to_string({})", val),
            _ => val,
        }
    }

    fn get_call_arg_str(&mut self, args: &[CallArg], idx: usize) -> String {
        args.get(idx).map(|a| match a {
            CallArg::Positional(e) => self.gen_expr(e, None),
            CallArg::Named { value, .. } => self.gen_expr(value, None),
        }).unwrap_or_default()
    }

    fn get_call_arg_type(&self, args: &[CallArg], idx: usize) -> NxType {
        args.get(idx).map(|a| match a {
            CallArg::Positional(e) => self.infer_expr_type(e),
            CallArg::Named { value, .. } => self.infer_expr_type(value),
        }).unwrap_or(NxType::Unknown)
    }

    fn builtin_free_fn_c_name(name: &str) -> Option<&'static str> {
        match name {
            "min" => Some("nx_min"),
            _ => None,
        }
    }

    fn gen_call(&mut self, callee: &Expr, args: &[CallArg]) -> String {
        if let Expr::Path(p) = callee {
            if p.len() == 2 && p[0] == "io" {
                return self.gen_io_call(&p[1], args);
            }

            if p.len() == 1 && p[0] == "List" {
                return "nx_list_new()".to_string();
            }
            if p.len() == 1 && p[0] == "Map" {
                return "nx_map_new()".to_string();
            }

            if p.len() == 1 {
                if let Some(c_name) = Self::builtin_free_fn_c_name(&p[0]) {
                    let args_str: Vec<_> = args.iter().map(|a| match a {
                        CallArg::Positional(e) => self.gen_expr(e, None),
                        CallArg::Named { value, .. } => self.gen_expr(value, None),
                    }).collect();
                    return format!("{}({})", c_name, args_str.join(", "));
                }
            }

            if p.len() == 1 && self.ctx.structs.contains_key(&p[0]) {
                let type_name = p[0].clone();
                let tmp = self.fresh_tmp();
                self.emit_line(&format!("NxStruct_{}* {} = GC_MALLOC(sizeof(NxStruct_{}));", type_name, tmp, type_name));
                let struct_fields = self.ctx.structs.get(&type_name).map(|s| s.fields.clone()).unwrap_or_default();
                let named: Vec<_> = args.iter().filter_map(|a| {
                    if let CallArg::Named { name, value } = a { Some((name.clone(), value.clone())) } else { None }
                }).collect();
                let positional: Vec<_> = args.iter().filter_map(|a| {
                    if let CallArg::Positional(e) = a { Some(e.clone()) } else { None }
                }).collect();
                if !named.is_empty() {
                    for (fname, fval) in &named {
                        let field_type = struct_fields.iter().find(|(n, _)| n == fname).map(|(_, t)| t.clone());
                        let val = self.gen_expr(fval, field_type.as_ref());
                        self.emit_line(&format!("{}->{} = {};", tmp, fname, val));
                    }
                } else {
                    let mut sorted_fields = struct_fields.clone();
                    sorted_fields.sort_by_key(|(n, _)| n.clone());
                    for (i, (fname, fty)) in sorted_fields.iter().enumerate() {
                        if let Some(e) = positional.get(i) {
                            let val = self.gen_expr(e, Some(fty));
                            self.emit_line(&format!("{}->{} = {};", tmp, fname, val));
                        }
                    }
                }
                self.var_types.insert(tmp.clone(), NxType::Named(type_name));
                return tmp;
            }

            let fn_name = if p.len() == 1 && p[0].starts_with("nx_") {
                p[0].clone()
            } else if p.len() == 1 {
                format!("nx_fn_{}", p[0])
            } else {
                format!("nx_fn_{}", p.join("_"))
            };
            let args_str: Vec<_> = args.iter().map(|a| {
                match a {
                    CallArg::Positional(e) => self.gen_expr(e, None),
                    CallArg::Named { value, .. } => self.gen_expr(value, None),
                }
            }).collect();
            return format!("{}({})", fn_name, args_str.join(", "));
        }
        let callee_str = self.gen_expr(callee, None);
        let args_str: Vec<_> = args.iter().map(|a| {
            match a {
                CallArg::Positional(e) => self.gen_expr(e, None),
                CallArg::Named { value, .. } => self.gen_expr(value, None),
            }
        }).collect();
        format!("{}({})", callee_str, args_str.join(", "))
    }

    fn gen_io_call(&mut self, method: &str, args: &[CallArg]) -> String {
        match method {
            "println" => {
                if args.len() == 1 {
                    let arg_expr = match &args[0] {
                        CallArg::Positional(e) => e.clone(),
                        CallArg::Named { value, .. } => value.clone(),
                    };
                    let arg_type = self.infer_expr_type(&arg_expr);
                    let fn_name = match &arg_type {
                        NxType::Int | NxType::Long => "nx_println_int",
                        NxType::Bool => "nx_println_bool",
                        NxType::Char => "nx_println_char",
                        _ => "nx_println_string",
                    };
                    let arg_str = self.gen_expr(&arg_expr, Some(&arg_type));
                    format!("{}({})", fn_name, arg_str)
                } else {
                    "nx_println_string(\"\")".to_string()
                }
            }
            "print" => {
                if args.len() == 1 {
                    let arg_expr = match &args[0] {
                        CallArg::Positional(e) => e.clone(),
                        CallArg::Named { value, .. } => value.clone(),
                    };
                    let arg_type = self.infer_expr_type(&arg_expr);
                    let arg_str = self.gen_expr(&arg_expr, Some(&arg_type));
                    match &arg_type {
                        NxType::Int | NxType::Long => format!("nx_print_int({})", arg_str),
                        _ => format!("nx_print_string({})", arg_str),
                    }
                } else {
                    "0".to_string()
                }
            }
            "readLine" => "nx_read_line()".to_string(),
            "readFile" => {
                let path = self.get_call_arg_str(args, 0);
                format!("nx_read_file({})", path)
            }
            "writeFile" => {
                let path = self.get_call_arg_str(args, 0);
                let content = self.get_call_arg_str(args, 1);
                format!("nx_write_file({}, {})", path, content)
            }
            _ => format!("nx_fn_io_{}()", method),
        }
    }

    fn gen_method_call(&mut self, receiver: &Expr, method: &str, args: &[CallArg], _safe: bool) -> String {
        if let Expr::Path(p) = receiver {
            if p.len() == 1 && p[0] == "io" {
                return self.gen_io_call(method, args);
            }
        }

        let recv_type = self.infer_expr_type(receiver);
        let recv = self.gen_expr(receiver, Some(&recv_type));

        if matches!(recv_type, NxType::StringType) {
            return match method {
                "length" => format!("nx_string_len({})", recv),
                "charAt" => {
                    let idx = self.get_call_arg_str(args, 0);
                    format!("nx_string_char_at({}, {})", recv, idx)
                }
                "toInt" => format!("nx_string_to_int({})", recv),
                "equals" => {
                    let other = self.get_call_arg_str(args, 0);
                    format!("nx_string_equals({}, {})", recv, other)
                }
                "concat" => {
                    let other = self.get_call_arg_str(args, 0);
                    format!("nx_string_concat({}, {})", recv, other)
                }
                "substring" => {
                    let start = self.get_call_arg_str(args, 0);
                    let end = self.get_call_arg_str(args, 1);
                    format!("nx_string_substring({}, {}, {})", recv, start, end)
                }
                "startsWith" => {
                    let prefix = self.get_call_arg_str(args, 0);
                    format!("nx_string_starts_with({}, {})", recv, prefix)
                }
                "endsWith" => {
                    let suffix = self.get_call_arg_str(args, 0);
                    format!("nx_string_ends_with({}, {})", recv, suffix)
                }
                "contains" => {
                    let sub = self.get_call_arg_str(args, 0);
                    format!("nx_string_contains({}, {})", recv, sub)
                }
                "toString" => recv,
                _ => format!("nx_string_{}({})", method, recv),
            };
        }

        if let NxType::List(elem_ty) = &recv_type.clone() {
            let elem_ty = elem_ty.clone();
            return match method {
                "len" | "size" => format!("nx_list_len({})", recv),
                "isEmpty" => format!("(nx_list_len({}) == 0)", recv),
                "push" | "add" => {
                    let val = self.get_call_arg_str(args, 0);
                    let arg_ty = self.get_call_arg_type(args, 0);
                    let cast = self.void_cast_in(&arg_ty, &val);
                    format!("nx_list_push({}, {})", recv, cast)
                }
                "get" => {
                    let idx = self.get_call_arg_str(args, 0);
                    self.void_cast_out(&elem_ty, &format!("nx_list_get({}, {})", recv, idx))
                }
                "set" => {
                    let idx = self.get_call_arg_str(args, 0);
                    let val = self.get_call_arg_str(args, 1);
                    let arg_ty = self.get_call_arg_type(args, 1);
                    let cast = self.void_cast_in(&arg_ty, &val);
                    format!("nx_list_set({}, {}, {})", recv, idx, cast)
                }
                _ => format!("nx_list_{}({}, /* args */)", method, recv),
            };
        }

        if let NxType::Map(key_ty, val_ty) = &recv_type.clone() {
            let val_ty = val_ty.clone();
            let _ = key_ty;
            return match method {
                "len" | "size" => format!("nx_map_len({})", recv),
                "contains" | "containsKey" => {
                    let key = self.get_call_arg_str(args, 0);
                    format!("nx_map_contains({}, {})", recv, key)
                }
                "get" => {
                    let key = self.get_call_arg_str(args, 0);
                    self.void_cast_out(&val_ty, &format!("nx_map_get({}, {})", recv, key))
                }
                "getInt" => {
                    let key = self.get_call_arg_str(args, 0);
                    format!("(NxInt)(intptr_t)nx_map_get({}, {})", recv, key)
                }
                "insert" | "put" => {
                    let key = self.get_call_arg_str(args, 0);
                    let val = self.get_call_arg_str(args, 1);
                    let arg_ty = self.get_call_arg_type(args, 1);
                    let cast = self.void_cast_in(&arg_ty, &val);
                    format!("nx_map_insert({}, {}, {})", recv, key, cast)
                }
                _ => format!("nx_map_{}({})", method, recv),
            };
        }

        if matches!(recv_type, NxType::Int | NxType::Long) {
            if method == "toString" {
                return format!("nx_int_to_string({})", recv);
            }
        }

        if matches!(recv_type, NxType::Bool) {
            if method == "toString" {
                return format!("nx_bool_to_string({})", recv);
            }
        }

        if matches!(recv_type, NxType::Char) {
            if method == "toString" {
                return format!("nx_char_to_string({})", recv);
            }
        }

        let type_name = match &recv_type {
            NxType::Named(n) => n.clone(),
            NxType::Nullable(inner) => match inner.as_ref() {
                NxType::Named(n) => n.clone(),
                _ => "void".to_string(),
            },
            _ => "void".to_string(),
        };
        let mut all_args = vec![recv.clone()];
        for a in args {
            let s = match a {
                CallArg::Positional(e) => self.gen_expr(e, None),
                CallArg::Named { value, .. } => self.gen_expr(value, None),
            };
            all_args.push(s);
        }
        format!("nx_method_{}_{} ({})", type_name, method, all_args.join(", "))
    }

    fn infer_expr_type(&self, expr: &Expr) -> NxType {
        match expr {
            Expr::Int(_) => NxType::Int,
            Expr::Float(_) => NxType::Double,
            Expr::Bool(_) => NxType::Bool,
            Expr::CharLit(_) => NxType::Char,
            Expr::StringLit(_) => NxType::StringType,
            Expr::Null => NxType::Unknown,
            Expr::This => {
                if let Some(t) = &self.current_class {
                    NxType::Named(t.clone())
                } else {
                    NxType::Unknown
                }
            }
            Expr::Path(p) => {
                if p.len() == 1 {
                    if let Some(ty) = self.var_types.get(&p[0]) {
                        return ty.clone();
                    }
                    if let Some(ty) = self.ctx.globals.get(&p[0]) {
                        return ty.clone();
                    }
                    NxType::Unknown
                } else {
                    NxType::Unknown
                }
            }
            Expr::Call { callee, .. } => {
                if let Expr::Path(p) = callee.as_ref() {
                    if p.len() == 1 {
                        if p[0] == "List" { return NxType::List(Box::new(NxType::Unknown)); }
                        if p[0] == "Map" { return NxType::Map(Box::new(NxType::Unknown), Box::new(NxType::Unknown)); }
                        if Self::builtin_free_fn_c_name(&p[0]).is_some() {
                            return NxType::Int;
                        }
                        if let Some(fi) = self.ctx.functions.get(&p[0]) {
                            return fi.return_type.clone();
                        }
                        if self.ctx.structs.contains_key(&p[0]) {
                            return NxType::Named(p[0].clone());
                        }
                    }
                }
                NxType::Unknown
            }
            Expr::MethodCall { receiver, method, .. } | Expr::SafeCall { receiver, method, .. } => {
                self.infer_method_type(receiver, method)
            }
            Expr::FieldAccess { receiver, field } | Expr::SafeFieldAccess { receiver, field } => {
                let recv_type = self.infer_expr_type(receiver);
                if matches!(recv_type, NxType::StringType) && field == "length" {
                    return NxType::Int;
                }
                let type_name = match &recv_type {
                    NxType::Named(n) => n.clone(),
                    NxType::Nullable(inner) => match inner.as_ref() {
                        NxType::Named(n) => n.clone(),
                        _ => return NxType::Unknown,
                    },
                    _ => return NxType::Unknown,
                };
                if let Some(si) = self.ctx.structs.get(&type_name) {
                    if let Some((_, ty)) = si.fields.iter().find(|(n, _)| n == field) {
                        return ty.clone();
                    }
                }
                NxType::Unknown
            }
            Expr::Binary { op, left, right } => match op {
                BinaryOp::Add => {
                    let lt = self.infer_expr_type(left);
                    let rt = self.infer_expr_type(right);
                    if matches!(lt, NxType::StringType) || matches!(rt, NxType::StringType) {
                        NxType::StringType
                    } else {
                        NxType::Int
                    }
                }
                BinaryOp::Sub | BinaryOp::Mul | BinaryOp::Div | BinaryOp::Mod => NxType::Int,
                _ => NxType::Bool,
            },
            Expr::Unary { op, .. } => match op {
                UnaryOp::Neg => NxType::Int,
                UnaryOp::Not => NxType::Bool,
            },
            Expr::ListLit(_) => NxType::List(Box::new(NxType::Unknown)),
            Expr::Index { base, .. } => {
                if let NxType::List(elem_ty) = self.infer_expr_type(base) { *elem_ty } else { NxType::Unknown }
            }
            Expr::NullAssert(e) => {
                if let NxType::Nullable(inner) = self.infer_expr_type(e) { *inner } else { self.infer_expr_type(e) }
            }
            Expr::Elvis { left, .. } => {
                if let NxType::Nullable(inner) = self.infer_expr_type(left) { *inner } else { self.infer_expr_type(left) }
            }
            _ => NxType::Unknown,
        }
    }

    fn infer_method_type(&self, receiver: &Expr, method: &str) -> NxType {
        let recv_type = self.infer_expr_type(receiver);
        match &recv_type {
            NxType::StringType => match method {
                "length" | "toInt" => NxType::Int,
                "charAt" => NxType::Char,
                "equals" | "startsWith" | "endsWith" | "contains" => NxType::Bool,
                "concat" | "substring" | "toString" => NxType::StringType,
                _ => NxType::Unknown,
            },
            NxType::List(elem) => match method {
                "len" | "size" => NxType::Int,
                "get" => *elem.clone(),
                "isEmpty" => NxType::Bool,
                "push" | "add" | "set" => NxType::Void,
                _ => NxType::Unknown,
            },
            NxType::Map(_, v) => match method {
                "len" | "size" => NxType::Int,
                "get" => *v.clone(),
                "getInt" => NxType::Int,
                "contains" | "containsKey" => NxType::Bool,
                "insert" | "put" => NxType::Void,
                _ => NxType::Unknown,
            },
            NxType::Int | NxType::Long => match method {
                "toString" => NxType::StringType,
                _ => NxType::Unknown,
            },
            NxType::Bool => match method {
                "toString" => NxType::StringType,
                _ => NxType::Unknown,
            },
            NxType::Char => match method {
                "toString" => NxType::StringType,
                _ => NxType::Unknown,
            },
            NxType::Named(type_name) => {
                if let Some(methods) = self.ctx.methods.get(type_name.as_str()) {
                    if let Some(fi) = methods.get(method) {
                        return fi.return_type.clone();
                    }
                }
                NxType::Unknown
            }
            NxType::Nullable(inner) => {
                if let NxType::Named(type_name) = inner.as_ref() {
                    if let Some(methods) = self.ctx.methods.get(type_name.as_str()) {
                        if let Some(fi) = methods.get(method) {
                            return NxType::Nullable(Box::new(fi.return_type.clone()));
                        }
                    }
                }
                NxType::Unknown
            }
            _ => NxType::Unknown,
        }
    }

    fn nx_type_to_c(&self, ty: &NxType) -> String {
        match ty {
            NxType::Int | NxType::Long => "NxInt".to_string(),
            NxType::Float | NxType::Double => "double".to_string(),
            NxType::Bool => "NxBool".to_string(),
            NxType::Char => "NxChar".to_string(),
            NxType::StringType => "NxString".to_string(),
            NxType::Void => "void".to_string(),
            NxType::Null => "void*".to_string(),
            NxType::Named(n) => {
                if self.ctx.structs.contains_key(n) {
                    format!("NxStruct_{}*", n)
                } else {
                    "void*".to_string()
                }
            }
            NxType::List(_) => "NxList*".to_string(),
            NxType::Map(_, _) => "NxMap*".to_string(),
            NxType::Nullable(inner) => self.nx_type_to_c(inner),
            NxType::Unknown => "void*".to_string(),
        }
    }
}

pub fn generate_c(program: &Program, ctx: SemaContext) -> String {
    let gen = CGen::new(ctx);
    gen.generate(program)
}
