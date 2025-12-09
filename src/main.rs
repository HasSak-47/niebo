use std::path::Path;

use inkwell::{
    OptimizationLevel,
    context::Context,
    targets::{CodeModel, FileType, InitializationConfig, RelocMode, Target, TargetMachine},
};

use crate::lowlevel::{
    compiler::Compiler,
    repr::{self, Expression, FunctionBuilder, Literal, Repr, Statement},
    types::*,
};

mod ast;
mod lowlevel;
mod parser;

fn main() -> anyhow::Result<()> {
    let context = Context::create();
    let compiler = Compiler::new(&context);
    let repr = Repr::new(vec![
        FunctionBuilder::new("puts", Type::Primitive(PrimitiveType::Int)).build_declaration(),
        FunctionBuilder::new("main", Type::Primitive(PrimitiveType::Int))
            .add_param("", Type::Primitive(PrimitiveType::String))
            .add_statement(Statement::VariableDefinition {
                mutable: false,
                ident: "test_var".to_string(),
                ty: Type::Primitive(PrimitiveType::Int),
                expression: Box::new(repr::Expression::Literal(Literal::Int(0x69))),
            })
            .add_statement(Statement::Expression(repr::Expression::Call {
                operand: Box::new(repr::Expression::Identifier("puts".to_string())),
                params: vec![repr::Expression::Identifier("hello_worl_ptr".to_string())],
            }))
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

    return Ok(());
}
