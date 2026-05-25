pub mod cimports;
pub mod validator;

use std::{collections::HashMap, path::PathBuf};

use clang::Clang;
use inkwell::{
    AddressSpace,
    builder::Builder,
    context::Context,
    module::Module,
    targets::{Target, TargetMachine},
    types::{BasicMetadataTypeEnum, BasicType, BasicTypeEnum, FunctionType},
    values::{BasicValueEnum, FunctionValue, PointerValue},
};

use crate::{
    ast::{
        self,
        expressions::{
            Expression, ExpressionKind, Statement,
            block::Block,
            literal::{Literal, LiteralInfo},
            operations::{BinaryOperation, BinaryOperator, UnaryOperation, UnaryOperator},
        },
        function::Function,
        project::Project,
    },
    general::{
        naming::QualifiedName,
        types::{PrimitiveType, Type},
    },
    ir::cimports::CCache,
};

pub struct CodeGen<'ctx> {
    module: Module<'ctx>,
    builder: Builder<'ctx>,
    locals: HashMap<QualifiedName, PointerValue<'ctx>>,
    functions: HashMap<QualifiedName, FunctionValue<'ctx>>,
}

trait TypeIr {
    fn make_fn_type<'ctx>(
        &self,
        params: &[BasicMetadataTypeEnum<'ctx>],
        varidic: bool,
        ctx: &'ctx Context,
        codegen: &mut CodeGen<'ctx>,
    ) -> anyhow::Result<FunctionType<'ctx>>;
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
    ) -> anyhow::Result<Option<BasicValueEnum<'ctx>>>;

    fn to_ir_place<'ctx>(
        &self,
        ctx: &'ctx Context,
        codegen: &mut CodeGen<'ctx>,
    ) -> anyhow::Result<PointerValue<'ctx>>;
}

impl TypeIr for Type {
    fn make_fn_type<'ctx>(
        &self,
        params: &[BasicMetadataTypeEnum<'ctx>],
        varidic: bool,
        ctx: &'ctx Context,
        codegen: &mut CodeGen<'ctx>,
    ) -> anyhow::Result<FunctionType<'ctx>> {
        let _ = codegen;
        match self {
            Type::Primitive(p) => match p {
                PrimitiveType::Void => Ok(ctx.void_type().fn_type(params, varidic)),
                p => Ok(Type::Primitive(p.clone())
                    .to_ir_type(ctx, codegen)?
                    .fn_type(params, varidic)),
            },
            un => unimplemented!("{un:?}"),
        }
    }
    fn to_ir_type<'ctx>(
        &self,
        ctx: &'ctx Context,
        codegen: &mut CodeGen<'ctx>,
    ) -> anyhow::Result<BasicTypeEnum<'ctx>> {
        let _ = codegen;
        match self {
            Type::Primitive(p) => match p {
                PrimitiveType::Int(_) => Ok(ctx.i32_type().into()),
                PrimitiveType::String => Ok(ctx.ptr_type(AddressSpace::default()).into()),
                PrimitiveType::Void => unreachable!(),
                un => unimplemented!("{un:?}"),
            },
            Type::Pointer(ptr) => ptr.to_ir_type(ctx, codegen),
            un => unimplemented!("{un:?}"),
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
        let val = self.value.to_ir_value(ctx, codegen)?.unwrap();
        codegen.builder.build_store(p, val)?;

        return Ok(());
    }
}

impl ExpressionIr for QualifiedName {
    fn to_ir_value<'ctx>(
        &self,
        ctx: &'ctx Context,
        codegen: &mut CodeGen<'ctx>,
    ) -> anyhow::Result<Option<BasicValueEnum<'ctx>>> {
        let ptr = self.to_ir_place(ctx, codegen)?;
        let value = codegen.builder.build_load(ctx.i32_type(), ptr, "")?;
        Ok(Some(value))
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
    ) -> anyhow::Result<Option<BasicValueEnum<'ctx>>> {
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
            LiteralInfo::String => {
                let p = codegen
                    .builder
                    .build_global_string_ptr(&self.data, "")?
                    .as_pointer_value();

                return Ok(Some(p.into()));
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

impl ExpressionIr for UnaryOperation {
    fn to_ir_value<'ctx>(
        &self,
        ctx: &'ctx Context,
        codegen: &mut CodeGen<'ctx>,
    ) -> anyhow::Result<Option<BasicValueEnum<'ctx>>> {
        match &self.operator {
            UnaryOperator::Ref => {
                return Ok(Some(self.operand.to_ir_place(ctx, codegen)?.into()));
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

impl ExpressionIr for BinaryOperation {
    fn to_ir_value<'ctx>(
        &self,
        ctx: &'ctx Context,
        codegen: &mut CodeGen<'ctx>,
    ) -> anyhow::Result<Option<BasicValueEnum<'ctx>>> {
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
                        .into(),
                ));
            }

            BinaryOperator::Equal => {
                return Ok(Some(
                    codegen
                        .builder
                        .build_int_compare(inkwell::IntPredicate::EQ, a, b, "")?
                        .into(),
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
    ) -> anyhow::Result<Option<BasicValueEnum<'ctx>>> {
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

                // condition
                let con = while_.condition.to_ir_value(ctx, codegen)?.unwrap();
                codegen
                    .builder
                    .build_conditional_branch(con.into_int_value(), body_bb, end_bb)?;
                codegen.builder.position_at_end(body_bb);
                // continue loop
                while_.then.to_ir_value(ctx, codegen)?;
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
            ExpressionKind::Block(b) => return b.to_ir_value(ctx, codegen),
            ExpressionKind::Call(call) => {
                let function = match &*call.called.kind {
                    ExpressionKind::Identifier(path) => codegen.functions[path],
                    un => todo!("{un:?}"),
                };

                let mut args = Vec::with_capacity(call.parameters.len());
                for arg in &call.parameters {
                    let value = arg
                        .to_ir_value(ctx, codegen)?
                        .ok_or_else(|| anyhow::anyhow!("call argument did not produce a value"))?;
                    args.push(value.into());
                }

                let call = codegen.builder.build_call(function, &args, "")?;
                return Ok(call.try_as_basic_value().basic());
            }
            ExpressionKind::Assignment(a, b) => {
                let a = a.to_ir_place(ctx, codegen)?;
                let b = b.to_ir_value(ctx, codegen)?.unwrap();

                codegen.builder.build_store(a, b)?;
                return Ok(None);
            }
            ExpressionKind::UnaryOperation(unary) => {
                return Ok(Some(unary.to_ir_value(ctx, codegen)?.unwrap()));
            }
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
            Statement::Return(ex) => {
                match ex {
                    Some(s) => {
                        if let Some(val) = s.to_ir_value(ctx, codegen)? {
                            codegen.builder.build_return(Some(&val))?;
                        } else {
                            codegen.builder.build_return(None)?;
                        }
                    }
                    _ => {
                        codegen.builder.build_return(None)?;
                    }
                }

                return Ok(());
            }

            un => todo!("{un:?}"),
        }

        return Ok(());
    }
}

