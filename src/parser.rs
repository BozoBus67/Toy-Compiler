use crate::ast::{BinOp, Expr};
use crate::lexer::Token;

// Recursive descent, LL(1).
//
// Grammar:
//   expr   = term   { ("+" | "-") term }
//   term   = factor { ("*" | "/") factor }
//   factor = NUMBER | "-" factor | "(" expr ")"

struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn advance(&mut self) -> Option<Token> {
        let t = self.tokens.get(self.pos).cloned();
        self.pos += 1;
        t
    }

    fn parse_expr(&mut self) -> Expr {
        let mut left = self.parse_term();
        loop {
            let op = match self.peek() {
                Some(Token::Plus) => BinOp::Add,
                Some(Token::Minus) => BinOp::Sub,
                _ => return left,
            };
            self.advance();
            let right = self.parse_term();
            left = Expr::Bin {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
    }

    fn parse_term(&mut self) -> Expr {
        let mut left = self.parse_factor();
        loop {
            let op = match self.peek() {
                Some(Token::Star) => BinOp::Mul,
                Some(Token::Slash) => BinOp::Div,
                _ => return left,
            };
            self.advance();
            let right = self.parse_factor();
            left = Expr::Bin {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
    }

    fn parse_factor(&mut self) -> Expr {
        match self.advance() {
            Some(Token::Num(n)) => Expr::Num(n),
            Some(Token::Minus) => Expr::Neg(Box::new(self.parse_factor())),
            Some(Token::LParen) => {
                let e = self.parse_expr();
                match self.advance() {
                    Some(Token::RParen) => e,
                    other => panic!("parse error: expected ')', got {:?}", other),
                }
            }
            other => panic!("parse error: expected number, '-', or '(', got {:?}", other),
        }
    }
}

pub fn parse(tokens: Vec<Token>) -> Expr {
    let mut p = Parser::new(tokens);
    let expr = p.parse_expr();
    if p.pos < p.tokens.len() {
        panic!("parse error: trailing tokens: {:?}", &p.tokens[p.pos..]);
    }
    expr
}
