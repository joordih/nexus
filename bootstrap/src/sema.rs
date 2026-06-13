use std::collections::HashMap;
use crate::ast::*;

#[derive(Debug, Clone, PartialEq)]
pub enum NxType {
    Int,
    Long,
    Float,
    Double,
    Bool,
    Char,
    StringType,
    Void,
    Null,
    Named(String),
    List(Box<NxType>),
    Map(Box<NxType>, Box<NxType>),
    Nullable(Box<NxType>),
    Unknown,
}

impl std::fmt::Display for NxType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NxType::Int => write!(f, "Int"),
            NxType::Long => write!(f, "Long"),
            NxType::Float => write!(f, "Float"),
            NxType::Double => write!(f, "Double"),
            NxType::Bool => write!(f, "Bool"),
            NxType::Char => write!(f, "Char"),
            NxType::StringType => write!(f, "String"),
            NxType::Void => write!(f, "Void"),
            NxType::Null => write!(f, "null"),
            NxType::Named(n) => write!(f, "{}", n),
            NxType::List(t) => write!(f, "List<{}>", t),
            NxType::Map(k, v) => write!(f, "Map<{}, {}>", k, v),
            NxType::Nullable(t) => write!(f, "{}?", t),
            NxType::Unknown => write!(f, "?"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SemaError {
    pub msg: String,
}

impl std::fmt::Display for SemaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "error semántico: {}", self.msg)
    }
}

#[derive(Debug, Clone)]
pub struct StructInfo {
    pub fields: Vec<(String, NxType)>,
    pub is_data: bool,
}

#[derive(Debug, Clone)]
pub struct FnInfo {
    pub params: Vec<NxType>,
    pub return_type: NxType,
}

#[allow(dead_code)]
pub struct SemaContext {
    pub structs: HashMap<String, StructInfo>,
    pub functions: HashMap<String, FnInfo>,
    pub methods: HashMap<String, HashMap<String, FnInfo>>,
    pub imports: std::collections::HashSet<String>,
    pub globals: HashMap<String, NxType>,
    pub errors: Vec<SemaError>,
}

impl SemaContext {
    pub fn new() -> Self {
        SemaContext {
            structs: HashMap::new(),
            functions: HashMap::new(),
            methods: HashMap::new(),
            imports: std::collections::HashSet::new(),
            globals: HashMap::new(),
            errors: Vec::new(),
        }
    }

    fn error(&mut self, msg: String) {
        self.errors.push(SemaError { msg });
    }

    pub fn check_program(&mut self, program: &Program) {
        for item in &program.items {
            self.collect_item(item);
        }
        for item in &program.items {
            self.check_item(item);
        }
    }

    fn collect_item(&mut self, item: &Item) {
        match item {
            Item::ImportDecl(i) => {
                if let Some(last) = i.path.last() {
                    self.imports.insert(last.clone());
                }
            }
            Item::DataDecl(d) => {
                let fields: Vec<_> = d.fields.iter()
                    .map(|f| (f.name.clone(), self.resolve_type(&f.ty)))
                    .collect();
                self.structs.insert(d.name.clone(), StructInfo { fields, is_data: true });
            }
            Item::ValueDecl(v) => {
                let fields: Vec<_> = v.fields.iter()
                    .map(|f| (f.name.clone(), self.resolve_type(&f.ty)))
                    .collect();
                self.structs.insert(v.name.clone(), StructInfo { fields, is_data: true });
            }
            Item::ClassDecl(c) => {
                let mut fields: Vec<_> = c.primary_params.iter()
                    .map(|f| (f.name.clone(), self.resolve_type(&f.ty)))
                    .collect();
                for m in &c.members {
                    if let ClassMember::Field(f) = m {
                        fields.push((f.name.clone(), self.resolve_type(&f.ty)));
                    }
                }
                self.structs.insert(c.name.clone(), StructInfo { fields, is_data: false });
                for m in &c.members {
                    if let ClassMember::Method(f) = m {
                        let params: Vec<_> = f.params.iter()
                            .map(|p| self.resolve_type(&p.ty))
                            .collect();
                        let ret = f.return_type.as_ref()
                            .map(|t| self.resolve_type(t))
                            .unwrap_or(NxType::Void);
                        self.methods.entry(c.name.clone()).or_default()
                            .insert(f.name.clone(), FnInfo { params, return_type: ret });
                    }
                }
            }
            Item::InterfaceDecl(i) => {
                for m in &i.members {
                    let sig = match m {
                        InterfaceMember::AbstractMethod(s) => s,
                        InterfaceMember::DefaultMethod(f) => {
                            let params: Vec<_> = f.params.iter()
                                .map(|p| self.resolve_type(&p.ty))
                                .collect();
                            let ret = f.return_type.as_ref()
                                .map(|t| self.resolve_type(t))
                                .unwrap_or(NxType::Void);
                            self.methods.entry(i.name.clone()).or_default()
                                .insert(f.name.clone(), FnInfo { params, return_type: ret });
                            continue;
                        }
                    };
                    let params: Vec<_> = sig.params.iter()
                        .map(|p| self.resolve_type(&p.ty))
                        .collect();
                    let ret = sig.return_type.as_ref()
                        .map(|t| self.resolve_type(t))
                        .unwrap_or(NxType::Void);
                    self.methods.entry(i.name.clone()).or_default()
                        .insert(sig.name.clone(), FnInfo { params, return_type: ret });
                }
            }
            Item::Function(f) => {
                let params: Vec<_> = f.params.iter()
                    .map(|p| self.resolve_type(&p.ty))
                    .collect();
                let ret = f.return_type.as_ref()
                    .map(|t| self.resolve_type(t))
                    .unwrap_or(NxType::Void);
                self.functions.insert(f.name.clone(), FnInfo { params, return_type: ret });
            }
            Item::GlobalConst(g) => {
                let ty = g.ty.as_ref()
                    .map(|t| self.resolve_type(t))
                    .unwrap_or(NxType::Unknown);
                self.globals.insert(g.name.clone(), ty);
            }
            _ => {}
        }
    }

