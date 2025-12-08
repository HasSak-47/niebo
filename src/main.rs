mod ast;
mod parser;

use std::{
    fs::File,
    io::Read,
    path::{Path, PathBuf},
};

use inkwell::{
    OptimizationLevel,
    builder::Builder,
    context::Context,
    execution_engine,
    module::Module,
    targets::{
        CodeModel, FileType, InitializationConfig, RelocMode, Target, TargetMachine, TargetTriple,
    },
};

use crate::ast::{Expression, Statement};

fn main() -> anyhow::Result<()> {
    let context = Context::create();
    let module = context.create_module("main");
    let builder = context.create_builder();

    let llvm = ast::Compiler::new(&context, &module, &builder);
    let ast = ast::AST {
        sts: vec![Statement::FunctionDeclaration {
            ident: "main".to_string(),
            params: vec![],
            ret_ty: ast::PrimitiveType::Void,
            body: vec![Statement::VariableDeclaration {
                mutable: false,
                ident: "x".to_string(),
                ty: ast::PrimitiveType::Int,
                expression: Box::new(Expression::Literal(ast::Literal::Int(10))),
            }],
        }],
    };

    llvm.build_code(&ast);
    println!("{}", llvm.module.to_string());

    Target::initialize_native(&InitializationConfig::default())
        .expect("Failed to initialize native target");

    let target_triple = TargetMachine::get_default_triple();
    module.set_triple(&target_triple);

    let target = Target::from_triple(&target_triple).expect("Could not create target from triple");

    let cpu = "generic";
    let features = "";
    let opt = OptimizationLevel::Default;
    let reloc = RelocMode::Default;
    let model = CodeModel::Default;

    let target_machine = target
        .create_target_machine(&target_triple, cpu, features, opt, reloc, model)
        .expect("Could not create target machine");

    // 3. Emit object file
    let obj_path = Path::new("output.o");
    target_machine
        .write_to_file(&module, FileType::Object, obj_path)
        .expect("Failed to write object file");

    // 4. Link to native executable (using system linker via `cc`/`clang`)
    // Very simple example using `cc`:
    let status = std::process::Command::new("cc")
        .arg("output.o")
        .arg("-o")
        .arg("output_bin")
        .status()?;

    if !status.success() {
        panic!("Linker failed");
    }

    return Ok(());
}
