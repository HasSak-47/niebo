use crate::lowlevel::{compiler::ModuleCompiler, repr::registry::SymbolRegistry, types::*};

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
    pub fn get_expression_type<'a, 'ctx>(
        &self,
        symbols: &SymbolRegistry<'ctx>,
        compiler: &ModuleCompiler<'a, 'ctx>,
    ) -> Type {
        return match self {
            Self::Binary { operands, .. } => {
                if operands[0].get_expression_type(symbols, compiler)
                    == operands[1].get_expression_type(symbols, compiler)
                {
                    operands[0].get_expression_type(symbols, compiler)
                } else {
                    unreachable!()
                }
            }
            Self::Unary { operand, operator } => match operator {
                UnaryOperator::Ref => Type::pointer(operand.get_expression_type(symbols, compiler)),
                _ => todo!(),
            },
        };
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
    pub fn get_expression_type<'a, 'ctx>(
        &self,
        _symbols: &SymbolRegistry<'ctx>,
        _compiler: &ModuleCompiler<'a, 'ctx>,
    ) -> Type {
        match self {
            Self::Int(_) => Type::int(),
            _ => todo!(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct BlockExpression {
    pub ret_ty: Type,
    pub body: Vec<Statement>,
}

impl BlockExpression {
    pub fn get_expression_type<'a, 'ctx>(
        &self,
        symbols: &SymbolRegistry<'ctx>,
        compiler: &ModuleCompiler<'a, 'ctx>,
    ) -> Type {
        if self.body.last().is_none() {
            return Type::void();
        }

        if let Statement::Expression(e) = self.body.last().unwrap() {
            return e.get_expression_type(symbols, compiler);
        }
        return Type::void();
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

    pub fn get_expression_type<'a, 'ctx>(
        &self,
        symbols: &SymbolRegistry<'ctx>,
        compiler: &ModuleCompiler<'a, 'ctx>,
    ) -> Type {
        match self {
            Self::Literal(l) => l.get_expression_type(symbols, compiler),
            Self::Block(b) => b.get_expression_type(symbols, compiler),
            Self::Return(r) => r.get_expression_type(symbols, compiler),
            Self::Call { operand, .. } => {
                let ty = operand.get_expression_type(symbols, compiler);
                match ty {
                    Type::Function(f) => (*f.ret_ty).clone(),
                    _ => unreachable!(),
                }
            }
            Self::Identifier(ident) => symbols.get_symbol(ident).get_type().clone(),
            Self::Operator(op) => op.get_expression_type(symbols, compiler),
            r => todo!("{r:?} not implemented"),
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
