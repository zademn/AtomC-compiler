mod asdr;
mod asdr_dom;
mod lexer;
mod symbols;
use clap::{App, Arg};
use std::fs;

//use lexer::get_tokens;
use asdr_dom::SyntaxAnalyser;
use lexer::Lexer;

fn main() -> Result<(), &'static str> {
    let matches = App::new("AtomC compiler")
        .version("0.0")
        .author("Dacian Stroia")
        .arg(
            Arg::with_name("file")
                .short("f")
                .long("file")
                .help("The AtomC file to compile")
                .value_name("FILE")
                .takes_value(true)
                .required(true), // file is required
        )
        .get_matches();
    // get filename
    let filename = matches.value_of("file").expect("Please input a file");
    // Get contents to a file as a string
    let contents = fs::read_to_string(filename).expect("Something went wrong reading the file");
    // Print contents to debug
    //println!("{}", contents);

    let mut lexer = Lexer::from_string(contents);
    let token_vec = lexer.get_tokens();
    // for elem in token_vec.iter() {
    //     println!("{:?}", elem);
    // }

    let mut syntax_analyser: SyntaxAnalyser = SyntaxAnalyser::new(token_vec);
    syntax_analyser.analyse_syntax();
    Ok(())
}
