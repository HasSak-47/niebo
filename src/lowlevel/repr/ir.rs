use crate::lowlevel::types::*;

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
            Self::Int(_) => Type::int(),
            _ => todo!(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct BlockExpression {
    ret_ty: Type,
    body: Vec<Statement>,
}

impl BlockExpression {}

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

    pub fn new(statements: Vec<Statement>) -> Self {
        let mut s = Self { statements };
        s.validate();
        return s;
    }
}
