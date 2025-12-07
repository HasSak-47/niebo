use inkwell::{
    builder::Builder,
    context::Context,
    module::Module,
    passes::PassManager,
    values::{BasicValueUse, IntValue},
};

enum Operator {
    Binary {
        operands: [Box<Expression>; 2],
        operator: String,
    },
    Unary {
        operand: Box<Expression>,
        operator: String,
    },
}

enum Literal {
    Number(i64),
    String(String),
    Character(char),
}

impl Literal {
    fn code_gen(&self) {}
}

enum Expression {
    Literal(Literal),
    Operator(Operator),
    Identifier(String),
    Call {
        operand: Box<Expression>,
        params: Vec<Expression>,
    },
}

enum Statement {
    Declaration {
        mutable: bool,
        ident: String,
        ty: Option<String>,
        expression: Option<Box<Expression>>,
    },
    Expression(Expression),
}

struct Function {
    pub body: Vec<Statement>,
}
