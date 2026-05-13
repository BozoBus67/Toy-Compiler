// LLVM IR backend, phase 1 only for now (pure arithmetic).
//
// Compared to native.rs:
//   - no RegPool — LLVM IR has infinite virtual registers (SSA names %0, %1, ...)
//     so we just hand out fresh names from a counter and never spill
//   - no manual prologue/epilogue — `define i64 @main()` declares the function
//     shape, LLVM (via llc) lowers it to the platform ABI
//   - typed — every value carries `i64`. The verifier rejects mismatches
//     before we ever hit the assembler
//   - single basic block (`entry:`) for phase 1 — control flow stays linear
//     so we don't need branches/phi yet
//
// Output is textual LLVM IR (`.ll`). Drive it with:
//   cargo run -q -- --llvm "1 + 2 * 3" > out.ll
//   clang out.ll -o out
//   ./out; echo $?    # -> 7

use crate::ast::{BinOp, Expr};

pub fn codegen(expr: &Expr) -> String {
    let mut out = String::new();
    out.push_str("define i64 @main() {\n");
    out.push_str("entry:\n");

    let mut counter: u32 = 0;
    let result = emit(expr, &mut counter, &mut out);

    out.push_str(&format!("  ret i64 {}\n", result));
    out.push_str("}\n");
    out
}

// Returns the operand string holding this expression's value — either an SSA
// name like "%3" or an inline literal like "42". LLVM IR lets either appear
// wherever an `i64` operand is expected.
fn emit(expr: &Expr, counter: &mut u32, out: &mut String) -> String {
    match expr {
        Expr::Num(n) => n.to_string(),
        Expr::Neg(inner) => {
            let v = emit(inner, counter, out);
            let name = fresh(counter);
            out.push_str(&format!("  {} = sub i64 0, {}\n", name, v));
            name
        }
        Expr::Bin { op, left, right } => {
            let l = emit(left, counter, out);
            let r = emit(right, counter, out);
            let mnemonic = match op {
                BinOp::Add => "add",
                BinOp::Sub => "sub",
                BinOp::Mul => "mul",
                BinOp::Div => "sdiv",
            };
            let name = fresh(counter);
            out.push_str(&format!("  {} = {} i64 {}, {}\n", name, mnemonic, l, r));
            name
        }
        _ => panic!("codegen: phase 1 only — let bindings, comparisons, if/else, and blocks come in later passes"),
    }
}

fn fresh(counter: &mut u32) -> String {
    let n = *counter;
    *counter += 1;
    format!("%{}", n)
}
