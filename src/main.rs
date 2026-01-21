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
    // let mut core_project = Project::load("./core")?;
    // core_project.generate_ir()?;

    let mut test_project = Project::load("./test")?;
    test_project.generate_ir();

    return Ok(());
}
