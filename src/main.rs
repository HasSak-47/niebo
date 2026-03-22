use crate::ast::project::{IRGenerator, Project};

mod ast;
mod general;
mod ir;
mod parser;

fn main() -> anyhow::Result<()> {
    // let mut core_project = Project::load("./core")?;
    // core_project.generate_ir()?;

    let test_project = Project::load("./test")?;
    let mut ir = IRGenerator::default();
    ir.generate_ir(test_project)?;

    return Ok(());
}
