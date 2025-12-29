pub mod block;
pub mod call;
pub mod conditional;
pub mod literal;
pub mod loops;
pub mod operations;

use crate::ast::{
    Definition, Path,
    expressions::block::Block,
    expressions::literal::Literal,
    expressions::operations::{BinaryOperation, UnaryOperation},
};

use anyhow::{Result, anyhow};

#[derive(Debug, Clone)]
pub enum Statement {
    // DefinitionKind::Module and DefinitionKind::Trait not allowed
    Definition(Definition),
    Expression(Expression),
    Use(Path),
}

#[derive(Debug, Clone)]
pub enum ExpressionKind {
    Block(Block),
    If {
        condition: Expression,
        then: Expression,
        else_: Option<Expression>,
    },
    Loop {
        body: Expression,
    },
    While {
        condition: Expression,
        then: Expression,
    },

    Literal(Literal),
    Identifier(Path),
    BinaryOperation(BinaryOperation),
    UnaryOperation(UnaryOperation),
    Call {
        called: Expression,
        parameters: Vec<Expression>,
    },
    Return(Expression),
}

#[derive(Debug, Clone)]
pub struct Expression {
    pub kind: Box<ExpressionKind>,
}
