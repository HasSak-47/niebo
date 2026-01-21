use crate::{ast::expressions::Expression, general::path::PathIdent};

#[derive(Debug, Clone)]
pub struct MemberAccess {
    pub object: Expression,
    pub member: PathIdent,
}
