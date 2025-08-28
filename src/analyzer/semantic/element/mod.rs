use std::{collections::VecDeque, iter::Peekable};

use iterator_stage::Processor;

use crate::analyzer::{diagnostic::Diagnostic, syntactic::Result};

pub struct Analyzer<T>
where
    T: Iterator<Item = Result>,
{
    reader: Peekable<T>,
    trace: VecDeque<Diagnostic>,
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
