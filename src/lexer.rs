use logos::Logos;

#[derive(Clone, Copy, Logos, Debug, PartialEq, Eq, Hash)]
pub enum Token<'s> {
    #[regex(r#"#\[[^\]]*\]#"#)]
    #[regex("#.*\n")]
    Comment,

    #[regex("-?[0-9]+", |l| l.slice())]
    Integer(&'s str),

    #[regex(r#"[_a-zA-Z\+=\[\]][_a-zA-Z0-9\+=\[\]]*"#, |l| l.slice())]
    Ident(&'s str),
    #[token(":")]
    Colon,
    #[token("<")]
    Send,

    #[token("|")]
    Pipe,

    #[token("]")]
    RBracket,
    #[token("[")]
    LBracket,
    #[token(")")]
    Rparen,
    #[token("(")]
    LParen,

    #[regex(r"\s+")]
    Whitespace,

    #[error]
    Error,
}
