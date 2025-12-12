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
    pub fn get_expression_type<'a, 'ctx>(
        &self,
        symbols: &Registry<'ctx>,
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
        _symbols: &Registry<'ctx>,
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
        symbols: &Registry<'ctx>,
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
pub struct Call {
    pub operand: Box<ExpressionHandler>,
    pub params: Vec<ExpressionHandler>,
    pub store_to: Option<Box<ExpressionHandler>>,
}

impl Call {
    fn new(
        operand: ExpressionHandler,
        params: Vec<ExpressionHandler>,
        store_to: Option<Box<ExpressionHandler>>,
    ) -> Self {
        return Self {
            operand: Box::new(operand),
            params,
            store_to,
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
}

#[derive(Debug, Clone)]
pub struct ExpressionHandler {
    pub e: ExpressionEnum,
    pub ret_ty: Option<Type>,
}

impl ExpressionHandler {
    pub fn int(val: i32) -> Self {
        return Self {
            e: ExpressionEnum::Literal(Literal::Int(val as u64)),
            ret_ty: Some(Type::int()),
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
            e: ExpressionEnum::Call(Call::new(operand, params, None)),
            ret_ty: None,
        };
    }

    pub fn call_statement(operand: ExpressionHandler, params: Vec<ExpressionHandler>) -> Statement {
        return Statement::Expression(Self::call(operand, params));
    }

    pub fn get_inner_type<'a, 'ctx>(
        &self,
        symbols: &Registry<'ctx>,
        compiler: &ModuleCompiler<'a, 'ctx>,
    ) -> Type {
        use ExpressionEnum as ExpEnum;
        match &self.e {
            ExpEnum::Literal(l) => l.get_expression_type(symbols, compiler),
            ExpEnum::Block(b) => b.get_expression_type(symbols, compiler),
            ExpEnum::Return(r) => r
                .as_ref()
                .map(|p| p.get_expression_type(symbols, compiler))
                .unwrap_or(Type::void()),
            ExpEnum::Call(Call { operand, .. }) => {
                let ty = operand.get_expression_type(symbols, compiler);
                match ty {
                    Type::Function(f) => (*f.ret_ty).clone(),
                    _ => unreachable!(),
                }
            }
            ExpEnum::Identifier(ident) => symbols.get_symbol(ident).get_type().clone(),
            ExpEnum::Operator(op) => op.get_expression_type(symbols, compiler),
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

    pub fn validate_and_determine_expression_type<'a, 'ctx>(
        &mut self,
        symbols: &Registry<'ctx>,
        compiler: &ModuleCompiler<'a, 'ctx>,
    ) {
        if let Some(ty) = &self.ret_ty {
            let inner = self.get_inner_type(symbols, compiler);
            assert_eq!(*ty, inner);
        } else {
            self.ret_ty = Some(self.get_expression_type(symbols, compiler));
        }
    }

    pub fn get_expression_type<'a, 'ctx>(
        &self,
        symbols: &Registry<'ctx>,
        compiler: &ModuleCompiler<'a, 'ctx>,
    ) -> Type {
        if let Some(ty) = &self.ret_ty {
            return ty.clone();
        }
        return self.get_inner_type(symbols, compiler);
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
