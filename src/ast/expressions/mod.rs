pub mod block;
pub mod call;
pub mod conditional;
pub mod init;
pub mod intrinsic;
pub mod literal;
pub mod loops;
pub mod member_access;
pub mod operations;

use std::fmt::Display;

use crate::{
    ast::{
        Definition, Import,
        expressions::{
            block::Block,
            call::Call,
            conditional::Conditional,
            init::StructInit,
            intrinsic::{Intrinsic, IntrinsicKind},
            literal::Literal,
            loops::{LoopExpression, WhileLoop},
            member_access::{MemberAccess, MethodCall},
            operations::{BinaryOperation, BinaryOperator, UnaryOperation, UnaryOperator},
        },
    },
    general::{
        naming::{QualifiedName, QualifiedNameSegment},
        types::Type,
    },
};

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    // DefinitionKind::Module, DefinitionKind::Trait, and impl blocks not allowed
    Import(Import),
    Definition(Definition),
    Expression(Expression),
    Break(Option<String>),
    Continue,
    Use(QualifiedName),
    Return(Option<Expression>),

    // return value for the block expression
    Value(Option<Expression>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExpressionKind {
    MethodCall(MethodCall),
    MemberAccess(MemberAccess),
    Index(Expression, Expression),
    StructInit(StructInit),
    Block(Block),
    If(Conditional),
    Loop(LoopExpression),
    While(WhileLoop),
    Literal(Literal),
    Identifier(QualifiedName),
    Intrinsic(Intrinsic),
    BinaryOperation(BinaryOperation),
    UnaryOperation(UnaryOperation),
    Call(Call),
    Assignment(Expression, Expression),
}

impl Display for ExpressionKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExpressionKind::MethodCall(a) => write!(f, "{}", a),
            ExpressionKind::Index(a, b) => write!(f, "{}[{}]", a, b),
            ExpressionKind::MemberAccess(a) => write!(f, "{}", a),
            ExpressionKind::Block(a) => write!(f, "{}", a),
            ExpressionKind::If(a) => write!(f, "{}", a),
            ExpressionKind::Loop(a) => write!(f, "{}", a),
            ExpressionKind::While(a) => write!(f, "{}", a),
            ExpressionKind::Literal(a) => write!(f, "{}", a),
            ExpressionKind::Identifier(a) => write!(f, "{}", a),
            ExpressionKind::Intrinsic(a) => write!(f, "{}", a),
            ExpressionKind::BinaryOperation(a) => write!(f, "{}", a),
            ExpressionKind::UnaryOperation(a) => write!(f, "{}", a),
            ExpressionKind::Call(a) => write!(f, "{}", a),
            ExpressionKind::Assignment(a, b) => write!(f, "({} = {})", a, b),
            ExpressionKind::StructInit(a) => write!(f, "{}", a),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Expression {
    pub kind: Box<ExpressionKind>,
    pub ret_ty: Option<Type>,
    pub constant: bool,
}

impl Display for Expression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.kind)
    }
}

impl Expression {
    pub fn assignment(var: Expression, value: Expression) -> Self {
        return Self {
            kind: Box::new(ExpressionKind::Assignment(var, value)),
            ret_ty: None,
            constant: false,
        };
    }

    pub fn index_access(val: Expression, index: Expression) -> Self {
        return Self {
            kind: Box::new(ExpressionKind::Index(val, index)),
            ret_ty: None,
            constant: false,
        };
    }

    pub fn method_call<P: Into<QualifiedNameSegment>>(
        object: Expression,
        method: P,
        params: Vec<Expression>,
    ) -> Self {
        let method = method.into();

        return Expression {
            kind: Box::new(ExpressionKind::MethodCall(MethodCall {
                object,
                method,
                params,
            })),
            constant: true,
            ret_ty: None,
        };
    }

    pub fn member_access<P: Into<QualifiedNameSegment>>(object: Expression, member: P) -> Self {
        let member = member.into();

        return Expression {
            kind: Box::new(ExpressionKind::MemberAccess(MemberAccess {
                object,
                member,
            })),
            constant: true,
            ret_ty: None,
        };
    }

    pub fn from_literal(l: Literal) -> Self {
        return Expression {
            kind: Box::new(ExpressionKind::Literal(l)),
            ret_ty: None,
            constant: true,
        };
    }
    pub fn new(kind: ExpressionKind) -> Self {
        Self {
            ret_ty: None,
            kind: Box::new(kind),
            constant: false,
        }
    }

    pub fn block(block: Block) -> Self {
        Self::new(ExpressionKind::Block(block))
    }

    pub fn if_(conditional: Conditional) -> Self {
        Self::new(ExpressionKind::If(conditional))
    }

    pub fn loop_(body: Expression, label: Option<String>) -> Self {
        Self::new(ExpressionKind::Loop(LoopExpression { label, body }))
    }

    pub fn while_(while_loop: WhileLoop) -> Self {
        Self::new(ExpressionKind::While(while_loop))
    }

    pub fn literal<L: Into<Literal>>(literal: L) -> Self {
        let literal = literal.into();

        Self::new(ExpressionKind::Literal(literal))
    }

    pub fn identifier<P: Into<QualifiedName>>(path: P) -> Self {
        let path = path.into();
        Self::new(ExpressionKind::Identifier(path))
    }

    pub fn intrinsic(kind: IntrinsicKind, parameters: Vec<Expression>) -> Self {
        Self::new(ExpressionKind::Intrinsic(Intrinsic::new(kind, parameters)))
    }

    pub fn binary_operation(operation: BinaryOperator, a: Expression, b: Expression) -> Self {
        Self::new(ExpressionKind::BinaryOperation(BinaryOperation {
            operator: operation,
            operands: [a, b],
        }))
    }

    pub fn unary_operation(operator: UnaryOperator, operand: Expression) -> Self {
        return Self::new(ExpressionKind::UnaryOperation(UnaryOperation {
            operator: operator,
            operand: operand,
        }));
    }

    pub fn call(called: Expression, params: Vec<Expression>) -> Self {
        Self::new(ExpressionKind::Call(Call::new(called, params)))
    }
}
