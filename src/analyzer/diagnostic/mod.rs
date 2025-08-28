use std::{io::BufRead, rc::Rc};

use colored::Colorize;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Position {
    pub abs: usize,
    pub col: usize,
    pub row: usize,
}

#[derive(Clone, Debug, Default)]
pub struct Span {
    pub end: Position,
    pub source: Rc<String>,
    pub start: Position,
}

impl Span {
    pub fn new(source: Rc<String>, start: Position, end: Position) -> Self {
        Self { end, source, start }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Kind {
    Critical,
    Error,
    Debug,
    Warning,
}

#[derive(Clone, Debug)]
pub struct Diagnostic {
    pub kind: Kind,
    pub message: String,
    pub span: Span,
}

impl std::fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        #[rustfmt::skip]
        let kind =
        match self.kind {
            Kind::Critical => "critical".red().on_black(),
            Kind::Warning => "warning".yellow(),
            Kind::Debug => "debug".green(),
            Kind::Error => "error".red(),
        };

        #[rustfmt::skip]
        let source_line =
        if let Ok(file) = std::fs::File::open(&self.span.source.as_ref()) {
            let reader = std::io::BufReader::new(file);
            if let Some(line) = reader.lines().nth(self.span.start.row) {
                line.unwrap_or_default()
            } else {
                String::default()
            }
        } else {
            String::default()
        };

        let mut arrow_line = String::new();

        let space_padding = " ".repeat(self.span.start.col);
        let arrow_padding = "^".repeat(self.span.end.abs - self.span.start.abs);

        arrow_line.push_str(&space_padding);
        arrow_line.push_str(&arrow_padding);

        let location_info = format!(
            "{}:{}:{}",
            self.span.source,
            self.span.start.row + 1,
            self.span.start.col + 1
        );

        // The following looks horrible...
        let line_info = format!(
            "{:<3} | {}",
            self.span.start.row.to_string().blue().bold(),
            source_line
        );

        let formatted_message = format!(
            "{}{} {}\n   {} {}\n    |\n{}\n    | {}",
            kind.bold(),
            ":".bold(),
            self.message.bold(),
            "-->".blue().bold(),
            location_info,
            line_info,
            arrow_line.yellow().bold()
        );

        let message = formatted_message
            .replace("|", &"|".blue().bold().to_string())
            .replace("-->", &"-->".blue().bold().to_string());

        write!(f, "{}", message)
    }
}

impl std::error::Error for Diagnostic {}

impl Diagnostic {
    #[rustfmt::skip]
    pub fn new(kind: Kind, message: &str, span: Span) -> Self {
        Self { kind, message: message.to_string(), span }
    }
}
