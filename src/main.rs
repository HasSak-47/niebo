use std::path::PathBuf;

use clap::{Parser, ValueEnum};
use pretty_env_logger::init_custom_env;

use crate::{
    ast::project::{Project, ProjectPreprocessor},
    ir::compile,
};

mod ast;
mod general;
mod ir;
mod parser;

#[derive(Parser, Debug)]
#[command(name = "niebo")]
#[command(about = "Compile a Niebo project directory or a single .nb script")]
struct Cli {
    /// Path to a Niebo project directory or a single script file
    #[arg(value_name = "PATH")]
    path: PathBuf,

    /// Output file path
    #[arg(long, value_name = "OUT")]
    out: PathBuf,

    /// Treat PATH as a project directory or a standalone script
    #[arg(long, value_enum, default_value_t = Mode::Project)]
    mode: Mode,
}

#[derive(Copy, Clone, Debug, ValueEnum)]
enum Mode {
    Script,
    Project,
}

fn main() -> anyhow::Result<()> {
    init_custom_env("NIEBOC_LOG");
    let cli = Cli::parse();
    let project = match cli.mode {
        Mode::Script => Project::load_script(&cli.path)?,
        Mode::Project => Project::load(&cli.path)?,
    };

    let mut ir = ProjectPreprocessor::default();
    let project = ir.process_project(project)?;

    compile(project, cli.out)?;

    return Ok(());
}
