use chainmail::lexer;
use chainmail::parser;
use clap::Parser;
use logos::Logos;
use std::{fs::File, io::Read, path::PathBuf};

#[derive(Parser)]
struct Args {
    #[clap(short = 'k', long)]
    dump_tokens: bool,
    #[clap(short = 't', long)]
    time: bool,
    source: PathBuf,
}

fn main() -> std::io::Result<()> {
    let args = Args::parse();

    let mut file = File::open(args.source)?;
    let mut src = String::new();
    file.read_to_string(&mut src)?;

    let tokens = lexer::Token::lexer(&src).spanned().collect::<Vec<_>>();
    if args.dump_tokens {
        println!("{:?}", &tokens)
    }
    let ast = parser::parse(tokens);

    println!("{ast:?}");

    Ok(())
}
