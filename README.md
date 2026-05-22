# Niebo

Niebo is an experimental systems-programming language and compiler written in Rust. The project parses a custom source language, builds an AST, performs early project preprocessing, resolves selected C imports through `libclang`, and lowers the result into LLVM IR with `inkwell`.

This is a compiler engineering project intended to demonstrate language implementation work: parsing, AST design, symbol handling, foreign-function integration, and LLVM-based code generation.

## Highlights

- Custom language frontend built with `pest`
- Structured AST for modules, functions, statements, expressions, and types
- Project loader that reads a `chmura.toml` manifest and source tree
- Early preprocessing stage for future name and type resolution
- C header import support via `libclang`
- LLVM IR and object file generation through `inkwell`
- Explicit split between statement lowering and expression lowering in codegen

## Current Capabilities

The compiler can currently:

- Parse functions, blocks, variable definitions, calls, binary expressions, `while` loops, and string/integer literals
- Handle `\n` in string literals during parsing
- Resolve selected C function declarations from imported headers
- Emit LLVM IR to `out.ll`
- Emit an object file to `out.o`

The current test program in [`test/src/main.nb`](./test/src/main.nb) demonstrates:

- C import of `stdio::printf`
- local variable allocation and assignment
- integer comparison and addition
- `while` loop lowering
- external function calls

## Example

```niebo
header stdio::printf;

fn main() -> testType {
    let mut i : i32 = 0;
    while i < 10 {
        printf("%d\n", i);
        i = i + 1;
    }
}
```

## Architecture

- `src/parser/`
  Parses Niebo source into AST nodes using `pest`.
- `src/ast/`
  Defines the language syntax tree and project model.
- `src/ir/`
  Contains C import resolution and IR-related support code.
- `src/main.rs`
  Drives project loading, preprocessing, symbol registration, LLVM lowering, and object generation.

The code generator currently maintains separate symbol tables for:

- local variables as LLVM pointer values
- callable functions as LLVM function values

This keeps lvalue handling and function calls distinct, which is important for a compiler targeting SSA/LLVM IR.

## Build

```bash
cargo check
```

Run parser tests:

```bash
cargo test
```

Compile the sample project:

```bash
cargo run
```

This produces:

- `out.ll` — generated LLVM IR
- `out.o` — generated object file

## Tech Stack

- Rust
- `pest`
- `inkwell`
- LLVM
- `libclang`
- `serde` / `toml`

## Why This Project Matters

Niebo is a compiler project focused on core language implementation work rather than framework usage. It demonstrates:

- parser and grammar design
- AST and IR-oriented architecture
- symbol resolution strategy
- foreign-function interoperability
- LLVM code generation
- incremental compiler construction in a strongly typed language

## Status

This project is in active development. The frontend and code generation pipeline are functional for a small subset of the language, while broader type resolution, richer expression support, and more complete lowering are still in progress.
