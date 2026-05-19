use std::fmt::Display;

use crate::{ast::expressions::Expression, general::path::PathIdent};

#[derive(Debug, Clone, PartialEq)]
pub struct MemberAccess {
    pub object: Expression,
    pub member: PathIdent,
}

impl Display for MemberAccess {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}).{}", self.object, self.member)
    }
}
