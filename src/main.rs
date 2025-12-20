use crate::repr::{
    Definition, Expression, ExpressionKind, Literal, Project, function::FunctionBuilder,
    types::Type,
};

mod lowlevel;
mod parser;
mod repr;

fn main() -> anyhow::Result<()> {
    let mut project = Project::new("test_project", (0, 0, 0));
    let root = &mut project.root_module;

    root.add_function(
        "main",
        FunctionBuilder::new("main", Type::int())
            .add_definition(Definition::variable(
                "x",
                Expression {
                    kind: Box::new(ExpressionKind::Literal(Literal::Integer {
                        signed: Some(true),
                        precision: Some(32),
                        negative: false,
                        value: 0x69,
                    })),
                },
                None,
            )?)
            .build_def(),
    );
    return Ok(());
}
