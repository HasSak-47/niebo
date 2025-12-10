#[derive(Debug, Clone)]
pub enum LiteralExpression {}

#[derive(Debug, Clone)]
pub enum OperatorExpression {}

#[derive(Debug, Clone)]
pub enum CallExpression {}

#[derive(Debug, Clone)]
pub enum BlockExpression {}

#[derive(Debug, Clone)]
pub enum Expression {
    Return(Box<Expression>),
    Literal(LiteralExpression),
    Operator(OperatorExpression),
    Identifier(String),
    Call {
        operand: Box<Expression>,
        params: Vec<Expression>,
    },
    Block(BlockExpression),
}
