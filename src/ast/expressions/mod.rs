pub mod block;
pub mod call;
pub mod conditional;
pub mod literal;
pub mod loops;
pub mod operations;

use crate::{
    ast::{
        Definition, Import, Path,
        expressions::{
            block::Block,
            call::Call,
            conditional::Conditional,
            literal::Literal,
            loops::{LoopExpression, WhileLoop},
            operations::{BinaryOperation, UnaryOperation},
        },
    },
    general::types::Type,
};

#[derive(Debug, Clone)]
pub enum Statement {
    // DefinitionKind::Module and DefinitionKind::Trait not allowed
    Import(Import),
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
    Return(Option<Expression>),
}

#[derive(Debug, Clone)]
pub struct Expression {
    pub kind: Box<ExpressionKind>,
    pub ret_ty: Option<Type>,
}

impl Expression {
    pub fn new(kind: ExpressionKind) -> Self {
        Self {
            ret_ty: None,
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

    pub fn literal<L: Into<Literal>>(literal: L) -> Self {
        let literal = literal.into();

        Self::new(ExpressionKind::Literal(literal))
    }

    pub fn identifier<P: Into<Path>>(path: P) -> Self {
        let path = path.into();
        Self::new(ExpressionKind::Identifier(path))
    }

    pub fn binary_operation(operation: BinaryOperation) -> Self {
        Self::new(ExpressionKind::BinaryOperation(operation))
    }

    pub fn unary_operation(operation: UnaryOperation) -> Self {
        Self::new(ExpressionKind::UnaryOperation(operation))
    }

    pub fn call(called: Expression, params: Vec<Expression>) -> Self {
        Self::new(ExpressionKind::Call(Call::new(called, params)))
    }

    pub fn return_(value: Option<Expression>) -> Self {
        Self::new(ExpressionKind::Return(value))
    }
}
