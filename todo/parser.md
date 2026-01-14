# Parser TODO (Pest)

Missing grammar needed to build the current AST in `src/ast/**.rs`:

- Definition visibility modifiers (`pub`, `mod`, private/default) for `Definition`/`Trait`/`Function`.
- Module definitions (nested `Module` in `DefinitionKind::Module`) and a module syntax rule.
- Variable declarations: `let` should support `mut` to map to `Variable::mutable`.
- Function features: `const fn`/`fn` distinction and a varidic marker (AST has `varidic`).
- Statements: `break`, `continue`, and `return` (optional expression) to map to `Statement::Break`, `Statement::Continue`, and `ExpressionKind::Return`.
- `use` statements distinct from imports if you intend to map to `Statement::Use(Path)` (current grammar only treats `use` as `import`).
- Expression coverage:
  - `loop { ... }` (AST has `LoopExpression`).
  - Function calls (`Call`) with argument lists.
  - Operator precedence/associativity rules that map to `BinaryOperation`/`UnaryOperation` (currently a flat binary rule).
- Literal forms:
  - Integer sign (`-`), signedness, and precision suffixes.
  - Float precision suffixes.
- Type syntax to cover `general::types::Type`:
  - Primitive type keywords (`bool`, `int`, `uint`, `float`, `string`, `void`).
  - Struct/union/variant literal types and member lists.
  - Array types, pointer/reference types (mutable + immutable), function types, and template types/constraints.
