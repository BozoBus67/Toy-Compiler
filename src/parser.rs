use crate::ast::{BinOp, Block, CmpOp, Expr, Program, Stmt};
use crate::lexer::Token;

// Recursive descent, LL(1).
//
// Grammar:
//   program  = { let_stmt } expr
//   let_stmt = "let" IDENT "=" expr ";"
//   expr     = cmp_expr
//   cmp_expr = add_expr [ cmp_op add_expr ]                     (non-associative)
//   cmp_op   = "<" | ">" | "<=" | ">=" | "==" | "!="
//   add_expr = mul_expr { ("+" | "-") mul_expr }
//   mul_expr = factor   { ("*" | "/") factor }
//   factor   = NUMBER | "true" | "false" | IDENT
//            | "-" factor | "(" expr ")"
//            | block | if_expr
//   block    = "{" { let_stmt } expr "}"
//   if_expr  = "if" expr block "else" block

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
        let (stmts, result) = self.parse_block_body();
        Program { stmts, result }
    }

    fn parse_block_body(&mut self) -> (Vec<Stmt>, Expr) {
        let mut stmts = Vec::new();
        while let Some(Token::Let) = self.peek() {
            stmts.push(self.parse_let_stmt());
        }
        let result = self.parse_expr();
        (stmts, result)
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
        self.parse_cmp_expr()
    }

    fn parse_cmp_expr(&mut self) -> Expr {
        let left = self.parse_add_expr();
        let op = match self.peek() {
            Some(Token::Lt) => CmpOp::Lt,
            Some(Token::Gt) => CmpOp::Gt,
            Some(Token::Le) => CmpOp::Le,
            Some(Token::Ge) => CmpOp::Ge,
            Some(Token::EqEq) => CmpOp::Eq,
            Some(Token::BangEq) => CmpOp::Ne,
            _ => return left,
        };
        self.advance();
        let right = self.parse_add_expr();
        Expr::Cmp {
            op,
            left: Box::new(left),
            right: Box::new(right),
        }
    }

    fn parse_add_expr(&mut self) -> Expr {
        let mut left = self.parse_mul_expr();
        loop {
            let op = match self.peek() {
                Some(Token::Plus) => BinOp::Add,
                Some(Token::Minus) => BinOp::Sub,
                _ => return left,
            };
            self.advance();
            let right = self.parse_mul_expr();
            left = Expr::Bin {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
    }

    fn parse_mul_expr(&mut self) -> Expr {
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
            Some(Token::True) => Expr::Bool(true),
            Some(Token::False) => Expr::Bool(false),
            Some(Token::Ident(s)) => Expr::Ident(s),
            Some(Token::Minus) => Expr::Neg(Box::new(self.parse_factor())),
            Some(Token::LParen) => {
                let e = self.parse_expr();
                match self.advance() {
                    Some(Token::RParen) => e,
                    other => panic!("parse error: expected ')', got {:?}", other),
                }
            }
            Some(Token::LBrace) => {
                let (stmts, result) = self.parse_block_body();
                match self.advance() {
                    Some(Token::RBrace) => Expr::Block(Box::new(Block { stmts, result })),
                    other => panic!("parse error: expected '}}', got {:?}", other),
                }
            }
            Some(Token::If) => {
                let cond = Box::new(self.parse_expr());
                let then_branch = Box::new(self.parse_brace_block());
                match self.advance() {
                    Some(Token::Else) => {}
                    other => panic!("parse error: expected 'else' after if-then, got {:?}", other),
                }
                let else_branch = Box::new(self.parse_brace_block());
                Expr::If { cond, then_branch, else_branch }
            }
            other => panic!("parse error: expected primary expression, got {:?}", other),
        }
    }

    fn parse_brace_block(&mut self) -> Block {
        match self.advance() {
            Some(Token::LBrace) => {}
            other => panic!("parse error: expected '{{', got {:?}", other),
        }
        let (stmts, result) = self.parse_block_body();
        match self.advance() {
            Some(Token::RBrace) => {}
            other => panic!("parse error: expected '}}', got {:?}", other),
        }
        Block { stmts, result }
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
