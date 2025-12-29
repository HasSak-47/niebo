pub mod block;
pub mod call;
pub mod conditional;
pub mod literal;
pub mod loops;
pub mod operations;

use crate::ast::{
    Definition, Path,
    expressions::block::Block,
    expressions::call::Call,
    expressions::conditional::Conditional,
    expressions::literal::Literal,
    expressions::loops::{LoopExpression, WhileLoop},
    expressions::operations::{BinaryOperation, UnaryOperation},
};

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
    If(Conditional),
    Loop(LoopExpression),
    While(WhileLoop),
    Literal(Literal),
    Identifier(Path),
    BinaryOperation(BinaryOperation),
    UnaryOperation(UnaryOperation),
    Call(Call),
    Return(Expression),
}

#[derive(Debug, Clone)]
pub struct Expression {
    pub kind: Box<ExpressionKind>,
}

impl Expression {
    pub fn new(kind: ExpressionKind) -> Self {
        Self {
            kind: Box::new(kind),
        }
    }

    pub fn block(block: Block) -> Self {
        Self::new(ExpressionKind::Block(block))
    }

    pub fn if_(conditional: Conditional) -> Self {
        Self::new(ExpressionKind::If(conditional))
    }

    pub fn loop_(loop_expression: LoopExpression) -> Self {
        Self::new(ExpressionKind::Loop(loop_expression))
    }

    pub fn while_(while_loop: WhileLoop) -> Self {
        Self::new(ExpressionKind::While(while_loop))
    }

    pub fn literal(literal: Literal) -> Self {
        Self::new(ExpressionKind::Literal(literal))
    }

    pub fn identifier(path: Path) -> Self {
        Self::new(ExpressionKind::Identifier(path))
    }

    pub fn binary_operation(operation: BinaryOperation) -> Self {
        Self::new(ExpressionKind::BinaryOperation(operation))
    }

    pub fn unary_operation(operation: UnaryOperation) -> Self {
        Self::new(ExpressionKind::UnaryOperation(operation))
    }

    pub fn call(call: Call) -> Self {
        Self::new(ExpressionKind::Call(call))
    }

    pub fn return_(value: Expression) -> Self {
        Self::new(ExpressionKind::Return(value))
    }
}
