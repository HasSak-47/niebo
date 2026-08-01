pub mod cimports;

use std::{collections::HashMap, path::PathBuf, rc::Rc};

use clang::Clang;
use inkwell::{
    AddressSpace,
    basic_block::BasicBlock,
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
};

use cimports::CCache;

pub struct CodeGen<'ctx> {
    module: Module<'ctx>,
    builder: Builder<'ctx>,
    machine: TargetMachine,
    locals: HashMap<QualifiedName, (PointerValue<'ctx>, Type)>,
    constants: HashMap<QualifiedName, BasicValueEnum<'ctx>>,
    functions: HashMap<QualifiedName, FunctionValue<'ctx>>,
    types: HashMap<QualifiedName, Type>,

    named_blocks: HashMap<String, Rc<BasicBlock<'ctx>>>,
    latest_block: Option<Rc<BasicBlock<'ctx>>>,
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
            Type::Pointer(_) => Ok(ctx
                .ptr_type(AddressSpace::default())
                .fn_type(params, varidic)),
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
                PrimitiveType::Float(_) => Ok(ctx.f32_type().into()),
                PrimitiveType::Void => unreachable!(),
                un => unimplemented!("{un:?}"),
            },
            Type::Pointer(_) => Ok(ctx.ptr_type(AddressSpace::default()).into()),
            Type::MutablePointer(_) => Ok(ctx.ptr_type(AddressSpace::default()).into()),
            Type::Struct(s) => {
                let fields: Vec<_> = s
                    .members
                    .iter()
                    .map(|m| m.1.to_ir_type(&ctx, codegen).unwrap())
                    .collect();

                Ok(ctx.struct_type(fields.as_slice(), false).into())
            }
            Type::Named(named) => {
                let ty = codegen
                    .types
                    .get(named)
                    .expect(&format!("no type named: {named}"))
                    .clone();
                Ok(ty.to_ir_type(ctx, codegen)?)
            }
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

        codegen
            .locals
            .insert(name.into(), (p, self.ty.as_ref().unwrap().clone()));
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
        if codegen.constants.contains_key(self) {
            return Ok(Some(codegen.constants[self]));
        }

        let ptr = self.to_ir_place(ctx, codegen)?;
        return Ok(Some(match codegen.locals[self].1 {
            Type::Primitive(PrimitiveType::Int(32)) => {
                codegen
                    .builder
                    .build_load(ctx.i32_type(), ptr, "load_i32")?
            }
            Type::Pointer(_) => codegen.builder.build_load(
                ctx.ptr_type(AddressSpace::default()),
                ptr,
                "load_ptr",
            )?,
            Type::MutablePointer(_) => codegen.builder.build_load(
                ctx.ptr_type(AddressSpace::default()),
                ptr,
                "load_mut_ptr",
            )?,
            _ => todo!(),
        }));
    }

    fn to_ir_place<'ctx>(
        &self,
        ctx: &'ctx Context,
        codegen: &mut CodeGen<'ctx>,
    ) -> anyhow::Result<PointerValue<'ctx>> {
        let _ = ctx;
        if codegen.constants.contains_key(self) {
            anyhow::bail!("fukc")
        }
        Ok(codegen.locals[self].0)
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
            LiteralInfo::Integer { precision, .. } => match precision {
                Some(prec) => match prec {
                    32 => {
                        return Ok(Some(
                            ctx.i32_type()
                                .const_int_from_string(
                                    &self.data,
                                    inkwell::types::StringRadix::Decimal,
                                )
                                .unwrap()
                                .into(),
                        ));
                    }
                    _ => unimplemented!(""),
                },
                None => {
                    return Ok(Some(
                        ctx.i32_type()
                            .const_int_from_string(&self.data, inkwell::types::StringRadix::Decimal)
                            .unwrap()
                            .into(),
                    ));
                }
            },
            LiteralInfo::String => {
                let p = codegen
                    .builder
                    .build_global_string_ptr(&self.data, "global_string")?
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

fn handle_ptr_ptr<'ctx>(
    ctx: &'ctx Context,
    codegen: &mut CodeGen<'ctx>,
    a_val: BasicValueEnum<'ctx>,
    b_val: BasicValueEnum<'ctx>,
    operator: &BinaryOperator,
) -> anyhow::Result<Option<BasicValueEnum<'ctx>>> {
    let a = a_val.into_pointer_value();
    let b = b_val.into_pointer_value();

    match operator {
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

fn handle_int_int<'ctx>(
    ctx: &'ctx Context,
    codegen: &mut CodeGen<'ctx>,
    a_val: BasicValueEnum<'ctx>,
    b_val: BasicValueEnum<'ctx>,
    operator: &BinaryOperator,
) -> anyhow::Result<Option<BasicValueEnum<'ctx>>> {
    let a = a_val.into_int_value();
    let b = b_val.into_int_value();

    match operator {
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
        BinaryOperator::Multiplication => {
            return Ok(Some(codegen.builder.build_int_mul(a, b, "")?.into()));
        }
        un => unimplemented!("{un:?}"),
    }
}

