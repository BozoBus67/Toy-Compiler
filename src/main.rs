mod ast;
mod lexer;
mod parser;

use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    let input = if args.len() > 1 {
        args[1..].join(" ")
    } else {
        "1 + 2 * 3".to_string()
    };

    println!("input:  {}", input);

    let tokens = lexer::lex(&input);
    println!("tokens: {:?}", tokens);

    let ast = parser::parse(tokens);
    println!("ast:    {:#?}", ast);
}