    fn check_item(&mut self, item: &Item) {
        match item {
            Item::Function(f) => {
                let ret = f.return_type.as_ref()
                    .map(|t| self.resolve_type(t))
                    .unwrap_or(NxType::Void);
                let mut scope = Scope::new(None);
                for p in &f.params {
                    scope.define(p.name.clone(), self.resolve_type(&p.ty));
                }
                let returns = self.check_fn_body(&f.body, &mut scope, &ret);
                if ret != NxType::Void && ret != NxType::Unknown && !returns {
                    self.error(format!("función '{}' no siempre retorna un valor de tipo {}", f.name, ret));
                }
            }
            Item::ClassDecl(c) => {
                let class_name = c.name.clone();
                for m in &c.members {
                    if let ClassMember::Method(f) = m {
                        let ret = f.return_type.as_ref()
                            .map(|t| self.resolve_type(t))
                            .unwrap_or(NxType::Void);
                        let mut scope = Scope::new(None);
                        scope.define("this".to_string(), NxType::Named(class_name.clone()));
                        for p in &c.primary_params {
                            scope.define(p.name.clone(), self.resolve_type(&p.ty));
                        }
                        for p in &f.params {
                            scope.define(p.name.clone(), self.resolve_type(&p.ty));
                        }
                        let returns = self.check_fn_body(&f.body, &mut scope, &ret);
                        if ret != NxType::Void && ret != NxType::Unknown && !returns {
                            self.error(format!("método '{}::{}' no siempre retorna un valor de tipo {}", class_name, f.name, ret));
                        }
                    }
                }
            }
            _ => {}
        }
    }

    fn check_fn_body(&mut self, body: &FunctionBody, scope: &mut Scope, expected_ret: &NxType) -> bool {
        match body {
            FunctionBody::Block(b) => self.check_block(b, scope, expected_ret),
            FunctionBody::Expr(e) => {
                let ty = self.infer_expr(e, scope);
                if !self.types_compatible(&ty, expected_ret) {
                    self.error(format!("tipo de retorno incompatible: esperado {}, obtenido {}", expected_ret, ty));
                }
                true
            }
        }
    }

    fn check_block(&mut self, block: &Block, scope: &mut Scope, expected_ret: &NxType) -> bool {
        let mut always_returns = false;
        for stmt in &block.stmts {
            if self.check_stmt(stmt, scope, expected_ret) {
                always_returns = true;
            }
        }
        always_returns
    }

