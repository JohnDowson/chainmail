#![feature(box_syntax)]
#![feature(box_patterns)]

use std::path::Path;

pub mod eval;
pub mod lexer;
// mod object;
pub mod parser;

type Spanned<'p, T> = (Span<'p>, T);

#[derive(Clone, Copy)]
pub struct Span<'p> {
    file: &'p Path,
    start: usize,
    end: usize,
}
impl<'p> Span<'p> {
    pub fn merge(self, other: Self) -> Self {
        Self {
            file: self.file,
            start: self.start,
            end: other.end,
        }
    }
}
impl<'p> std::fmt::Debug for Span<'p> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}..{}", self.start, self.end)
    }
}
impl<'p> chumsky::Span for Span<'p> {
    type Context = &'p Path;
    type Offset = usize;

    fn new(context: Self::Context, range: std::ops::Range<Self::Offset>) -> Self {
        Self {
            file: context,
            start: range.start,
            end: range.end,
        }
    }

    fn context(&self) -> Self::Context {
        self.file
    }

    fn start(&self) -> Self::Offset {
        self.start
    }

    fn end(&self) -> Self::Offset {
        self.end
    }
}
