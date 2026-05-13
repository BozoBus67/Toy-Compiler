#[derive(Debug, Clone, Copy)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
}

#[derive(Debug, Clone, Copy)]
pub enum CmpOp {
    Lt,
    Gt,
    Le,
    Ge,
    Eq,
    Ne,
}

#[derive(Debug)]
pub enum Expr {
    Num(i64),
    Bool(bool),
    Ident(String),
    Neg(Box<Expr>),
    Bin {
        op: BinOp,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    Cmp {
        op: CmpOp,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    If {
        cond: Box<Expr>,
        then_branch: Box<Block>,
        else_branch: Box<Block>,
    },
    Block(Box<Block>),
}

#[derive(Debug)]
pub enum Stmt {
    Let { name: String, value: Expr },
}

#[derive(Debug)]
pub struct Block {
    pub stmts: Vec<Stmt>,
    pub result: Expr,
}

#[derive(Debug)]
pub struct Program {
    pub stmts: Vec<Stmt>,
    pub result: Expr,
}