impl ExpressionIr for BinaryOperation {
    fn to_ir_value<'ctx>(
        &self,
        ctx: &'ctx Context,
        codegen: &mut CodeGen<'ctx>,
    ) -> anyhow::Result<Option<BasicValueEnum<'ctx>>> {
        let a_val = self.operands[0]
            .to_ir_value(ctx, codegen)?
            .ok_or_else(|| anyhow::anyhow!("left operand did not produce a value"))?;
        let b_val = self.operands[1]
            .to_ir_value(ctx, codegen)?
            .ok_or_else(|| anyhow::anyhow!("left operand did not produce a value"))?;

        match (
            self.operands[0].ret_ty.as_ref().unwrap(),
            self.operands[1].ret_ty.as_ref().unwrap(),
        ) {
            (Type::Pointer(_), Type::Pointer(_)) => {
                handle_ptr_ptr(ctx, codegen, a_val, b_val, &self.operator)
            }
            (Type::MutablePointer(_), Type::Pointer(_)) => {
                handle_ptr_ptr(ctx, codegen, a_val, b_val, &self.operator)
            }
            (Type::MutablePointer(_), Type::MutablePointer(_)) => {
                handle_ptr_ptr(ctx, codegen, a_val, b_val, &self.operator)
            }
            (Type::Primitive(PrimitiveType::Int(_)), Type::Primitive(PrimitiveType::Int(_))) => {
                handle_int_int(ctx, codegen, a_val, b_val, &self.operator)
            }
            todo => todo!("{todo:#?}"),
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

                let cond_bb = ctx.insert_basic_block_after(current_block, "while.cond");
                let body_bb = ctx.insert_basic_block_after(current_block, "while.body");
                let end_bb = Rc::new(ctx.insert_basic_block_after(body_bb, "while.end"));
                if let Some(name) = &while_.label {
                    codegen.named_blocks.insert(name.clone(), end_bb.clone());
                }
                codegen.latest_block = Some(end_bb.clone());

                codegen.builder.build_unconditional_branch(cond_bb)?;
                codegen.builder.position_at_end(cond_bb);

                // condition
                let con = while_.condition.to_ir_value(ctx, codegen)?.unwrap();
                codegen
                    .builder
                    .build_conditional_branch(con.into_int_value(), body_bb, *end_bb)?;
                codegen.builder.position_at_end(body_bb);
                // continue loop
                while_.then.to_ir_value(ctx, codegen)?;
                codegen.builder.build_unconditional_branch(cond_bb).unwrap();

                // after loop
                codegen.builder.position_at_end(*end_bb);

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
            ExpressionKind::Loop(loop_) => {
                let current_block = codegen
                    .builder
                    .get_insert_block()
                    .ok_or_else(|| anyhow::anyhow!("builder has no insertion block"))?;

                let body_bb = ctx.insert_basic_block_after(current_block, "loop.body");
                let end_bb = Rc::new(ctx.insert_basic_block_after(body_bb, "loop.end"));

                if let Some(name) = &loop_.label {
                    codegen.named_blocks.insert(name.clone(), end_bb.clone());
                }
                codegen.latest_block = Some(end_bb.clone());

                // continue loop
                codegen.builder.build_unconditional_branch(body_bb)?;
                codegen.builder.position_at_end(body_bb);

                loop_.body.to_ir_value(ctx, codegen)?;

                if let Some(name) = &loop_.label {
                    codegen.named_blocks.remove(name);
                }
                codegen.latest_block = None;

                // after loop
                codegen.builder.build_unconditional_branch(body_bb)?;
                codegen.builder.position_at_end(*end_bb);

                return Ok(None);
            }
            ExpressionKind::If(if_) => {
                let current_block = codegen
                    .builder
                    .get_insert_block()
                    .ok_or_else(|| anyhow::anyhow!("builder has no insertion block"))?;

                let cond_bb = ctx.insert_basic_block_after(current_block, "if.cond");
                let body_bb = ctx.insert_basic_block_after(cond_bb, "if.then");
                let (else_bb, end_bb) = if if_.else_.is_some() {
                    let else_bb = ctx.insert_basic_block_after(body_bb, "if.else");
                    (else_bb, ctx.insert_basic_block_after(else_bb, "if.end"))
                } else {
                    let end = ctx.insert_basic_block_after(body_bb, "if.end");
                    (end, end)
                };

                codegen.builder.build_unconditional_branch(cond_bb)?;

                // condition
                codegen.builder.position_at_end(cond_bb);
                let con = if_.condition.to_ir_value(ctx, codegen)?.unwrap();
                codegen
                    .builder
                    .build_conditional_branch(con.into_int_value(), body_bb, else_bb)?;

                // then block
                codegen.builder.position_at_end(body_bb);
                if_.then.to_ir_value(ctx, codegen)?;

                // if last instruction of block is a br do not add an aditional br to if.end
                if match body_bb.get_last_instruction() {
                    Some(ins) => match ins.get_opcode() {
                        inkwell::values::InstructionOpcode::Br => false,
                        inkwell::values::InstructionOpcode::Return => false,
                        _ => true,
                    },
                    None => true,
                } {
                    codegen.builder.build_unconditional_branch(end_bb).unwrap();
                }

                // else block
                if let Some(ex) = &if_.else_ {
                    codegen.builder.position_at_end(else_bb);
                    ex.to_ir_value(ctx, codegen)?;
                    codegen.builder.build_unconditional_branch(end_bb).unwrap();
                }
                codegen.builder.position_at_end(end_bb);

                // fuck it I just want it to compiler rn
                return Ok(None);
            }
            ExpressionKind::Index(exp, idx) => {
                let offset = idx.to_ir_value(ctx, codegen)?.unwrap();
                let ptr = exp.to_ir_value(ctx, codegen)?.unwrap().into_pointer_value();

                let offset_ptr = unsafe {
                    let ir_ty = exp
                        .ret_ty
                        .as_ref()
                        .clone()
                        .unwrap()
                        .to_ir_type(ctx, codegen)?;
                    codegen.builder.build_gep(
                        ir_ty,
                        ptr,
                        &[offset.into_int_value()],
                        "ir_value_pointer_access",
                    )?
                };

                return Ok(Some(codegen.builder.build_load(
                    ctx.i32_type(),
                    offset_ptr,
                    "",
                )?));
            }
            ExpressionKind::StructInit(init) => {
                todo!()
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
            ExpressionKind::Index(exp, idx) => {
                let offset = idx.to_ir_value(ctx, codegen)?.unwrap();
                let ptr_val = exp.to_ir_value(ctx, codegen)?.unwrap().into_pointer_value();

                let offset_ptr = unsafe {
                    let ir_ty = match exp.ret_ty.clone().unwrap() {
                        Type::Array(r) => *r,
                        Type::MutablePointer(r) => *r,
                        _ => unreachable!(),
                    }
                    .to_ir_type(ctx, codegen)?;

                    codegen.builder.build_gep(
                        ir_ty,
                        ptr_val,
                        &[offset.into_int_value()],
                        "ir_place_pointer_access",
                    )?
                };

                return Ok(offset_ptr);
            }
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
            Statement::Break(break_) => match break_ {
                Some(s) => {
                    let end_bb = codegen
                        .named_blocks
                        .get(s)
                        .expect(&format!("{:?}", codegen.named_blocks));
                    codegen.builder.build_unconditional_branch(**end_bb)?;
                }
                None => {
                    unimplemented!()
                }
            },
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
        let basic_block = ctx.append_basic_block(func, "statement_block");
        codegen.builder.position_at_end(basic_block);

        for stmt in &self.body.statements {
            stmt.to_ir(String::new(), ctx, codegen)?;
        }

        return Ok(());
    }
}

