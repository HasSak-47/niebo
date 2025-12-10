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
use registry::*;

#[derive(Debug, Clone)]
pub enum BinaryOperator {
    Addition,
    Multiplication,
    Substraction,
    Division,
    Module,
    BitShitLeft,
    BitShitRight,
    Or,
    And,
    Xor,
}

#[derive(Debug, Clone)]
pub enum UnaryOperator {
    Deref,
    Ref,
    Negation,
}

#[derive(Debug, Clone)]
pub enum Operator {
    Binary {
        operands: [Box<Expression>; 2],
        operator: BinaryOperator,
    },
    Unary {
        operand: Box<Expression>,
        operator: UnaryOperator,
    },
}

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
                    _ => todo!(),
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum Literal {
    Int(u64),
    Uint(u64),
    Bool(bool),
    Float(f64),
    String(String),
}

impl Literal {
    pub fn get_expression_type(&self) -> Type {
        match self {
            Self::Int(_) => Type::Primitive(super::types::PrimitiveType::Int),
            _ => todo!(),
        }
    }

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

#[derive(Debug, Clone)]
pub struct BlockExpression {
    ret_ty: Type,
    body: Vec<Statement>,
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

#[derive(Debug, Clone)]
pub enum Expression {
    Return(Box<Expression>),
    Literal(Literal),
    Operator(Operator),
    Identifier(String),
    Call {
        operand: Box<Expression>,
        params: Vec<Expression>,
    },
    Block(BlockExpression),
}

impl Expression {
    pub fn int(val: i32) -> Expression {
        return Expression::Literal(Literal::Int(val as u64));
    }

    pub fn string<S: AsRef<str>>(s: S) -> Expression {
        let s = s.as_ref().to_string();
        return Expression::Literal(Literal::String(s));
    }

    pub fn identifier<S: AsRef<str>>(s: S) -> Expression {
        let s = s.as_ref().to_string();
        return Expression::Identifier(s);
    }

    pub fn return_statement(exp: Expression) -> Statement {
        Statement::Expression(Expression::Return(Box::new(exp)))
    }

    pub fn call_statement(operand: Expression, params: Vec<Expression>) -> Statement {
        Statement::Expression(Expression::Call {
            operand: Box::new(operand),
            params,
        })
    }

    pub fn code_gen<'a, 'ctx>(
        &self,
        symbols: &mut SymbolRegistry<'ctx>,
        compiler: &ModuleCompiler<'a, 'ctx>,
    ) -> Option<BasicMetadataValueEnum<'ctx>> {
        match self {
            Self::Literal(literal) => literal.code_gen(symbols, compiler),
            Self::Identifier(ident) => {
                return match symbols.get_symbol(ident) {
                    Symbol::Symbol { pointer, .. } => Some(pointer.clone().into()),
                    _ => todo!(),
                };
            }
            Self::Call { operand, params } => {
                let params: Vec<BasicMetadataValueEnum> = params
                    .iter()
                    .map(|v| {
                        let p = v.code_gen(symbols, compiler).unwrap();
                        compiler.builder.build_load(pointee_ty, p, name)
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
            }
            Self::Return(expr) => {
                let val: BasicValueEnum = expr
                    .code_gen(symbols, compiler)
                    .map(|f| f.try_into().unwrap())
                    .unwrap();
                compiler.builder.build_return(Some(&val)).unwrap();
                return None;
            }
            v => todo!("expression {v:?} not yet implemented"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Statement {
    GlobalVariableDeclaration {
        mutable: bool,
        ident: String,
        ty: Type,
    },
    VariableDeclaration {
        mutable: bool,
        ident: String,
        ty: Type,
    },
    FunctionDeclaration {
        ident: String,
        ret_ty: Type,
        params: Vec<(String, Type)>,
        varidic: bool,
    },
    Expression(Expression),
    VariableDefinition {
        ident: String,
        ty: Type,
        expression: Box<Expression>,
    },
    FunctionDefinition {
        ident: String,
        params: Vec<(String, Type)>,
        block: BlockExpression,
        varidic: bool,
    },
}

pub struct FunctionBuilder {
    ident: String,
    ret_ty: Type,
    params: Vec<(String, Type)>,
    varidic: bool,
    body: Option<BlockExpression>,
}

impl FunctionBuilder {
    pub fn new<S: AsRef<str>>(ident: S, ret_ty: Type) -> Self {
        Self {
            ident: ident.as_ref().to_string(),
            ret_ty,
            params: Vec::new(),
            varidic: false,
            body: None,
        }
    }

    pub fn varidic(mut self) -> Self {
        self.varidic = true;
        return self;
    }

    pub fn add_param<S: AsRef<str>>(mut self, ident: S, ty: Type) -> Self {
        self.params.push((ident.as_ref().to_string(), ty));
        return self;
    }

    pub fn add_statement(mut self, stmt: Statement) -> Self {
        if let Some(body) = &mut self.body {
            body.body.push(stmt);
        } else {
            self.body = Some(BlockExpression {
                ret_ty: self.ret_ty.clone(),
                body: vec![stmt],
            })
        }

        return self;
    }
    pub fn build_definition(self) -> Statement {
        assert!(self.body.is_some());
        Statement::FunctionDefinition {
            ident: self.ident,
            block: self.body.unwrap(),
            params: self.params,
            varidic: self.varidic,
        }
    }

    pub fn build_declaration(self) -> Statement {
        assert!(self.body.is_none());
        Statement::FunctionDeclaration {
            ident: self.ident,
            ret_ty: self.ret_ty,
            params: self.params,
            varidic: self.varidic,
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
                symbols.register_symbol(
                    &ident,
                    Symbol::Function {
                        pointer: fv,
                        external: false,
                        ty: Type::Function(ty),
                    },
                );

                let entry = compiler.context.append_basic_block(fv, ident);
                compiler.builder.position_at_end(entry);
                block.code_gen(symbols, compiler);
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
                symbols.register_symbol(&ident, Symbol::Symbol { pointer: var });
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
