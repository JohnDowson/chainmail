use crate::Spanned;
use internment::Intern;

mod parsers;
pub use parsers::parser;

#[derive(Debug)]
pub enum AstNode<'s, 'p> {
    Class(Spanned<'p, Intern<String>>),

    Let(
        Spanned<'p, Intern<String>>,
        Box<Spanned<'p, AstNode<'s, 'p>>>,
        Box<Spanned<'p, AstNode<'s, 'p>>>,
    ),
    Statement(
        Box<Spanned<'p, AstNode<'s, 'p>>>,
        Box<Spanned<'p, AstNode<'s, 'p>>>,
    ),

    Literal(Literal<'s>),
    List(Vec<Spanned<'p, AstNode<'s, 'p>>>),
    Ident(Intern<String>),
    Call(
        Box<Spanned<'p, AstNode<'s, 'p>>>,
        Vec<Spanned<'p, AstNode<'s, 'p>>>,
    ),
    Lambda(
        Vec<Spanned<'p, Intern<String>>>,
        Box<Spanned<'p, AstNode<'s, 'p>>>,
    ),
    UnaryOp(Op, Box<Spanned<'p, AstNode<'s, 'p>>>),
    BinOp(
        Box<Spanned<'p, AstNode<'s, 'p>>>,
        Op,
        Box<Spanned<'p, AstNode<'s, 'p>>>,
    ),
}

#[derive(Debug)]
pub enum Op {
    Assign,
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
