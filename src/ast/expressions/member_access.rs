use std::fmt::Display;

use crate::{ast::expressions::Expression, general::naming::QualifiedNameSegment};

#[derive(Debug, Clone, PartialEq)]
pub struct MemberAccess {
    pub object: Expression,
    pub member: QualifiedNameSegment,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MethodCall {
    pub object: Expression,
    pub method: QualifiedNameSegment,
    pub params: Vec<Expression>,
}

impl Display for MemberAccess {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}).{}", self.object, self.member)
    }
}

impl Display for MethodCall {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}).{}({:?})", self.object, self.method, self.params)
    }
}
