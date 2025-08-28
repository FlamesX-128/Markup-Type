use std::{collections::VecDeque, iter::Peekable, rc::Rc};

use iterator_stage::Processor;

use crate::analyzer::{
    diagnostic::{self, Diagnostic},
    syntactic::{Element, Kind, Node, ProcessingInstruction, Result},
};

pub struct Analyzer<T>
where
    T: Iterator<Item = Result>,
{
    reader: Peekable<T>,
    stack: Vec<Rc<Node>>,
    trace: VecDeque<Diagnostic>,
}

impl<T> Analyzer<T>
where
    T: Iterator<Item = Result>,
{
    fn validate_attribute_context(&mut self, _: &Node) {
        while let Some(node) = self.stack.pop() {
            if let Kind::ProcessingInstruction(ProcessingInstruction::Borrow(_)) | Kind::Text(_) =
                &node.kind
            {
                let message = format!("attribute cannot be followed by a {}", node.kind);
                let diagnostic =
                    Diagnostic::new(diagnostic::Kind::Error, &message, node.span.clone());

                self.trace.push_back(diagnostic);
            }
        }
    }

    fn validate_attribute_name_and_value(&mut self, _element: &Element) {
        while let Some(_node) = self.stack.pop() {}
    }
}

impl<T> Iterator for Analyzer<T>
where
    T: Iterator<Item = Result>,
{
    type Item = Result;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(diagnostic) = self.trace.pop_front() {
            return Some(Result::from(diagnostic));
        }

        let result = self.reader.next();

        if let Some(Result::Value(node)) = &result {
            match &node.kind {
                Kind::Attribute(_) => {
                    self.stack.push(Rc::clone(node));
                }
                Kind::ProcessingInstruction(ProcessingInstruction::Borrow(_)) | Kind::Text(_) => {
                    self.validate_attribute_context(&node);
                }
                Kind::Element(element) => {
                    self.validate_attribute_name_and_value(&element);
                }
                _ => {}
            }
        }

        result
    }
}

impl<T> Analyzer<T>
where
    T: Iterator<Item = Result>,
{
    pub fn new(reader: T) -> Self {
        Self {
            reader: reader.peekable(),
            stack: Vec::new(),
            trace: VecDeque::new(),
        }
    }
}

impl<T> Processor<T> for Analyzer<T>
where
    T: Iterator<Item = Result>,
{
    fn build(upstream: T) -> Self {
        Analyzer::new(upstream)
    }
}