    fn check_stmt(&mut self, stmt: &Stmt, scope: &mut Scope, expected_ret: &NxType) -> bool {
        match stmt {
            Stmt::Var(v) => {
                let val_ty = self.infer_expr(&v.value, scope);
                if let Some(declared_ty) = &v.ty {
                    let declared = self.resolve_type(declared_ty);
                    if !self.types_compatible(&val_ty, &declared) {
                        self.error(format!(
                            "tipo incompatible en '{}': esperado {}, obtenido {}",
                            v.name, declared, val_ty
                        ));
                    }
                    scope.define(v.name.clone(), declared);
                } else {
                    scope.define(v.name.clone(), val_ty);
                }
                false
            }
            Stmt::Return(r) => {
                let ret_ty = r.value.as_ref()
                    .map(|e| self.infer_expr(e, scope))
                    .unwrap_or(NxType::Void);
                if !self.types_compatible(&ret_ty, expected_ret) {
                    self.error(format!("tipo de retorno incompatible: esperado {}, obtenido {}", expected_ret, ret_ty));
                }
                true
            }
            Stmt::Break | Stmt::Continue => false,
            Stmt::If(i) => {
                let cond_ty = self.infer_expr(&i.cond, scope);
                if cond_ty != NxType::Bool && cond_ty != NxType::Unknown {
                    self.error(format!("condición de if debe ser Bool, obtenido {}", cond_ty));
                }
                let mut then_scope = Scope::new(Some(scope));
                let then_ret = self.check_block(&i.then_block, &mut then_scope, expected_ret);
                let else_ret = match &i.else_branch {
                    Some(ElseBranch::Block(b)) => {
                        let mut else_scope = Scope::new(Some(scope));
                        self.check_block(b, &mut else_scope, expected_ret)
                    }
                    Some(ElseBranch::If(i2)) => {
                        self.check_stmt(&Stmt::If(*i2.clone()), scope, expected_ret)
                    }
                    None => false,
                };
                then_ret && else_ret
            }
            Stmt::While(w) => {
                let cond_ty = self.infer_expr(&w.cond, scope);
                if cond_ty != NxType::Bool && cond_ty != NxType::Unknown {
                    self.error(format!("condición de while debe ser Bool, obtenido {}", cond_ty));
                }
                let mut body_scope = Scope::new(Some(scope));
                self.check_block(&w.body, &mut body_scope, expected_ret);
                false
            }
            Stmt::For(f) => {
                let iter_ty = self.infer_expr(&f.iterable, scope);
                let elem_ty = if let NxType::List(elem) = iter_ty { *elem } else { NxType::Unknown };
                let mut body_scope = Scope::new(Some(scope));
                body_scope.define(f.var.clone(), elem_ty);
                self.check_block(&f.body, &mut body_scope, expected_ret);
                false
            }
            Stmt::Switch(s) => {
                self.infer_expr(&s.subject, scope);
                let mut all_return = !s.arms.is_empty();
                for arm in &s.arms {
                    if let SwitchPattern::Case(e) = &arm.pattern {
                        self.infer_expr(e, scope);
                    }
                    let arm_ret = match &arm.body {
                        SwitchBody::Expr(e) => { self.infer_expr(e, scope); false }
                        SwitchBody::Block(b) => {
                            let mut arm_scope = Scope::new(Some(scope));
                            self.check_block(b, &mut arm_scope, expected_ret)
                        }
                    };
                    if !arm_ret { all_return = false; }
                }
                all_return
            }
            Stmt::Expr(e) => {
                self.infer_expr(&e.expr, scope);
                if let Some(assign_val) = &e.assign {
                    self.infer_expr(assign_val, scope);
                }
                false
            }
        }
    }

