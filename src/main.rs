#![allow(warnings)]

use std::{fs::File, io::Read};

use crate::{
    ast::{
        Definition, Project,
        expressions::{
            Expression, ExpressionKind, Statement, call::Call, literal::Literal,
            operations::BinaryOperator,
        },
        function::FunctionBuilder,
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
    let module = parser::parse_module(buf)?;
    println!("{module:#?}");

    return Ok(());
}
