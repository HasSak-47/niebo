use std::fmt::Display;

use crate::{ast::expressions::call::Call, general::naming::QualifiedName};

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
    pub fn into_operation_call(&self) -> QualifiedName {
        let (operation, method) = match self {
            BinaryOperator::Addition => ("Add", "add"),
            BinaryOperator::Substraction => ("Sub", "sub"),
            BinaryOperator::Multiplication => ("Mul", "mul"),
            BinaryOperator::Division => ("Div", "div"),
            BinaryOperator::Equal => ("Eq", "eq"),
            BinaryOperator::NotEqual => ("NEq", "neq"),
            BinaryOperator::LesserOrEqual => ("LEq", "leq"),
            BinaryOperator::GreaterOrEqual => ("GEq", "geq"),
            BinaryOperator::Lesser => ("Les", "les"),
            BinaryOperator::Greater => ("Gre", "gre"),
            _ => todo!(),
        };

        let mut path = QualifiedName::from(operation);
        path.add_segment(method);
        path
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum UnaryOperator {
    Deref,
    Ref,
    Negation,
    Increase,
    Decrease,
    EarlyRet,
}

impl UnaryOperator {
    pub fn into_operation_call(&self) -> QualifiedName {
        let (operation, method) = match self {
            UnaryOperator::Negation => ("Neg", "neg"),
            _ => todo!(),
        };

        let mut path = QualifiedName::from(operation);
        path.add_segment(method);
        path
    }
}

impl Display for UnaryOperator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Deref => write!(f, "*"),
            Self::Ref => write!(f, "&"),
            Self::Negation => write!(f, "-"),

            Self::Increase => write!(f, "--"),
            Self::Decrease => write!(f, "++"),
            Self::EarlyRet => write!(f, "?"),
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

    pub fn simple_desugar(&self) -> Expression {
        return Expression::call(
            Expression::identifier(self.operator.into_operation_call()),
            vec![self.operand.clone()],
        );
    }
}

impl BinaryOperation {
    pub fn new(operator: BinaryOperator, left: Expression, right: Expression) -> Self {
        Self {
            operator,
            operands: [left, right],
        }
    }

    pub fn simple_desugar(&self) -> Expression {
        return Expression::call(
            Expression::identifier(self.operator.into_operation_call()),
            vec![self.operands[0].clone(), self.operands[1].clone()],
        );
    }
}
