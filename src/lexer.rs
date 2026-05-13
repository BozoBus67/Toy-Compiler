#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Num(i64),
    Ident(String),
    Let,
    If,
    Else,
    True,
    False,
    Eq,
    EqEq,
    BangEq,
    Lt,
    Gt,
    Le,
    Ge,
    Semi,
    Plus,
    Minus,
    Star,
    Slash,
    LParen,
    RParen,
    LBrace,
    RBrace,
}

pub fn lex(input: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut chars = input.chars().peekable();

    while let Some(&c) = chars.peek() {
        match c {
            ' ' | '\t' | '\n' | '\r' => { chars.next(); }
            '+' => { chars.next(); tokens.push(Token::Plus); }
            '-' => { chars.next(); tokens.push(Token::Minus); }
            '*' => { chars.next(); tokens.push(Token::Star); }
            '/' => { chars.next(); tokens.push(Token::Slash); }
            '(' => { chars.next(); tokens.push(Token::LParen); }
            ')' => { chars.next(); tokens.push(Token::RParen); }
            '{' => { chars.next(); tokens.push(Token::LBrace); }
            '}' => { chars.next(); tokens.push(Token::RBrace); }
            ';' => { chars.next(); tokens.push(Token::Semi); }
            '=' => {
                chars.next();
                if let Some(&'=') = chars.peek() {
                    chars.next();
                    tokens.push(Token::EqEq);
                } else {
                    tokens.push(Token::Eq);
                }
            }
            '!' => {
                chars.next();
                if let Some(&'=') = chars.peek() {
                    chars.next();
                    tokens.push(Token::BangEq);
                } else {
                    panic!("lex error: '!' must be followed by '='");
                }
            }
            '<' => {
                chars.next();
                if let Some(&'=') = chars.peek() {
                    chars.next();
                    tokens.push(Token::Le);
                } else {
                    tokens.push(Token::Lt);
                }
            }
            '>' => {
                chars.next();
                if let Some(&'=') = chars.peek() {
                    chars.next();
                    tokens.push(Token::Ge);
                } else {
                    tokens.push(Token::Gt);
                }
            }
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
                    "if" => tokens.push(Token::If),
                    "else" => tokens.push(Token::Else),
                    "true" => tokens.push(Token::True),
                    "false" => tokens.push(Token::False),
                    _ => tokens.push(Token::Ident(s)),
                }
            }
            _ => panic!("lex error: unexpected character {:?}", c),
        }
    }

    tokens
}
