use std::fmt::Display;

use crate::ast::expressions::ExpressionKind;

use super::Expression;

#[derive(Debug, Clone, PartialEq)]
pub struct LoopExpression {
    pub label: Option<String>,
    pub body: Expression,
}

impl From<LoopExpression> for Expression {
    fn from(value: LoopExpression) -> Self {
        Expression {
            kind: Box::new(ExpressionKind::Loop(value)),
            ret_ty: None,
            constant: false,
        }
    }
}

impl LoopExpression {
    pub fn new(body: Expression) -> Self {
        Self { body, label: None }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WhileLoop {
    pub label: Option<String>,
    pub condition: Expression,
    pub then: Expression,
}

impl WhileLoop {
    pub fn new(condition: Expression, then: Expression) -> Self {
        Self {
            condition,
            then,
            label: None,
        }
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
