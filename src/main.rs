use std::{env::current_dir, path::PathBuf};

use crate::ast::{project::Project, validator::Validator};
use clap::{Parser, ValueEnum};

use pretty_env_logger::init_custom_env;

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
    #[arg(long, value_name = "OUT", default_value_os_t = current_dir().unwrap())]
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

    let mut v = Validator::new();
    let project = v.process_project(project)?;

    // compile(project, cli.out)?;

    return Ok(());
}
