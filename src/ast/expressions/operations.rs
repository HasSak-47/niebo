use crate::general::path::Path;

use super::Expression;

#[derive(Debug, Clone)]
pub enum BinaryOperator {
    // boolean operations
    Greater,
    Lesser,
    GreaterOrEqual,
    LesserOrEqual,
    Equal,
    NotEqual,

    // arithmetic operations
    Addition,
    Multiplication,
    Substraction,
    Division,
    Module,

    // bitwise operations
    BitShitLeft,
    BitShitRight,
    BitwiseOr,
    BitwiseAnd,
    BitwiseXor,
}

impl BinaryOperator {
    pub fn into_path(&self) -> Path {
        let val = match self {
            BinaryOperator::Addition => "add",
            _ => todo!(),
        }
        .to_string();
        return Path {
            v: vec![val.into(), "core".into()],
        };
    }
}

#[derive(Debug, Clone)]
pub enum UnaryOperator {
    Deref,
    Ref,
    Negation,
}

#[derive(Debug, Clone)]
pub struct UnaryOperation {
    pub operator: UnaryOperator,
    pub operand: Expression,
}

#[derive(Debug, Clone)]
pub struct BinaryOperation {
    pub operator: BinaryOperator,
    pub operands: [Expression; 2],
}

impl UnaryOperation {
    pub fn new(operator: UnaryOperator, operand: Expression) -> Self {
        Self { operator, operand }
    }
}

impl BinaryOperation {
    pub fn new(operator: BinaryOperator, left: Expression, right: Expression) -> Self {
        Self {
            operator,
            operands: [left, right],
        }
    }
}
