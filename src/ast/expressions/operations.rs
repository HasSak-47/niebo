use super::*;
use anyhow::{Result, anyhow};

#[derive(Debug, Clone)]
pub enum BinaryOperator {
    Addition,
    Multiplication,
    Substraction,
    Division,
    Module,
    BitShitLeft,
    BitShitRight,
    Or,
    And,
    Xor,
}

#[derive(Debug, Clone)]
pub enum UnaryOperator {
    Deref,
    Ref,
    Negation,
}

#[derive(Debug, Clone)]
pub struct UnaryOperation {
    operator: BinaryOperator,
    operand: Expression,
}

#[derive(Debug, Clone)]
pub struct BinaryOperation {
    operator: BinaryOperator,
    operands: [Expression; 2],
}
