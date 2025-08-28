use std::{collections::VecDeque, iter::Peekable};

use iterator_stage::Processor;

use crate::analyzer::diagnostic::Diagnostic;

use super::Result;

pub struct Analyzer<I>
where
    I: Iterator<Item = Result>,
{
    trace: VecDeque<Diagnostic>,
    upstream: Peekable<I>,
}

impl<I> Iterator for Analyzer<I>
where
    I: Iterator<Item = Result>,
{
    type Item = Result;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(diagnostic) = self.trace.pop_front() {
            return Some(Result::from(diagnostic));
        }

        self.upstream.next()
    }
}

impl<I> Analyzer<I>
where
    I: Iterator<Item = Result>,
{
    pub fn new(upstream: I) -> Self {
        let upstream = upstream.peekable();
        let trace = VecDeque::new();

        Self { trace, upstream }
    }
}

impl<I> Processor<I> for Analyzer<I>
where
    I: Iterator<Item = Result>,
{
    fn build(upstream: I) -> Self {
        Self::new(upstream)
    }
}
