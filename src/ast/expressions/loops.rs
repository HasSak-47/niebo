use std::fmt::Display;

use crate::ast::expressions::{ExpressionKind, conditional::ConditionalBuilder};

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

    pub fn simple_desugar(&self) -> Expression {
        let id = self.label.clone();
        let while_header_ = ConditionalBuilder::new(
            self.condition.clone(),
            Expression::block(crate::ast::expressions::block::Block {
                statements: vec![crate::ast::statements::Statement::Break(id.clone())],
            }),
        );

        let mut body = crate::ast::expressions::Block::new();
        body.add_statement(crate::ast::statements::Statement::Expression(
            Expression::if_(while_header_.build()),
        ));
        body.add_statement(crate::ast::statements::Statement::Expression(
            self.then.clone(),
        ));

        return Expression::loop_(Expression::block(body), id);
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
