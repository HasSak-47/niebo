use std::collections::HashMap;

use crate::lowlevel::types::Type;

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
    variable_registry: Vec<(String, Type)>,
    ret_ty: Type,
    body: Vec<Statement>,
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
    },
}

pub struct Repr {
    statements: Vec<Statement>,
    type_registry: HashMap<String, Type>,
    global_registry: HashMap<String, Type>,
}

impl Repr {
    fn build_registry(&self) {
        for statement in &self.statements {
            if let Statement::Expression(_) = &statement {
                panic!("no expressions are allowed in module declaration");
            }
        }
    }
}
