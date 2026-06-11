use crate::lexer::{Token, TokenKind};
use crate::ast::*;

pub struct ParseError {
    pub msg: String,
    pub line: usize,
    pub col: usize,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "error de sintaxis en {}:{}: {}", self.line, self.col, self.msg)
    }
}

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
    errors: Vec<ParseError>,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, pos: 0, errors: Vec::new() }
    }

    fn current(&self) -> &Token {
        &self.tokens[self.pos.min(self.tokens.len() - 1)]
    }

    fn peek_kind(&self) -> &TokenKind {
        &self.current().kind
    }

    fn peek_kind_at(&self, offset: usize) -> &TokenKind {
        let idx = (self.pos + offset).min(self.tokens.len() - 1);
        &self.tokens[idx].kind
    }

    fn advance(&mut self) -> &Token {
        let tok = &self.tokens[self.pos.min(self.tokens.len() - 1)];
        if self.pos < self.tokens.len() - 1 { self.pos += 1; }
        tok
    }

    fn expect(&mut self, kind: &TokenKind) -> Result<&Token, ParseError> {
        if self.peek_kind() == kind {
            Ok(self.advance())
        } else {
            let tok = self.current();
            Err(ParseError {
                msg: format!("esperado {:?}, encontrado {:?}", kind, tok.kind),
                line: tok.line,
                col: tok.col,
            })
        }
    }

    fn expect_ident(&mut self) -> Result<String, ParseError> {
        let tok = self.current().clone();
        if let TokenKind::Ident(name) = &tok.kind {
            let name = name.clone();
            self.advance();
            Ok(name)
        } else {
            Err(ParseError {
                msg: format!("esperado identificador, encontrado {:?}", tok.kind),
                line: tok.line,
                col: tok.col,
            })
        }
    }

    fn skip_semis(&mut self) {
        while *self.peek_kind() == TokenKind::Semi { self.advance(); }
    }

    fn is_stmt_start(&self) -> bool {
        matches!(self.peek_kind(),
            TokenKind::Var | TokenKind::Final | TokenKind::Return |
            TokenKind::Break | TokenKind::Continue | TokenKind::If |
            TokenKind::While | TokenKind::For | TokenKind::Switch |
            TokenKind::RBrace | TokenKind::Eof | TokenKind::At
        )
    }

    pub fn parse_program(&mut self) -> Result<Program, Vec<ParseError>> {
        let mut items = Vec::new();
        self.skip_semis();
        while *self.peek_kind() != TokenKind::Eof {
            match self.parse_item() {
                Ok(item) => items.push(item),
                Err(e) => {
                    self.errors.push(e);
                    self.recover_to_item_boundary();
                }
            }
            self.skip_semis();
        }
        if self.errors.is_empty() {
            Ok(Program { items })
        } else {
            Err(std::mem::take(&mut self.errors))
        }
    }

    fn recover_to_item_boundary(&mut self) {
        loop {
            match self.peek_kind() {
                TokenKind::Class | TokenKind::Data | TokenKind::Value |
                TokenKind::Interface | TokenKind::Annotation | TokenKind::Module |
                TokenKind::Import | TokenKind::At | TokenKind::Eof => break,
                TokenKind::Ident(_) if self.is_function_start() => break,
                _ => { self.advance(); }
            }
        }
    }

    fn is_function_start(&self) -> bool {
        if let TokenKind::Ident(_) = self.peek_kind() {
            matches!(self.peek_kind_at(1), TokenKind::LParen | TokenKind::Lt)
        } else {
            false
        }
    }

    fn parse_annotations(&mut self) -> Vec<Annotation> {
        let mut annotations = Vec::new();
        while *self.peek_kind() == TokenKind::At {
            self.advance();
            if let TokenKind::Ident(name) = self.peek_kind().clone() {
                annotations.push(Annotation { name: name.clone() });
                self.advance();
            }
        }
        annotations
    }

    fn parse_item(&mut self) -> Result<Item, ParseError> {
        let annotations = self.parse_annotations();
        match self.peek_kind() {
            TokenKind::Module => Ok(Item::ModuleDecl(self.parse_module_decl()?)),
            TokenKind::Import => Ok(Item::ImportDecl(self.parse_import_decl()?)),
            TokenKind::Class => Ok(Item::ClassDecl(self.parse_class_decl(annotations)?)),
            TokenKind::Data => Ok(Item::DataDecl(self.parse_data_decl(annotations)?)),
            TokenKind::Value => Ok(Item::ValueDecl(self.parse_value_decl(annotations)?)),
            TokenKind::Interface => Ok(Item::InterfaceDecl(self.parse_interface_decl(annotations)?)),
            TokenKind::Annotation => Ok(Item::AnnotationDecl(self.parse_annotation_decl()?)),
            TokenKind::Ident(_) => Ok(Item::Function(self.parse_function(annotations)?)),
            _ => {
                let tok = self.current().clone();
                Err(ParseError {
                    msg: format!("elemento inesperado: {:?}", tok.kind),
                    line: tok.line,
                    col: tok.col,
                })
            }
        }
    }

    fn parse_dot_path(&mut self) -> Result<Vec<String>, ParseError> {
        let mut parts = Vec::new();
        parts.push(self.expect_ident()?);
        while *self.peek_kind() == TokenKind::Dot {
            if let TokenKind::Ident(_) = self.peek_kind_at(1) {
                self.advance();
                parts.push(self.expect_ident()?);
            } else {
                break;
            }
        }
        Ok(parts)
    }

    fn parse_module_decl(&mut self) -> Result<ModuleDecl, ParseError> {
        self.expect(&TokenKind::Module)?;
        let path = self.parse_dot_path()?;
        self.skip_semis();
        Ok(ModuleDecl { path })
    }

    fn parse_import_decl(&mut self) -> Result<ImportDecl, ParseError> {
        self.expect(&TokenKind::Import)?;
        let path = self.parse_dot_path()?;
        self.skip_semis();
        Ok(ImportDecl { path })
    }

    fn parse_class_decl(&mut self, annotations: Vec<Annotation>) -> Result<ClassDecl, ParseError> {
        self.expect(&TokenKind::Class)?;
        let name = self.expect_ident()?;
        let type_params = self.parse_type_params_opt()?;

        let primary_params = if *self.peek_kind() == TokenKind::LParen {
            self.advance();
            let params = self.parse_field_list()?;
            self.expect(&TokenKind::RParen)?;
            params
        } else {
            Vec::new()
        };

        let extends = if *self.peek_kind() == TokenKind::Extends {
            self.advance();
            Some(self.parse_type()?)
        } else {
            None
        };

        let mut implements = Vec::new();
        if *self.peek_kind() == TokenKind::Implements {
            self.advance();
            implements.push(self.parse_type()?);
            while *self.peek_kind() == TokenKind::Comma {
                self.advance();
                implements.push(self.parse_type()?);
            }
        }

        self.expect(&TokenKind::LBrace)?;
        let mut members = Vec::new();
        self.skip_semis();
        while *self.peek_kind() != TokenKind::RBrace && *self.peek_kind() != TokenKind::Eof {
            let ann = self.parse_annotations();
            members.push(self.parse_class_member(ann)?);
            self.skip_semis();
        }
        self.expect(&TokenKind::RBrace)?;

        Ok(ClassDecl { annotations, name, type_params, primary_params, extends, implements, members })
    }

    fn parse_class_member(&mut self, annotations: Vec<Annotation>) -> Result<ClassMember, ParseError> {
        if let TokenKind::Ident(_) = self.peek_kind() {
            // field: Name: Type  OR  method: Name(params): RetType { ... }
            if self.is_function_start() {
                Ok(ClassMember::Method(self.parse_function(annotations)?))
            } else {
                // field declaration
                let name = self.expect_ident()?;
                self.expect(&TokenKind::Colon)?;
                let ty = self.parse_type()?;
                self.skip_semis();
                Ok(ClassMember::Field(FieldDecl { name, ty }))
            }
        } else {
            let tok = self.current().clone();
            Err(ParseError {
                msg: format!("miembro de clase inesperado: {:?}", tok.kind),
                line: tok.line,
                col: tok.col,
            })
        }
    }

    fn parse_data_decl(&mut self, annotations: Vec<Annotation>) -> Result<DataDecl, ParseError> {
        self.expect(&TokenKind::Data)?;
        let name = self.expect_ident()?;
        self.expect(&TokenKind::LBrace)?;
        let fields = self.parse_field_body()?;
        self.expect(&TokenKind::RBrace)?;
        Ok(DataDecl { annotations, name, fields })
    }

    fn parse_value_decl(&mut self, annotations: Vec<Annotation>) -> Result<ValueDecl, ParseError> {
        self.expect(&TokenKind::Value)?;
        let name = self.expect_ident()?;
        self.expect(&TokenKind::LBrace)?;
        let fields = self.parse_field_body()?;
        self.expect(&TokenKind::RBrace)?;
        Ok(ValueDecl { annotations, name, fields })
    }

    fn parse_field_body(&mut self) -> Result<Vec<FieldDecl>, ParseError> {
        let mut fields = Vec::new();
        self.skip_semis();
        while *self.peek_kind() != TokenKind::RBrace && *self.peek_kind() != TokenKind::Eof {
            let f = self.parse_single_field()?;
            fields.push(f);
            self.skip_semis();
        }
        Ok(fields)
    }

    fn parse_single_field(&mut self) -> Result<FieldDecl, ParseError> {
        let name = self.expect_ident()?;
        self.expect(&TokenKind::Colon)?;
        let ty = self.parse_type()?;
        Ok(FieldDecl { name, ty })
    }

    fn parse_field_list(&mut self) -> Result<Vec<FieldDecl>, ParseError> {
        let mut fields = Vec::new();
        if *self.peek_kind() == TokenKind::RParen { return Ok(fields); }
        fields.push(self.parse_single_field()?);
        while *self.peek_kind() == TokenKind::Comma {
            self.advance();
            if *self.peek_kind() == TokenKind::RParen { break; }
            fields.push(self.parse_single_field()?);
        }
        Ok(fields)
    }

    fn parse_interface_decl(&mut self, annotations: Vec<Annotation>) -> Result<InterfaceDecl, ParseError> {
        self.expect(&TokenKind::Interface)?;
        let name = self.expect_ident()?;
        let type_params = self.parse_type_params_opt()?;
        self.expect(&TokenKind::LBrace)?;
        let mut members = Vec::new();
        self.skip_semis();
        while *self.peek_kind() != TokenKind::RBrace && *self.peek_kind() != TokenKind::Eof {
            let ann = self.parse_annotations();
            members.push(self.parse_interface_member(ann)?);
            self.skip_semis();
        }
        self.expect(&TokenKind::RBrace)?;
        Ok(InterfaceDecl { annotations, name, type_params, members })
    }

    fn parse_interface_member(&mut self, annotations: Vec<Annotation>) -> Result<InterfaceMember, ParseError> {
        let name = self.expect_ident()?;
        let type_params = self.parse_type_params_opt()?;
        self.expect(&TokenKind::LParen)?;
        let params = self.parse_params()?;
        self.expect(&TokenKind::RParen)?;
        let return_type = if *self.peek_kind() == TokenKind::Colon {
            self.advance();
            Some(self.parse_type()?)
        } else {
            None
        };
        self.skip_semis();
        if *self.peek_kind() == TokenKind::LBrace {
            let body = self.parse_block()?;
            Ok(InterfaceMember::DefaultMethod(Function {
                annotations, name, type_params, params, return_type, body: FunctionBody::Block(body),
            }))
        } else {
            Ok(InterfaceMember::AbstractMethod(FnSignature { name, params, return_type }))
        }
    }

    fn parse_annotation_decl(&mut self) -> Result<AnnotationDecl, ParseError> {
        self.expect(&TokenKind::Annotation)?;
        let name = self.expect_ident()?;
        let fields = if *self.peek_kind() == TokenKind::LBrace {
            self.advance();
            let f = self.parse_field_body()?;
            self.expect(&TokenKind::RBrace)?;
            f
        } else {
            Vec::new()
        };
        Ok(AnnotationDecl { name, fields })
    }

    fn parse_type_params_opt(&mut self) -> Result<Vec<String>, ParseError> {
        if *self.peek_kind() != TokenKind::Lt { return Ok(Vec::new()); }
        self.advance();
        let mut params = Vec::new();
        params.push(self.parse_type_param_name()?);
        while *self.peek_kind() == TokenKind::Comma {
            self.advance();
            if *self.peek_kind() == TokenKind::Gt { break; }
            params.push(self.parse_type_param_name()?);
        }
        self.expect(&TokenKind::Gt)?;
        Ok(params)
    }

    fn parse_type_param_name(&mut self) -> Result<String, ParseError> {
        let name = self.expect_ident()?;
        if *self.peek_kind() == TokenKind::Extends {
            self.advance();
            self.parse_type()?; // consume bound, discard for now
        }
        Ok(name)
    }

    fn parse_function(&mut self, annotations: Vec<Annotation>) -> Result<Function, ParseError> {
        let name = self.expect_ident()?;
        let type_params = self.parse_type_params_opt()?;
        self.expect(&TokenKind::LParen)?;
        let params = self.parse_params()?;
        self.expect(&TokenKind::RParen)?;
        let return_type = if *self.peek_kind() == TokenKind::Colon {
            self.advance();
            Some(self.parse_type()?)
        } else {
            None
        };
        let body = if *self.peek_kind() == TokenKind::FatArrow {
            self.advance();
            FunctionBody::Expr(Box::new(self.parse_expr(false)?))
        } else {
            FunctionBody::Block(self.parse_block()?)
        };
        Ok(Function { annotations, name, type_params, params, return_type, body })
    }

    fn parse_params(&mut self) -> Result<Vec<Param>, ParseError> {
        let mut params = Vec::new();
        if *self.peek_kind() == TokenKind::RParen { return Ok(params); }
        params.push(self.parse_param()?);
        while *self.peek_kind() == TokenKind::Comma {
            self.advance();
            if *self.peek_kind() == TokenKind::RParen { break; }
            params.push(self.parse_param()?);
        }
        Ok(params)
    }

    fn parse_param(&mut self) -> Result<Param, ParseError> {
        let name = self.expect_ident()?;
        self.expect(&TokenKind::Colon)?;
        let ty = self.parse_type()?;
        Ok(Param { name, ty })
    }

    fn parse_type(&mut self) -> Result<Type, ParseError> {
        let path = self.parse_dot_path()?;
        let args = if *self.peek_kind() == TokenKind::Lt {
            self.advance();
            let mut args = Vec::new();
            args.push(self.parse_type()?);
            while *self.peek_kind() == TokenKind::Comma {
                self.advance();
                if *self.peek_kind() == TokenKind::Gt { break; }
                args.push(self.parse_type()?);
            }
            self.expect(&TokenKind::Gt)?;
            args
        } else {
            Vec::new()
        };
        let base = Type::Named { path, args };
        if *self.peek_kind() == TokenKind::Question {
            self.advance();
            Ok(Type::Nullable(Box::new(base)))
        } else {
            Ok(base)
        }
    }

    fn parse_block(&mut self) -> Result<Block, ParseError> {
        self.expect(&TokenKind::LBrace)?;
        let mut stmts = Vec::new();
        self.skip_semis();
        while *self.peek_kind() != TokenKind::RBrace && *self.peek_kind() != TokenKind::Eof {
            stmts.push(self.parse_stmt()?);
            self.skip_semis();
        }
        self.expect(&TokenKind::RBrace)?;
        Ok(Block { stmts })
    }

    fn parse_stmt(&mut self) -> Result<Stmt, ParseError> {
        match self.peek_kind() {
            TokenKind::Var => Ok(Stmt::Var(self.parse_var_stmt(false)?)),
            TokenKind::Final => Ok(Stmt::Var(self.parse_var_stmt(true)?)),
            TokenKind::Return => Ok(Stmt::Return(self.parse_return_stmt()?)),
            TokenKind::Break => { self.advance(); self.skip_semis(); Ok(Stmt::Break) }
            TokenKind::Continue => { self.advance(); self.skip_semis(); Ok(Stmt::Continue) }
            TokenKind::If => Ok(Stmt::If(self.parse_if_stmt()?)),
            TokenKind::While => Ok(Stmt::While(self.parse_while_stmt()?)),
            TokenKind::For => Ok(Stmt::For(self.parse_for_stmt()?)),
            TokenKind::Switch => Ok(Stmt::Switch(self.parse_switch_stmt()?)),
            _ => Ok(Stmt::Expr(self.parse_expr_stmt()?)),
        }
    }

    fn parse_var_stmt(&mut self, is_final: bool) -> Result<VarStmt, ParseError> {
        self.advance(); // consume var/final
        let name = self.expect_ident()?;
        let ty = if *self.peek_kind() == TokenKind::Colon {
            self.advance();
            Some(self.parse_type()?)
        } else {
            None
        };
        self.expect(&TokenKind::Eq)?;
        let value = self.parse_expr(false)?;
        self.skip_semis();
        Ok(VarStmt { is_final, name, ty, value })
    }

    fn parse_return_stmt(&mut self) -> Result<ReturnStmt, ParseError> {
        self.expect(&TokenKind::Return)?;
        let value = if !self.is_stmt_start() && *self.peek_kind() != TokenKind::Semi {
            Some(self.parse_expr(false)?)
        } else {
            None
        };
        self.skip_semis();
        Ok(ReturnStmt { value })
    }

    fn parse_if_stmt(&mut self) -> Result<IfStmt, ParseError> {
        self.expect(&TokenKind::If)?;
        self.expect(&TokenKind::LParen)?;
        let cond = self.parse_expr(false)?;
        self.expect(&TokenKind::RParen)?;
        let then_block = self.parse_block()?;
        let else_branch = if *self.peek_kind() == TokenKind::Else {
            self.advance();
            if *self.peek_kind() == TokenKind::If {
                Some(ElseBranch::If(Box::new(self.parse_if_stmt()?)))
            } else {
                Some(ElseBranch::Block(self.parse_block()?))
            }
        } else {
            None
        };
        Ok(IfStmt { cond, then_block, else_branch })
    }

    fn parse_while_stmt(&mut self) -> Result<WhileStmt, ParseError> {
        self.expect(&TokenKind::While)?;
        self.expect(&TokenKind::LParen)?;
        let cond = self.parse_expr(false)?;
        self.expect(&TokenKind::RParen)?;
        let body = self.parse_block()?;
        Ok(WhileStmt { cond, body })
    }

    fn parse_for_stmt(&mut self) -> Result<ForStmt, ParseError> {
        self.expect(&TokenKind::For)?;
        let var = self.expect_ident()?;
        self.expect(&TokenKind::In)?;
        let iterable = self.parse_expr(false)?;
        let body = self.parse_block()?;
        Ok(ForStmt { var, iterable, body })
    }

    fn parse_switch_stmt(&mut self) -> Result<SwitchStmt, ParseError> {
        self.expect(&TokenKind::Switch)?;
        self.expect(&TokenKind::LParen)?;
        let subject = self.parse_expr(false)?;
        self.expect(&TokenKind::RParen)?;
        self.expect(&TokenKind::LBrace)?;
        let mut arms = Vec::new();
        self.skip_semis();
        while *self.peek_kind() != TokenKind::RBrace && *self.peek_kind() != TokenKind::Eof {
            let arm = self.parse_switch_arm()?;
            arms.push(arm);
            self.skip_semis();
        }
        self.expect(&TokenKind::RBrace)?;
        Ok(SwitchStmt { subject, arms })
    }

    fn parse_switch_arm(&mut self) -> Result<SwitchArm, ParseError> {
        let pattern = if *self.peek_kind() == TokenKind::Default {
            self.advance();
            SwitchPattern::Default
        } else {
            self.expect(&TokenKind::Case)?;
            let expr = self.parse_expr(true)?;
            SwitchPattern::Case(expr)
        };
        self.expect(&TokenKind::Arrow)?;
        let body = if *self.peek_kind() == TokenKind::LBrace {
            SwitchBody::Block(self.parse_block()?)
        } else {
            SwitchBody::Expr(self.parse_expr(false)?)
        };
        self.skip_semis();
        Ok(SwitchArm { pattern, body })
    }

    fn parse_expr_stmt(&mut self) -> Result<ExprStmt, ParseError> {
        let expr = self.parse_expr(false)?;
        let assign = if *self.peek_kind() == TokenKind::Eq {
            self.advance();
            Some(self.parse_expr(false)?)
        } else {
            None
        };
        self.skip_semis();
        Ok(ExprStmt { expr, assign })
    }

    fn parse_expr(&mut self, no_struct_init: bool) -> Result<Expr, ParseError> {
        self.parse_elvis(no_struct_init)
    }

    fn parse_elvis(&mut self, no_struct_init: bool) -> Result<Expr, ParseError> {
        let left = self.parse_or_expr(no_struct_init)?;
        if *self.peek_kind() == TokenKind::QuestionColon {
            self.advance();
            let right = self.parse_or_expr(no_struct_init)?;
            Ok(Expr::Elvis { left: Box::new(left), right: Box::new(right) })
        } else {
            Ok(left)
        }
    }

    fn parse_or_expr(&mut self, no_struct_init: bool) -> Result<Expr, ParseError> {
        let mut left = self.parse_and_expr(no_struct_init)?;
        while *self.peek_kind() == TokenKind::PipePipe {
            self.advance();
            let right = self.parse_and_expr(no_struct_init)?;
            left = Expr::Binary { op: BinaryOp::Or, left: Box::new(left), right: Box::new(right) };
        }
        Ok(left)
    }

    fn parse_and_expr(&mut self, no_struct_init: bool) -> Result<Expr, ParseError> {
        let mut left = self.parse_equality(no_struct_init)?;
        while *self.peek_kind() == TokenKind::AmpAmp {
            self.advance();
            let right = self.parse_equality(no_struct_init)?;
            left = Expr::Binary { op: BinaryOp::And, left: Box::new(left), right: Box::new(right) };
        }
        Ok(left)
    }

    fn parse_equality(&mut self, no_struct_init: bool) -> Result<Expr, ParseError> {
        let mut left = self.parse_comparison(no_struct_init)?;
        loop {
            let op = match self.peek_kind() {
                TokenKind::EqEq => BinaryOp::Eq,
                TokenKind::BangEq => BinaryOp::Ne,
                _ => break,
            };
            self.advance();
            let right = self.parse_comparison(no_struct_init)?;
            left = Expr::Binary { op, left: Box::new(left), right: Box::new(right) };
        }
        Ok(left)
    }

    fn parse_comparison(&mut self, no_struct_init: bool) -> Result<Expr, ParseError> {
        let mut left = self.parse_term(no_struct_init)?;
        loop {
            let op = match self.peek_kind() {
                TokenKind::Lt => BinaryOp::Lt,
                TokenKind::LtEq => BinaryOp::Le,
                TokenKind::Gt => BinaryOp::Gt,
                TokenKind::GtEq => BinaryOp::Ge,
                _ => break,
            };
            self.advance();
            let right = self.parse_term(no_struct_init)?;
            left = Expr::Binary { op, left: Box::new(left), right: Box::new(right) };
        }
        Ok(left)
    }

    fn parse_term(&mut self, no_struct_init: bool) -> Result<Expr, ParseError> {
        let mut left = self.parse_factor(no_struct_init)?;
        loop {
            let op = match self.peek_kind() {
                TokenKind::Plus => BinaryOp::Add,
                TokenKind::Minus => BinaryOp::Sub,
                _ => break,
            };
            self.advance();
            let right = self.parse_factor(no_struct_init)?;
            left = Expr::Binary { op, left: Box::new(left), right: Box::new(right) };
        }
        Ok(left)
    }

    fn parse_factor(&mut self, no_struct_init: bool) -> Result<Expr, ParseError> {
        let mut left = self.parse_unary(no_struct_init)?;
        loop {
            let op = match self.peek_kind() {
                TokenKind::Star => BinaryOp::Mul,
                TokenKind::Slash => BinaryOp::Div,
                TokenKind::Percent => BinaryOp::Mod,
                _ => break,
            };
            self.advance();
            let right = self.parse_unary(no_struct_init)?;
            left = Expr::Binary { op, left: Box::new(left), right: Box::new(right) };
        }
        Ok(left)
    }

    fn parse_unary(&mut self, no_struct_init: bool) -> Result<Expr, ParseError> {
        match self.peek_kind() {
            TokenKind::Bang => {
                self.advance();
                let expr = self.parse_unary(no_struct_init)?;
                Ok(Expr::Unary { op: UnaryOp::Not, expr: Box::new(expr) })
            }
            TokenKind::Minus => {
                self.advance();
                let expr = self.parse_unary(no_struct_init)?;
                Ok(Expr::Unary { op: UnaryOp::Neg, expr: Box::new(expr) })
            }
            _ => self.parse_postfix(no_struct_init),
        }
    }

    fn parse_postfix(&mut self, no_struct_init: bool) -> Result<Expr, ParseError> {
        let mut expr = self.parse_primary(no_struct_init)?;
        loop {
            match self.peek_kind() {
                TokenKind::Dot => {
                    self.advance();
                    let field = self.expect_ident()?;
                    if *self.peek_kind() == TokenKind::LParen {
                        let args = self.parse_call_args()?;
                        expr = Expr::MethodCall { receiver: Box::new(expr), method: field, args };
                    } else {
                        expr = Expr::FieldAccess { receiver: Box::new(expr), field };
                    }
                }
                TokenKind::QuestionDot => {
                    self.advance();
                    let field = self.expect_ident()?;
                    if *self.peek_kind() == TokenKind::LParen {
                        let args = self.parse_call_args()?;
                        expr = Expr::SafeCall { receiver: Box::new(expr), method: field, args };
                    } else {
                        expr = Expr::SafeFieldAccess { receiver: Box::new(expr), field };
                    }
                }
                TokenKind::LParen => {
                    let args = self.parse_call_args()?;
                    expr = Expr::Call { callee: Box::new(expr), args };
                }
                TokenKind::LBracket => {
                    self.advance();
                    let index = self.parse_expr(false)?;
                    self.expect(&TokenKind::RBracket)?;
                    expr = Expr::Index { base: Box::new(expr), index: Box::new(index) };
                }
                TokenKind::Bang => {
                    // null assert (postfix !)
                    self.advance();
                    expr = Expr::NullAssert(Box::new(expr));
                }
                _ => break,
            }
        }
        Ok(expr)
    }

    fn parse_call_args(&mut self) -> Result<Vec<CallArg>, ParseError> {
        self.expect(&TokenKind::LParen)?;
        let mut args = Vec::new();
        if *self.peek_kind() != TokenKind::RParen {
            args.push(self.parse_call_arg()?);
            while *self.peek_kind() == TokenKind::Comma {
                self.advance();
                if *self.peek_kind() == TokenKind::RParen { break; }
                args.push(self.parse_call_arg()?);
            }
        }
        self.expect(&TokenKind::RParen)?;
        Ok(args)
    }

    fn parse_call_arg(&mut self) -> Result<CallArg, ParseError> {
        // named arg: IDENT COLON expr
        if let TokenKind::Ident(name) = self.peek_kind().clone() {
            if *self.peek_kind_at(1) == TokenKind::Colon {
                let name = name.clone();
                self.advance(); // consume ident
                self.advance(); // consume colon
                let value = self.parse_expr(false)?;
                return Ok(CallArg::Named { name, value });
            }
        }
        Ok(CallArg::Positional(self.parse_expr(false)?))
    }

    fn is_lambda_paren(&self) -> bool {
        // Check: ( IDENT (, IDENT)* ) =>
        let mut i = self.pos + 1; // skip (
        let len = self.tokens.len();
        loop {
            if i >= len { return false; }
            match &self.tokens[i].kind {
                TokenKind::RParen => {
                    i += 1;
                    return i < len && matches!(&self.tokens[i].kind, TokenKind::FatArrow);
                }
                TokenKind::Ident(_) => {
                    i += 1;
                    if i < len {
                        match &self.tokens[i].kind {
                            TokenKind::Comma => { i += 1; }
                            TokenKind::RParen => {}
                            _ => return false,
                        }
                    }
                }
                _ => return false,
            }
        }
    }

    fn parse_primary(&mut self, _no_struct_init: bool) -> Result<Expr, ParseError> {
        match self.peek_kind().clone() {
            TokenKind::Int(n) => { self.advance(); Ok(Expr::Int(n)) }
            TokenKind::Float(n) => { self.advance(); Ok(Expr::Float(n)) }
            TokenKind::StringLit(s) => { self.advance(); Ok(Expr::StringLit(s)) }
            TokenKind::CharLit(c) => { self.advance(); Ok(Expr::CharLit(c)) }
            TokenKind::True => { self.advance(); Ok(Expr::Bool(true)) }
            TokenKind::False => { self.advance(); Ok(Expr::Bool(false)) }
            TokenKind::Null => { self.advance(); Ok(Expr::Null) }
            TokenKind::This => { self.advance(); Ok(Expr::This) }
            TokenKind::Switch => {
                let s = self.parse_switch_stmt()?;
                Ok(Expr::Switch(Box::new(s)))
            }
            TokenKind::LBracket => {
                self.advance();
                let mut elems = Vec::new();
                if *self.peek_kind() != TokenKind::RBracket {
                    elems.push(self.parse_expr(false)?);
                    while *self.peek_kind() == TokenKind::Comma {
                        self.advance();
                        if *self.peek_kind() == TokenKind::RBracket { break; }
                        elems.push(self.parse_expr(false)?);
                    }
                }
                self.expect(&TokenKind::RBracket)?;
                Ok(Expr::ListLit(elems))
            }
            TokenKind::LParen => {
                if self.is_lambda_paren() {
                    // lambda: (params) => expr
                    self.advance(); // consume (
                    let mut params = Vec::new();
                    while let TokenKind::Ident(name) = self.peek_kind().clone() {
                        params.push(name.clone());
                        self.advance();
                        if *self.peek_kind() == TokenKind::Comma { self.advance(); }
                    }
                    self.expect(&TokenKind::RParen)?;
                    self.expect(&TokenKind::FatArrow)?;
                    let body = self.parse_expr(false)?;
                    Ok(Expr::Lambda { params, body: Box::new(body) })
                } else {
                    self.advance();
                    let expr = self.parse_expr(false)?;
                    self.expect(&TokenKind::RParen)?;
                    Ok(expr)
                }
            }
            TokenKind::Ident(_) => {
                // Check for single-param lambda: IDENT =>
                if matches!(self.peek_kind_at(1), TokenKind::FatArrow) {
                    let name = self.expect_ident()?;
                    self.advance(); // consume =>
                    let body = self.parse_expr(false)?;
                    return Ok(Expr::Lambda { params: vec![name], body: Box::new(body) });
                }
                // Single identifier — dots handled by postfix loop as field/method access
                let name = self.expect_ident()?;
                Ok(Expr::Path(vec![name]))
            }
            _ => {
                let tok = self.current().clone();
                Err(ParseError {
                    msg: format!("expresión inesperada: {:?}", tok.kind),
                    line: tok.line,
                    col: tok.col,
                })
            }
        }
    }

    pub fn take_errors(&mut self) -> Vec<ParseError> {
        std::mem::take(&mut self.errors)
    }
}
