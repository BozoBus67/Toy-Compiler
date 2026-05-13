// LLVM IR backend — phases 1, 2, and 3.
//
// What lands at each phase:
//   phase 1: arithmetic and unary minus  -> add/sub/mul/sdiv/neg in one block
//   phase 2: let bindings + ident reads  -> alloca/store/load + symbol table
//   phase 3: if/else, comparisons,       -> multi-block IR with br + phi,
//            booleans, block expressions    plus i1 typing for booleans
//
// Drive with:
//   cargo run -q -- --llvm "..." > out.ll && clang out.ll -o out && ./out; echo $?

use std::collections::HashMap;

use crate::ast::{BinOp, Block, CmpOp, Expr, Program, Stmt};

// Every emitted value carries its LLVM type — i64 for ints, i1 for booleans.
// That matters because `br` wants an i1 cond, `phi` needs a type, and `ret`
// wants the function's declared return type.
type Value = (String, &'static str);

struct Codegen {
    out: String,
    next_id: u32,
    current_block: String,
    // source name -> (slot operand like "%var1_slot_3", element type like "i64")
    symbols: HashMap<String, (String, &'static str)>,
}

impl Codegen {
    fn new() -> Self {
        Self {
            out: String::new(),
            next_id: 0,
            current_block: "entry".to_string(),
            symbols: HashMap::new(),
        }
    }

    fn fresh(&mut self, hint: &str) -> String {
        let id = self.next_id;
        self.next_id += 1;
        format!("%{}{}", hint, id)
    }

    fn fresh_label(&mut self, base: &str) -> String {
        let id = self.next_id;
        self.next_id += 1;
        format!("{}{}", base, id)
    }

    fn emit_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Let { name, value } => {
                let (v, ty) = self.emit_expr(value);
                let slot = self.fresh(&format!("{}_slot_", name));
                self.out.push_str(&format!("  {} = alloca {}\n", slot, ty));
                self.out.push_str(&format!("  store {} {}, ptr {}\n", ty, v, slot));
                self.symbols.insert(name.clone(), (slot, ty));
            }
        }
    }

    fn emit_block(&mut self, block: &Block) -> Value {
        // Snapshot the symbol table so bindings declared inside the block
        // don't leak out — block scoping. Stack slots themselves remain
        // allocated for the function's lifetime, only the name->slot map
        // is restored.
        let saved = self.symbols.clone();
        for stmt in &block.stmts {
            self.emit_stmt(stmt);
        }
        let result = self.emit_expr(&block.result);
        self.symbols = saved;
        result
    }

    fn emit_expr(&mut self, expr: &Expr) -> Value {
        match expr {
            Expr::Num(n) => (n.to_string(), "i64"),
            Expr::Bool(b) => ((if *b { "true" } else { "false" }).to_string(), "i1"),
            Expr::Ident(name) => {
                let (slot, ty) = self
                    .symbols
                    .get(name)
                    .cloned()
                    .unwrap_or_else(|| panic!("codegen: undefined identifier {:?}", name));
                let dest = self.fresh("v");
                self.out
                    .push_str(&format!("  {} = load {}, ptr {}\n", dest, ty, slot));
                (dest, ty)
            }
            Expr::Neg(inner) => {
                let (v, _) = self.emit_expr(inner);
                let dest = self.fresh("v");
                self.out
                    .push_str(&format!("  {} = sub i64 0, {}\n", dest, v));
                (dest, "i64")
            }
            Expr::Bin { op, left, right } => {
                let (l, _) = self.emit_expr(left);
                let (r, _) = self.emit_expr(right);
                let mnemonic = match op {
                    BinOp::Add => "add",
                    BinOp::Sub => "sub",
                    BinOp::Mul => "mul",
                    BinOp::Div => "sdiv",
                };
                let dest = self.fresh("v");
                self.out.push_str(&format!(
                    "  {} = {} i64 {}, {}\n",
                    dest, mnemonic, l, r
                ));
                (dest, "i64")
            }
            Expr::Cmp { op, left, right } => {
                let (l, _) = self.emit_expr(left);
                let (r, _) = self.emit_expr(right);
                let pred = match op {
                    CmpOp::Lt => "slt",
                    CmpOp::Gt => "sgt",
                    CmpOp::Le => "sle",
                    CmpOp::Ge => "sge",
                    CmpOp::Eq => "eq",
                    CmpOp::Ne => "ne",
                };
                let dest = self.fresh("v");
                self.out.push_str(&format!(
                    "  {} = icmp {} i64 {}, {}\n",
                    dest, pred, l, r
                ));
                (dest, "i1")
            }
            Expr::If {
                cond,
                then_branch,
                else_branch,
            } => {
                let (cv, _) = self.emit_expr(cond);
                let then_label = self.fresh_label("if.then.");
                let else_label = self.fresh_label("if.else.");
                let merge_label = self.fresh_label("if.merge.");

                self.out.push_str(&format!(
                    "  br i1 {}, label %{}, label %{}\n",
                    cv, then_label, else_label
                ));

                self.out.push_str(&format!("{}:\n", then_label));
                self.current_block = then_label.clone();
                let (tv, t_ty) = self.emit_block(then_branch);
                // The branch we'll jump to merge from is the CURRENT block at
                // this moment — which might not be `then_label` anymore if the
                // then-branch contained nested control flow. Capture it now.
                let then_exit = self.current_block.clone();
                self.out
                    .push_str(&format!("  br label %{}\n", merge_label));

                self.out.push_str(&format!("{}:\n", else_label));
                self.current_block = else_label.clone();
                let (ev, e_ty) = self.emit_block(else_branch);
                let else_exit = self.current_block.clone();
                self.out
                    .push_str(&format!("  br label %{}\n", merge_label));

                assert_eq!(
                    t_ty, e_ty,
                    "codegen: if/else branches must produce matching types"
                );

                self.out.push_str(&format!("{}:\n", merge_label));
                self.current_block = merge_label;
                let dest = self.fresh("v");
                self.out.push_str(&format!(
                    "  {} = phi {} [{}, %{}], [{}, %{}]\n",
                    dest, t_ty, tv, then_exit, ev, else_exit
                ));
                (dest, t_ty)
            }
            Expr::Block(b) => self.emit_block(b),
        }
    }
}

pub fn codegen(program: &Program) -> String {
    let mut cg = Codegen::new();
    cg.out.push_str("define i64 @main() {\n");
    cg.out.push_str("entry:\n");

    for stmt in &program.stmts {
        cg.emit_stmt(stmt);
    }
    let (result, ty) = cg.emit_expr(&program.result);

    // main is declared to return i64. If the program's final value is an i1
    // (a boolean), zero-extend it so `ret i64 ...` is well-typed and the
    // shell sees 0/1 for false/true.
    let final_result = if ty == "i64" {
        result
    } else {
        let z = cg.fresh("v");
        cg.out
            .push_str(&format!("  {} = zext {} {} to i64\n", z, ty, result));
        z
    };
    cg.out
        .push_str(&format!("  ret i64 {}\n", final_result));
    cg.out.push_str("}\n");
    cg.out
}
