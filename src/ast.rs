#[derive(Debug, Clone, Copy)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
}

#[derive(Debug)]
pub enum Expr {
    Num(i64),
    Ident(String),
    Neg(Box<Expr>),
    Bin {
        op: BinOp,
        left: Box<Expr>,
        right: Box<Expr>,
    },
}

#[derive(Debug)]
pub enum Stmt {
    Let { name: String, value: Expr },
}

#[derive(Debug)]
pub struct Program {
    pub stmts: Vec<Stmt>,
    pub result: Expr,
}
