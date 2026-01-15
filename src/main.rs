#![allow(warnings)]

use std::{fs::File, io::Read};

use crate::{
    ast::{
        Definition,
        expressions::{
            Expression, ExpressionKind, Statement, call::Call, literal::Literal,
            operations::BinaryOperator,
        },
        function::FunctionBuilder,
        project::Project,
    },
    ir::IR,
};

mod ast;
mod general;
mod ir;
mod parser;

fn main() -> anyhow::Result<()> {
    let mut file = File::open("test.nb")?;
    let mut buf = String::new();
    file.read_to_string(&mut buf)?;
    let mut project = Project::new("test_project", (0, 1, 0));
    project.root_module = parser::parse_module(buf)?;
    println!("{:#?}", project.root_module);

    return Ok(());
}
