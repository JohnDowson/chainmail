use crate::Spanned;
use internment::Intern;
pub use parsers::expr;

mod parsers;

#[derive(Debug)]
pub enum Expr<'s, 'p> {
    Literal(Literal<'s>),
    Ident(Intern<String>),
    Lambda(
        Vec<Spanned<'p, Intern<String>>>,
        Box<Spanned<'p, Expr<'s, 'p>>>,
    ),
    UnaryOp(Op, Box<Spanned<'p, Expr<'s, 'p>>>),
    BinOp(
        Box<Spanned<'p, Expr<'s, 'p>>>,
        Op,
        Box<Spanned<'p, Expr<'s, 'p>>>,
    ),
}

#[derive(Debug)]
pub enum Op {
    Field,
    Pipe,
    Send,
    Add,
    Sub,
    Mul,
    Div,
    Eq,
    Neq,
    Lt,
    Gt,
    Ge,
    Le,
}

#[derive(Debug)]
pub enum Literal<'s> {
    String(&'s str),
    Symbol(Intern<String>),
    Int(i64),
    UInt(u64),
    Float(f64),
}
