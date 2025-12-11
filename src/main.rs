use std::{fs::File, io::Write, path::Path};

use inkwell::{
    OptimizationLevel,
    context::Context,
    targets::{CodeModel, FileType, InitializationConfig, RelocMode, Target, TargetMachine},
};

use crate::lowlevel::{
    compiler::Compiler,
    repr::{
        Repr,
        ir::{Expression, FunctionBuilder, Statement, UnaryOperator},
    },
    types::*,
};

mod ast;
mod lowlevel;
mod parser;

fn main() -> anyhow::Result<()> {
    let context = Context::create();
    let compiler = Compiler::new(&context);
    let repr = Repr::new(vec![
        // fn printf
        FunctionBuilder::new("printf", Type::int())
            .add_param("", Type::string())
            .varidic()
            .build_declaration(),
        // fn scanf
        FunctionBuilder::new("scanf", Type::int())
            .add_param("", Type::string())
            .varidic()
            .build_declaration(),
        // fn bar
        FunctionBuilder::new("bar", Type::void())
            .add_param("x", Type::int())
            .add_statement(Expression::call_statement(
                Expression::identifier("printf"),
                vec![Expression::string("bar %d"), Expression::identifier("x")],
            ))
            .add_statement(Statement::Expression(Expression::Return(None)))
            .build_definition(),
        // fn foo
        FunctionBuilder::new("foo", Type::void())
            .add_param("x", Type::int())
            .add_statement(Expression::call_statement(
                Expression::identifier("printf"),
                vec![Expression::string("foo %d"), Expression::identifier("x")],
            ))
            .add_statement(Statement::Expression(Expression::Return(None)))
            .build_definition(),
        // fn main
        FunctionBuilder::new("main", Type::int())
            .add_statement(Statement::var_define(
                "opt",
                Type::int(),
                Expression::int(0),
            ))
            .add_statement(Statement::var_define("x", Type::int(), Expression::int(0)))
            .add_statement(Expression::call_statement(
                Expression::identifier("scanf"),
                vec![
                    Expression::string("select opt and val %d %d\n"),
                    Expression::Operator(lowlevel::repr::ir::Operator::Unary {
                        operator: UnaryOperator::Ref,
                        operand: Box::new(Expression::identifier("opt")),
                    }),
                    Expression::Operator(lowlevel::repr::ir::Operator::Unary {
                        operator: UnaryOperator::Ref,
                        operand: Box::new(Expression::identifier("x")),
                    }),
                ],
            ))
            .add_statement(Expression::call_statement(
                Expression::identifier("printf"),
                vec![
                    Expression::string("opt %d val %d func %p!\n"),
                    Expression::identifier("opt"),
                    Expression::identifier("x"),
                ],
            ))
            .add_statement(Expression::return_statement(Expression::int(0x00)))
            .build_definition(),
    ]);
    let module = compiler.new_module(repr, "test_module".into());
    module.compile();
    let llvmir = module.get_ll_code();
    println!("{llvmir}");
    let mut f = File::create("output.ll")?;
    f.write_all(llvmir.as_bytes())?;

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
