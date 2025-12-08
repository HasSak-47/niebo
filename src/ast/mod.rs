use inkwell::{
    builder::Builder,
    context::Context,
    module::Module,
    types::{BasicType, BasicTypeEnum},
    values::{BasicValue, BasicValueEnum},
};

pub struct Compiler<'a, 'ctx> {
    pub context: &'ctx Context,
    pub module: &'a Module<'ctx>,
    pub builder: &'a Builder<'ctx>,
}

impl<'a, 'ctx> Compiler<'a, 'ctx> {
    pub fn new(
        context: &'ctx Context,
        module: &'a Module<'ctx>,
        builder: &'a Builder<'ctx>,
    ) -> Self {
        Self {
            context,
            module,
            builder,
        }
    }
    pub fn build_code(&'a self, ast: &AST) {
        for st in &ast.sts {
            if let Statement::Expression(_) = st {
                continue;
            }
            st.code_gen(self);
        }
    }
}

pub enum Operator {
    Binary {
        operands: [Box<Expression>; 2],
        operator: String,
    },
    Unary {
        operand: Box<Expression>,
        operator: String,
    },
}

pub enum Literal {
    Int(u64),
    Float(f64),
    String(String),
}

impl Literal {
    pub fn code_gen<'a, 'ctx>(&self, llvm: &'a Compiler<'a, 'ctx>) -> BasicValueEnum<'a> {
        match self {
            Self::Int(n) => llvm
                .context
                .i64_type()
                .const_int(*n, false)
                .as_basic_value_enum(),
            Self::Float(f) => llvm
                .context
                .f64_type()
                .const_float(*f)
                .as_basic_value_enum(),
            _ => todo!(),
        }
    }
}

pub enum Expression {
    Literal(Literal),
    Operator(Operator),
    Identifier(String),
    Call {
        operand: Box<Expression>,
        params: Vec<Expression>,
    },
}

impl Expression {
    pub fn code_gen<'a, 'ctx>(&self, llvm: &'a Compiler<'a, 'ctx>) -> BasicValueEnum<'a> {
        match self {
            Self::Literal(lit) => lit.code_gen(llvm),
            _ => todo!(),
        }
    }
}

pub enum PrimitiveType {
    Void,
    Int,
    Uint,
    Float,
    String,
}

impl PrimitiveType {
    pub fn get_basic_type<'a, 'ctx>(&self, llvm: &'a Compiler<'a, 'ctx>) -> BasicTypeEnum<'a> {
        match self {
            Self::Int => llvm.context.i64_type().as_basic_type_enum(),
            _ => todo!(),
        }
    }
}

pub enum Statement {
    VariableDeclaration {
        mutable: bool,
        ident: String,
        ty: PrimitiveType,
        expression: Box<Expression>,
    },
    Expression(Expression),
    FunctionDeclaration {
        ident: String,
        ret_ty: PrimitiveType,
        params: Vec<(String, PrimitiveType)>,
        body: Vec<Statement>,
    },
}

impl Statement {
    pub fn code_gen<'a, 'ctx>(&self, llvm: &'a Compiler<'a, 'ctx>) {
        match self {
            Self::VariableDeclaration {
                ident,
                ty,
                expression,
                ..
            } => {
                let var = llvm
                    .builder
                    .build_alloca(ty.get_basic_type(llvm), ident.as_str())
                    .unwrap();
                llvm.builder
                    .build_store(var, expression.code_gen(llvm))
                    .unwrap();
            }
            Self::FunctionDeclaration { ident, body, .. } => {
                let fn_t = llvm.context.void_type().fn_type(&[], false);
                let fn_v = llvm.module.add_function(ident.as_str(), fn_t, None);
                let entry = llvm.context.append_basic_block(fn_v, "entry");
                llvm.builder.position_at_end(entry);
                for element in body {
                    element.code_gen(llvm);
                }
            }
            _ => todo!(),
        }
    }
}

pub struct AST {
    pub sts: Vec<Statement>,
}
