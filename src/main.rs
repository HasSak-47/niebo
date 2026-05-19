use crate::ast::project::{Project, ProjectPreprocessor};

mod ast;
mod general;
mod ir;
mod parser;

fn main() -> anyhow::Result<()> {
    // let mut core_project = Project::load("./core")?;
    // core_project.generate_ir()?;

    let test_project = Project::load("./test")?;
    let mut ir = ProjectPreprocessor::default();
    ir.process_project(test_project)?;

    return Ok(());
}
