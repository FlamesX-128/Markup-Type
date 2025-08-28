pub mod converter;
pub mod desugaring;
pub mod expansion;

use std::rc::Rc;

use crate::analyzer::diagnostic::Span;

use super::diagnostic::Diagnostic;

#[derive(Clone, Debug)]
pub struct Attribute {
    pub name: String,
    pub value: Option<String>,
}

impl Attribute {
    pub fn new(name: String, value: Option<String>) -> Self {
        Self { name, value }
    }
}

#[derive(Clone, Debug)]
pub struct Borrow {
    pub name: String,
}

impl Borrow {
    pub fn new(name: String) -> Self {
        Self { name }
    }
}

#[derive(Clone, Debug)]
pub struct Borrowable {
    pub name: String,
}

impl Borrowable {
    pub fn new(name: String) -> Self {
        Self { name }
    }
}

#[derive(Clone, Debug)]
pub struct Comment {
    pub value: String,
}

impl Comment {
    pub fn new(value: String) -> Self {
        Self { value }
    }
}

#[derive(Clone, Debug)]
pub struct Element {
    pub name: String,
}

impl Element {
    pub fn new(name: String) -> Self {
        Self { name }
    }
}

#[derive(Clone, Debug)]
pub enum ProcessingInstruction {
    Borrow(Borrow),
    Borrowable(Borrowable),
}

impl From<Borrow> for ProcessingInstruction {
    fn from(value: Borrow) -> Self {
        Self::Borrow(value)
    }
}

impl From<Borrowable> for ProcessingInstruction {
    fn from(value: Borrowable) -> Self {
        Self::Borrowable(value)
    }
}

#[derive(Clone, Debug)]
pub struct Text {
    pub value: String,
}

impl Text {
    pub fn new(value: String) -> Self {
        Self { value }
    }
}

#[derive(Clone, Debug)]
pub enum Kind {
    Attribute(Attribute),
    Comment(Comment),
    Element(Element),
    ProcessingInstruction(ProcessingInstruction),
    Text(Text),
}

impl std::fmt::Display for Kind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        #[rustfmt::skip]
        let identifier =
        match self {
            Kind::Attribute(_) => "attribute",
            Kind::Comment(_) => "comment",
            Kind::Element(_) => "element",
            Kind::ProcessingInstruction(kind) => match kind {
                ProcessingInstruction::Borrow(_) => "borrow",
                ProcessingInstruction::Borrowable(_) => "borrowable",
            },
            Kind::Text(_) => "text",
        };

        write!(f, "{}", identifier)
    }
}

impl From<Attribute> for Kind {
    fn from(value: Attribute) -> Self {
        Self::Attribute(value)
    }
}

impl From<Comment> for Kind {
    fn from(value: Comment) -> Self {
        Self::Comment(value)
    }
}

impl From<Element> for Kind {
    fn from(value: Element) -> Self {
        Self::Element(value)
    }
}

impl From<ProcessingInstruction> for Kind {
    fn from(value: ProcessingInstruction) -> Self {
        Self::ProcessingInstruction(value)
    }
}

impl From<Text> for Kind {
    fn from(value: Text) -> Self {
        Self::Text(value)
    }
}

impl Kind {
    pub fn new<T>(value: T) -> Self
    where
        T: Into<Self>,
    {
        value.into()
    }
}

#[derive(Clone, Debug)]
pub struct Node {
    pub depth: u16,
    pub kind: Kind,
    pub span: Span,
}

impl Node {
    pub fn new<T>(depth: u16, kind: T, span: Span) -> Self
    where
        T: Into<Kind>,
    {
        let kind = kind.into();

        Self { depth, kind, span }
    }
}

#[derive(Clone, Debug)]
pub enum Result {
    Diagnostic(Diagnostic),
    Value(Rc<Node>),
}

impl From<Diagnostic> for Result {
    fn from(value: Diagnostic) -> Self {
        Self::Diagnostic(value)
    }
}

impl From<Node> for Result {
    fn from(value: Node) -> Self {
        Self::Value(Rc::new(value))
    }
}

impl Result {
    pub fn new<T>(value: T) -> Self
    where
        T: Into<Self>,
    {
        value.into()
    }
}
