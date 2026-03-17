use crate::ast::project::Project;

mod ast;
mod general;
mod ir;
mod parser;

fn main() -> anyhow::Result<()> {
    // let mut core_project = Project::load("./core")?;
    // core_project.generate_ir()?;

    let mut test_project = Project::load("./test")?;
    test_project.generate_ir()?;

    return Ok(());
}
