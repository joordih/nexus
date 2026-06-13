use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    Ident(String),
    Int(i64),
    Float(f64),
    StringLit(String),
    CharLit(char),
    True,
    False,
    Null,
    Var,
    Final,
    Return,
    If,
    Else,
    While,
    Break,
    Continue,
    For,
    In,
    Switch,
    Case,
    Default,
    Class,
    Data,
    Value,
    Interface,
    Extends,
    Implements,
    Annotation,
    Module,
    Import,
    This,
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    EqEq,
    BangEq,
    Lt,
    LtEq,
    Gt,
    GtEq,
    AmpAmp,
    PipePipe,
    Bang,
    Eq,
    Arrow,
    FatArrow,
    Question,
    QuestionDot,
    QuestionColon,
    At,
    Dot,
    Comma,
    Colon,
    Semi,
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    Eof,
}

impl fmt::Display for TokenKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TokenKind::Ident(s) => write!(f, "Ident({})", s),
            TokenKind::Int(n) => write!(f, "Int({})", n),
            TokenKind::Float(n) => write!(f, "Float({})", n),
            TokenKind::StringLit(s) => write!(f, "String({:?})", s),
            TokenKind::CharLit(c) => write!(f, "Char({:?})", c),
            TokenKind::True => write!(f, "true"),
            TokenKind::False => write!(f, "false"),
            TokenKind::Null => write!(f, "null"),
            TokenKind::Var => write!(f, "var"),
            TokenKind::Final => write!(f, "final"),
            TokenKind::Return => write!(f, "return"),
            TokenKind::If => write!(f, "if"),
            TokenKind::Else => write!(f, "else"),
            TokenKind::While => write!(f, "while"),
            TokenKind::Break => write!(f, "break"),
            TokenKind::Continue => write!(f, "continue"),
            TokenKind::For => write!(f, "for"),
            TokenKind::In => write!(f, "in"),
            TokenKind::Switch => write!(f, "switch"),
            TokenKind::Case => write!(f, "case"),
            TokenKind::Default => write!(f, "default"),
            TokenKind::Class => write!(f, "class"),
            TokenKind::Data => write!(f, "data"),
            TokenKind::Value => write!(f, "value"),
            TokenKind::Interface => write!(f, "interface"),
            TokenKind::Extends => write!(f, "extends"),
            TokenKind::Implements => write!(f, "implements"),
            TokenKind::Annotation => write!(f, "annotation"),
            TokenKind::Module => write!(f, "module"),
            TokenKind::Import => write!(f, "import"),
            TokenKind::This => write!(f, "this"),
            TokenKind::Plus => write!(f, "+"),
            TokenKind::Minus => write!(f, "-"),
            TokenKind::Star => write!(f, "*"),
            TokenKind::Slash => write!(f, "/"),
            TokenKind::Percent => write!(f, "%"),
            TokenKind::EqEq => write!(f, "=="),
            TokenKind::BangEq => write!(f, "!="),
            TokenKind::Lt => write!(f, "<"),
            TokenKind::LtEq => write!(f, "<="),
            TokenKind::Gt => write!(f, ">"),
            TokenKind::GtEq => write!(f, ">="),
            TokenKind::AmpAmp => write!(f, "&&"),
            TokenKind::PipePipe => write!(f, "||"),
            TokenKind::Bang => write!(f, "!"),
            TokenKind::Eq => write!(f, "="),
            TokenKind::Arrow => write!(f, "->"),
            TokenKind::FatArrow => write!(f, "=>"),
            TokenKind::Question => write!(f, "?"),
            TokenKind::QuestionDot => write!(f, "?."),
            TokenKind::QuestionColon => write!(f, "?:"),
            TokenKind::At => write!(f, "@"),
            TokenKind::Dot => write!(f, "."),
            TokenKind::Comma => write!(f, ","),
            TokenKind::Colon => write!(f, ":"),
            TokenKind::Semi => write!(f, ";"),
            TokenKind::LParen => write!(f, "("),
            TokenKind::RParen => write!(f, ")"),
            TokenKind::LBrace => write!(f, "{{"),
            TokenKind::RBrace => write!(f, "}}"),
            TokenKind::LBracket => write!(f, "["),
            TokenKind::RBracket => write!(f, "]"),
            TokenKind::Eof => write!(f, "EOF"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub line: usize,
    pub col: usize,
}

