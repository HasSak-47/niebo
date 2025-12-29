use super::Expression;

#[derive(Debug, Clone)]
pub struct Call {
    pub called: Expression,
    pub parameters: Vec<Expression>,
}

impl Call {
    pub fn new(called: Expression, parameters: Vec<Expression>) -> Self {
        Self { called, parameters }
    }

    pub fn add_parameter(&mut self, parameter: Expression) {
        self.parameters.push(parameter);
    }
}
