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
        ir::{ExpressionHandler, FunctionBuilder, Operator, Statement, UnaryOperator},
    },
    types::*,
};

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
            .add_statement(ExpressionHandler::call_statement(
                ExpressionHandler::identifier("printf"),
                vec![
                    ExpressionHandler::string("bar %d"),
                    ExpressionHandler::identifier("x"),
                ],
            ))
            .add_statement(Statement::Expression(ExpressionHandler::return_expression(
                None,
            )))
            .build_definition(),
        // fn foo
        FunctionBuilder::new("foo", Type::void())
            .add_param("x", Type::int())
            .add_statement(ExpressionHandler::call_statement(
                ExpressionHandler::identifier("printf"),
                vec![
                    ExpressionHandler::string("foo %d"),
                    ExpressionHandler::identifier("x"),
                ],
            ))
            .add_statement(Statement::Expression(ExpressionHandler::return_expression(
                None,
            )))
            .build_definition(),
        // fn main
        FunctionBuilder::new("main", Type::int())
            .add_statement(Statement::var_define(
                "opt",
                Type::int(),
                ExpressionHandler::int(0),
            ))
            .add_statement(Statement::var_define(
                "x",
                Type::int(),
                ExpressionHandler::int(0),
            ))
            .add_statement(Statement::var_define(
                "r",
                Type::int(),
                ExpressionHandler::call(
                    ExpressionHandler::identifier("scanf"),
                    vec![
                        ExpressionHandler::string("select opt and val %d %d\n"),
                        ExpressionHandler::unary_operation(
                            UnaryOperator::Ref,
                            ExpressionHandler::identifier("opt"),
                        ),
                        ExpressionHandler::unary_operation(
                            UnaryOperator::Ref,
                            ExpressionHandler::identifier("x"),
                        ),
                    ],
                ),
            ))
            .add_statement(ExpressionHandler::call_statement(
                ExpressionHandler::identifier("printf"),
                vec![
                    ExpressionHandler::string("opt %d val %d func %p!\n"),
                    ExpressionHandler::identifier("opt"),
                    ExpressionHandler::identifier("x"),
                ],
            ))
            .add_statement(ExpressionHandler::return_statement(Some(
                ExpressionHandler::int(0x00),
            )))
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

    /*
    std::process::Command::new("clang")
        .args(["output.o"])
        .status()
        .unwrap();

    std::process::Command::new("./a.out").status().unwrap();
    */

    return Ok(());
}
