use super::ir::*;
use inkwell::{
    AddressSpace,
    values::{
        AnyValue, AnyValueEnum, ArrayValue, BasicMetadataValueEnum, BasicValue, BasicValueEnum,
    },
};

use crate::lowlevel::{
    compiler::ModuleCompiler,
    repr::registry::{Registry, Symbol},
    types::{PrimitiveType, Type},
};

pub trait CodeGenerator {
    // fn get_expression_type<'a, 'ctx>(
    //     &self,
    //     symbols: &SymbolRegistry<'ctx>,
    //     compiler: &mut ModuleCompiler<'a, 'ctx>,
    // ) -> Type;

    fn code_gen<'a, 'ctx>(
        &self,
        symbols: &mut Registry<'ctx>,
        compiler: &mut ModuleCompiler<'a, 'ctx>,
        assign_to: Option<Box<dyn CodeGenerator>>,
    ) -> Option<AnyValueEnum<'ctx>>;
}

impl CodeGenerator for Operator {
    fn code_gen<'a, 'ctx>(
        &self,
        symbols: &mut Registry<'ctx>,
        compiler: &mut ModuleCompiler<'a, 'ctx>,
        assign_to: Option<Box<dyn CodeGenerator>>,
    ) -> Option<AnyValueEnum<'ctx>> {
        match self {
            Operator::Binary { operands, operator } => {
                let a = operands[0].code_gen(symbols, compiler, None).unwrap();
                let b = operands[1].code_gen(symbols, compiler, None).unwrap();
                match operator {
                    _ => todo!(),
                }
            }
            Operator::Unary { operand, operator } => {
                let a = operand
                    .code_gen(symbols, compiler, None)
                    .expect(&format!("{operand:?}"));
                match operator {
                    UnaryOperator::Ref => {
                        return Some(a);
                    }
                    _ => todo!(),
                }
            }
        }
    }
}

impl CodeGenerator for Literal {
    fn code_gen<'a, 'ctx>(
        &self,
        _symbols: &mut Registry,
        compiler: &mut ModuleCompiler<'a, 'ctx>,
        assign_to: Option<Box<dyn CodeGenerator>>,
    ) -> Option<AnyValueEnum<'ctx>> {
        match self {
            Literal::Int(val) => {
                return Some(compiler.context.i32_type().const_int(*val, true).into());
            }
            Literal::String(string) => {
                let bytes = string.as_bytes();
                let mut buf = Vec::with_capacity(bytes.len() + 1);
                buf.extend(bytes);
                buf.push(0);

                let char_ty = compiler.context.i8_type();
                let buf: Vec<_> = buf
                    .into_iter()
                    .map(|v| char_ty.const_int(v as u64, false))
                    .collect();

                let arr_ty = char_ty.array_type(buf.len() as u32);
                let const_arr = unsafe { ArrayValue::new_const_array(&char_ty, buf.as_slice()) };

                let global =
                    compiler
                        .module
                        .add_global(arr_ty, Some(AddressSpace::default()), "strlit");
                global.set_initializer(&const_arr);
                global.set_constant(true);

                Some(global.as_basic_value_enum().into())
            }
            Literal::Uint(u) => {
                return Some(compiler.context.i32_type().const_int(*u, true).into());
            }
            Literal::Bool(b) => {
                return Some(
                    compiler
                        .context
                        .i32_type()
                        .const_int(*b as u64, true)
                        .into(),
                );
            }
            _ => todo!(),
        }
    }
}

impl CodeGenerator for BlockExpression {
    fn code_gen<'a, 'ctx>(
        &self,
        symbols: &mut Registry<'ctx>,
        compiler: &mut ModuleCompiler<'a, 'ctx>,
        _assign_to: Option<Box<dyn CodeGenerator>>,
    ) -> Option<AnyValueEnum<'ctx>> {
        if self.body.len() == 0 {
            return None;
        }

        if let Type::Primitive(ty) = &self.get_expression_type(symbols) {
            if let PrimitiveType::Void = ty {
                for stmt in &self.body {
                    stmt.code_gen(symbols, compiler, None);
                }
                return None;
            }
        }
        for stmt in &self.body[0..(self.body.len() - 1)] {
            stmt.code_gen(symbols, compiler, None);
        }
        if let Statement::Expression(exp) = self.body.last().unwrap() {
            return exp.code_gen(symbols, compiler, None);
        }
        panic!("no ending expression!");
    }
}

impl CodeGenerator for Identifier {
    fn code_gen<'a, 'ctx>(
        &self,
        symbols: &mut Registry<'ctx>,
        _compiler: &mut ModuleCompiler<'a, 'ctx>,
        _assign_to: Option<Box<dyn CodeGenerator>>,
    ) -> Option<AnyValueEnum<'ctx>> {
        assert!(self.name.len() > 0);
        let v = match symbols.get_symbol(&self) {
            Symbol::Label { pointer, .. } => pointer
                .as_ref()
                .and_then(|x| Some(x.clone().as_any_value_enum())),
            Symbol::Value { value: pointer, .. } => pointer
                .as_ref()
                .and_then(|x| Some(x.clone().as_any_value_enum())),
            _ => todo!(),
        }
        .expect(&format!("{self:?} has no value in registry: {symbols:#?}"));

