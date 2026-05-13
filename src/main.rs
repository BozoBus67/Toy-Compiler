mod ast;
mod codegen;
mod lexer;
mod native;
mod parser;

use std::env;

enum Mode {
    Debug,
    Native,
    Llvm,
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut mode = Mode::Debug;
    let mut input_parts: Vec<String> = Vec::new();
    for a in &args[1..] {
        match a.as_str() {
            "--native" => mode = Mode::Native,
            "--llvm" => mode = Mode::Llvm,
            _ => input_parts.push(a.clone()),
        }
    }
    let input = if input_parts.is_empty() {
        "1 + 2 * 3".to_string()
    } else {
        input_parts.join(" ")
    };

    let tokens = lexer::lex(&input);

    if matches!(mode, Mode::Debug) {
        println!("input:  {}", input);
        println!("tokens: {:?}", tokens);
    }

    let ast = parser::parse(tokens);

    match mode {
        Mode::Debug => {
            println!("ast:    {:#?}", ast);
        }
        Mode::Native => {
            if !ast.stmts.is_empty() {
                panic!("native demo: only phase 1 expressions (no let bindings)");
            }
            print!("{}", native::codegen(&ast.result));
        }
        Mode::Llvm => {
            if !ast.stmts.is_empty() {
                panic!("llvm codegen: phase 1 only for now (no let bindings)");
            }
            print!("{}", codegen::codegen(&ast.result));
        }
    }
}
