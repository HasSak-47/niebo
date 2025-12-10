use std::path::Path;

use inkwell::{
    OptimizationLevel,
    context::Context,
    targets::{CodeModel, FileType, InitializationConfig, RelocMode, Target, TargetMachine},
};

use crate::lowlevel::{
    compiler::Compiler,
    repr::{Expression, FunctionBuilder, Repr, Statement},
    types::*,
};

mod ast;
mod lowlevel;
mod parser;

fn main() -> anyhow::Result<()> {
    let context = Context::create();
    let compiler = Compiler::new(&context);
    let repr = Repr::new(vec![
        FunctionBuilder::new("scanf", Type::int())
            .varidic()
            .add_param("", Type::string())
            .build_declaration(),
        FunctionBuilder::new("printf", Type::int())
            .varidic()
            .add_param("", Type::string())
            .build_declaration(),
        FunctionBuilder::new("main", Type::int())
            .add_statement(Statement::var_define(
                "test_var",
                Type::int(),
                Expression::int(0x69),
            ))
            .add_statement(Expression::call_statement(
                Expression::identifier("printf"),
                vec![
                    Expression::string("hello world %d!\n"),
                    Expression::Identifier("test_var".to_string()),
                ],
            ))
            .add_statement(Expression::return_statement(Expression::int(0x00)))
            .build_definition(),
    ]);
    let module = compiler.new_module(repr, "test_module".into());
    module.compile();
    println!("code: {}", module.get_ll_code());

    Target::initialize_native(&InitializationConfig::default())
        .expect("Failed to initialize native target");

    let target_triple = TargetMachine::get_default_triple();
    module.module.set_triple(&target_triple);

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
        .write_to_file(&module.module, FileType::Object, obj_path)
        .expect("Failed to write object file");

    std::process::Command::new("clang")
        .args(["output.o"])
        .status()
        .unwrap();

    std::process::Command::new("./a.out").status().unwrap();

    return Ok(());
}
