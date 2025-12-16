use crate::lowlevel::{compiler::ModuleCompiler, repr::registry::Registry, types::*};

#[derive(Debug, Clone)]
pub struct Identifier {
    pub name: String,
    pub path: Vec<String>,
}

impl Identifier {
    fn new<S: Into<String>>(name: S, path: Vec<String>) -> Self {
        let name = name.into();
        return Self { name, path };
    }
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
        operands: [Box<ExpressionHandler>; 2],
        operator: BinaryOperator,
    },
    Unary {
        operand: Box<ExpressionHandler>,
        operator: UnaryOperator,
    },
}

impl Operator {
    pub fn get_expression_type<'a, 'ctx>(&self, symbols: &Registry<'ctx>) -> Type {
        return match self {
            Self::Binary { operands, .. } => {
                if operands[0].get_expression_type(symbols)
                    == operands[1].get_expression_type(symbols)
                {
                    operands[0].get_expression_type(symbols)
                } else {
                    unreachable!()
                }
            }
            Self::Unary { operand, operator } => match operator {
                UnaryOperator::Ref => Type::pointer(operand.get_expression_type(symbols)),
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

impl From<bool> for Literal {
    fn from(value: bool) -> Self {
        Literal::Bool(value)
    }
}

impl From<i64> for Literal {
    fn from(value: i64) -> Self {
        Literal::Int(value as u64)
    }
}

impl From<u64> for Literal {
    fn from(value: u64) -> Self {
        Literal::Uint(value)
    }
}

impl Literal {
    pub fn get_expression_type<'a, 'ctx>(&self, _symbols: &Registry<'ctx>) -> Type {
        match self {
            Self::Int(_) => Type::int(),
            Self::Uint(_) => Type::uint(),
            Self::Bool(_) => Type::bool(),
            lit => todo!("{lit:?} not implemented"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct BlockExpression {
    pub body: Vec<Statement>,
}

impl BlockExpression {
    pub fn new(body: Vec<Statement>) -> Self {
        if body.len() == 0 {
            return Self { body };
        }
        if let Statement::Expression(e) = &body.last().unwrap() {
            return Self { body: body.clone() };
        }

        return Self { body };
    }
    pub fn get_expression_type<'a, 'ctx>(&self, symbols: &Registry<'ctx>) -> Type {
        if self.body.last().is_none() {
            return Type::void();
        }

        if let Statement::Expression(e) = self.body.last().unwrap() {
            return e.get_expression_type(symbols);
        }
        return Type::void();
    }
}

#[derive(Debug, Clone)]
pub struct Call {
    pub operand: Box<ExpressionHandler>,
    pub params: Vec<ExpressionHandler>,
}

impl Call {
    fn new(operand: ExpressionHandler, params: Vec<ExpressionHandler>) -> Self {
        return Self {
            operand: Box::new(operand),
            params,
        };
    }
}

#[derive(Debug, Clone)]
pub struct Conditional {
    pub condition: Box<ExpressionHandler>,
    pub then: Box<ExpressionHandler>,
    pub els_: Option<Box<ExpressionHandler>>,
}

impl Conditional {
    pub fn new(condition: ExpressionHandler, then: ExpressionHandler) -> Self {
        return Self {
            condition: Box::new(condition),
            then: Box::new(then),
            els_: None,
        };
    }

    pub fn set_else(&mut self, then: ExpressionHandler) {
        self.els_ = Some(Box::new(then));
    }

    pub fn validate_and_determine_expression_type<'ctx>(&mut self, symbols: &mut Registry<'ctx>) {
        if let Type::Primitive(PrimitiveType::Bool) = self.condition.get_expression_type(symbols) {
        } else {
            panic!("if condition is not boolean")
        }
    }
}

#[derive(Debug, Clone)]
pub struct ConditionalBuilder {
    pub ifs: Vec<Conditional>,
    pub e: Option<ExpressionHandler>,
}

impl ConditionalBuilder {
    pub fn new(condition: ExpressionHandler, then: ExpressionHandler) -> Self {
        return Self {
            ifs: vec![Conditional::new(condition, then)],
            e: None,
        };
    }

    pub fn add_if(mut self, condition: ExpressionHandler, then: ExpressionHandler) -> Self {
        self.ifs.push(Conditional::new(condition, then));
        self
    }

    pub fn set_else(mut self, e: ExpressionHandler) -> Self {
        self.e = Some(e);
        self
    }

    pub fn build(mut self) -> ExpressionHandler {
        let mut last = self.ifs.pop().unwrap();
        if let Some(e) = self.e {
            last.set_else(e);
        }
        for mut if_ in self.ifs.into_iter().rev() {
            if_.set_else(ExpressionHandler {
                e: ExpressionEnum::Condition(last),
                ret_ty: None,
            });
            last = if_;
        }

        return ExpressionHandler {
            e: ExpressionEnum::Condition(last),
            ret_ty: None,
        };
    }
}

#[derive(Debug, Clone)]
pub enum ExpressionEnum {
    Return(Option<Box<ExpressionHandler>>),
    Literal(Literal),
    Operator(Operator),
    Identifier(Identifier),
    Call(Call),
    Block(BlockExpression),
    Condition(Conditional),
}

#[derive(Debug, Clone)]
pub struct ExpressionHandler {
    pub e: ExpressionEnum,
    pub ret_ty: Option<Type>,
}

impl ExpressionHandler {
    pub fn literal<L: Into<Literal>>(literal: L) -> Self {
        let literal = literal.into();
        let r = Registry::new("");
        let ret_ty = Some(literal.get_expression_type(&r));
        return Self {
            e: ExpressionEnum::Literal(literal),
            ret_ty,
        };
    }

    pub fn string<S: AsRef<str>>(s: S) -> Self {
        let s = s.as_ref().to_string();
        return Self {
            e: ExpressionEnum::Literal(Literal::String(s)),
            ret_ty: Some(Type::string()),
        };
    }

    pub fn identifier<S: Into<String>>(s: S) -> Self {
        return Self {
            e: ExpressionEnum::Identifier(Identifier::new(s, vec![])),
            ret_ty: None,
        };
    }

    pub fn return_expression(exp: Option<ExpressionHandler>) -> Self {
        return Self {
            e: ExpressionEnum::Return(exp.and_then(|f| Some(Box::new(f)))),
            ret_ty: None,
        };
    }

    pub fn return_statement(exp: Option<ExpressionHandler>) -> Statement {
        return Statement::Expression(Self::return_expression(exp));
    }

    pub fn call(operand: ExpressionHandler, params: Vec<ExpressionHandler>) -> Self {
        return Self {
            e: ExpressionEnum::Call(Call::new(operand, params)),
            ret_ty: None,
        };
    }

    pub fn call_statement(operand: ExpressionHandler, params: Vec<ExpressionHandler>) -> Statement {
        return Statement::Expression(Self::call(operand, params));
    }

    pub fn get_inner_type<'a, 'ctx>(&self, symbols: &Registry<'ctx>) -> Type {
        use ExpressionEnum as ExpEnum;
        match &self.e {
            ExpEnum::Literal(l) => l.get_expression_type(symbols),
            ExpEnum::Block(b) => b.get_expression_type(symbols),
            ExpEnum::Return(r) => r
                .as_ref()
                .map(|p| p.get_expression_type(symbols))
                .unwrap_or(Type::void()),
            ExpEnum::Call(Call { operand, .. }) => {
                let ty = operand.get_expression_type(symbols);
                match ty {
                    Type::Function(f) => (*f.ret_ty).clone(),
                    _ => unreachable!(),
                }
            }
            ExpEnum::Identifier(ident) => symbols.get_symbol(ident).get_type().clone(),
            ExpEnum::Operator(op) => op.get_expression_type(symbols),
            r => todo!("{r:?} not implemented"),
        }
    }

    pub fn unary_operation(operator: UnaryOperator, a: ExpressionHandler) -> Self {
        return Self {
            e: ExpressionEnum::Operator(Operator::Unary {
                operand: Box::new(a),
                operator,
            }),
            ret_ty: None,
        };
    }

    #[allow(dead_code)]
    pub fn binary_operation(
        operator: BinaryOperator,
        a: ExpressionHandler,
        b: ExpressionHandler,
    ) -> Self {
        return Self {
            e: ExpressionEnum::Operator(Operator::Binary {
                operands: [Box::new(a), Box::new(b)],
                operator,
            }),
            ret_ty: None,
        };
    }

    pub fn validate_and_determine_expression_type<'a, 'ctx>(&mut self, symbols: &Registry<'ctx>) {
        if let Some(ty) = &self.ret_ty {
            let inner = self.get_inner_type(symbols);
            assert_eq!(*ty, inner);
        } else {
            self.ret_ty = Some(self.get_expression_type(symbols));
        }
    }

    pub fn get_expression_type<'a, 'ctx>(&self, symbols: &Registry<'ctx>) -> Type {
        if let Some(ty) = &self.ret_ty {
            return ty.clone();
        }
        return self.get_inner_type(symbols);
    }

    pub fn new_block(body: Vec<Statement>) -> Self {
        Self {
            e: ExpressionEnum::Block(BlockExpression { body }),
            ret_ty: None,
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
    Expression(ExpressionHandler),
    VariableDefinition {
        ident: String,
        ty: Type,
        expression: Box<ExpressionHandler>,
    },
    FunctionDefinition {
        ident: String,
        params: Vec<(String, Type)>,
        block: ExpressionHandler,
        varidic: bool,
    },
}

impl Statement {
    pub fn validate_statement<'ctx>(&mut self, symbols: &Registry<'ctx>) {
        match self {
            Self::VariableDefinition { expression, .. } => {
                expression.validate_and_determine_expression_type(symbols)
            }
            Self::FunctionDefinition { block, .. } => {
                block.validate_and_determine_expression_type(symbols)
            }
            Self::Expression(expression) => {
                expression.validate_and_determine_expression_type(symbols)
            }
            _ => {}
        }
    }
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
            self.body = Some(BlockExpression { body: vec![stmt] })
        }

        return self;
    }
    pub fn build_definition(self) -> Statement {
        assert!(self.body.is_some());
        Statement::FunctionDefinition {
            ident: self.ident,
            block: ExpressionHandler {
                e: ExpressionEnum::Block(self.body.unwrap()),
                ret_ty: Some(self.ret_ty),
            },
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
