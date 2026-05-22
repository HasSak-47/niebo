use std::collections::HashMap;

use clang::Clang;
use inkwell::{
    builder::Builder,
    context::Context,
    module::Module,
    types::{AnyType, BasicTypeEnum, IntType, VoidType},
    values::{AnyValue, AnyValueEnum, PointerValue},
};

use crate::{
    ast::{
        expressions::{
            Expression, ExpressionKind, Statement,
            block::Block,
            literal::{Literal, LiteralInfo},
            operations::{BinaryOperation, BinaryOperator},
        },
        function::Function,
        project::{Project, ProjectPreprocessor},
    },
    general::{
        path::Path,
        types::{PrimitiveType, Type},
    },
    ir::cimports::CCache,
};

mod ast;
mod general;
mod ir;
mod parser;

struct CodeGen<'ctx> {
    module: Module<'ctx>,
    builder: Builder<'ctx>,
    locals: HashMap<Path, PointerValue<'ctx>>,
}

trait TypeIr {
    fn to_ir_type<'ctx>(
        &self,
        ctx: &'ctx Context,
        codegen: &mut CodeGen<'ctx>,
    ) -> anyhow::Result<BasicTypeEnum<'ctx>>;
}

trait StatementIr {
    fn to_ir<'ctx>(
        &self,
        name: String,
        ctx: &'ctx Context,
        codegen: &mut CodeGen<'ctx>,
    ) -> anyhow::Result<()>;
}

trait ExpressionIr {
    fn to_ir_value<'ctx>(
        &self,
        ctx: &'ctx Context,
        codegen: &mut CodeGen<'ctx>,
    ) -> anyhow::Result<Option<AnyValueEnum<'ctx>>>;

    fn to_ir_place<'ctx>(
        &self,
        ctx: &'ctx Context,
        codegen: &mut CodeGen<'ctx>,
    ) -> anyhow::Result<PointerValue<'ctx>>;
}

impl TypeIr for Type {
    fn to_ir_type<'ctx>(
        &self,
        ctx: &'ctx Context,
        codegen: &mut CodeGen<'ctx>,
    ) -> anyhow::Result<BasicTypeEnum<'ctx>> {
        let _ = codegen;
        match self {
            Type::Primitive(p) => match p {
                PrimitiveType::Int(_) => Ok(ctx.i32_type().into()),
                _ => todo!(),
            },
            _ => todo!(),
        }
    }
}

impl StatementIr for ast::Variable {
    fn to_ir<'ctx>(
        &self,
        name: String,
        ctx: &'ctx Context,
        codegen: &mut CodeGen<'ctx>,
    ) -> anyhow::Result<()> {
        let ty = self.ty.clone().unwrap().to_ir_type(ctx, codegen)?;
        let p = codegen.builder.build_alloca(ty, &name)?;

        codegen.locals.insert(name.into(), p);

        return Ok(());
    }
}

impl ExpressionIr for Path {
    fn to_ir_value<'ctx>(
        &self,
        ctx: &'ctx Context,
        codegen: &mut CodeGen<'ctx>,
    ) -> anyhow::Result<Option<AnyValueEnum<'ctx>>> {
        let ptr = self.to_ir_place(ctx, codegen)?;
        let value = codegen.builder.build_load(ctx.i32_type(), ptr, "")?;
        Ok(Some(value.as_any_value_enum()))
    }

    fn to_ir_place<'ctx>(
        &self,
        ctx: &'ctx Context,
        codegen: &mut CodeGen<'ctx>,
    ) -> anyhow::Result<PointerValue<'ctx>> {
        let _ = ctx;
        Ok(codegen.locals[self])
    }
}

impl ExpressionIr for Literal {
    fn to_ir_value<'ctx>(
        &self,
        ctx: &'ctx Context,
        codegen: &mut CodeGen<'ctx>,
    ) -> anyhow::Result<Option<AnyValueEnum<'ctx>>> {
        let _ = codegen;
        match &self.info {
            LiteralInfo::Integer { .. } => {
                return Ok(Some(
                    ctx.i32_type()
                        .const_int_from_string(&self.data, inkwell::types::StringRadix::Decimal)
                        .unwrap()
                        .into(),
                ));
            }
            _ => todo!(),
        }
    }

    fn to_ir_place<'ctx>(
        &self,
        ctx: &'ctx Context,
        codegen: &mut CodeGen<'ctx>,
    ) -> anyhow::Result<PointerValue<'ctx>> {
        let _ = (ctx, codegen);
        todo!()
    }
}

impl ExpressionIr for BinaryOperation {
    fn to_ir_value<'ctx>(
        &self,
        ctx: &'ctx Context,
        codegen: &mut CodeGen<'ctx>,
    ) -> anyhow::Result<Option<AnyValueEnum<'ctx>>> {
        let a = self.operands[0]
            .to_ir_value(ctx, codegen)?
            .ok_or_else(|| anyhow::anyhow!("left operand did not produce a value"))?
            .into_int_value();
        let b = self.operands[1]
            .to_ir_value(ctx, codegen)?
            .ok_or_else(|| anyhow::anyhow!("right operand did not produce a value"))?
            .into_int_value();
        match &self.operator {
            BinaryOperator::Lesser => {
                return Ok(Some(
                    codegen
                        .builder
                        .build_int_compare(inkwell::IntPredicate::SLT, a, b, "")?
                        .as_any_value_enum(),
                ));
            }

            BinaryOperator::Equal => {
                return Ok(Some(
                    codegen
                        .builder
                        .build_int_compare(inkwell::IntPredicate::EQ, a, b, "")?
                        .as_any_value_enum(),
                ));
            }
            BinaryOperator::Addition => {
                return Ok(Some(codegen.builder.build_int_add(a, b, "")?.into()));
            }
            un => unimplemented!("{un:?}"),
        }
    }

    fn to_ir_place<'ctx>(
        &self,
        ctx: &'ctx Context,
        codegen: &mut CodeGen<'ctx>,
    ) -> anyhow::Result<PointerValue<'ctx>> {
        let _ = (ctx, codegen);
        todo!()
    }
}

