use crate::ast::{
    Definition, Expression, ExpressionKind, Literal, Project, function::FunctionBuilder,
    types::Type,
};

mod ast;
mod lowlevel;
mod parser;

fn main() -> anyhow::Result<()> {
    let mut project = Project::new("test_project", (0, 0, 0));
    let root = &mut project.root_module;
    root.add_function(
        FunctionBuilder::new("main").add_definition(Definition::variable(
            "x",
            Expression {
                kind: Box::new(ExpressionKind::Literal(Literal::integer("10", None, None))),
            },
            None,
        )?),
    );
    return Ok(());
}
