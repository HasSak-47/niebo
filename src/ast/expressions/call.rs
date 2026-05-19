use std::fmt::Display;

use super::Expression;

#[derive(Debug, Clone, PartialEq)]
pub struct Call {
    pub called: Expression,
    pub parameters: Vec<Expression>,
}

impl Display for Call {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({})({:?})", self.called, self.parameters)
    }
}

impl Call {
    pub fn new(called: Expression, parameters: Vec<Expression>) -> Self {
        Self { called, parameters }
    }

    pub fn add_parameter(&mut self, parameter: Expression) {
        self.parameters.push(parameter);
    }
}
