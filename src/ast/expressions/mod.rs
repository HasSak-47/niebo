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
            operations::{BinaryOperation, BinaryOperator, UnaryOperation, UnaryOperator},
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
    Break,
    Continue,
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
    pub fn from_literal(l: Literal) -> Self {
        return Expression {
            kind: Box::new(ExpressionKind::Literal(l)),
            ret_ty: None,
        };
    }
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

    pub fn binary_operation(operation: BinaryOperator, a: Expression, b: Expression) -> Self {
        Self::new(ExpressionKind::BinaryOperation(BinaryOperation {
            operator: operation,
            operands: [a, b],
        }))
    }

    pub fn unary_operation(operator: UnaryOperator, operand: Expression) -> Self {
        return Self::new(ExpressionKind::UnaryOperation(UnaryOperation {
            operator: operator,
            operand: operand,
        }));
    }

    pub fn call(called: Expression, params: Vec<Expression>) -> Self {
        Self::new(ExpressionKind::Call(Call::new(called, params)))
    }

    pub fn return_(value: Option<Expression>) -> Self {
        Self::new(ExpressionKind::Return(value))
    }
}
