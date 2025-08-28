use std::{
    collections::{HashMap, VecDeque},
    iter::Peekable,
};

use iterator_stage::Processor;

use crate::analyzer::{
    diagnostic::{self, Diagnostic},
    syntactic::{Borrow, Kind, Node, ProcessingInstruction, Result},
};

struct BorrowChecker {
    scope: Vec<VecDeque<Node>>,
    trace: Vec<HashMap<String, Node>>,
    transferring: u16,
}

impl BorrowChecker {
    pub fn alloc(&mut self, node: &Node) {
        if self.transferring > 0 {
            if let Some(deque) = self.scope.last_mut() {
                let kind = node.kind.clone();
                let span = node.span.clone();

                let node = deque.front().unwrap_or(node);

                deque.push_back(Node::new(node.depth, kind, span));
            }
        }
    }

    pub fn dealloc(&mut self, node: &Node, peek: &Node) {
        if peek.depth < node.depth {
            self.transferring = self
                .transferring
                .saturating_sub(node.depth.saturating_sub(peek.depth));
        }
    }

    pub fn contains(&self, borrow: &Borrow, node: &Node) -> bool {
        let mut scope = self.scope.clone();

        while let Some(deque) = scope.last_mut() {
            while let Some(borrowable) = deque.pop_front() {
                if borrowable.depth - 1 > node.depth {
                    break;
                }

                if borrowable.depth - 1 == node.depth {
                    if let Kind::Element(element) = &borrowable.kind {
                        if element.name == borrow.name {
                            return true;
                        }
                    }
                } else {
                    return false;
                }
            }

            scope.pop();
        }

        false
    }
}

impl BorrowChecker {
    pub fn borrowable(&mut self) {
        self.transferring += 1;

        if self.transferring == 1 {
            self.scope.push(VecDeque::new());
            self.trace.push(HashMap::new());
        }
    }

    fn borrowing(&mut self, borrow: &Borrow, node: &Node) {
        while let Some(deque) = self.scope.last_mut() {
            while let Some(borrowable) = deque.pop_front() {
                if borrowable.depth > node.depth + 1 {
                    break;
                }

                if borrowable.depth == node.depth + 1 {
                    if let Kind::Element(element) = &borrowable.kind {
                        if let Some(trace) = self.trace.last_mut() {
                            trace.insert(element.name.clone(), node.clone());
                        }

                        if element.name == borrow.name {
                            return deque.push_front(borrowable);
                        }
                    }
                }
            }

            self.scope.pop();
            self.trace.pop();
        }
    }

    pub fn borrow(&mut self, borrow: &Borrow, node: &Node) -> Option<Diagnostic> {
        if self.contains(borrow, node) {
            self.borrowing(borrow, node);

            return None;
        }

        if let Some(trace) = self.trace.get(node.depth as usize) {
            if let Some(_) = trace.get(&borrow.name) {
                let message = format!("the borrowable '{}' has been already borrowed", borrow.name);

                let kind = diagnostic::Kind::Error;
                let diagnostic = Diagnostic::new(kind, &message, node.span.clone());

                return Some(diagnostic);
            }
        }

        let message = format!(
            "the borrowable '{}' was expected to be used, but it was not found",
            borrow.name
        );

        let kind = diagnostic::Kind::Error;
        let diagnostic = Diagnostic::new(kind, &message, node.span.clone());

        return Some(diagnostic);
    }
}

impl BorrowChecker {
    pub fn new() -> Self {
        Self {
            scope: Vec::new(),
            trace: Vec::new(),
            transferring: 0,
        }
    }
}

pub struct Analyzer<T>
where
    T: Iterator<Item = Result>,
{
    checker: BorrowChecker,
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
                Kind::Element(_) => {
                    self.checker.alloc(node);
                }
                Kind::ProcessingInstruction(kind) => match kind {
                    ProcessingInstruction::Borrow(borrow) => {
                        if let Some(diagnostic) = self.checker.borrow(borrow, node) {
                            self.trace.push_back(diagnostic);
                        }
                    }
                    ProcessingInstruction::Borrowable(_) => {
                        self.checker.borrowable();
                    }
                },
                _ => {}
            }

            if let Some(Result::Value(peek)) = self.reader.peek() {
                self.checker.dealloc(node, peek)
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
            checker: BorrowChecker::new(),
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

#[cfg(test)]
mod test {
    use iterator_stage::Stage;

    use crate::analyzer::{
        filesystem::FileReader,
        lexical::tokenizer,
        semantic::{attribute, element, ownership},
        syntactic::{converter, desugaring},
    };

    #[test]
    fn test() -> std::io::Result<()> {
        let path = "/home/flames/Github/FlamesX-128/markup-type/examples/input.mt";

        let reader = FileReader::new(&path)?;

        let mut analyzer = tokenizer::Analyzer::new(&path, reader)
            .chain_infer::<converter::Analyzer<_>>()
            .chain_infer::<desugaring::Analyzer<_>>()
            .chain_infer::<attribute::Analyzer<_>>()
            .chain_infer::<element::Analyzer<_>>()
            .chain_infer::<ownership::Analyzer<_>>();

        while let Some(node) = analyzer.next() {
            match node {
                crate::analyzer::syntactic::Result::Diagnostic(diagnostic) => {
                    println!("{}", diagnostic);
                }
                _ => {}
            }
        }

        Ok(())
    }
}
