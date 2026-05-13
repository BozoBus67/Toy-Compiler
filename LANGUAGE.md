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
| 3 | `if`/`else`, comparisons (`< > == !=`), booleans | basic blocks, branches, phi nodes, SSA form |
| 4 | functions with parameters, recursion | **System V AMD64 ABI** in IR, call convention, multiple functions per module |
| 5 | `print(x)` for integers, then strings | libc interop, **linker relocations**, global constants, `declare i32 @printf(...)` |
| 6 | loops (`while`), arrays | mutable state in SSA, GEP (`getelementptr`), bounds (optional) |
| 7+ | (open) — types, structs, modules, optimization passes, your own backend... | as needed |

Each phase is one small grammar extension + one new IR construct + one new toolchain insight.
