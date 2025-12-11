pub mod ir;
pub mod prelude;
pub mod registry;

use inkwell::{
    AddressSpace,
    module::Linkage,
    values::{
        AnyValue, ArrayValue, BasicMetadataValueEnum, BasicValue, BasicValueEnum, FunctionValue,
        PointerValue,
    },
};

use crate::lowlevel::{
    compiler::ModuleCompiler,
    types::{FunctionType, PrimitiveType, Type},
};
use ir::*;
use registry::*;

impl Operator {
    pub fn code_gen<'a, 'ctx>(
        &self,
        symbols: &mut SymbolRegistry<'ctx>,
        compiler: &ModuleCompiler<'a, 'ctx>,
    ) -> Option<BasicMetadataValueEnum<'ctx>> {
        match self {
            Operator::Binary { operands, operator } => {
                let a = operands[0].code_gen(symbols, compiler).unwrap();
                let b = operands[1].code_gen(symbols, compiler).unwrap();
                match operator {
                    _ => todo!(),
                }
            }
            Operator::Unary { operand, operator } => {
                let a = operand.code_gen(symbols, compiler).unwrap();
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

impl Literal {
    pub fn code_gen<'a, 'ctx>(
        &self,
        _symbols: &mut SymbolRegistry,
        compiler: &ModuleCompiler<'a, 'ctx>,
    ) -> Option<BasicMetadataValueEnum<'ctx>> {
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

impl BlockExpression {
    pub fn code_gen<'a, 'ctx>(
        &self,
        symbols: &mut SymbolRegistry<'ctx>,
        compiler: &ModuleCompiler<'a, 'ctx>,
    ) -> Option<BasicMetadataValueEnum<'ctx>> {
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
            return exp.code_gen(symbols, compiler);
        }
        panic!("no ending expression!");
    }
}

impl Expression {
    pub fn code_gen<'a, 'ctx>(
        &self,
        symbols: &mut SymbolRegistry<'ctx>,
        compiler: &ModuleCompiler<'a, 'ctx>,
    ) -> Option<BasicMetadataValueEnum<'ctx>> {
        match self {
            Self::Literal(literal) => literal.code_gen(symbols, compiler),
            Self::Identifier(ident) => {
                assert!(ident.len() > 0);
                return match symbols.get_symbol(ident) {
                    Symbol::Symbol { pointer, .. } => Some(pointer.clone().into()),
                    Symbol::SymbolVal { pointer, .. } => Some(pointer.clone().into()),
                    _ => todo!(),
                };
            }
            Self::Call { operand, params } => {
                let ty = operand.get_expression_type(symbols, compiler);
                if let Type::Function(func_ty) = ty {
                    let params: Vec<BasicMetadataValueEnum> = params
                        .iter()
                        .enumerate()
                        .map(|(i, expr)| {
                            let p = expr.code_gen(symbols, compiler).unwrap();
                            if let BasicMetadataValueEnum::PointerValue(ptr) = p {
                                if i >= func_ty.params.len() && func_ty.varidic {
                                    let ret_ty = expr.get_expression_type(symbols, compiler);
                                    if let Type::Pointer(_) = ret_ty {
                                        return ptr.into();
                                    }

                                    return compiler
                                        .builder
                                        .build_load(ret_ty.to_llvm_basic_type(compiler), ptr, "")
                                        .unwrap()
                                        .into();
                                } else {
                                    let (name, ty) = &func_ty.params[i];
                                    match ty {
                                        Type::Primitive(PrimitiveType::String)
                                        | Type::Primitive(PrimitiveType::Void)
                                        | Type::Pointer(_) => return p,
                                        ty => {
                                            return compiler
                                                .builder
                                                .build_load(
                                                    ty.to_llvm_basic_type(compiler),
                                                    ptr,
                                                    name,
                                                )
                                                .unwrap()
                                                .into();
                                        }
                                    }
                                }
                            } else {
                                return p;
                            }
                        })
                        .collect();
                    match &**operand {
                        Expression::Identifier(ident) => {
                            let func = symbols.get_symbol(ident);
                            match func {
                                Symbol::Function { pointer, .. } => {
                                    compiler
                                        .builder
                                        .build_call(*pointer, params.as_slice(), "")
                                        .unwrap();
                                }
                                _ => todo!(),
                            };
                        }
                        _ => todo!(),
                    }
                    return None;
                } else {
                    unreachable!()
                }
            }
            Self::Return(expr) => {
                if let Some(expr) = expr {
                    let val: BasicValueEnum = expr
                        .code_gen(symbols, compiler)
                        .map(|f| f.try_into().unwrap())
                        .unwrap();
                    compiler.builder.build_return(Some(&val)).unwrap();
                } else {
                    compiler.builder.build_return(None).unwrap();
                }
                return None;
            }
            Self::Operator(op) => {
                return op.code_gen(symbols, compiler);
            }
            v => todo!("expression {v:?} not yet implemented"),
        }
    }
}

impl Statement {
    pub fn var_define<S: AsRef<str>>(ident: S, ty: Type, expr: Expression) -> Self {
        return Self::VariableDefinition {
            ident: ident.as_ref().to_string(),
            ty,
            expression: Box::new(expr),
        };
    }
    pub fn code_gen<'a, 'ctx>(
        &self,
        symbols: &mut SymbolRegistry<'ctx>,
        compiler: &ModuleCompiler<'a, 'ctx>,
    ) {
        match self {
            Self::FunctionDeclaration {
                ident,
                params,
                ret_ty,
                varidic,
            } => {
                let ty = FunctionType {
                    params: params.clone(),
                    ret_ty: Box::new(ret_ty.clone()),
                    varidic: varidic.clone(),
                };
                let llvm_ty = ty.build_fn_type(compiler.context);
                let val = compiler
                    .module
                    .add_function(ident, llvm_ty, Some(Linkage::External));

                symbols.register_symbol(
                    &ident,
                    Symbol::Function {
                        pointer: val,
                        external: true,
                        ty: Type::Function(ty),
                    },
                );
            }
            Self::FunctionDefinition {
                ident,
                params,
                block,
                varidic,
            } => {
                let ty = FunctionType {
                    params: params.clone(),
                    ret_ty: Box::new(block.ret_ty.clone()),
                    varidic: varidic.clone(),
                };
                let llvm_ty = ty.build_fn_type(compiler.context);
                let fv = compiler
                    .module
                    .add_function(ident, llvm_ty, Some(Linkage::External));

                symbols.register_symbol_scope(
                    &ident,
                    Symbol::Function {
                        pointer: fv,
                        external: false,
                        ty: Type::Function(ty.clone()),
                    },
                );

                symbols.push_scope();
                for (idx, (ident, ty)) in ty.params.iter().enumerate() {
                    let param = fv.get_nth_param(idx as u32).unwrap();
                    param.set_name(&ident);
                    symbols.register_symbol_scope(
                        &ident,
                        Symbol::SymbolVal {
                            ty: ty.clone(),
                            pointer: param,
                        },
                    );
                }
                let entry = compiler.context.append_basic_block(fv, ident);
                compiler.builder.position_at_end(entry);
                block.code_gen(symbols, compiler);
                symbols.pop_scope();
            }
            Self::VariableDefinition {
                ident,
                ty,
                expression,
                ..
            } => {
                let var = compiler
                    .builder
                    .build_alloca(ty.to_llvm_basic_type(compiler), ident)
                    .unwrap();
                compiler
                    .builder
                    .build_store(
                        var,
                        TryInto::<BasicValueEnum<'ctx>>::try_into(
                            expression.code_gen(symbols, compiler).unwrap(),
                        )
                        .unwrap(),
                    )
                    .unwrap();
                symbols.register_symbol(
                    &ident,
                    Symbol::Symbol {
                        ty: ty.clone(),
                        pointer: var,
                    },
                );
            }
            Self::Expression(e) => {
                e.code_gen(symbols, compiler);
            }
            v => todo!("statement {v:?} not yet implemented"),
        }
    }
}

pub struct Repr {
    statements: Vec<Statement>,
}

impl Repr {
    pub fn validate(&mut self) {
        for statement in &self.statements {
            if let Statement::Expression(_) = statement {
                panic!("no expressions are allowed in module declaration");
            }
        }
    }

    pub fn code_gen<'a, 'ctx>(&self, compiler: &ModuleCompiler<'a, 'ctx>) {
        let mut r = SymbolRegistry::new(&compiler.ident);
        for stmt in &self.statements {
            stmt.code_gen(&mut r, compiler);
        }
    }

    pub fn new(statements: Vec<Statement>) -> Self {
        let mut s = Self { statements };
        s.validate();
        return s;
    }
}
