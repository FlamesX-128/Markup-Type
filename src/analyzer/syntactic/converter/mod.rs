use std::iter::Peekable;

use iterator_stage::Processor;

use crate::analyzer::{
    diagnostic::{self, Span},
    lexical::{self, Token},
};

use super::{
    Attribute, Borrow, Borrowable, Comment, Element, Kind, Node, ProcessingInstruction, Result, Text
};

pub struct Analyzer<I>
where
    I: Iterator<Item = Token>,
{
    depth: u16,
    upstream: Peekable<I>,
}

impl<I> Analyzer<I>
where
    I: Iterator<Item = Token>,
{
    fn parse_delimiter(&mut self, kind: lexical::Delimiter) -> Option<Result> {
        self.upstream.next().unwrap();

        match kind {
            lexical::Delimiter::LeftBrace => {
                self.depth += 1;
            }
            lexical::Delimiter::RightBrace => {
                self.depth -= 1;
            }
            lexical::Delimiter::Semicolon => {}
        };

        self.next()
    }
}

impl<I> Analyzer<I>
where
    I: Iterator<Item = Token>,
{
    fn parse_comment_lexeme(&mut self, value: String) -> Option<Result> {
        let token = self.upstream.next().unwrap();

        let data = Comment::new(value);

        let kind = Kind::from(data);
        let node = Node::new(self.depth, kind, token.span);

        Some(Result::from(node))
    }

    fn parse_identifier_lexeme(&mut self, value: String) -> Option<Result> {
        let namet = self.upstream.next().unwrap();

        if let Some(item) = self.upstream.peek() {
            if let lexical::Kind::Operator(lexical::Operator::Borrow) = item.kind {
                let item = self.upstream.next().unwrap();

                let data = Borrow::new(value);
                let span = Span::new(namet.span.source, namet.span.start, item.span.end);

                let pi = ProcessingInstruction::from(data);

                let kind = Kind::from(pi);
                let node = Node::new(self.depth, kind, span);

                return Some(Result::from(node));
            }

            if let lexical::Kind::Operator(lexical::Operator::Borrowable) = item.kind {
                let item = self.upstream.next().unwrap();

                let data = Borrowable::new(value);
                let span = Span::new(namet.span.source, namet.span.start, item.span.end);

                let pi = ProcessingInstruction::from(data);

                let kind = Kind::from(pi);
                let node = Node::new(self.depth, kind, span);

                return Some(Result::from(node));
            }
        }

        let data = Element::new(value);

        let kind = Kind::from(data);
        let node = Node::new(self.depth, kind, namet.span);

        Some(Result::from(node))
    }

    fn parse_literal_lexeme(&mut self, value: String) -> Option<Result> {
        let item = self.upstream.next().unwrap();

        let data = Text::new(value);
        let kind = Kind::new(data);

        let node = Node::new(self.depth, kind, item.span);

        Some(Result::from(node))
    }

    fn parse_lexeme(&mut self, kind: lexical::Lexeme) -> Option<Result> {
        match kind {
            lexical::Lexeme::Comment(value) => self.parse_comment_lexeme(value),
            lexical::Lexeme::Identifier(value) => self.parse_identifier_lexeme(value),
            lexical::Lexeme::Literal(value) => self.parse_literal_lexeme(value),
        }
    }
}

impl<I> Analyzer<I>
where
    I: Iterator<Item = Token>,
{
    fn parse_pipe_operator(&mut self, token: lexical::Token) -> Option<Result> {
        if let Some(namet) = self.upstream.next() {
            if let lexical::Kind::Lexeme(
                lexical::Lexeme::Identifier(name) | lexical::Lexeme::Literal(name),
            ) = namet.kind
            {
                if let Some(valuet) = self.upstream.peek() {
                    if let lexical::Kind::Lexeme(lexical::Lexeme::Literal(value)) =
                        valuet.kind.clone()
                    {
                        let valuet = self.upstream.next().unwrap();

                        let data = Attribute::new(name, Some(value));
                        let span = Span::new(token.span.source, token.span.start, valuet.span.end);

                        let kind = Kind::from(data);
                        let node = Node::new(self.depth, kind, span);

                        return Some(Result::from(node));
                    }
                }

                let data = Attribute::new(name, None);
                let span = Span::new(token.span.source, token.span.start, namet.span.end);

                let kind = Kind::from(data);
                let node = Node::new(self.depth, kind, span);

                return Some(Result::from(node));
            }
        }

        let message = "an identifier was expected after the property declaration";

        let diagnostic = diagnostic::Diagnostic::new(diagnostic::Kind::Error, &message, token.span);
        let result = Result::from(diagnostic);

        Some(result)
    }

    fn parse_operator(&mut self, kind: lexical::Operator) -> Option<Result> {
        let token = self.upstream.next().unwrap();

        match kind {
            lexical::Operator::Pipe => self.parse_pipe_operator(token),
            _ => self.parse_unknown(),
        }
    }
}

impl<I> Analyzer<I>
where
    I: Iterator<Item = Token>,
{
    pub fn parse_unknown(&mut self) -> Option<Result> {
        let token = self.upstream.next().unwrap();

        let message = "an unknown token was unexpectedly found";

        let diagnostic = diagnostic::Diagnostic::new(diagnostic::Kind::Error, &message, token.span);
        let result = Result::from(diagnostic);

        Some(result)
    }
}

impl<I> Iterator for Analyzer<I>
where
    I: Iterator<Item = Token>,
{
    type Item = Result;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(token) = self.upstream.peek() {
            match token.kind.clone() {
                lexical::Kind::Delimiter(delimiter) => self.parse_delimiter(delimiter),
                lexical::Kind::Lexeme(lexeme) => self.parse_lexeme(lexeme),
                lexical::Kind::Operator(operator) => self.parse_operator(operator),
                _ => self.parse_unknown(),
            }
        } else {
            None
        }
    }
}

impl<I> Analyzer<I>
where
    I: Iterator<Item = Token>,
{
    pub fn new(upstream: I) -> Self {
        let upstream = upstream.peekable();
        let depth = 0;

        Self { depth, upstream }
    }
}

impl<I> Processor<I> for Analyzer<I>
where
    I: Iterator<Item = Token>,
{
    fn build(upstream: I) -> Self {
        Self::new(upstream)
    }
}
