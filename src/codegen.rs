// LLVM IR backend — phases 1, 2, 3, and 4.
//
// What lands at each phase:
//   phase 1: arithmetic and unary minus  -> add/sub/mul/sdiv/neg in one block
//   phase 2: let bindings + ident reads  -> alloca/store/load + symbol table
//   phase 3: if/else, comparisons,       -> multi-block IR with br + phi,
//            booleans, block expressions    plus i1 typing for booleans
//   phase 4: function defs + calls       -> multiple `define`s per module,
//            with recursion                 `call`, per-function SSA reset
//
// Drive with:
//   cargo run -q -- --llvm "..." > out.ll && clang out.ll -o out && ./out; echo $?

use std::collections::HashMap;

use crate::ast::{BinOp, Block, CmpOp, Expr, FnDef, Program, Stmt};

// Every emitted value carries its LLVM type — i64 for ints, i1 for booleans.
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

    fn reset_fn_state(&mut self) {
        self.next_id = 0;
        self.current_block = "entry".to_string();
        self.symbols.clear();
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

    fn coerce_to_i64(&mut self, v: String, ty: &'static str) -> String {
        if ty == "i64" {
            v
        } else {
            let z = self.fresh("v");
            self.out
                .push_str(&format!("  {} = zext {} {} to i64\n", z, ty, v));
            z
        }
    }

    fn emit_fn(&mut self, fn_def: &FnDef) {
        self.reset_fn_state();

        // Signature. Every param is i64 — our toy language has no parameter
        // type annotations, so we pick i64 and zext booleans at call sites.
        self.out.push_str(&format!("define i64 @{}(", fn_def.name));
        let mut param_bindings = Vec::new();
        for (i, p) in fn_def.params.iter().enumerate() {
            if i > 0 {
                self.out.push_str(", ");
            }
            let pname = format!("%{}_arg", p);
            self.out.push_str(&format!("i64 {}", pname));
            param_bindings.push((p.clone(), pname));
        }
        self.out.push_str(") {\n");
        self.out.push_str("entry:\n");

        // Pour each SSA parameter into a stack slot so reads inside the body
        // look identical to reads of `let` bindings. Without this, every
        // Expr::Ident codegen path would need to know "are you a param or a
        // local?" — uniformity is cheaper.
        for (p_name, p_arg) in &param_bindings {
            let slot = self.fresh(&format!("{}_slot_", p_name));
            self.out
                .push_str(&format!("  {} = alloca i64\n", slot));
            self.out
                .push_str(&format!("  store i64 {}, ptr {}\n", p_arg, slot));
            self.symbols.insert(p_name.clone(), (slot, "i64"));
        }

        let (result, ty) = self.emit_block(&fn_def.body);
        let final_result = self.coerce_to_i64(result, ty);
        self.out.push_str(&format!("  ret i64 {}\n", final_result));
        self.out.push_str("}\n\n");
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
                // Capture the CURRENT block right before emitting `br label
                // %merge` — nested control flow inside the branch may have
                // moved us out of `then_label`.
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
            Expr::Call { name, args } => {
                // Lower each arg, coerce to i64 (calling convention here
                // treats all parameters as i64).
                let mut arg_strs = Vec::new();
                for a in args {
                    let (v, ty) = self.emit_expr(a);
                    let v = self.coerce_to_i64(v, ty);
                    arg_strs.push(format!("i64 {}", v));
                }
                let dest = self.fresh("v");
                self.out.push_str(&format!(
                    "  {} = call i64 @{}({})\n",
                    dest,
                    name,
                    arg_strs.join(", ")
                ));
                (dest, "i64")
            }
        }
    }
}

pub fn codegen(program: &Program) -> String {
    let mut cg = Codegen::new();

    // Emit user-defined functions first. LLVM doesn't require forward
    // declarations — calls resolve by symbol name — but the file reads
    // more naturally top-down.
    for fn_def in &program.fns {
        cg.emit_fn(fn_def);
    }

    // Emit @main from program-level lets + final expression.
    cg.reset_fn_state();
    cg.out.push_str("define i64 @main() {\n");
    cg.out.push_str("entry:\n");
    for stmt in &program.stmts {
        cg.emit_stmt(stmt);
    }
    let (result, ty) = cg.emit_expr(&program.result);
    let final_result = cg.coerce_to_i64(result, ty);
    cg.out
        .push_str(&format!("  ret i64 {}\n", final_result));
    cg.out.push_str("}\n");

    cg.out
}
