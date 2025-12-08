mod ast;
mod parser;

use std::{fs::File, io::Read, path::PathBuf};

use inkwell::{
    OptimizationLevel,
    builder::Builder,
    context::Context,
    execution_engine,
    module::Module,
    targets::{CodeModel, Target, TargetMachine, TargetTriple},
};

fn main() -> anyhow::Result<()> {
    let context = Context::create();
    let module = context.create_module("main");
    let builder = context.create_builder();

    let i32_t = context.i32_type();
    let void_t = context.void_type();
    let func_t = void_t.fn_type(&[i32_t.into(), i32_t.into()], false);

    let func_v = module.add_function("test", func_t, None);
    let entry = context.append_basic_block(func_v, "entry");
    builder.position_at_end(entry);
    let a = func_v.get_nth_param(0).unwrap().into_int_value();
    let b = func_v.get_nth_param(1).unwrap().into_int_value();
    let c = builder.build_int_add(a, b, "sum").unwrap();
    builder.build_return(Some(&c)).unwrap();
    println!("{}", module.to_string());

    return Ok(());
}
