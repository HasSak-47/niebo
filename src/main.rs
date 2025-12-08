mod ast;
mod lowlevel;
mod parser;

use std::path::Path;

use inkwell::{
    OptimizationLevel,
    context::Context,
    targets::{CodeModel, FileType, InitializationConfig, RelocMode, Target, TargetMachine},
};

use crate::ast::{Expression, Statement};

fn main() -> anyhow::Result<()> {
    let context = Context::create();
    let module = context.create_module("main");
    let builder = context.create_builder();

    let llvm = ast::ModuleCompiler::new(&context, &module, &builder);
    let ast = ast::AST {
        sts: vec![Statement::FunctionDefinition {
            ident: "test_function".to_string(),
            params: vec![],
            ret_ty: ast::PrimitiveType::Void,
            body: vec![Statement::VariableDefinition {
                mutable: false,
                ident: "x".to_string(),
                ty: ast::PrimitiveType::Int,
                expression: Box::new(Expression::Literal(ast::Literal::Int(0x69))),
            }],
        }],
    };

    llvm.build_code(&ast);
    println!("{}", module.to_string());

    Target::initialize_native(&InitializationConfig::default())
        .expect("Failed to initialize native target");

    let target_triple = TargetMachine::get_default_triple();
    module.set_triple(&target_triple);

    let target = Target::from_triple(&target_triple).expect("Could not create target from triple");

    let cpu = "generic";
    let features = "";
    let opt = OptimizationLevel::None;
    let reloc = RelocMode::Default;
    let model = CodeModel::Default;

    let target_machine = target
        .create_target_machine(&target_triple, cpu, features, opt, reloc, model)
        .expect("Could not create target machine");

    let obj_path = Path::new("output.o");
    target_machine
        .write_to_file(&module, FileType::Object, obj_path)
        .expect("Failed to write object file");

    return Ok(());
}
