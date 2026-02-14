use gbasic_common::span::Span;
use logos::Logos;

/// Callback to normalize identifiers/keywords to lowercase for case-insensitivity.
fn to_lowercase(lex: &logos::Lexer<'_, RawToken>) -> String {
    lex.slice().to_ascii_lowercase()
}

/// Process escape sequences in a string literal.
fn process_escapes(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('n') => out.push('\n'),
                Some('t') => out.push('\t'),
                Some('\\') => out.push('\\'),
                Some('"') => out.push('"'),
                Some('{') => out.push('{'),
                Some('}') => out.push('}'),
                Some(other) => {
                    out.push('\\');
                    out.push(other);
                }
                None => out.push('\\'),
            }
        } else {
            out.push(c);
        }
    }
    out
}

/// Raw token produced by logos before keyword classification.
#[derive(Logos, Debug, Clone, PartialEq)]
#[logos(skip r"[ \t\r]+")]
#[logos(skip r"//[^\n]*")]
#[logos(skip r"/\*([^*]|\*[^/])*\*/")]
pub enum RawToken {
    // Literals
    #[regex(r"[0-9]+\.[0-9]+([eE][+-]?[0-9]+)?", |lex| lex.slice().parse::<f64>().ok())]
    Float(f64),

    #[regex(r"[0-9]+", |lex| lex.slice().parse::<i64>().ok(), priority = 3)]
    Int(i64),

    #[regex(r#""([^"\\]|\\.)*""#, |lex| {
        let s = lex.slice();
        Some(process_escapes(&s[1..s.len()-1]))
    })]
    String(String),

    // Identifiers (captured as lowercase for case-insensitivity)
    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*", to_lowercase)]
    Ident(String),

    // Operators
    #[token("+")]
    Plus,
    #[token("-")]
    Minus,
    #[token("*")]
    Star,
    #[token("/")]
    Slash,
    #[token("%")]
    Percent,
    #[token("==")]
    EqEq,
    #[token("!=")]
    BangEq,
    #[token("<=")]
    LtEq,
    #[token(">=")]
    GtEq,
    #[token("<")]
    Lt,
    #[token(">")]
    Gt,
    #[token("&&")]
    AmpAmp,
    #[token("||")]
    PipePipe,
    #[token("!")]
    Bang,
    #[token("=")]
    Eq,

    // Delimiters
    #[token("(")]
    LParen,
    #[token(")")]
    RParen,
    #[token("{")]
    LBrace,
    #[token("}")]
    RBrace,
    #[token("[")]
    LBracket,
    #[token("]")]
    RBracket,
    #[token(",")]
    Comma,
    #[token("..")]
    DotDot,
    #[token(".")]
    Dot,
    #[token(":")]
    Colon,
    #[token(";")]
    Semicolon,
    #[token("->")]
    Arrow,

    #[token("\n")]
    Newline,
}

/// Classified token with keywords resolved.
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Keywords
    Let,
    Fun, // primary keyword for functions
    Fn,  // alias for fun
    If,
    Else,
    For,
    In,
    While,
    Match,
    Return,
    Break,
    Continue,
    True,
    False,
    And,
    Or,
    Not,

    // Namespaces
    Screen,
    Sound,
    Input,
    Math,
    System,
    Memory,
    IO,
    Asset,

    // Type keywords
    TyInt,
    TyFloat,
    TyString,
    TyBool,
    TyVoid,

    // Literals
    Int(i64),
    Float(f64),
    String(String),

    // Identifier
    Ident(String),

    // Operators
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    EqEq,
    BangEq,
    LtEq,
    GtEq,
    Lt,
    Gt,
    AmpAmp,
    PipePipe,
    Bang,
    Eq,

    // Delimiters
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    Comma,
    DotDot,
    Dot,
    Colon,
    Semicolon,
    Arrow,

    Newline,
    Eof,
    Error,
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::Let => write!(f, "let"),
            Token::Fun => write!(f, "fun"),
            Token::Fn => write!(f, "fn"),
            Token::If => write!(f, "if"),
            Token::Else => write!(f, "else"),
            Token::For => write!(f, "for"),
            Token::In => write!(f, "in"),
            Token::While => write!(f, "while"),
            Token::Match => write!(f, "match"),
            Token::Return => write!(f, "return"),
            Token::Break => write!(f, "break"),
            Token::Continue => write!(f, "continue"),
            Token::True => write!(f, "true"),
            Token::False => write!(f, "false"),
            Token::And => write!(f, "and"),
            Token::Or => write!(f, "or"),
            Token::Not => write!(f, "not"),
            Token::Screen => write!(f, "Screen"),
            Token::Sound => write!(f, "Sound"),
            Token::Input => write!(f, "Input"),
            Token::Math => write!(f, "Math"),
            Token::System => write!(f, "System"),
            Token::Memory => write!(f, "Memory"),
            Token::IO => write!(f, "IO"),
            Token::Asset => write!(f, "Asset"),
            Token::TyInt => write!(f, "Int"),
            Token::TyFloat => write!(f, "Float"),
            Token::TyString => write!(f, "String"),
            Token::TyBool => write!(f, "Bool"),
            Token::TyVoid => write!(f, "Void"),
            Token::Int(v) => write!(f, "{v}"),
            Token::Float(v) => write!(f, "{v}"),
            Token::String(s) => write!(f, "\"{s}\""),
            Token::Ident(s) => write!(f, "{s}"),
            Token::Plus => write!(f, "+"),
            Token::Minus => write!(f, "-"),
            Token::Star => write!(f, "*"),
            Token::Slash => write!(f, "/"),
            Token::Percent => write!(f, "%"),
            Token::EqEq => write!(f, "=="),
            Token::BangEq => write!(f, "!="),
            Token::LtEq => write!(f, "<="),
            Token::GtEq => write!(f, ">="),
            Token::Lt => write!(f, "<"),
            Token::Gt => write!(f, ">"),
            Token::AmpAmp => write!(f, "&&"),
            Token::PipePipe => write!(f, "||"),
            Token::Bang => write!(f, "!"),
            Token::Eq => write!(f, "="),
            Token::LParen => write!(f, "("),
            Token::RParen => write!(f, ")"),
            Token::LBrace => write!(f, "{{"),
            Token::RBrace => write!(f, "}}"),
            Token::LBracket => write!(f, "["),
            Token::RBracket => write!(f, "]"),
            Token::Comma => write!(f, ","),
            Token::DotDot => write!(f, ".."),
            Token::Dot => write!(f, "."),
            Token::Colon => write!(f, ":"),
            Token::Semicolon => write!(f, ";"),
            Token::Arrow => write!(f, "->"),
            Token::Newline => write!(f, "\\n"),
            Token::Eof => write!(f, "EOF"),
            Token::Error => write!(f, "<error>"),
        }
    }
}

