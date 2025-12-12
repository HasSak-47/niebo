use super::ir::*;
use inkwell::{
    AddressSpace,
    module::Linkage,
    values::{
        AnyValue, AnyValueEnum, ArrayValue, BasicMetadataValueEnum, BasicValue, BasicValueEnum,
        FunctionValue, PointerValue,
    },
};

use crate::lowlevel::{
    compiler::ModuleCompiler,
    repr::registry::{Registry, Symbol},
    types::{PrimitiveType, Type},
};

pub trait Expression {
    // fn get_expression_type<'a, 'ctx>(
    //     &self,
    //     symbols: &SymbolRegistry<'ctx>,
    //     compiler: &ModuleCompiler<'a, 'ctx>,
    // ) -> Type;

    fn code_gen<'a, 'ctx>(
        &self,
        symbols: &mut Registry<'ctx>,
        compiler: &ModuleCompiler<'a, 'ctx>,
        assing_to: Option<Box<Self>>,
    ) -> Option<AnyValueEnum<'ctx>>;
}

impl Expression for Operator {
    fn code_gen<'a, 'ctx>(
        &self,
        symbols: &mut Registry<'ctx>,
        compiler: &ModuleCompiler<'a, 'ctx>,
        _assing_to: Option<Box<Self>>,
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
                let a = operand.code_gen(symbols, compiler, None).unwrap();
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

impl Expression for Literal {
    fn code_gen<'a, 'ctx>(
        &self,
        _symbols: &mut Registry,
        compiler: &ModuleCompiler<'a, 'ctx>,
        _assing_to: Option<Box<Self>>,
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
            _ => todo!(),
        }
    }
}

impl Expression for BlockExpression {
    fn code_gen<'a, 'ctx>(
        &self,
        symbols: &mut Registry<'ctx>,
        compiler: &ModuleCompiler<'a, 'ctx>,
        _assing_to: Option<Box<Self>>,
    ) -> Option<AnyValueEnum<'ctx>> {
        if self.body.len() == 0 {
            return None;
        }

        if let Type::Primitive(ty) = &self.ret_ty {
            if let PrimitiveType::Void = ty {
                for stmt in &self.body {
                    stmt.code_gen(symbols, compiler);
                }
                return None;
            }
        }
        for stmt in &self.body[0..(self.body.len() - 1)] {
            stmt.code_gen(symbols, compiler);
        }
        if let Statement::Expression(exp) = self.body.last().unwrap() {
            return exp.code_gen(symbols, compiler, None);
        }
        panic!("no ending expression!");
    }
}

impl Expression for Identifier {
    fn code_gen<'a, 'ctx>(
        &self,
        symbols: &mut Registry<'ctx>,
        compiler: &ModuleCompiler<'a, 'ctx>,
        assing_to: Option<Box<Self>>,
    ) -> Option<AnyValueEnum<'ctx>> {
        assert!(self.name.len() > 0);
        return match symbols.get_symbol(&self) {
            Symbol::Label { pointer, .. } => pointer
                .as_ref()
                .and_then(|x| Some(x.clone().as_any_value_enum())),
            Symbol::Value { pointer, .. } => pointer
                .as_ref()
                .and_then(|x| Some(x.clone().as_any_value_enum())),
            _ => todo!(),
        };
    }
}

impl Expression for Call {
    fn code_gen<'a, 'ctx>(
        &self,
        symbols: &mut Registry<'ctx>,
        compiler: &ModuleCompiler<'a, 'ctx>,
        _assing_to: Option<Box<Self>>,
    ) -> Option<AnyValueEnum<'ctx>> {
        let func_ty =
            if let Type::Function(func_ty) = self.operand.get_expression_type(symbols, compiler) {
                func_ty
            } else {
                unreachable!()
            };
        let params: Vec<BasicMetadataValueEnum> = self
            .params
            .iter()
            .enumerate()
            .map(|(i, expr)| {
                let p = expr.code_gen(symbols, compiler, None).unwrap();
                if i >= func_ty.params.len() && func_ty.varidic {
                    let ret_ty = expr.get_expression_type(symbols, compiler);
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

impl Expression for ExpressionHandler {
    fn code_gen<'a, 'ctx>(
        &self,
        symbols: &mut Registry<'ctx>,
        compiler: &ModuleCompiler<'a, 'ctx>,
        _assing_to: Option<Box<Self>>,
    ) -> Option<AnyValueEnum<'ctx>> {
        use ExpressionEnum as ExpEnum;
        match &self.e {
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
            v => todo!("expression {v:?} not yet implemented"),
        }
    }
}
