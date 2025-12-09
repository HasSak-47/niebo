use std::collections::HashMap;

use inkwell::{
    module::Linkage,
    values::{BasicValue, FunctionValue, PointerValue},
};

use crate::lowlevel::{
    compiler::ModuleCompiler,
    types::{FunctionType, PrimitiveType, Type},
};

pub struct SymbolRegistry<'ctx> {
    reg: HashMap<String, Symbol<'ctx>>,
    scope: SymbolScope<'ctx>,
}

impl<'ctx> SymbolRegistry<'ctx> {
    pub fn new<S: AsRef<str>>(namespace: S) -> Self {
        let mut reg = HashMap::new();
        reg.insert(
            namespace.as_ref().to_string(),
            Symbol::Registry(SymbolRegistry {
                reg: HashMap::new(),
                scope: SymbolScope::new(),
            }),
        );
        return Self {
            reg,
            scope: SymbolScope::new(),
        };
    }

    pub fn get_symbol<S: AsRef<str>>(&self, ident: S) -> &Symbol<'ctx> {
        let ident = ident.as_ref();
        if let Some(s) = self.scope.get_symbol(ident) {
            return s;
        }

        for (id, symbol) in &self.reg {
            if id == ident {
                return &symbol;
            }
        }
        panic!("symbol not found!");
    }

    pub fn register_symbol<S: AsRef<str>>(&mut self, ident: S, symbol: Symbol<'ctx>) {
        if let Some(_) = self.reg.insert(ident.as_ref().to_string(), symbol) {
            panic!("symbol redefined!");
        }
    }
}

pub struct SymbolScope<'ctx> {
    scope: Vec<HashMap<String, Symbol<'ctx>>>,
}

impl<'ctx> SymbolScope<'ctx> {
    pub fn new() -> Self {
        Self { scope: Vec::new() }
    }

    pub fn push_scope(&mut self) {
        self.scope.push(HashMap::new());
    }

    pub fn pop_scope(&mut self) {
        self.scope.pop();
    }

    pub fn register_symbol<S: AsRef<str>>(&mut self, ident: S, symbol: Symbol<'ctx>) {
        let repeat = self
            .scope
            .last_mut()
            .unwrap()
            .insert(ident.as_ref().to_string(), symbol);
        if repeat.is_some() {
            panic!("symbol redefined!");
        }
    }

    pub fn get_symbol<S: AsRef<str>>(&self, ident: S) -> Option<&Symbol<'ctx>> {
        let ident = ident.as_ref();
        for symbols in self.scope.iter().rev() {
            if symbols.contains_key(ident) {
                return Some(&symbols[ident]);
            }
        }

        return None;
    }
}

enum Symbol<'ctx> {
    Function {
        pointer: Option<FunctionValue<'ctx>>,
        external: bool,
        ty: Type,
    },
    Symbol {
        pointer: Option<PointerValue<'ctx>>,
    },
    Registry(SymbolRegistry<'ctx>),
}

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
        symbols: &mut SymbolRegistry,
        compiler: &ModuleCompiler<'a, 'ctx>,
    ) {
    }
}

#[derive(Debug, Clone)]
pub struct BlockExpression {
    local_registry: HashMap<String, Type>,
    ret_ty: Type,
    body: Vec<Statement>,
}

impl BlockExpression {
    pub fn code_gen<'a, 'ctx>(
        &self,
        symbols: &mut SymbolRegistry<'ctx>,
        compiler: &ModuleCompiler<'a, 'ctx>,
    ) -> Option<Box<dyn BasicValue<'ctx> + 'ctx>> {
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
    pub fn code_gen<'a, 'ctx>(
        &self,
        symbols: &mut SymbolRegistry<'ctx>,
        compiler: &ModuleCompiler<'a, 'ctx>,
    ) -> Option<Box<dyn BasicValue<'ctx> + 'ctx>> {
        match self {
            Self::Literal(literal) => match literal {
                Literal::Int(val) => {
                    return Some(Box::new(
                        compiler
                            .context
                            .i32_type()
                            .const_int(*val, true)
                            .as_basic_value_enum(),
                    ));
                }
                _ => todo!(),
            },
            Self::Identifier(ident) => {
                return match symbols.get_symbol(ident) {
                    Symbol::Symbol { pointer, .. } => Some(Box::new(pointer.unwrap().clone())),
                    _ => todo!(),
                };
            }
            Self::Call { operand, params } => {
                return None;
            }
            Self::Return(expr) => {
                let val = expr.code_gen(symbols, compiler).unwrap();
                let v = compiler.builder.build_return(Some(&*val)).unwrap();
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
        mutable: bool,
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
                local_registry: HashMap::new(),
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
                }
                .build_fn_type(compiler.context);
                compiler
                    .module
                    .add_function(ident, ty, Some(Linkage::External));
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
                }
                .build_fn_type(compiler.context);
                let fv = compiler
                    .module
                    .add_function(ident, ty, Some(Linkage::External));
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
                        expression
                            .code_gen(symbols, compiler)
                            .unwrap()
                            .as_basic_value_enum(),
                    )
                    .unwrap();
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
