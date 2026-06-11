#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub items: Vec<Item>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Item {
    ModuleDecl(ModuleDecl),
    ImportDecl(ImportDecl),
    ClassDecl(ClassDecl),
    DataDecl(DataDecl),
    ValueDecl(ValueDecl),
    InterfaceDecl(InterfaceDecl),
    AnnotationDecl(AnnotationDecl),
    Function(Function),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ModuleDecl {
    pub path: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ImportDecl {
    pub path: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClassDecl {
    pub annotations: Vec<Annotation>,
    pub name: String,
    pub type_params: Vec<String>,
    pub primary_params: Vec<FieldDecl>,
    pub extends: Option<Type>,
    pub implements: Vec<Type>,
    pub members: Vec<ClassMember>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ClassMember {
    Field(FieldDecl),
    Method(Function),
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataDecl {
    pub annotations: Vec<Annotation>,
    pub name: String,
    pub fields: Vec<FieldDecl>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ValueDecl {
    pub annotations: Vec<Annotation>,
    pub name: String,
    pub fields: Vec<FieldDecl>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct InterfaceDecl {
    pub annotations: Vec<Annotation>,
    pub name: String,
    pub type_params: Vec<String>,
    pub members: Vec<InterfaceMember>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum InterfaceMember {
    AbstractMethod(FnSignature),
    DefaultMethod(Function),
}

#[derive(Debug, Clone, PartialEq)]
pub struct FnSignature {
    pub name: String,
    pub params: Vec<Param>,
    pub return_type: Option<Type>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AnnotationDecl {
    pub name: String,
    pub fields: Vec<FieldDecl>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Annotation {
    pub name: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FieldDecl {
    pub name: String,
    pub ty: Type,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Function {
    pub annotations: Vec<Annotation>,
    pub name: String,
    pub type_params: Vec<String>,
    pub params: Vec<Param>,
    pub return_type: Option<Type>,
    pub body: FunctionBody,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FunctionBody {
    Block(Block),
    Expr(Box<Expr>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Param {
    pub name: String,
    pub ty: Type,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Named { path: Vec<String>, args: Vec<Type> },
    Nullable(Box<Type>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Block {
    pub stmts: Vec<Stmt>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Var(VarStmt),
    Return(ReturnStmt),
    Break,
    Continue,
    If(IfStmt),
    While(WhileStmt),
    For(ForStmt),
    Switch(SwitchStmt),
    Expr(ExprStmt),
}

#[derive(Debug, Clone, PartialEq)]
pub struct VarStmt {
    pub is_final: bool,
    pub name: String,
    pub ty: Option<Type>,
    pub value: Expr,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReturnStmt {
    pub value: Option<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct IfStmt {
    pub cond: Expr,
    pub then_block: Block,
    pub else_branch: Option<ElseBranch>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ElseBranch {
    Block(Block),
    If(Box<IfStmt>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct WhileStmt {
    pub cond: Expr,
    pub body: Block,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForStmt {
    pub var: String,
    pub iterable: Expr,
    pub body: Block,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SwitchStmt {
    pub subject: Expr,
    pub arms: Vec<SwitchArm>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SwitchArm {
    pub pattern: SwitchPattern,
    pub body: SwitchBody,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SwitchPattern {
    Case(Expr),
    Default,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SwitchBody {
    Expr(Expr),
    Block(Block),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExprStmt {
    pub expr: Expr,
    pub assign: Option<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Int(i64),
    Float(f64),
    StringLit(String),
    CharLit(char),
    Bool(bool),
    Null,
    This,
    Path(Vec<String>),
    Unary { op: UnaryOp, expr: Box<Expr> },
    Binary { op: BinaryOp, left: Box<Expr>, right: Box<Expr> },
    Call { callee: Box<Expr>, args: Vec<CallArg> },
    MethodCall { receiver: Box<Expr>, method: String, args: Vec<CallArg> },
    SafeCall { receiver: Box<Expr>, method: String, args: Vec<CallArg> },
    FieldAccess { receiver: Box<Expr>, field: String },
    SafeFieldAccess { receiver: Box<Expr>, field: String },
    Index { base: Box<Expr>, index: Box<Expr> },
    NullAssert(Box<Expr>),
    Elvis { left: Box<Expr>, right: Box<Expr> },
    Lambda { params: Vec<String>, body: Box<Expr> },
    Switch(Box<SwitchStmt>),
    ListLit(Vec<Expr>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum CallArg {
    Positional(Expr),
    Named { name: String, value: Expr },
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOp {
    Neg,
    Not,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    And,
    Or,
}
