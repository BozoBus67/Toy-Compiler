# tinylang

A tiny made-up language we compile to LLVM IR, then through LLVM's toolchain to a native binary.

Built in phases. Each phase adds **one** concept that forces exposure to **one** new compiler/toolchain layer. Rename freely.

File extension: `.tl`

---

## Phase 1: arithmetic expressions

A program is a single integer expression. Its value becomes the program's exit code.

### Grammar (EBNF-ish)

```
program  = expr ;
expr     = term   { ("+" | "-") term } ;
term     = factor { ("*" | "/") factor } ;
factor   = NUMBER
         | "-" factor
         | "(" expr ")" ;
NUMBER   = digit { digit } ;
digit    = "0" | "1" | ... | "9" ;
```

---

## Phase 2: let bindings

A program is zero or more `let` bindings followed by a final expression. Bindings live in a single flat scope (no nested blocks yet). The final expression's value is the program's exit code.

### Grammar (EBNF-ish)

```
program  = { let_stmt } expr ;
let_stmt = "let" IDENT "=" expr ";" ;
expr     = term   { ("+" | "-") term } ;
term     = factor { ("*" | "/") factor } ;
factor   = NUMBER
         | IDENT
         | "-" factor
         | "(" expr ")" ;
IDENT    = letter { letter | digit | "_" } ;
letter   = "a" | ... | "z" | "A" | ... | "Z" | "_" ;
```

### Keywords

`let` is reserved and cannot be used as an identifier.

### Examples

```
let x = 5;
let y = 10;
x + y               # = 15

let a = 1 + 2;
let b = a * 3;
b - a               # = 6
```

### Scoping

One flat scope. Later `let` shadows earlier (TBD — for now: redefining is allowed and the latest binding wins at codegen). Forward references are an error.

---

## Phase 3: control flow

`if`/`else` expressions, comparisons producing booleans, block expressions, and the `true`/`false` literals. `if` is an *expression* — its value is the value of whichever branch ran. Both branches are blocks (`{ let_stmt* expr }`). Comparisons are non-associative — at most one comparison op per `cmp_expr` level (`a < b < c` is a parse error).

### Grammar (EBNF-ish)

```
program  = { let_stmt } expr ;
let_stmt = "let" IDENT "=" expr ";" ;
expr     = cmp_expr ;
cmp_expr = add_expr [ cmp_op add_expr ] ;
cmp_op   = "<" | ">" | "<=" | ">=" | "==" | "!=" ;
add_expr = mul_expr { ("+" | "-") mul_expr } ;
mul_expr = factor   { ("*" | "/") factor } ;
factor   = NUMBER | "true" | "false" | IDENT
         | "-" factor | "(" expr ")"
         | block | if_expr ;
block    = "{" { let_stmt } expr "}" ;
if_expr  = "if" expr block "else" block ;
```

### Keywords added

`if`, `else`, `true`, `false`.

### Precedence (low → high)

1. comparisons (`< > <= >= == !=`) — non-associative
2. additive (`+ -`)
3. multiplicative (`* /`)
4. unary `-`
5. parens, blocks, `if`-expressions, literals

### Examples

```
if 1 < 2 { 10 } else { 20 }                       # = 10
let x = 5; if x == 5 { x * 2 } else { x }         # = 10
{ let a = 1; let b = 2; a + b }                   # = 3
if (1 + 2) * 3 == 9 { 1 } else { 0 }              # = 1
```

### Types

Comparisons return a boolean. `if`'s condition must be a boolean. Arithmetic operates on integers only. Types aren't checked at parse time — semantic checking will live in codegen (or a dedicated typeck pass later).

---

## Phase 4: functions and recursion

Top-level function definitions with positional parameters and recursion. Every parameter is implicitly an integer (the only "general" type the language has — booleans are zero-extended on the way in if needed). A program can now contain `fn`-blocks before its `main` body; the main body is still `{ let_stmt } expr`.

### Grammar (EBNF-ish)

```
program  = { fn_def } { let_stmt } expr ;
fn_def   = "fn" IDENT "(" [ IDENT { "," IDENT } ] ")" block ;
factor   = NUMBER | "true" | "false"
         | IDENT [ "(" [ expr { "," expr } ] ")" ]
         | "-" factor | "(" expr ")"
         | block | if_expr ;
```

Everything else is unchanged from phase 3.

### Keywords added

`fn`.

### Semantics

- Functions are top-level. No nested functions, no closures.
- Recursion is allowed (a function can call itself by name).
- Forward references are allowed — definition order in source doesn't matter at the IR level since calls are resolved by symbol name.
- Every parameter is i64 (booleans coerce via `zext` at call sites).
- Every function returns i64 (booleans coerce on the way out).

### Examples

```
fn add(x, y) { x + y }
let r = add(3, 4);
r                                                # = 7

fn fact(n) {
  if n == 0 { 1 } else { n * fact(n - 1) }
}
fact(5)                                          # = 120

fn fib(n) {
  if n < 2 { n } else { fib(n - 1) + fib(n - 2) }
}
fib(10)                                          # = 55
```

The `{ ... }` means "zero or more." Curly braces (instead of writing recursion) implicitly produce **left-associative** parsing in a recursive-descent parser, which is what you want for `1 - 2 - 3 = (1 - 2) - 3 = -4`.

### Precedence (low → high)

1. `+` `-` (binary, left-associative)
2. `*` `/` (binary, left-associative)
3. unary `-`
4. parentheses

### Examples

```
1 + 2 * 3           # = 7
(1 + 2) * 3         # = 9
-5 + 10             # = 5
100 / 3             # = 33   (integer division, truncates toward zero)
-(2 + 3) * 4        # = -20
```

### Whitespace

Spaces, tabs, newlines (`\n`, `\r`) — all ignored.

### Errors

Anything that doesn't fit the grammar = parse error. Phase 1 just panics with a message. Real error recovery comes later.

### Integer semantics

64-bit signed (`i64`). Overflow is undefined behavior for now (LLVM IR's `add` instruction doesn't check). Division by zero is also UB for now.

---

## Roadmap

| Phase | Adds | Forces you to learn |
|-------|------|---------------------|
| 1 | arithmetic expressions | lexer, parser, AST design, LLVM IR basics, `llc`/`clang` pipeline |
| 2 | `let` bindings, multi-statement programs (`;` separator) | scopes, symbol tables, IR stack slots (`alloca`, `store`, `load`) ← parser landed; codegen still pending |
| 3 | `if`/`else`, comparisons (`< > <= >= == !=`), booleans, blocks | basic blocks, branches, phi nodes, SSA form |
| 4 | functions with parameters, recursion | **System V AMD64 ABI** in IR (`define`/`call`), multiple functions per module, per-function SSA namespace |
| 5 | `print(x)` for integers, then strings | libc interop, **linker relocations**, global constants, `declare i32 @printf(...)` |
| 6 | loops (`while`), arrays | mutable state in SSA, GEP (`getelementptr`), bounds (optional) |
| 7+ | (open) — types, structs, modules, optimization passes, your own backend... | as needed |

Each phase is one small grammar extension + one new IR construct + one new toolchain insight.