impl ExpressionIr for Expression {
    fn to_ir_value<'ctx>(
        &self,
        ctx: &'ctx Context,
        codegen: &mut CodeGen<'ctx>,
    ) -> anyhow::Result<Option<AnyValueEnum<'ctx>>> {
        match &*self.kind {
            ExpressionKind::While(while_) => {
                let current_block = codegen
                    .builder
                    .get_insert_block()
                    .ok_or_else(|| anyhow::anyhow!("builder has no insertion block"))?;
                let function = current_block
                    .get_parent()
                    .ok_or_else(|| anyhow::anyhow!("insertion block has no parent function"))?;

                let cond_bb = ctx.append_basic_block(function, "while.cond");
                let body_bb = ctx.append_basic_block(function, "while.body");
                let end_bb = ctx.append_basic_block(function, "while.end");

                codegen.builder.build_unconditional_branch(cond_bb)?;
                codegen.builder.position_at_end(cond_bb);

                // TODO: lower `while_.condition` into an i1/int predicate.
                let con = while_.condition.to_ir_value(ctx, codegen)?.unwrap();
                codegen
                    .builder
                    .build_conditional_branch(con.into_int_value(), body_bb, end_bb)?;
                // continue loop
                codegen.builder.build_unconditional_branch(cond_bb).unwrap();

                // after loop
                codegen.builder.position_at_end(end_bb);

                return Ok(None);
            }
            ExpressionKind::BinaryOperation(oper) => {
                return oper.to_ir_value(ctx, codegen);
            }
            ExpressionKind::Identifier(id) => {
                return id.to_ir_value(ctx, codegen);
            }
            ExpressionKind::Literal(lit) => return lit.to_ir_value(ctx, codegen),
            un => todo!("{un:?}"),
        }
    }

    fn to_ir_place<'ctx>(
        &self,
        ctx: &'ctx Context,
        codegen: &mut CodeGen<'ctx>,
    ) -> anyhow::Result<PointerValue<'ctx>> {
        match &*self.kind {
            ExpressionKind::Identifier(id) => id.to_ir_place(ctx, codegen),
            un => todo!("{un:?}"),
        }
    }
}

impl StatementIr for Statement {
    fn to_ir<'ctx>(
        &self,
        name: String,
        ctx: &'ctx Context,
        codegen: &mut CodeGen<'ctx>,
    ) -> anyhow::Result<()> {
        match self {
            Statement::Definition(def) => match &def.kind {
                ast::DefinitionKind::Variable(v) => {
                    v.to_ir(def.name.clone(), ctx, codegen)?;
                }
                _ => todo!(),
            },
            Statement::Expression(exp) => {
                let _ = name;
                let _ = exp.to_ir_value(ctx, codegen)?;
            }
            un => todo!("{un:?}"),
        }

        return Ok(());
    }
}

impl StatementIr for Block {
    fn to_ir<'ctx>(
        &self,
        name: String,
        ctx: &'ctx Context,
        codegen: &mut CodeGen<'ctx>,
    ) -> anyhow::Result<()> {
        let _ = name;
        let mut iter = self.statements.iter();
        while let Some(stmt) = iter.next() {
            stmt.to_ir(String::new(), ctx, codegen)?;
        }
        return Ok(());
    }
}

impl StatementIr for Function {
    fn to_ir<'ctx>(
        &self,
        name: String,
        ctx: &'ctx Context,
        codegen: &mut CodeGen<'ctx>,
    ) -> anyhow::Result<()> {
        let fn_type = ctx.void_type().fn_type(&[], false);
        let func = codegen.module.add_function(&name, fn_type, None);
        let basic_block = ctx.append_basic_block(func, "");
        codegen.builder.position_at_end(basic_block);

        for stmt in &self.body.statements {
            stmt.to_ir(String::new(), ctx, codegen)?;
        }

        codegen.builder.build_return(None)?;
        return Ok(());
    }
}

fn main() -> anyhow::Result<()> {
    // let mut core_project = Project::load("./core")?;
    // core_project.generate_ir()?;

    let project = Project::load("./test")?;
    let mut ir = ProjectPreprocessor::default();
    let project = ir.process_project(project)?;

    let clang = Clang::new().unwrap();
    let mut ccache = CCache::new(&clang)?;

    for import in &project.root_module.imports {
        if import.c_import {
            ccache.resolve_c_definitions(&import.path.get(0).ident)?;
            let mut path = Path::new();
            path.add_segment(&import.path.get(1).ident);
            // self.global.symbols.insert(path, );
        }
    }
    let context = Context::create();
    let mut codegen = CodeGen {
        module: context.create_module(&project.name),
        builder: context.create_builder(),
        locals: HashMap::new(),
    };
    for def in &project.root_module.definitions {
        match &def.kind {
            ast::DefinitionKind::Function(f) => {
                f.to_ir(def.name.clone(), &context, &mut codegen)?
            }
            _ => unimplemented!(),
        }
    }

    codegen.module.print_to_file("out.ll").unwrap();

    return Ok(());
}
