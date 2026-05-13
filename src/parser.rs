use crate::ast::{BinOp, Expr, Program, Stmt};
use crate::lexer::Token;

// Recursive descent, LL(1).
//
// Grammar:
//   program  = { let_stmt ";" } expr
//   let_stmt = "let" IDENT "=" expr
//   expr     = term   { ("+" | "-") term }
//   term     = factor { ("*" | "/") factor }
//   factor   = NUMBER | IDENT | "-" factor | "(" expr ")"

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

    fn parse_program(&mut self) -> Program {
        let mut stmts = Vec::new();
        while let Some(Token::Let) = self.peek() {
            stmts.push(self.parse_let_stmt());
        }
        let result = self.parse_expr();
        Program { stmts, result }
    }

    fn parse_let_stmt(&mut self) -> Stmt {
        match self.advance() {
            Some(Token::Let) => {}
            other => panic!("parse error: expected 'let', got {:?}", other),
        }
        let name = match self.advance() {
            Some(Token::Ident(s)) => s,
            other => panic!("parse error: expected identifier after 'let', got {:?}", other),
        };
        match self.advance() {
            Some(Token::Eq) => {}
            other => panic!("parse error: expected '=' after let name, got {:?}", other),
        }
        let value = self.parse_expr();
        match self.advance() {
            Some(Token::Semi) => {}
            other => panic!("parse error: expected ';' after let value, got {:?}", other),
        }
        Stmt::Let { name, value }
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
            Some(Token::Ident(s)) => Expr::Ident(s),
            Some(Token::Minus) => Expr::Neg(Box::new(self.parse_factor())),
            Some(Token::LParen) => {
                let e = self.parse_expr();
                match self.advance() {
                    Some(Token::RParen) => e,
                    other => panic!("parse error: expected ')', got {:?}", other),
                }
            }
            other => panic!("parse error: expected number, identifier, '-', or '(', got {:?}", other),
        }
    }
}

pub fn parse(tokens: Vec<Token>) -> Program {
    let mut p = Parser::new(tokens);
    let program = p.parse_program();
    if p.pos < p.tokens.len() {
        panic!("parse error: trailing tokens: {:?}", &p.tokens[p.pos..]);
    }
    program
}
