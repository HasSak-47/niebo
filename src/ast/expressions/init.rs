use std::fmt::Display;

use crate::{ast::expressions::Expression, general::naming::QualifiedName};

#[derive(Debug, Default, Clone, PartialEq)]
pub struct StructInit {
    pub ident: QualifiedName,
    pub params: Vec<(String, Expression)>,
}

impl Display for StructInit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {{ {:?} }}", self.ident, self.params)
    }
}
