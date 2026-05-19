use std::fmt::Display;

use super::Expression;

#[derive(Debug, Clone, PartialEq)]
pub struct LoopExpression {
    pub body: Expression,
}

impl LoopExpression {
    pub fn new(body: Expression) -> Self {
        Self { body }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WhileLoop {
    pub condition: Expression,
    pub then: Expression,
}

impl WhileLoop {
    pub fn new(condition: Expression, then: Expression) -> Self {
        Self { condition, then }
    }
}

impl Display for LoopExpression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "loop {{ {} }}", self.body)
    }
}

impl Display for WhileLoop {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "while ({}) {{ {} }}", self.condition, self.then)
    }
}