impl Token {
    pub fn new(kind: TokenKind, line: usize, col: usize) -> Self {
        Token { kind, line, col }
    }
}

pub struct Lexer {
    source: Vec<char>,
    pos: usize,
    line: usize,
    col: usize,
}

impl Lexer {
    pub fn new(source: &str) -> Self {
        Lexer { source: source.chars().collect(), pos: 0, line: 1, col: 1 }
    }

    fn current(&self) -> Option<char> {
        self.source.get(self.pos).copied()
    }

    fn peek(&self) -> Option<char> {
        self.source.get(self.pos + 1).copied()
    }

    fn advance(&mut self) -> Option<char> {
        let ch = self.source.get(self.pos).copied();
        if let Some(c) = ch {
            self.pos += 1;
            if c == '\n' { self.line += 1; self.col = 1; } else { self.col += 1; }
        }
        ch
    }

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.current() {
            if c == ' ' || c == '\t' || c == '\r' || c == '\n' { self.advance(); } else { break; }
        }
    }

    fn skip_line_comment(&mut self) {
        while let Some(c) = self.current() {
            if c == '\n' { break; }
            self.advance();
        }
    }

    fn skip_block_comment(&mut self) {
        while let Some(c) = self.current() {
            if c == '*' && self.peek() == Some('/') { self.advance(); self.advance(); return; }
            self.advance();
        }
    }

    fn read_string(&mut self, line: usize, col: usize) -> Result<Token, LexError> {
        let mut s = String::new();
        loop {
            match self.current() {
                None => return Err(LexError { msg: "cadena sin cerrar".to_string(), line, col }),
                Some('"') => { self.advance(); return Ok(Token::new(TokenKind::StringLit(s), line, col)); }
                Some('\\') => {
                    self.advance();
                    match self.current() {
                        Some('"') => { s.push('"'); self.advance(); }
                        Some('\'') => { s.push('\''); self.advance(); }
                        Some('\\') => { s.push('\\'); self.advance(); }
                        Some('n') => { s.push('\n'); self.advance(); }
                        Some('t') => { s.push('\t'); self.advance(); }
                        Some('r') => { s.push('\r'); self.advance(); }
                        Some('0') => { s.push('\0'); self.advance(); }
                        Some(c) => return Err(LexError { msg: format!("escape desconocido: \\{}", c), line, col }),
                        None => return Err(LexError { msg: "cadena sin cerrar".to_string(), line, col }),
                    }
                }
                Some(c) => { s.push(c); self.advance(); }
            }
        }
    }

    fn read_char_literal(&mut self, line: usize, col: usize) -> Result<Token, LexError> {
        let ch = match self.current() {
            None => return Err(LexError { msg: "literal de carácter sin cerrar".to_string(), line, col }),
            Some('\\') => {
                self.advance();
                match self.current() {
                    Some('\'') => { self.advance(); '\'' }
                    Some('\\') => { self.advance(); '\\' }
                    Some('n') => { self.advance(); '\n' }
                    Some('t') => { self.advance(); '\t' }
                    Some('r') => { self.advance(); '\r' }
                    Some('0') => { self.advance(); '\0' }
                    Some(c) => return Err(LexError { msg: format!("escape desconocido: \\{}", c), line, col }),
                    None => return Err(LexError { msg: "literal de carácter sin cerrar".to_string(), line, col }),
                }
            }
            Some(c) => { self.advance(); c }
        };
        match self.current() {
            Some('\'') => { self.advance(); Ok(Token::new(TokenKind::CharLit(ch), line, col)) }
            _ => Err(LexError { msg: "literal de carácter sin cerrar".to_string(), line, col }),
        }
    }

    fn read_ident_or_keyword(&mut self, first: char, line: usize, col: usize) -> Token {
        let mut s = String::new();
        s.push(first);
        while let Some(c) = self.current() {
            if c.is_ascii_alphanumeric() || c == '_' { s.push(c); self.advance(); } else { break; }
        }
        let kind = match s.as_str() {
            "var" => TokenKind::Var,
            "final" => TokenKind::Final,
            "return" => TokenKind::Return,
            "if" => TokenKind::If,
            "else" => TokenKind::Else,
            "while" => TokenKind::While,
            "break" => TokenKind::Break,
            "continue" => TokenKind::Continue,
            "for" => TokenKind::For,
            "in" => TokenKind::In,
            "switch" => TokenKind::Switch,
            "case" => TokenKind::Case,
            "default" => TokenKind::Default,
            "class" => TokenKind::Class,
            "data" => TokenKind::Data,
            "value" => TokenKind::Value,
            "interface" => TokenKind::Interface,
            "extends" => TokenKind::Extends,
            "implements" => TokenKind::Implements,
            "annotation" => TokenKind::Annotation,
            "module" => TokenKind::Module,
            "import" => TokenKind::Import,
            "this" => TokenKind::This,
            "true" => TokenKind::True,
            "false" => TokenKind::False,
            "null" => TokenKind::Null,
            _ => TokenKind::Ident(s),
        };
        Token::new(kind, line, col)
    }

    fn read_number(&mut self, first: char, line: usize, col: usize) -> Token {
        let mut s = String::new();
        s.push(first);
        while let Some(c) = self.current() {
            if c.is_ascii_digit() { s.push(c); self.advance(); } else { break; }
        }
        if self.current() == Some('.') && self.peek().map(|c| c.is_ascii_digit()).unwrap_or(false) {
            s.push('.');
            self.advance();
            while let Some(c) = self.current() {
                if c.is_ascii_digit() { s.push(c); self.advance(); } else { break; }
            }
            let n: f64 = s.parse().unwrap_or(0.0);
            Token::new(TokenKind::Float(n), line, col)
        } else {
            let n: i64 = s.parse().unwrap_or(0);
            Token::new(TokenKind::Int(n), line, col)
        }
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, LexError> {
        let mut tokens = Vec::new();
        loop {
            self.skip_whitespace();
            let line = self.line;
            let col = self.col;
            match self.current() {
                None => { tokens.push(Token::new(TokenKind::Eof, line, col)); break; }
                Some('/') if self.peek() == Some('/') => { self.advance(); self.advance(); self.skip_line_comment(); }
                Some('/') if self.peek() == Some('*') => { self.advance(); self.advance(); self.skip_block_comment(); }
                Some('"') => { self.advance(); tokens.push(self.read_string(line, col)?); }
                Some('\'') => { self.advance(); tokens.push(self.read_char_literal(line, col)?); }
                Some(c) if c.is_ascii_alphabetic() || c == '_' => {
                    self.advance();
                    tokens.push(self.read_ident_or_keyword(c, line, col));
                }
                Some(c) if c.is_ascii_digit() => { self.advance(); tokens.push(self.read_number(c, line, col)); }
                Some('+') => { self.advance(); tokens.push(Token::new(TokenKind::Plus, line, col)); }
                Some('-') => {
                    self.advance();
                    if self.current() == Some('>') { self.advance(); tokens.push(Token::new(TokenKind::Arrow, line, col)); }
                    else { tokens.push(Token::new(TokenKind::Minus, line, col)); }
                }
                Some('*') => { self.advance(); tokens.push(Token::new(TokenKind::Star, line, col)); }
                Some('/') => { self.advance(); tokens.push(Token::new(TokenKind::Slash, line, col)); }
                Some('%') => { self.advance(); tokens.push(Token::new(TokenKind::Percent, line, col)); }
                Some('=') => {
                    self.advance();
                    if self.current() == Some('=') { self.advance(); tokens.push(Token::new(TokenKind::EqEq, line, col)); }
                    else if self.current() == Some('>') { self.advance(); tokens.push(Token::new(TokenKind::FatArrow, line, col)); }
                    else { tokens.push(Token::new(TokenKind::Eq, line, col)); }
                }
                Some('!') => {
                    self.advance();
                    if self.current() == Some('=') { self.advance(); tokens.push(Token::new(TokenKind::BangEq, line, col)); }
                    else { tokens.push(Token::new(TokenKind::Bang, line, col)); }
                }
                Some('<') => {
                    self.advance();
                    if self.current() == Some('=') { self.advance(); tokens.push(Token::new(TokenKind::LtEq, line, col)); }
                    else { tokens.push(Token::new(TokenKind::Lt, line, col)); }
                }
                Some('>') => {
                    self.advance();
                    if self.current() == Some('=') { self.advance(); tokens.push(Token::new(TokenKind::GtEq, line, col)); }
                    else { tokens.push(Token::new(TokenKind::Gt, line, col)); }
                }
                Some('&') => {
                    self.advance();
                    if self.current() == Some('&') { self.advance(); tokens.push(Token::new(TokenKind::AmpAmp, line, col)); }
                    else { return Err(LexError { msg: "carácter ilegal: &".to_string(), line, col }); }
                }
                Some('|') => {
                    self.advance();
                    if self.current() == Some('|') { self.advance(); tokens.push(Token::new(TokenKind::PipePipe, line, col)); }
                    else { return Err(LexError { msg: "carácter ilegal: |".to_string(), line, col }); }
                }
                Some('?') => {
                    self.advance();
                    if self.current() == Some('.') { self.advance(); tokens.push(Token::new(TokenKind::QuestionDot, line, col)); }
                    else if self.current() == Some(':') { self.advance(); tokens.push(Token::new(TokenKind::QuestionColon, line, col)); }
                    else { tokens.push(Token::new(TokenKind::Question, line, col)); }
                }
                Some('@') => { self.advance(); tokens.push(Token::new(TokenKind::At, line, col)); }
                Some('.') => { self.advance(); tokens.push(Token::new(TokenKind::Dot, line, col)); }
                Some(',') => { self.advance(); tokens.push(Token::new(TokenKind::Comma, line, col)); }
                Some(':') => { self.advance(); tokens.push(Token::new(TokenKind::Colon, line, col)); }
                Some(';') => { self.advance(); tokens.push(Token::new(TokenKind::Semi, line, col)); }
                Some('(') => { self.advance(); tokens.push(Token::new(TokenKind::LParen, line, col)); }
                Some(')') => { self.advance(); tokens.push(Token::new(TokenKind::RParen, line, col)); }
                Some('{') => { self.advance(); tokens.push(Token::new(TokenKind::LBrace, line, col)); }
                Some('}') => { self.advance(); tokens.push(Token::new(TokenKind::RBrace, line, col)); }
                Some('[') => { self.advance(); tokens.push(Token::new(TokenKind::LBracket, line, col)); }
                Some(']') => { self.advance(); tokens.push(Token::new(TokenKind::RBracket, line, col)); }
                Some(c) => { return Err(LexError { msg: format!("carácter ilegal: {}", c), line, col }); }
            }
        }
        Ok(tokens)
    }
}

#[derive(Debug)]
pub struct LexError {
    pub msg: String,
    pub line: usize,
    pub col: usize,
}

impl fmt::Display for LexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "error léxico en {}:{}: {}", self.line, self.col, self.msg)
    }
}
