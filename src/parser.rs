use crate::{lexer::Token, Span, Spanned};
use chumsky::prelude::*;
use internment::Intern;

#[derive(Debug)]
pub enum Expr<'s, 'p> {
    Let(
        Spanned<'p, Intern<String>>,
        Box<Spanned<'p, Expr<'s, 'p>>>,
        Box<Spanned<'p, Expr<'s, 'p>>>,
    ),
    Statement(
        Box<Spanned<'p, Expr<'s, 'p>>>,
        Box<Spanned<'p, Expr<'s, 'p>>>,
    ),

    Literal(Literal<'s>),
    List(Vec<Spanned<'p, Expr<'s, 'p>>>),
    Ident(Intern<String>),
    Call(
        Box<Spanned<'p, Expr<'s, 'p>>>,
        Vec<Spanned<'p, Expr<'s, 'p>>>,
    ),
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

pub fn parser<'s: 'parser, 'path: 'parser, 'parser>(
) -> impl Parser<Token<'s>, Spanned<'path, Expr<'s, 'path>>, Error = Simple<Token<'s>, Span<'path>>>
       + Clone
       + 'parser {
    let raw_ident = select!(Token::Ident(i) => i).map_with_span(|i, s| (s, i));
    let expr = recursive(|expr| {
        let literal = select! {
            Token::Integer(i) => Literal::Int(i as _),
            Token::Symbol(s) => Literal::Symbol(s)
        }
        .map(Expr::Literal);

        let spanned_expr = expr.clone().map_with_span(|e, s| (s, e));
        let lambda = just(Token::Lambda)
            .ignore_then(raw_ident.separated_by(just(Token::Comma)))
            .then_ignore(just(Token::ThinArrow))
            .then(spanned_expr.clone())
            .map(|(args, body)| Expr::Lambda(args, box body))
            .boxed();

        let ident = select! {Token::Ident(ident) => Expr::Ident(ident)};
        let parenthesised = expr
            .clone()
            .delimited_by(just(Token::LParen), just(Token::RParen));
        let call_or_ctor = choice((
            ident,
            lambda
                .clone()
                .delimited_by(just(Token::LParen), just(Token::RParen)),
        ))
        .map_with_span(|e, s| (s, e))
        .then(
            spanned_expr
                .clone()
                .separated_by(just(Token::Comma))
                .delimited_by(just(Token::LParen), just(Token::RParen)),
        )
        .map(|(r, a)| Expr::Call(box r, a))
        .boxed();

        let list = spanned_expr
            .separated_by(just(Token::Comma))
            .delimited_by(just(Token::LBracket), just(Token::RBracket))
            .map(Expr::List)
            .boxed();

        let atom = choice((call_or_ctor, ident, lambda, literal, parenthesised, list)).boxed();

        let sub = select!(Token::Sub => Op::Sub);
        let unary_sub = sub
            .repeated()
            .then(atom.clone().map_with_span(|e, s| (s, e)))
            .foldr(|op, (s, rhs)| (s, Expr::UnaryOp(op, box (s, rhs))))
            .map(|(_, e)| e)
            .boxed();

        let field = select!(Token::Field => Op::Field);
        let access = unary_sub
            .clone()
            .map_with_span(|e, s: Span| (s, e))
            .then(
                field
                    .then(unary_sub.clone().map_with_span(|e, s| (s, e)))
                    .repeated(),
            )
            .foldl(|(lspan, lhs), (op, (rspan, rhs))| {
                (
                    lspan.merge(rspan),
                    Expr::BinOp(box (lspan, lhs), op, box (rspan, rhs)),
                )
            })
            .map(|(_, e)| e)
            .boxed();

        let send = select!(Token::Send => Op::Send);
        let sending = access
            .clone()
            .map_with_span(|e, s: Span| (s, e))
            .then(
                send.then(access.clone().map_with_span(|e, s| (s, e)))
                    .repeated(),
            )
            .foldl(|(lspan, lhs), (op, (rspan, rhs))| {
                (
                    lspan.merge(rspan),
                    Expr::BinOp(box (lspan, lhs), op, box (rspan, rhs)),
                )
            })
            .map(|(_, e)| e)
            .boxed();

        let op = select! {
            Token::Mul => Op::Mul,
            Token::Div => Op::Div,
        };
        let product = sending
            .clone()
            .map_with_span(|e, s: Span| (s, e))
            .then(
                op.then(sending.clone().map_with_span(|e, s| (s, e)))
                    .repeated(),
            )
            .foldl(|(lspan, lhs), (op, (rspan, rhs))| {
                (
                    lspan.merge(rspan),
                    Expr::BinOp(box (lspan, lhs), op, box (rspan, rhs)),
                )
            })
            .map(|(_, e)| e)
            .boxed();

        let op = select! {
            Token::Add => Op::Add,
            Token::Sub => Op::Sub,
        };
        let sum = product
            .clone()
            .map_with_span(|e, s: Span| (s, e))
            .then(
                op.then(product.clone().map_with_span(|e, s| (s, e)))
                    .repeated(),
            )
            .foldl(|(lspan, lhs), (op, (rspan, rhs))| {
                (
                    lspan.merge(rspan),
                    Expr::BinOp(box (lspan, lhs), op, box (rspan, rhs)),
                )
            })
            .map(|(_, e)| e);

        let op = select! {
            Token::Assign => Op::Assign,
        };
        sum.clone()
            .map_with_span(|e, s: Span| (s, e))
            .then(op.then(sum.clone().map_with_span(|e, s| (s, e))).repeated())
            .foldl(|(lspan, lhs), (op, (rspan, rhs))| {
                (
                    lspan.merge(rspan),
                    Expr::BinOp(box (lspan, lhs), op, box (rspan, rhs)),
                )
            })
            .map(|(_, e)| e)
    })
    .map_with_span(|e, s| (s, e))
    .boxed();

    let statement = recursive(|statement| {
        let r#let = just(Token::KwLet)
            .ignore_then(raw_ident)
            .then_ignore(just(Token::Assign))
            .then(expr.clone())
            .then_ignore(just(Token::Semicol))
            .then(statement.clone())
            .map_with_span(|((i, e), then), s| (s, Expr::Let(i, box e, box then)))
            .labelled("let")
            .boxed();

        let stmt = expr
            .clone()
            .then_ignore(just(Token::Semicol))
            .then(statement)
            .map(|(s, then)| Expr::Statement(box s, box then))
            .map_with_span(|e, s| (s, e))
            .labelled("stmt")
            .boxed();

        choice((r#let, stmt, expr))
    })
    .labelled("statement")
    .boxed();

    statement.then_ignore(end())
}
