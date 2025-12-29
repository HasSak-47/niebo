use super::Expression;

#[derive(Debug, Clone)]
pub struct LoopExpression {
    pub body: Expression,
}

impl LoopExpression {
    pub fn new(body: Expression) -> Self {
        Self { body }
    }
}

#[derive(Debug, Clone)]
pub struct WhileLoop {
    pub condition: Expression,
    pub then: Expression,
}

impl WhileLoop {
    pub fn new(condition: Expression, then: Expression) -> Self {
        Self { condition, then }
    }
}
