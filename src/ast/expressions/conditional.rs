use super::Expression;

#[derive(Debug, Clone)]
pub struct Conditional {
    pub condition: Expression,
    pub then: Expression,
    pub else_: Option<Expression>,
}

impl Conditional {
    pub fn new(condition: Expression, then: Expression) -> Self {
        Self {
            condition,
            then,
            else_: None,
        }
    }

    pub fn set_else(&mut self, else_: Expression) {
        self.else_ = Some(else_);
    }
}

#[derive(Debug, Clone)]
pub struct ConditionalBuilder {
    ifs: Vec<Conditional>,
    else_: Option<Expression>,
}

impl ConditionalBuilder {
    pub fn new(condition: Expression, then: Expression) -> Self {
        Self {
            ifs: vec![Conditional::new(condition, then)],
            else_: None,
        }
    }

    pub fn add_if(mut self, condition: Expression, then: Expression) -> Self {
        self.ifs.push(Conditional::new(condition, then));
        self
    }

    pub fn set_else(mut self, else_: Expression) -> Self {
        self.else_ = Some(else_);
        self
    }

    pub fn build(mut self) -> Conditional {
        let mut last = self.ifs.pop().unwrap();
        if let Some(else_) = self.else_ {
            last.set_else(else_);
        }

        for mut if_ in self.ifs.into_iter().rev() {
            if_.set_else(Expression::if_(last));
            last = if_;
        }

        last
    }
}
