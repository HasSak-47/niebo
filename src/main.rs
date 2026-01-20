#![allow(warnings)]

use std::{fs::File, io::Read};

use crate::ast::{
    Definition,
    expressions::{
        Expression, ExpressionKind, Statement, call::Call, literal::Literal,
        operations::BinaryOperator,
    },
    function::FunctionBuilder,
    project::Project,
};

mod ast;
mod general;
mod ir;
mod parser;

fn main() -> anyhow::Result<()> {
    let mut project = Project::load("./core")?;
    project.generate_ir()?;

    return Ok(());
}
