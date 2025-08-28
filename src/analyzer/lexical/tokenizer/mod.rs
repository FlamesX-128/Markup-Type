use std::{iter::Peekable, rc::Rc};

use iterator_stage::ConfigurableProcessor;

use crate::analyzer::diagnostic::{Position, Span};

use super::{Delimiter, Kind, Lexeme, Operator, Token};

pub struct Analyzer<T>
where
    T: Iterator<Item = char>,
{
    reader: Peekable<T>,
    position: Position,
    source: Rc<String>,
}

impl<T> Analyzer<T>
where
    T: Iterator<Item = char>,
{
    fn advance(&mut self) -> Option<char> {
        let item = self.reader.next();

        if let Some(char) = item {
            self.position.abs += 1;

            if char == '\n' {
                self.position.row += 1;
                self.position.col = 0;
            } else {
                self.position.col += 1;
            }
        }

        item
    }
}

impl<T> Analyzer<T>
where
    T: Iterator<Item = char>,
{
    fn tokenize_delimiter(&mut self) -> Token {
        let position = self.position.clone();
        let char = self.advance().unwrap();

        #[rustfmt::skip]
        let kind = match char {
            '{' => {
                Kind::Delimiter(Delimiter::LeftBrace)
            },
            '}' => {
                Kind::Delimiter(Delimiter::RightBrace)
            },
            ';' => {
                Kind::Delimiter(Delimiter::Semicolon)
            },
            _ => {
                unreachable!();
            }
        };

        let span = Span::new(self.source.clone(), position, self.position);
        let token = Token::new(kind, span);

        token
    }
}

impl<T> Analyzer<T>
where
    T: Iterator<Item = char>,
{
    fn tokenize_comment_lexeme(&mut self) -> Kind {
        let mut content = String::new();

        while let Some(char) = self.advance() {
            #[rustfmt::skip]
            match char {
                | '\n'
                | '\r' => {
                    break;
                },
                _ => {
                    content.push(char);
                }
            };
        }

        let lexeme = Lexeme::Comment(content);
        let kind = Kind::Lexeme(lexeme);

        kind
    }

    fn tokenize_identifier_lexeme(&mut self) -> Kind {
        let mut content = String::new();

        while let Some(&char) = self.reader.peek() {
            #[rustfmt::skip]
            match char {
                | '0'..='9'
                | 'A'..='Z'
                | 'a'..='z'
                | '-'
                | '_' => {
                    content.push(char);
                },
                _ => {
                    break;
                }
            };

            self.advance();
        }

        let lexeme = Lexeme::Identifier(content);
        let kind = Kind::Lexeme(lexeme);

        kind
    }

    fn tokenize_literal_lexeme(&mut self) -> Kind {
        let delimiter = self.advance().unwrap();

        let mut content = String::new();
        let mut escape = false;

        while let Some(char) = self.advance() {
            if escape {
                content.push(char);
                escape = false;
            } else if char == '\\' {
                escape = true;
            } else if char == delimiter {
                break;
            } else {
                content.push(char);
            }
        }

        let lexeme = Lexeme::Literal(content);
        let kind = Kind::Lexeme(lexeme);

        kind
    }

    fn tokenize_lexeme(&mut self) -> Token {
        let position = self.position.clone();
        let char = self.reader.peek().unwrap();

        #[rustfmt::skip]
        let kind = match char {
            '#' => {
                self.tokenize_comment_lexeme()
            },
            | 'A'..='Z'
            | 'a'..='z'
            | '-'
            | '_' => {
                self.tokenize_identifier_lexeme()
            },
            | '\''
            | '"' => {
                self.tokenize_literal_lexeme()
            },
            _ => {
                unreachable!()
            }
        };

        let span = Span::new(self.source.clone(), position, self.position);
        let token = Token::new(kind, span);

        token
    }
}

impl<T> Analyzer<T>
where
    T: Iterator<Item = char>,
{
    fn tokenize_borrow_operator(&mut self) -> Kind {
        let symbol = "<".into();

        if let Some(&char) = self.reader.peek() {
            if char == '-' {
                self.advance();

                Kind::Operator(Operator::Borrow)
            } else {
                Kind::Unknown(symbol)
            }
        } else {
            Kind::Unknown(symbol)
        }
    }

    fn tokenize_borrowable_transfer_operator(&mut self) -> Kind {
        let symbol = "-".into();

        if let Some(&char) = self.reader.peek() {
            if char == '>' {
                self.advance();

                Kind::Operator(Operator::Borrowable)
            } else {
                Kind::Unknown(symbol)
            }
        } else {
            Kind::Unknown(symbol)
        }
    }

    fn tokenize_pipe_operator(&mut self) -> Kind {
        Kind::Operator(Operator::Pipe)
    }

    fn tokenize_operator(&mut self) -> Token {
        let position = self.position.clone();
        let char = self.advance().unwrap();

        #[rustfmt::skip]
        let kind = match char {
            '<' => {
                self.tokenize_borrow_operator()
            },
            '-' => {
                self.tokenize_borrowable_transfer_operator()
            }
            '|' => {
                self.tokenize_pipe_operator()
            },
            _ => {
                unreachable!()
            }
        };

        let span = Span::new(self.source.clone(), position, self.position);
        let token = Token::new(kind, span);

        token
    }
}

impl<T> Analyzer<T>
where
    T: Iterator<Item = char>,
{
    fn tokenize_unknown(&mut self) -> Token {
        let position = self.position.clone();

        let mut content = String::new();

        while let Some(&char) = self.reader.peek() {
            match char {
                '\n' | '\r' | '\t' | ' ' => break,
                _ => {
                    content.push(char);
                }
            }

            self.advance();
        }

        let kind = Kind::Unknown(content);

        let span = Span::new(self.source.clone(), position, self.position);
        let token = Token::new(kind, span);

        token
    }
}

impl<T> Iterator for Analyzer<T>
where
    T: Iterator<Item = char>,
{
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(char) = self.reader.peek() {
            let token = match char {
                '\n' | '\r' | '\t' | ' ' => {
                    self.advance();
                    self.next()
                }
                '{' | '}' | ';' => {
                    let token = self.tokenize_delimiter();

                    Some(token)
                }
                'A'..='Z' | 'a'..='z' | '\'' | '"' | '#' | '_' => {
                    let token = self.tokenize_lexeme();

                    Some(token)
                }
                '<' | '-' | '|' => {
                    let token = self.tokenize_operator();

                    Some(token)
                }
                _ => {
                    let token = self.tokenize_unknown();

                    Some(token)
                }
            };

            token
        } else {
            None
        }
    }
}

impl<T> Analyzer<T>
where
    T: Iterator<Item = char>,
{
    pub fn new<Y>(source: Y, reader: T) -> Self
    where
        Y: AsRef<str>,
    {
        Self {
            reader: reader.peekable(),
            position: Position::default(),
            source: Rc::new(source.as_ref().into()),
        }
    }
}

pub struct Configurator<'a>(pub &'a str);

impl<'a, I> ConfigurableProcessor<I> for Configurator<'a>
where
    I: Iterator<Item = char>,
{
    type Iterator = Analyzer<I>;

    fn build(self, upstream: I) -> Self::Iterator {
        Self::Iterator::new(self.0, upstream)
    }
}
