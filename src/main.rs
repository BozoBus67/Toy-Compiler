mod ast;
mod lexer;
mod native;
mod parser;

use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut native_mode = false;
    let mut input_parts: Vec<String> = Vec::new();
    for a in &args[1..] {
        if a == "--native" {
            native_mode = true;
        } else {
            input_parts.push(a.clone());
        }
    }
    let input = if input_parts.is_empty() {
        "1 + 2 * 3".to_string()
    } else {
        input_parts.join(" ")
    };

    let tokens = lexer::lex(&input);

    if !native_mode {
        println!("input:  {}", input);
        println!("tokens: {:?}", tokens);
    }

    let ast = parser::parse(tokens);

    if native_mode {
        if !ast.stmts.is_empty() {
            panic!("native demo: only phase 1 expressions (no let bindings)");
        }
        print!("{}", native::codegen(&ast.result));
    } else {
        println!("ast:    {:#?}", ast);
    }
}