pub fn compile(project: Project, out: PathBuf) -> anyhow::Result<()> {
    let clang = Clang::new().unwrap();
    let mut ccache = CCache::new(&clang)?;

    let context = Context::create();

    let triple = TargetMachine::get_default_triple();

    // creating code module
    let module = context.create_module(&project.name);
    module.set_triple(&triple);

    // creating target machine
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
        .ok_or(anyhow::anyhow!("failed to create machine"))?;
    println!("generated machine");

    let mut codegen = CodeGen {
        types: HashMap::new(),
        module: module,
        machine: machine,
        builder: context.create_builder(),
        locals: HashMap::new(),
        constants: HashMap::new(),
        functions: HashMap::new(),
        named_blocks: HashMap::new(),
        latest_block: None,
    };

    codegen.constants.insert(
        "nullptr".into(),
        context
            .ptr_type(AddressSpace::default())
            .const_null()
            .into(),
    );

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
        println!("generating def {}", def.name);
        match &def.kind {
            ast::DefinitionKind::Function(f) => {
                f.to_ir(def.name.clone(), &context, &mut codegen)?
            }
            // do jackshit the types are handled when needed
            // probably would be smart to keep a registry of them
            ast::DefinitionKind::Type(ty) => {
                codegen.types.insert(def.name.clone().into(), ty.clone());
                println!("{ty:?}");
            }
            _ => unimplemented!(),
        }
    }

    let mut out_ll = out.clone();
    out_ll.push("out");
    out_ll.set_extension("ll");

    codegen.module.print_to_file(out_ll)?;
    println!("generated .ll output");

    codegen
        .machine
        .write_to_file(&codegen.module, inkwell::targets::FileType::Object, &out_o)?;
    println!("generated .o output");

    return Ok(());
}
