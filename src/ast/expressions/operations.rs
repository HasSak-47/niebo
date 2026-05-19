use std::fmt::Display;

use crate::general::path::Path;

use super::Expression;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum BinaryOperator {
    // bitwise operations
    BitShitLeft,
    BitShitRight,
    BitwiseOr,
    BitwiseAnd,
    BitwiseXor,

    // arithmetic operations
    Addition,
    Multiplication,
    Substraction,
    Division,
    Module,

    // boolean operations
    Greater,
    Lesser,
    GreaterOrEqual,
    LesserOrEqual,
    Equal,
    NotEqual,
}

impl Display for BinaryOperator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BinaryOperator::BitShitLeft => write!(f, ">>"),
            BinaryOperator::BitShitRight => write!(f, "<<"),
            BinaryOperator::BitwiseOr => write!(f, "|"),
            BinaryOperator::BitwiseAnd => write!(f, "&"),
            BinaryOperator::BitwiseXor => write!(f, "^"),
            BinaryOperator::Addition => write!(f, "+"),
            BinaryOperator::Multiplication => write!(f, "*"),
            BinaryOperator::Substraction => write!(f, "-"),
            BinaryOperator::Division => write!(f, "/"),
            BinaryOperator::Module => write!(f, "%"),
            BinaryOperator::Greater => write!(f, ">"),
            BinaryOperator::Lesser => write!(f, "<"),
            BinaryOperator::GreaterOrEqual => write!(f, ">="),
            BinaryOperator::LesserOrEqual => write!(f, "<="),
            BinaryOperator::Equal => write!(f, "=="),
            BinaryOperator::NotEqual => write!(f, "!="),
        }
    }
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

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum UnaryOperator {
    Deref,
    Ref,
    Negation,
}

impl Display for UnaryOperator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Deref => write!(f, "*"),
            Self::Ref => write!(f, "&"),
            Self::Negation => write!(f, "!"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnaryOperation {
    pub operator: UnaryOperator,
    pub operand: Expression,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BinaryOperation {
    pub operator: BinaryOperator,
    pub operands: [Expression; 2],
}

impl Display for UnaryOperation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({} {})", self.operand, self.operator)
    }
}

impl Display for BinaryOperation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "({} {} {})",
            self.operands[0], self.operator, self.operands[1]
        )
    }
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
