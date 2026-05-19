use std::fmt::Display;

use super::{Expression, Statement};
use anyhow::{Result, anyhow};

#[derive(Debug, Clone, PartialEq)]
pub struct Block {
    pub statements: Vec<Statement>,
}

impl Block {
    // TODO: validate statement
    pub fn add_statement(&mut self, stmt: Statement) {
        self.statements.push(stmt);
    }

    pub fn new() -> Self {
        return Self { statements: vec![] };
    }

    pub fn into_expression(self) -> Result<Expression> {
        match self.statements.last() {
            Some(Statement::Expression(_)) => Ok(Expression::block(self)),
            _ => Err(anyhow!("block does not end with an expression")),
        }
    }
}

impl Display for Block {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{ {:?} }}", self.statements)
    }
}
