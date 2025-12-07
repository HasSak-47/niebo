use std::{fs::File, io::Read, path::PathBuf};

use inkwell::{
    OptimizationLevel,
    builder::Builder,
    context::Context,
    execution_engine,
    module::Module,
    targets::{CodeModel, Target, TargetMachine, TargetTriple},
};

mod parser;

fn main() -> anyhow::Result<()> {
    return Ok(());
}