    fn infer_expr(&mut self, expr: &Expr, scope: &mut Scope) -> NxType {
        match expr {
            Expr::Int(_) => NxType::Int,
            Expr::Float(_) => NxType::Double,
            Expr::StringLit(_) => NxType::StringType,
            Expr::CharLit(_) => NxType::Char,
            Expr::Bool(_) => NxType::Bool,
            Expr::Null => NxType::Null,
            Expr::This => scope.lookup("this").unwrap_or(NxType::Unknown),
            Expr::Path(path) => {
                if path.len() == 1 {
                    scope.lookup(&path[0]).unwrap_or_else(|| {
                        if let Some(ty) = self.globals.get(&path[0]).cloned() {
                            return ty;
                        }
                        const BUILTINS: &[&str] = &["Map", "List", "Int", "Long", "Float",
                            "Double", "Bool", "Char", "String", "Void"];
                        if self.functions.contains_key(&path[0])
                            || self.structs.contains_key(&path[0])
                            || self.imports.contains(&path[0])
                            || BUILTINS.contains(&path[0].as_str()) {
                            NxType::Unknown
                        } else {
                            self.error(format!("nombre no resuelto: {}", path[0]));
                            NxType::Unknown
                        }
                    })
                } else {
                    NxType::Unknown
                }
            }
            Expr::Unary { op, expr } => {
                let ty = self.infer_expr(expr, scope);
                match op {
                    UnaryOp::Neg => {
                        if ty != NxType::Int && ty != NxType::Double && ty != NxType::Unknown {
                            self.error(format!("operador - requiere numérico, obtenido {}", ty));
                        }
                        ty
                    }
                    UnaryOp::Not => {
                        if ty != NxType::Bool && ty != NxType::Unknown {
                            self.error(format!("operador ! requiere Bool, obtenido {}", ty));
                        }
                        NxType::Bool
                    }
                }
            }
            Expr::Binary { op, left, right } => {
                let lt = self.infer_expr(left, scope);
                let rt = self.infer_expr(right, scope);
                match op {
                    BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul | BinaryOp::Div | BinaryOp::Mod => {
                        if lt == NxType::StringType || rt == NxType::StringType {
                            NxType::StringType
                        } else {
                            NxType::Int
                        }
                    }
                    BinaryOp::Eq | BinaryOp::Ne => NxType::Bool,
                    BinaryOp::Lt | BinaryOp::Le | BinaryOp::Gt | BinaryOp::Ge => {
                        let is_numeric = |t: &NxType| matches!(t,
                            NxType::Int | NxType::Long | NxType::Double | NxType::Float | NxType::Char | NxType::Unknown);
                        if !is_numeric(&lt) || !is_numeric(&rt) {
                            self.error(format!("comparación requiere numérico, obtenido {} y {}", lt, rt));
                        }
                        NxType::Bool
                    }
                    BinaryOp::And | BinaryOp::Or => {
                        if (lt != NxType::Bool && lt != NxType::Unknown)
                            || (rt != NxType::Bool && rt != NxType::Unknown)
                        {
                            self.error(format!("operador lógico requiere Bool, obtenido {} y {}", lt, rt));
                        }
                        NxType::Bool
                    }
                }
            }
            Expr::Call { callee, args } => {
                for a in args {
                    match a {
                        CallArg::Positional(e) => { self.infer_expr(e, scope); }
                        CallArg::Named { value, .. } => { self.infer_expr(value, scope); }
                    }
                }
                if let Expr::Path(path) = callee.as_ref() {
                    if path.len() == 1 {
                        if let Some(fi) = self.functions.get(&path[0]).cloned() {
                            return fi.return_type;
                        }
                        if self.structs.contains_key(&path[0]) {
                            return NxType::Named(path[0].clone());
                        }
                    }
                }
                NxType::Unknown
            }
            Expr::MethodCall { receiver, method, args } | Expr::SafeCall { receiver, method, args } => {
                let recv_ty = self.infer_expr(receiver, scope);
                for a in args {
                    match a {
                        CallArg::Positional(e) => { self.infer_expr(e, scope); }
                        CallArg::Named { value, .. } => { self.infer_expr(value, scope); }
                    }
                }
                let base_ty = match &recv_ty {
                    NxType::Nullable(inner) => inner.as_ref().clone(),
                    other => other.clone(),
                };
                if let NxType::Named(type_name) = &base_ty {
                    if let Some(methods) = self.methods.get(type_name).cloned() {
                        if let Some(fi) = methods.get(method.as_str()) {
                            return fi.return_type.clone();
                        }
                    }
                }
                NxType::Unknown
            }
            Expr::FieldAccess { receiver, field } | Expr::SafeFieldAccess { receiver, field } => {
                let recv_ty = self.infer_expr(receiver, scope);
                let base_ty = match &recv_ty {
                    NxType::Nullable(inner) => inner.as_ref().clone(),
                    other => other.clone(),
                };
                if let NxType::Named(type_name) = &base_ty {
                    if let Some(si) = self.structs.get(type_name).cloned() {
                        if let Some((_, ty)) = si.fields.iter().find(|(n, _)| n == field) {
                            return ty.clone();
                        } else {
                            self.error(format!("campo '{}' no existe en '{}'", field, type_name));
                        }
                    }
                }
                NxType::Unknown
            }
            Expr::Index { base, index } => {
                let base_ty = self.infer_expr(base, scope);
                self.infer_expr(index, scope);
                if let NxType::List(elem_ty) = base_ty { *elem_ty } else { NxType::Unknown }
            }
            Expr::NullAssert(e) => {
                let ty = self.infer_expr(e, scope);
                if let NxType::Nullable(inner) = ty { *inner } else { ty }
            }
            Expr::Elvis { left, right } => {
                let lt = self.infer_expr(left, scope);
                self.infer_expr(right, scope);
                if let NxType::Nullable(inner) = lt { *inner } else { lt }
            }
            Expr::Lambda { params, body } => {
                let mut lam_scope = Scope::new(Some(scope));
                for p in params {
                    lam_scope.define(p.clone(), NxType::Unknown);
                }
                self.infer_expr(body, &mut lam_scope);
                NxType::Unknown
            }
            Expr::Switch(s) => {
                self.infer_expr(&s.subject, scope);
                for arm in &s.arms {
                    if let SwitchPattern::Case(e) = &arm.pattern {
                        self.infer_expr(e, scope);
                    }
                    match &arm.body {
                        SwitchBody::Expr(e) => { self.infer_expr(e, scope); }
                        SwitchBody::Block(b) => {
                            let mut arm_scope = Scope::new(Some(scope));
                            self.check_block(b, &mut arm_scope, &NxType::Unknown);
                        }
                    }
                }
                NxType::Unknown
            }
            Expr::ListLit(elems) => {
                for e in elems { self.infer_expr(e, scope); }
                NxType::List(Box::new(NxType::Unknown))
            }
        }
    }

