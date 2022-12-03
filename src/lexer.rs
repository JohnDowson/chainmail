use internment::Intern;
use logos::Logos;

#[derive(Clone, Copy, Logos, Debug, PartialEq, Eq, Hash)]
pub enum Token<'s> {
    #[regex(r"#\[[^\]]*\]#", logos::skip)]
    #[regex("#[^\n]*", logos::skip)]
    Comment,

    #[regex(r#""([^"\\]|\\t|\\u|\\n|\\")*""#, |l| l.slice())]
    String(&'s str),

    #[regex("[0-9]+", |l| l.slice().parse())]
    Integer(u64),

    #[regex(r"[0-9]*\.[0-9]+", |l| l.slice())]
    Float(&'s str),

    #[regex(r"[_a-zA-Z][_a-zA-Z0-9]*", |l| Intern::new(l.slice().into()))]
    Ident(Intern<String>),
    #[regex(r":[_a-zA-Z][_a-zA-Z0-9]*", |l| Intern::new(l.slice()[1..].into()))]
    Symbol(Intern<String>),

    #[token(".")]
    Field,

    #[token(",")]
    Comma,

    #[token("<|")]
    Send,
    #[token("|")]
    Pipe,
    #[token("\\")]
    Lambda,
    #[token("->")]
    ThinArrow,

    #[token("+")]
    Add,
    #[token("-")]
    Sub,
    #[token("*")]
    Mul,
    #[token("/")]
    Div,
    #[token("==")]
    Eq,
    #[token("!=")]
    Neq,
    #[token("<")]
    Lt,
    #[token(">")]
    Gt,
    #[token("<=")]
    Ge,
    #[token(">=")]
    Le,

    #[token("]")]
    RBracket,
    #[token("[")]
    LBracket,
    #[token(")")]
    RParen,
    #[token("(")]
    LParen,

    #[regex(r"[ \t\f]+", logos::skip)]
    Whitespace,
    #[regex(r"\n+")]
    Newline,

    #[error]
    Error,
}
