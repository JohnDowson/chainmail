use std::{ops::Range, path::Path};

use crate::lexer::Token;

use chumsky::{
    prelude::{just, Simple},
    select, Parser, Stream,
};
use logos::Lexer;

#[derive(Debug)]
pub enum Expr<'s> {
    Symbol(&'s str),
}

#[derive(Clone, Copy)]
pub struct Span<'p> {
    file: &'p Path,
    start: usize,
    end: usize,
}

impl<'p> std::fmt::Debug for Span<'p> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}..{}", self.start, self.end)
    }
}

pub struct AstNode<'s, 'p> {
    expr: Expr<'s>,
    span: Span<'p>,
}

fn symbol<'s>() -> impl Parser<Token<'s>, Expr<'s>, Error = Simple<Token<'s>, Range<usize>>> {
    just(Token::Colon).ignore_then(select! {
        Token::Ident(sym) => Expr::Symbol(sym)
    })
}

pub fn parse(
    tokens: Vec<(Token, Range<usize>)>,
) -> Result<Vec<Expr>, Vec<Simple<Token, Range<usize>>>> {
    symbol().repeated().parse(Stream::from_iter(
        tokens.last().unwrap().1.clone(),
        tokens.into_iter(),
    ))
}
