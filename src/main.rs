use chainmail::{lexer, Span};
use chumsky::{Parser, Span as _, Stream};
use clap::Parser as CParser;
use logos::Logos;
use std::{fs::File, io::Read, path::PathBuf};

#[derive(CParser)]
struct Args {
    #[clap(short = 'k', long)]
    dump_tokens: bool,
    #[clap(short = 't', long)]
    time: bool,
    source: PathBuf,
}

fn main() -> std::io::Result<()> {
    let args = Args::parse();

    let mut file = File::open(&args.source)?;
    let mut src = String::new();
    file.read_to_string(&mut src)?;

    let tokens = lexer::Token::lexer(&src)
        .spanned()
        .map(|(t, s)| (t, Span::new(args.source.as_ref(), s)))
        .collect::<Vec<_>>();
    if args.dump_tokens {
        println!("{:?}", &tokens)
    }
    let ast = chainmail::parser::expr().parse(Stream::from_iter(
        Span::new(args.source.as_ref(), 0..0),
        tokens.into_iter(),
    ));

    println!("{ast:#?}");

    Ok(())
}
