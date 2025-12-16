pub mod codegen;
pub mod ir;
pub mod prelude;
pub mod registry;

use inkwell::{
    module::Linkage,
    values::{AnyValueEnum, FunctionValue, IntValue, PointerValue},
};

use crate::lowlevel::{
    compiler::ModuleCompiler,
    types::{FunctionType, PrimitiveType, Type},
};
use codegen::CodeGenerator;
use ir::*;
use registry::*;

impl Statement {
    pub fn var_define<S: AsRef<str>>(ident: S, ty: Type, expr: ExpressionHandler) -> Self {
        return Self::VariableDefinition {
            ident: ident.as_ref().to_string(),
            ty,
            expression: Box::new(expr),
        };
    }
}

impl CodeGenerator for Statement {
    fn code_gen<'a, 'ctx>(
        &self,
        symbols: &mut Registry<'ctx>,
        compiler: &mut ModuleCompiler<'a, 'ctx>,
        assign_to: Option<Box<dyn CodeGenerator>>,
    ) -> Option<AnyValueEnum<'ctx>> {
        assert!(assign_to.is_none());
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
                    ident,
                    Symbol::Function {
                        pointer: Some(val),
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
                    ret_ty: Box::new(block.ret_ty.as_ref().unwrap().clone()),
                    varidic: varidic.clone(),
                };
                let llvm_ty = ty.build_fn_type(compiler.context);
                let fv = compiler.module.add_function(ident, llvm_ty, None);

                // register function in global namespace
                symbols.register_symbol(
                    ident,
                    Symbol::Function {
                        pointer: Some(fv),
                        external: false,
                        ty: Type::Function(ty.clone()),
                    },
                );

                // create new scope
                symbols.push_scope();
                // add parameters to scope
                for (idx, (ident, ty)) in ty.params.iter().enumerate() {
                    let param = fv.get_nth_param(idx as u32).unwrap();
                    param.set_name(&ident);
                    symbols.register_symbol_scope(
                        ident,
                        Symbol::Value {
                            ty: ty.clone(),
                            value: Some(param),
                        },
                    );
                }
                // create code block
                let entry = compiler.context.append_basic_block(fv, ident);
                compiler.add_block(entry);
                compiler.builder.position_at_end(entry);
                block.code_gen(symbols, compiler, None);
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
                symbols.register_symbol_scope(
                    ident,
                    Symbol::Label {
                        ty: ty.clone(),
                        pointer: Some(var),
                    },
                );
                expression
                    .code_gen(
                        symbols,
                        compiler,
                        Some(Box::new(ExpressionHandler::identifier(ident))),
                    )
                    .expect(&format!("{expression:?} doesn't return value"));
            }
            Self::Expression(e) => {
                e.code_gen(symbols, compiler, None);
            }
            v => todo!("statement {v:?} not yet implemented"),
        }

        return None;
    }
}

pub struct Repr {
    pub statements: Vec<Statement>,
}

impl Repr {
    pub fn validate(&mut self) {
        for statement in &self.statements {
            if let Statement::Expression(_) = statement {
                panic!("no expressions are allowed in module declaration");
            }
        }
    }

    pub fn code_gen<'a, 'ctx>(&self, compiler: &mut ModuleCompiler<'a, 'ctx>) {
        let mut r = Registry::new(&compiler.ident);
        for stmt in &self.statements {
            stmt.code_gen(&mut r, compiler, None);
        }
    }

    pub fn new(statements: Vec<Statement>) -> Self {
        let mut s = Self { statements };
        s.validate();
        return s;
    }
}
