// Hand-rolled x86-64 backend for phase 1 only (pure arithmetic on i64).
//
// What this exists to teach:
//   - tree-walk register allocation (one register per live value, recurse)
//   - the x86-64 register pool we can clobber freely (caller-saved scratch)
//   - what the function prologue/epilogue look like and why
//   - the return-in-rax convention on macOS/Linux x86-64
//
// What it deliberately *doesn't* handle, and why it stops there:
//   - division: idiv requires rdx:rax / src and writes quotient to rax,
//     remainder to rdx. That breaks the "any register from the pool" assumption
//     and would force us to special-case rax/rdx. LLVM IR makes this disappear.
//   - spilling: if the expression tree is deeper than the pool, we panic.
//     A real backend evaluates the heavier subtree first (Sethi-Ullman) and
//     spills to the stack when forced. Both are nontrivial.
//   - let bindings: would need a symbol table mapping name -> stack slot,
//     plus alloca-style stack frame management (sub rsp, N; mov [rbp-k], reg).
//   - if/else: would need labels and conditional branches; the value of an
//     `if` expression has to land in the same place from both branches.
//   - function calls: full System V AMD64 ABI — args in rdi/rsi/rdx/rcx/r8/r9,
//     16-byte stack alignment before `call`, caller-saved regs preserved
//     across calls.
//
// We're stopping at phase 1 so the *exposure* lands: you can read the asm,
// see exactly which register holds which subexpression, and feel where
// "what if I run out?" starts to bite. Then we hand off to LLVM IR.

use crate::ast::{BinOp, Expr};

struct RegPool {
    free: Vec<&'static str>,
}

impl RegPool {
    fn new() -> Self {
        // Caller-saved scratch regs. rax is reserved (return value).
        // rsp/rbp are reserved (stack frame). rbx is callee-saved in the
        // System V ABI but since we're a leaf function that doesn't call
        // anyone, we can clobber it without saving — included here for
        // a slightly larger pool. (A non-leaf function would need to push/pop it.)
        // Listed so .pop() hands out r11 first, then r10, etc.
        Self {
            free: vec!["rbx", "rcx", "rdx", "r8", "r9", "r10", "r11"],
        }
    }

    fn alloc(&mut self) -> &'static str {
        self.free
            .pop()
            .expect("native demo: ran out of registers (spilling not implemented)")
    }

    fn free(&mut self, reg: &'static str) {
        self.free.push(reg);
    }
}

pub fn codegen(expr: &Expr) -> String {
    let mut out = String::new();
    out.push_str(".intel_syntax noprefix\n");
    out.push_str(".global _main\n");
    out.push_str("_main:\n");
    out.push_str("    push rbp\n");
    out.push_str("    mov  rbp, rsp\n");

    let mut pool = RegPool::new();
    emit(expr, "rax", &mut pool, &mut out);

    out.push_str("    pop  rbp\n");
    out.push_str("    ret\n");
    out
}

fn emit(expr: &Expr, dest: &str, pool: &mut RegPool, out: &mut String) {
    match expr {
        Expr::Num(n) => {
            out.push_str(&format!("    mov  {}, {}\n", dest, n));
        }
        Expr::Neg(inner) => {
            emit(inner, dest, pool, out);
            out.push_str(&format!("    neg  {}\n", dest));
        }
        Expr::Bin { op, left, right } => {
            // Compute left into the destination register, then compute right
            // into a freshly-allocated temporary, then fold them with one op.
            emit(left, dest, pool, out);
            let tmp = pool.alloc();
            emit(right, tmp, pool, out);
            let mnemonic = match op {
                BinOp::Add => "add ",
                BinOp::Sub => "sub ",
                BinOp::Mul => "imul",
                BinOp::Div => panic!(
                    "native demo: division omitted — idiv's fixed rdx:rax/src \
                     register constraints don't fit a generic pool-based scheme. \
                     LLVM hides this."
                ),
            };
            out.push_str(&format!("    {} {}, {}\n", mnemonic, dest, tmp));
            pool.free(tmp);
        }
        _ => panic!("native demo: only phase 1 (arithmetic) supported"),
    }
}
