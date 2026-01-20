# AST -> IR Roadmap

## Pipeline Overview
- Parse -> build raw AST.
- Validate -> structural + scope + type rules; collect diagnostics.
- Fill -> resolve names, infer/annotate types, desugar sugar forms.
- Lower -> construct IR (or reuse typed/desugared AST as IR).

## 1) Parse
- Expand `parse_module` to emit `DefinitionKind::Function`, `DefinitionKind::Variable`, and expression trees.
- Keep `Expression.ret_ty = None` at parse time.

## 2) Structural Validation (AST-only)
- Block restrictions: forbid `DefinitionKind::Module` and `DefinitionKind::Trait` inside blocks.
- Duplicate names: check within the same module/block scope.
- Imports: ensure `c_import` has exactly two segments.
- Control flow: `break/continue` only inside loops; `return` only inside functions.
- While/loop: ensure valid condition/body shapes.

## 3) Name Resolution
- Build a symbol table per module with definitions + imports + external modules.
- Resolve `ExpressionKind::Identifier(Path)` to a definition/type/trait.
- Decide storage: add a `resolved` field or keep a side table keyed by node id.
- Resolve operator paths for desugared calls (e.g., `core::op::Add::add`).

## 4) Type Checking + Inference (fill `ret_ty`)
- Implement `evaluate_expression` cases:
  - Literal: map to primitive types.
  - Identifier: lookup resolved symbol type.
  - Call: verify params, set return type.
  - Return: ensure inside function, set `void` or returned type.
  - Loop: define return type behavior.
- Validate function body return type matches signature.

## 5) Desugaring (already in IR)
- Rewrite `while` -> `loop + if + break`.
- Rewrite binary/unary ops -> calls into `core::op`.
- Run before typing so typing is done on uniform forms.

## 6) Lower to IR
- `IR::from_project` can remain: add core module, collect traits, desugar, type-check.
- If IR diverges from AST later, lower into a separate IR struct after typing.

## Recommended Order in Code
1) `parse_module` -> raw `Project/Module`.
2) `ast::validate::structural(&project)`.
3) `ast::resolve::names(&mut project)` (or side table).
4) `ast::typeck::infer(&mut project)` fills `Expression.ret_ty`.
5) `ir::from_project(project)` (desugar + final checks).
