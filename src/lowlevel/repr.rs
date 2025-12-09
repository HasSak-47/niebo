use std::collections::HashMap;

use inkwell::module::Linkage;

use crate::lowlevel::{
    compiler::ModuleCompiler,
    types::{FunctionType, Type},
};

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

pub enum UnaryOperator {
    Deref,
    Ref,
    Negation,
}

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
}

pub struct BlockExpression {
    local_registry: HashMap<String, Type>,
    ret_ty: Type,
    body: Vec<Statement>,
}

impl BlockExpression {
    pub fn code_gen<'a, 'ctx>(&self, compiler: &ModuleCompiler<'a, 'ctx>) {}
}

pub enum Expression {
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
    pub fn code_gen<'a, 'ctx>(&self, compiler: &ModuleCompiler<'a, 'ctx>) {}
    pub fn get_expression_type(
        &self,
        global_idents: HashMap<String, Type>,
        local_idents: HashMap<String, Type>,
    ) -> Type {
        return match self {
            Self::Literal(literal) => literal.get_expression_type(),
            Self::Block(block) => block.ret_ty.clone(),
            Self::Call { operand, .. } => {
                let ty = operand.get_expression_type(global_idents, local_idents);
                if let Type::Function(func) = ty {
                    *func.ret_ty.clone()
                } else {
                    panic!("operand is not callable!")
                }
            }
            Self::Identifier(ident) => {
                if local_idents.contains_key(ident) {
                    local_idents[ident].clone()
                } else if global_idents.contains_key(ident) {
                    global_idents[ident].clone()
                } else {
                    panic!("found unknown identifier {ident}")
                }
            }
            _ => todo!(),
        };
    }
}

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

impl Statement {
    pub fn code_gen<'a, 'ctx>(&self, compiler: &ModuleCompiler<'a, 'ctx>) {
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
                compiler
                    .module
                    .add_function(ident, ty, Some(Linkage::External));
                block.code_gen(compiler)
            }
            _ => todo!(),
        }
    }
}

pub struct RegistryMember {
    external: bool,
    ty: Type,
}

pub struct Repr {
    statements: Vec<Statement>,
    global_registry: HashMap<String, RegistryMember>,
}

impl Repr {
    pub fn build_registry(&mut self) {
        for statement in &self.statements {
            if let Statement::Expression(_) = &statement {}

            match &statement {
                Statement::Expression(_) => {
                    panic!("no expressions are allowed in module declaration");
                }
                Statement::FunctionDefinition {
                    ident,
                    block,
                    params,
                    varidic,
                } => {
                    if self.global_registry.contains_key(ident) {
                        panic!("redefined symbol");
                    }

                    self.global_registry.insert(
                        ident.clone(),
                        RegistryMember {
                            external: false,
                            ty: Type::Function(FunctionType {
                                params: params.clone(),
                                ret_ty: Box::new(block.ret_ty.clone()),
                                varidic: varidic.clone(),
                            }),
                        },
                    );
                }
                Statement::FunctionDeclaration {
                    ident,
                    ret_ty,
                    params,
                    varidic,
                } => {
                    if self.global_registry.contains_key(ident) {
                        panic!("redefined symbol");
                    }

                    self.global_registry.insert(
                        ident.clone(),
                        RegistryMember {
                            external: false,
                            ty: Type::Function(FunctionType {
                                params: params.clone(),
                                ret_ty: Box::new(ret_ty.clone()),
                                varidic: varidic.clone(),
                            }),
                        },
                    );
                }
                _ => todo!(),
            }
        }
    }

    pub fn code_gen<'a, 'ctx>(&self, compiler: &ModuleCompiler<'a, 'ctx>) {
        for stmt in &self.statements {
            stmt.code_gen(compiler);
        }
    }

    pub fn new(statements: Vec<Statement>) -> Self {
        let mut s = Self {
            statements,
            global_registry: HashMap::new(),
        };
        s.build_registry();
        return s;
    }
}
