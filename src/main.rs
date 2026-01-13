use crate::{
    ast::{
        Definition, Project,
        expressions::{
            Expression, ExpressionKind, Statement, call::Call, literal::Literal,
            operations::BinaryOperator,
        },
        function::FunctionBuilder,
    },
    ir::IR,
};

mod ast;
mod general;
mod ir;
mod parser;

fn main() -> anyhow::Result<()> {
    let mut project = Project::new("test_project", (0, 0, 0));
    let root = &mut project.root_module;
    root.add_c_import("printf");
    root.add_function(
        FunctionBuilder::new("main")
            .add_definition(Definition::variable(
                "x",
                Expression {
                    kind: Box::new(ExpressionKind::Literal(Literal::integer("10", None, None))),
                    ret_ty: None,
                },
                None,
            )?)
            .add_statement(Statement::Expression(Expression::call(
                Expression::identifier("printf"),
                vec![
                    Expression::literal("test_string %d"),
                    Expression::binary_operation(
                        BinaryOperator::Addition,
                        Expression::identifier("x"),
                        Expression::from_literal(Literal {
                            data: "10".to_string(),
                            info: ast::expressions::literal::LiteralInfo::Integer {
                                signed: None,
                                precision: None,
                            },
                        }),
                    ),
                ],
            ))),
    );

    let ir = IR::from_project(project);
    println!("{ir:#?}");

    return Ok(());
}
