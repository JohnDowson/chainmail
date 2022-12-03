use super::{Expr, Literal, Op};
use crate::{lexer::Token, Span, Spanned};
use chumsky::prelude::*;

pub fn expr<'s: 'parser, 'path: 'parser, 'parser>(
) -> impl Parser<Token<'s>, Spanned<'path, Expr<'s, 'path>>, Error = Simple<Token<'s>, Span<'path>>>
       + Clone
       + 'parser {
    recursive(|expr| {
        let literal = select! {
            Token::Integer(i) => Literal::Int(i as _),
            Token::Symbol(s) => Literal::Symbol(s)
        }
        .map(Expr::Literal);

        let lambda = just(Token::Lambda)
            .ignore_then(select!(Token::Ident(i) => i).separated_by(just(Token::Comma)))
            .then(just(Token::ThinArrow))
            .ignore_then(expr.clone())
            .map(
                |(args, body): (Vec<Spanned<internment::Intern<String>>>, Spanned<Expr>)| {
                    Expr::Lambda(args, box body)
                },
            );

        let ident = select! {Token::Ident(ident) => Expr::Ident(ident)};
        let parenthesised = expr.delimited_by(just(Token::LParen), just(Token::RParen));
        let atom = choice((ident, lambda, literal, parenthesised));

        let sub = select!(Token::Sub => Op::Sub);
        let unary_sub = sub
            .repeated()
            .then(atom.clone().map_with_span(|e, s| (s, e)))
            .foldr(|op, (s, rhs)| (s, Expr::UnaryOp(op, box (s, rhs))))
            .map(|(_, e)| e);

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
            .map(|(_, e)| e);

        let op = select! {
            Token::Mul => Op::Mul,
            Token::Div => Op::Div,
        };
        let product = access
            .clone()
            .map_with_span(|e, s: Span| (s, e))
            .then(
                op.then(access.clone().map_with_span(|e, s| (s, e)))
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
            Token::Add => Op::Add,
            Token::Sub => Op::Sub,
        };
        product
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
            .map(|(_, e)| e)
    })
    .map_with_span(|e, s| (s, e))
}
