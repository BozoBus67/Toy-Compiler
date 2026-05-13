#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Num(i64),
    Ident(String),
    Let,
    Eq,
    Semi,
    Plus,
    Minus,
    Star,
    Slash,
    LParen,
    RParen,
}

pub fn lex(input: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut chars = input.chars().peekable();

    while let Some(&c) = chars.peek() {
        match c {
            ' ' | '\t' | '\n' | '\r' => {
                chars.next();
            }
            '+' => { chars.next(); tokens.push(Token::Plus); }
            '-' => { chars.next(); tokens.push(Token::Minus); }
            '*' => { chars.next(); tokens.push(Token::Star); }
            '/' => { chars.next(); tokens.push(Token::Slash); }
            '(' => { chars.next(); tokens.push(Token::LParen); }
            ')' => { chars.next(); tokens.push(Token::RParen); }
            '=' => { chars.next(); tokens.push(Token::Eq); }
            ';' => { chars.next(); tokens.push(Token::Semi); }
            '0'..='9' => {
                let mut n: i64 = 0;
                while let Some(&d) = chars.peek() {
                    if let Some(digit) = d.to_digit(10) {
                        n = n * 10 + digit as i64;
                        chars.next();
                    } else {
                        break;
                    }
                }
                tokens.push(Token::Num(n));
            }
            'a'..='z' | 'A'..='Z' | '_' => {
                let mut s = String::new();
                while let Some(&d) = chars.peek() {
                    if d.is_ascii_alphanumeric() || d == '_' {
                        s.push(d);
                        chars.next();
                    } else {
                        break;
                    }
                }
                match s.as_str() {
                    "let" => tokens.push(Token::Let),
                    _ => tokens.push(Token::Ident(s)),
                }
            }
            _ => panic!("lex error: unexpected character {:?}", c),
        }
    }

    tokens
}