        return Some(v);
    }
}

impl CodeGenerator for Call {
    fn code_gen<'a, 'ctx>(
        &self,
        symbols: &mut Registry<'ctx>,
        compiler: &mut ModuleCompiler<'a, 'ctx>,
        _assign_to: Option<Box<dyn CodeGenerator>>,
    ) -> Option<AnyValueEnum<'ctx>> {
        let func_ty = if let Type::Function(func_ty) = self.operand.get_expression_type(symbols) {
            func_ty
        } else {
            unreachable!()
        };
        // build parameter loading
        let params: Vec<BasicMetadataValueEnum> = self
            .params
            .iter()
            .enumerate()
            .map(|(i, expr)| {
                let p = expr.code_gen(symbols, compiler, None).unwrap();
                if i >= func_ty.params.len() && func_ty.varidic {
                    let ret_ty = expr.get_expression_type(symbols);
                    return ret_ty.build_load(p, "", compiler).try_into().unwrap();
                } else {
                    let (name, ty) = &func_ty.params[i];
                    return ty.build_load(p, name, compiler).try_into().unwrap();
                }
            })
            .collect();
        match &self.operand.e {
            ExpressionEnum::Identifier(ident) => {
                let func = symbols.get_symbol(ident);
                match func {
                    Symbol::Function { pointer, .. } => {
                        return Some(
                            compiler
                                .builder
                                .build_call(*pointer.as_ref().unwrap(), params.as_slice(), "")
                                .unwrap()
                                .as_any_value_enum(),
                        );
                    }
                    _ => todo!(),
                };
            }
            _ => todo!(),
        }
    }
}

impl CodeGenerator for ExpressionHandler {
    fn code_gen<'a, 'ctx>(
        &self,
        symbols: &mut Registry<'ctx>,
        compiler: &mut ModuleCompiler<'a, 'ctx>,
        assign_to: Option<Box<dyn CodeGenerator>>,
    ) -> Option<AnyValueEnum<'ctx>> {
        use ExpressionEnum as ExpEnum;
        let val = match &self.e {
            ExpEnum::Literal(literal) => literal.code_gen(symbols, compiler, None),
            ExpEnum::Identifier(ident) => ident.code_gen(symbols, compiler, None),
            ExpEnum::Call(call) => call.code_gen(symbols, compiler, None),
            ExpEnum::Return(expr) => {
                if let Some(expr) = expr {
                    let val: BasicValueEnum = expr
                        .code_gen(symbols, compiler, None)
                        .map(|f| f.try_into().unwrap())
                        .unwrap();
                    compiler.builder.build_return(Some(&val)).unwrap();
                } else {
                    compiler.builder.build_return(None).unwrap();
                }
                return None;
            }
            ExpEnum::Operator(op) => {
                return op.code_gen(symbols, compiler, None);
            }
            ExpEnum::Block(blk) => {
                return blk.code_gen(symbols, compiler, None);
            }
            ExpEnum::Condition(conds) => {
                return conds.code_gen(symbols, compiler, None);
            }
            #[allow(unreachable_patterns)]
            v => todo!("expression {v:?} not yet implemented"),
        };

        if let Some(expr) = &assign_to {
            let ptr = expr
                .code_gen(symbols, compiler, None)
                .unwrap()
                .into_pointer_value();
            let val = val.unwrap();
            match val {
                AnyValueEnum::IntValue(val) => {
                    compiler.builder.build_store(ptr, val).unwrap();
                }
                e => todo!("{e:?}"),
            }
        }

        return val;
    }
}

impl CodeGenerator for Conditional {
    fn code_gen<'a, 'ctx>(
        &self,
        symbols: &mut Registry<'ctx>,
        compiler: &mut ModuleCompiler<'a, 'ctx>,
        _assign_to: Option<Box<dyn CodeGenerator>>,
    ) -> Option<AnyValueEnum<'ctx>> {
        let ret_ty = self.condition.get_expression_type(symbols);
        assert_eq!(ret_ty, Type::bool());
        let val = self.condition.code_gen(symbols, compiler, None).unwrap();

        let curr_block = *compiler.current_block.last().unwrap();
        let then_block = compiler.context.insert_basic_block_after(curr_block, "");
        let else_block = compiler.context.insert_basic_block_after(then_block, "");
        let cont_block = compiler.context.insert_basic_block_after(else_block, "");

        compiler
            .builder
            .build_conditional_branch(val.into_int_value(), then_block, else_block)
            .unwrap();

        // build then block
        compiler.builder.position_at_end(then_block);
        compiler.add_block(then_block);
        self.then.code_gen(symbols, compiler, None);
        compiler
            .builder
            .build_unconditional_branch(cont_block)
            .unwrap();

        compiler.remove_block();

        // build else block
        compiler.builder.position_at_end(else_block);
        compiler.add_block(else_block);
        self.then.code_gen(symbols, compiler, None);
        compiler
            .builder
            .build_unconditional_branch(cont_block)
            .unwrap();

        compiler.remove_block();

        compiler.builder.position_at_end(cont_block);
        compiler.add_block(cont_block);

        return None;
    }
}
