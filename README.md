# Niebo

Niebo is an experimental compiler and language project. It is aimed at end-to-end compiler construction rather than isolated parsing work, so the repository covers the full path from grammar and parsing through AST construction, project loading, preprocessing, LLVM lowering, and native object generation.

The current implementation already includes a custom frontend built with Pest, a project model loaded from `chmura.toml`, a code generator built on Inkwell, and early C interop support through `libclang`. During code generation, local variables and callable functions are tracked separately, which keeps lvalue handling and call lowering clean and matches the shape of an LLVM-backed compiler.

## Language Scope

Niebo is intended as a systems language with explicit types, compiled native output, and visible low-level interop. At the language-design level, the project is aiming for modules, interfaces, inherent method blocks, trait-style extension blocks, algebraic and low-level data types, generics, pattern matching, and direct access to C headers through declarations such as `header stdio::printf;`.

## Example

```niebo
header stdio::printf;

func main() -> testType {
    let mut i : i32 = 0;
    while i < 10 {
        printf("%d\n", i);
        i = i + 1;
    }
}
```

## How To Use

For normal use, start from the `stable` branch:

```bash
git checkout stable
```

The `main` branch may contain work-in-progress compiler changes, while `stable` is the better default for trying the project, demos, and portfolio review.

The compiler can load either a Niebo project directory or a single `.nb` script file. To compile a project, run:

```bash
cargo run -- ./test --mode project --out out.o
```

This treats `./test` as a project directory containing `chmura.toml` and a `src/` tree. To compile a standalone script instead, run:

```bash
cargo run -- ./test/src/main.nb --mode script --out out.o
```

In both cases, `PATH` is the input path, `--mode` selects whether that path is interpreted as a project or a single script, and `--out` chooses the backend output path.

## Output

The backend currently writes LLVM-generated output to the path you provide. In practice, the pipeline is structured around LLVM IR generation followed by native object emission.

## Tech Stack

The project is built primarily

- Rust
- Pest
- Inkwell
- LLVM
- `libclang`
- `serde`
- `toml`

## Status

The parser and AST already describe more of that surface than the backend fully lowers today. In practice, the implemented subset is smaller but real: functions, blocks, local variables, integer and string literals, arithmetic and comparison expressions, assignments, `while` loops, direct calls, and C-style imports are all present in the current pipeline.

Niebo is in active development. The frontend and backend pipeline are functional for a limited subset of the language, while broader type resolution, richer expression support, and more complete lowering are still being built out.
