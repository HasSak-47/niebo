use crate::ast::{Definition, Expression, Import, QualifiedName};

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    // DefinitionKind::Module, DefinitionKind::Trait, and impl blocks not allowed
    Import(Import),
    Definition(Definition),
    Expression(Expression),
    Break(Option<String>),
    Continue,
    Use(QualifiedName),
    Return(Option<Expression>),

    // return value for the block expression
    // no implicit returns! fuck you rust
    Value(Option<Expression>),
}