impl ExpressionIr for Block {
    fn to_ir_value<'ctx>(
        &self,
        ctx: &'ctx Context,
        codegen: &mut CodeGen<'ctx>,
    ) -> anyhow::Result<Option<BasicValueEnum<'ctx>>> {
        let mut iter = self.statements.iter();
        while let Some(stmt) = iter.next() {
            stmt.to_ir(String::new(), ctx, codegen)?;
        }
        return Ok(None);
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

impl StatementIr for Function {
    fn to_ir<'ctx>(
        &self,
        name: String,
        ctx: &'ctx Context,
        codegen: &mut CodeGen<'ctx>,
    ) -> anyhow::Result<()> {
        let fn_type = ctx.void_type().fn_type(&[], false);
        let func = codegen.module.add_function(&name, fn_type, None);
        codegen.functions.insert(name.into(), func);
        let basic_block = ctx.append_basic_block(func, "");
        codegen.builder.position_at_end(basic_block);

        for stmt in &self.body.statements {
            stmt.to_ir(String::new(), ctx, codegen)?;
        }

        codegen.builder.build_return(None)?;
        return Ok(());
    }
}

pub fn compile(project: Project, out: PathBuf) -> anyhow::Result<()> {
    let clang = Clang::new().unwrap();
    let mut ccache = CCache::new(&clang)?;

    let context = Context::create();
    let mut codegen = CodeGen {
        module: context.create_module(&project.name),
        builder: context.create_builder(),
        locals: HashMap::new(),
        functions: HashMap::new(),
    };

    for import in &project.root_module.imports {
        if import.c_import {
            ccache.resolve_c_definitions(&import.path.get(0).ident)?;
            let mut name = QualifiedName::new();
            let mut header_path = QualifiedName::new();
            header_path.add_segment(&import.path.get(0).ident);
            header_path.add_segment(&import.path.get(1).ident);
            name.add_segment(&import.path.get(1).ident);

            let func = ccache.get_definition(&header_path)?;
            match &func.kind {
                ast::DefinitionKind::FunctionC(f) => {
                    let mut args = Vec::with_capacity(f.parameters.len());
                    for arg in &f.parameters {
                        let ty = arg.1.to_ir_type(&context, &mut codegen)?;
                        args.push(ty.into());
                    }

                    let ty = f.return_ty.clone().unwrap().make_fn_type(
                        args.as_slice(),
                        f.varidic,
                        &context,
                        &mut codegen,
                    )?;

                    let func = codegen.module.add_function(
                        &name.get(0).ident,
                        ty,
                        Some(inkwell::module::Linkage::External),
                    );

                    codegen.functions.insert(name, func);
                }
                ast::DefinitionKind::VarC(_) => {}
                _ => unreachable!(),
            }
        }
    }

    for def in &project.root_module.definitions {
        match &def.kind {
            ast::DefinitionKind::Function(f) => {
                f.to_ir(def.name.clone(), &context, &mut codegen)?
            }
            _ => unimplemented!(),
        }
    }

    let mut out_ll = out.clone();
    out_ll.push("out");
    out_ll.set_extension("ll");

    codegen.module.print_to_file(out_ll)?;
    let triple = TargetMachine::get_default_triple();
    codegen.module.set_triple(&triple);

    let target = Target::from_triple(&triple).unwrap();

    let mut out_o = out.clone();
    out_o.push("out");
    out_o.set_extension("o");

    let machine = target
        .create_target_machine(
            &triple,
            "generic",
            "",
            inkwell::OptimizationLevel::None,
            inkwell::targets::RelocMode::Default,
            inkwell::targets::CodeModel::Default,
        )
        .unwrap();

    machine
        .write_to_file(&codegen.module, inkwell::targets::FileType::Object, &out_o)
        .unwrap();

    return Ok(());
}
