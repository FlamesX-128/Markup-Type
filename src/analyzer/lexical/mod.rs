pub mod tokenizer;

use crate::analyzer::diagnostic::Span;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Delimiter {
    LeftBrace,
    RightBrace,
    Semicolon,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Lexeme {
    Comment(String),
    Identifier(String),
    Literal(String),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Operator {
    Borrow,
    Borrowable,
    Pipe,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Kind {
    Delimiter(Delimiter),
    Lexeme(Lexeme),
    Operator(Operator),
    Unknown(String),
}

#[derive(Clone, Debug)]
pub struct Token {
    pub kind: Kind,
    pub span: Span,
}

impl Token {
    pub fn new(kind: Kind, span: Span) -> Self {
        Self { kind, span }
    }
}
