pub mod operations;
use super::Expression;

use super::types::*;

use anyhow::{Result, anyhow};

pub trait ExpressionTrait {
    fn get_return_type(&self) -> Type;
    fn resolve_and_validate(&mut self) -> Result<()>;
    fn is_constant_expr(&self) -> bool {
        false
    }
}