    fn types_compatible(&self, got: &NxType, expected: &NxType) -> bool {
        if *got == NxType::Unknown || *expected == NxType::Unknown { return true; }
        if *got == NxType::Null {
            return matches!(expected, NxType::Nullable(_));
        }
        if let NxType::Nullable(inner_exp) = expected {
            if let NxType::Nullable(inner_got) = got {
                return self.types_compatible(inner_got, inner_exp);
            }
            return self.types_compatible(got, inner_exp);
        }
        if let NxType::Nullable(inner_got) = got {
            return self.types_compatible(inner_got, expected);
        }
        got == expected
    }

    pub fn resolve_type(&self, ty: &Type) -> NxType {
        match ty {
            Type::Nullable(inner) => NxType::Nullable(Box::new(self.resolve_type(inner))),
            Type::Named { path, args } => {
                let name = path.last().cloned().unwrap_or_default();
                match name.as_str() {
                    "Int" => NxType::Int,
                    "Long" => NxType::Long,
                    "Float" => NxType::Float,
                    "Double" => NxType::Double,
                    "Bool" => NxType::Bool,
                    "Char" => NxType::Char,
                    "String" => NxType::StringType,
                    "Void" => NxType::Void,
                    "List" => {
                        let elem = args.first().map(|t| self.resolve_type(t)).unwrap_or(NxType::Unknown);
                        NxType::List(Box::new(elem))
                    }
                    "Set" => {
                        let elem = args.first().map(|t| self.resolve_type(t)).unwrap_or(NxType::Unknown);
                        NxType::List(Box::new(elem))
                    }
                    "Map" => {
                        let key = args.first().map(|t| self.resolve_type(t)).unwrap_or(NxType::Unknown);
                        let val = args.get(1).map(|t| self.resolve_type(t)).unwrap_or(NxType::Unknown);
                        NxType::Map(Box::new(key), Box::new(val))
                    }
                    _ => NxType::Named(name),
                }
            }
        }
    }
}

pub struct Scope<'a> {
    vars: HashMap<String, NxType>,
    parent: Option<&'a Scope<'a>>,
}

impl<'a> Scope<'a> {
    pub fn new(parent: Option<&'a Scope<'a>>) -> Self {
        Scope { vars: HashMap::new(), parent }
    }

    pub fn define(&mut self, name: String, ty: NxType) {
        self.vars.insert(name, ty);
    }

    pub fn lookup(&self, name: &str) -> Option<NxType> {
        if let Some(ty) = self.vars.get(name) {
            Some(ty.clone())
        } else if let Some(parent) = self.parent {
            parent.lookup(name)
        } else {
            None
        }
    }
}

pub fn check_program(program: &Program) -> Result<SemaContext, Vec<SemaError>> {
    let mut ctx = SemaContext::new();
    ctx.check_program(program);
    if ctx.errors.is_empty() {
        Ok(ctx)
    } else {
        Err(ctx.errors.clone())
    }
}
