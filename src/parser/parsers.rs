use super::{AstNode, Literal, Op};
use crate::{lexer::Token, Span, Spanned};
use chumsky::prelude::*;
use internment::Intern;

pub fn parser<'s: 'parser, 'path: 'parser, 'parser>(
) -> impl Parser<Token<'s>, Spanned<'path, AstNode<'s, 'path>>, Error = Simple<Token<'s>, Span<'path>>>
       + Clone
       + 'parser {
    statement().then_ignore(end())
}
fn statement<'s: 'parser, 'path: 'parser, 'parser>(
) -> impl Parser<Token<'s>, Spanned<'path, AstNode<'s, 'path>>, Error = Simple<Token<'s>, Span<'path>>>
       + Clone
       + 'parser {
    recursive(|statement| {
        let r#let = just(Token::KwLet)
            .ignore_then(raw_ident())
            .then_ignore(just(Token::Assign))
            .then(expr())
            .then_ignore(just(Token::Semicol))
            .then(statement.clone())
            .map_with_span(|((i, e), then), s| (s, AstNode::Let(i, box e, box then)))
            .labelled("let")
            .boxed();

        let field = raw_ident()
            .then_ignore(just(Token::Colon))
            .then(raw_ident());
        let class = just(Token::KwClass)
            .ignore_then(raw_ident())
            .then(field.repeated())
            .then_ignore(just(Token::KwEnd))
            .map(|(name, _fields)| AstNode::Class(name))
            .map_with_span(|e, s| (s, e))
            .labelled("class")
            .boxed();

        let stmt = expr()
            .then_ignore(just(Token::Semicol))
            .then(statement)
            .map(|(s, then)| AstNode::Statement(box s, box then))
            .map_with_span(|e, s| (s, e))
            .labelled("stmt")
            .boxed();

        choice((r#let, class, stmt, expr()))
    })
    .labelled("statement")
    .boxed()
}

fn expr<'s: 'parser, 'path: 'parser, 'parser>(
) -> impl Parser<Token<'s>, Spanned<'path, AstNode<'s, 'path>>, Error = Simple<Token<'s>, Span<'path>>>
       + Clone
       + 'parser {
    recursive(|expr| {
        let spanned_expr = expr.clone().map_with_span(|e, s| (s, e));
        let lambda = just(Token::Lambda)
            .ignore_then(raw_ident().separated_by(just(Token::Comma)))
            .then_ignore(just(Token::ThinArrow))
            .then(spanned_expr.clone())
            .map(|(args, body)| AstNode::Lambda(args, box body))
            .boxed();

        let parenthesised = expr
            .clone()
            .delimited_by(just(Token::LParen), just(Token::RParen));
        let call_or_ctor = choice((
            ident(),
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
        .map(|(r, a)| AstNode::Call(box r, a))
        .boxed();

        let list = spanned_expr
            .separated_by(just(Token::Comma))
            .delimited_by(just(Token::LBracket), just(Token::RBracket))
            .map(AstNode::List)
            .boxed();

        let atom = choice((
            call_or_ctor,
            ident(),
            lambda,
            literal(),
            parenthesised,
            list,
        ))
        .boxed();

        let sub = select!(Token::Sub => Op::Sub);
        let unary_sub = sub
            .repeated()
            .then(atom.clone().map_with_span(|e, s| (s, e)))
            .foldr(|op, (s, rhs)| (s, AstNode::UnaryOp(op, box (s, rhs))))
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
                    AstNode::BinOp(box (lspan, lhs), op, box (rspan, rhs)),
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
                    AstNode::BinOp(box (lspan, lhs), op, box (rspan, rhs)),
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
                    AstNode::BinOp(box (lspan, lhs), op, box (rspan, rhs)),
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
                    AstNode::BinOp(box (lspan, lhs), op, box (rspan, rhs)),
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
                    AstNode::BinOp(box (lspan, lhs), op, box (rspan, rhs)),
                )
            })
            .map(|(_, e)| e)
    })
    .map_with_span(|e, s| (s, e))
    .labelled("expr")
    .boxed()
}

fn ident<'s: 'parser, 'path: 'parser, 'parser>(
) -> impl Parser<Token<'s>, AstNode<'s, 'path>, Error = Simple<Token<'s>, Span<'path>>> + Clone + 'parser
{
    select!(Token::Ident(ident) => AstNode::Ident(ident))
}

fn raw_ident<'s: 'parser, 'path: 'parser, 'parser>(
) -> impl Parser<Token<'s>, Spanned<'path, Intern<String>>, Error = Simple<Token<'s>, Span<'path>>>
       + Clone
       + 'parser {
    select!(Token::Ident(i) => i).map_with_span(|i, s| (s, i))
}

fn literal<'s: 'parser, 'path: 'parser, 'parser>(
) -> impl Parser<Token<'s>, AstNode<'s, 'path>, Error = Simple<Token<'s>, Span<'path>>> + Clone + 'parser
{
    select! {
        Token::Integer(i) => Literal::Int(i as _),
        Token::Symbol(s) => Literal::Symbol(s)
    }
    .map(AstNode::Literal)
}

#[cfg(test)]
mod tests {
    #[test]
    fn literals() {}
}