/// A token with its source span.
#[derive(Debug, Clone)]
pub struct SpannedToken {
    pub token: Token,
    pub span: Span,
}

/// Classify a raw ident into keyword or identifier.
fn classify_ident(s: &str) -> Token {
    match s {
        "let" => Token::Let,
        "fun" => Token::Fun,
        "fn" => Token::Fn,
        "if" => Token::If,
        "else" => Token::Else,
        "for" => Token::For,
        "in" => Token::In,
        "while" => Token::While,
        "match" => Token::Match,
        "return" => Token::Return,
        "break" => Token::Break,
        "continue" => Token::Continue,
        "true" => Token::True,
        "false" => Token::False,
        "and" => Token::And,
        "or" => Token::Or,
        "not" => Token::Not,
        "screen" => Token::Screen,
        "sound" => Token::Sound,
        "input" => Token::Input,
        "math" => Token::Math,
        "system" => Token::System,
        "memory" => Token::Memory,
        "io" => Token::IO,
        "asset" => Token::Asset,
        "int" => Token::TyInt,
        "float" => Token::TyFloat,
        "string" => Token::TyString,
        "bool" => Token::TyBool,
        "void" => Token::TyVoid,
        _ => Token::Ident(s.to_string()),
    }
}

/// Tokenize source code into a vector of spanned tokens.
pub fn tokenize(source: &str) -> Vec<SpannedToken> {
    let mut tokens = Vec::new();
    let lexer = RawToken::lexer(source);

    for (result, range) in lexer.spanned() {
        let span = Span::new(range.start, range.end);
        let token = match result {
            Ok(raw) => match raw {
                RawToken::Int(v) => Token::Int(v),
                RawToken::Float(v) => Token::Float(v),
                RawToken::String(s) => Token::String(s),
                RawToken::Ident(s) => classify_ident(&s),
                RawToken::Plus => Token::Plus,
                RawToken::Minus => Token::Minus,
                RawToken::Star => Token::Star,
                RawToken::Slash => Token::Slash,
                RawToken::Percent => Token::Percent,
                RawToken::EqEq => Token::EqEq,
                RawToken::BangEq => Token::BangEq,
                RawToken::LtEq => Token::LtEq,
                RawToken::GtEq => Token::GtEq,
                RawToken::Lt => Token::Lt,
                RawToken::Gt => Token::Gt,
                RawToken::AmpAmp => Token::AmpAmp,
                RawToken::PipePipe => Token::PipePipe,
                RawToken::Bang => Token::Bang,
                RawToken::Eq => Token::Eq,
                RawToken::LParen => Token::LParen,
                RawToken::RParen => Token::RParen,
                RawToken::LBrace => Token::LBrace,
                RawToken::RBrace => Token::RBrace,
                RawToken::LBracket => Token::LBracket,
                RawToken::RBracket => Token::RBracket,
                RawToken::Comma => Token::Comma,
                RawToken::DotDot => Token::DotDot,
                RawToken::Dot => Token::Dot,
                RawToken::Colon => Token::Colon,
                RawToken::Semicolon => Token::Semicolon,
                RawToken::Arrow => Token::Arrow,
                RawToken::Newline => Token::Newline,
            },
            Err(()) => Token::Error,
        };
        tokens.push(SpannedToken { token, span });
    }

    tokens.push(SpannedToken {
        token: Token::Eof,
        span: Span::new(source.len(), source.len()),
    });

    tokens
}
