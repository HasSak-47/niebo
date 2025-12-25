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

impl ExpressionTrait for UnaryOperation {
    fn resolve_and_validate(&mut self) -> Result<()> {
        self.operand.resolve_and_validate()?;

        match self.operand.get_return_type() {
            Type::Primitive(_) | Type::Pointer(_) => {}
            _ => {
                return Err(anyhow!(
                    "unary operation only available for primitives and pointers"
                ));
            }
        }

        return Ok(());
    }

    fn get_return_type(&self) -> Type {
        return self.operand.get_return_type();
    }
}

impl ExpressionTrait for BinaryOperation {
    fn resolve_and_validate(&mut self) -> Result<()> {
        self.operands[0].resolve_and_validate()?;
        self.operands[1].resolve_and_validate()?;

        if self.operands[0].get_return_type() != self.operands[1].get_return_type() {
            return Err(anyhow!("binary operation has two different types"));
        }
        match self.operands[0].get_return_type() {
            Type::Primitive(_) | Type::Pointer(_) => {}
            _ => {
                return Err(anyhow!(
                    "binary operation only available for primitives and pointers"
                ));
            }
        }

        return Ok(());
    }

    fn get_return_type(&self) -> Type {
        return self.operands[0].get_return_type();
    }
}
