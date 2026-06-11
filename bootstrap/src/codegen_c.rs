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

        // Forward type declarations
        let mut sorted_structs: Vec<_> = self.ctx.structs.keys().cloned().collect();
        sorted_structs.sort();
        for name in &sorted_structs {
            self.emit_line(&format!("typedef struct NxStruct_{} NxStruct_{};", name, name));
        }
        if !sorted_structs.is_empty() { self.output.push('\n'); }

        // Struct definitions
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

        // Forward function declarations
        let mut forward_decls = Vec::new();
        let items: Vec<_> = program.items.clone();
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
            self.emit_line("int main(void) {");
            self.emit_line("    nexus_init();");
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
                            let pname = format!("this->{}", p.name);
                            self.var_types.insert(p.name.clone(), self.ctx.resolve_type(&p.ty));
                            let _ = pname;
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
                self.emit_line(&format!("{} {} = ({})nx_list_get({}, {});", elem_c, f.var, elem_c, iter_tmp, idx_tmp));
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
                    format!("{} == {}", subj_tmp, case_val)
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
            Expr::FieldAccess { receiver, field } => {
                let recv = self.gen_lvalue(receiver);
                format!("{}->{}", recv, field)
            }
            _ => "_nx_invalid_lvalue".to_string(),
        }
    }

    fn gen_expr(&mut self, expr: &Expr, _hint: Option<&NxType>) -> String {
        match expr {
            Expr::Int(n) => format!("((NxInt){}LL)", n),
            Expr::Float(n) => format!("((double){})", n),
            Expr::Bool(b) => if *b { "NX_TRUE".to_string() } else { "NX_FALSE".to_string() },
            Expr::CharLit(c) => format!("((NxChar)'{}')", c),
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
                let recv = self.gen_expr(receiver, None);
                format!("{}->{}", recv, field)
            }
            Expr::SafeFieldAccess { receiver, field } => {
                let recv = self.gen_expr(receiver, None);
                // safe: if null return null else access
                format!("(({}) != NULL ? ({})->{} : NULL)", recv, recv, field)
            }
            Expr::Index { base, index } => {
                let base_str = self.gen_expr(base, None);
                let idx = self.gen_expr(index, Some(&NxType::Int));
                format!("nx_list_get({}, {})", base_str, idx)
            }
            Expr::NullAssert(e) => {
                let inner = self.gen_expr(e, None);
                format!("(({}) != NULL ? ({}) : (nx_panic(\"null assertion failed\"), ({})NULL))", inner, inner, self.nx_type_to_c(&self.infer_expr_type(e)))
            }
            Expr::Elvis { left, right } => {
                let l = self.gen_expr(left, None);
                let r = self.gen_expr(right, None);
                format!("(({}) != NULL ? ({}) : ({}))", l, l, r)
            }
            Expr::Lambda { .. } => {
                // lambdas are not directly code-generated in bootstrap; they appear as call args
                "NULL".to_string()
            }
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
                    let val = self.gen_expr(e, None);
                    self.emit_line(&format!("nx_list_push({}, (void*)(intptr_t){});", tmp, val));
                }
                tmp
            }
        }
    }

    fn gen_call(&mut self, callee: &Expr, args: &[CallArg]) -> String {
        if let Expr::Path(p) = callee {
            // io.println special case
            if p.len() == 2 && p[0] == "io" && p[1] == "println" {
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
                    return format!("{}({})", fn_name, arg_str);
                }
            }

            // Constructor call for data/class: Type(named: val, ...)
            if p.len() == 1 && self.ctx.structs.contains_key(&p[0]) {
                let type_name = p[0].clone();
                let tmp = self.fresh_tmp();
                self.emit_line(&format!("NxStruct_{}* {} = GC_MALLOC(sizeof(NxStruct_{}));", type_name, tmp, type_name));
                let struct_fields = self.ctx.structs.get(&type_name).map(|s| s.fields.clone()).unwrap_or_default();
                // named args: assign by name
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
                    // positional: match by order
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

            let fn_name = if p.len() == 1 {
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

    fn gen_method_call(&mut self, receiver: &Expr, method: &str, args: &[CallArg], _safe: bool) -> String {
        // io.println special case
        if let Expr::Path(p) = receiver {
            if p.len() == 1 && p[0] == "io" && method == "println" {
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
                    return format!("{}({})", fn_name, arg_str);
                }
            }
        }
        let recv_type = self.infer_expr_type(receiver);
        let recv = self.gen_expr(receiver, Some(&recv_type));
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
                    self.var_types.get(&p[0]).cloned().unwrap_or(NxType::Unknown)
                } else {
                    NxType::Unknown
                }
            }
            Expr::Call { callee, .. } => {
                if let Expr::Path(p) = callee.as_ref() {
                    if p.len() == 1 {
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
                let recv_type = self.infer_expr_type(receiver);
                let type_name = match &recv_type {
                    NxType::Named(n) => n.clone(),
                    NxType::Nullable(inner) => match inner.as_ref() {
                        NxType::Named(n) => n.clone(),
                        _ => return NxType::Unknown,
                    },
                    _ => return NxType::Unknown,
                };
                if let Some(methods) = self.ctx.methods.get(&type_name) {
                    if let Some(fi) = methods.get(method.as_str()) {
                        return fi.return_type.clone();
                    }
                }
                NxType::Unknown
            }
            Expr::FieldAccess { receiver, field } | Expr::SafeFieldAccess { receiver, field } => {
                let recv_type = self.infer_expr_type(receiver);
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
            Expr::Binary { op, .. } => match op {
                BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul | BinaryOp::Div | BinaryOp::Mod => NxType::Int,
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
