use crate::ast::{
    DefinitionKind,
    expressions::{
        Expression,
        ExpressionKind::{self},
        Statement,
        block::Block,
        conditional::ConditionalBuilder,
    },
    project::Project,
};

fn basic_simplify_expression(expr: &mut Expression) -> Option<Expression> {
    match expr.kind.as_mut() {
        ExpressionKind::BinaryOperation(bin) => Some(Expression::method_call(
            bin.operands[0].clone(),
            bin.operator.into_member_access(),
            vec![bin.operands[1].clone()],
        )),
        ExpressionKind::UnaryOperation(un) => Some(Expression::method_call(
            un.operand.clone(),
            un.operator.into_member_access(),
            vec![],
        )),
        ExpressionKind::While(while_loop) => {
            let mut if_body = Block::new();
            if_body.statements.push(Statement::Break(None));

            let cond = Expression::if_(
                ConditionalBuilder::new(while_loop.condition.clone(), Expression::block(if_body))
                    .build(),
            );
            let mut loop_body = Block::new();
            loop_body.add_statement(Statement::Expression(cond));
            loop_body.add_statement(Statement::Expression(while_loop.then.clone()));

            return Some(Expression::loop_(Expression::block(loop_body), None));
        }
        ExpressionKind::Block(block) => {
            return Some(Expression::block(basic_simplify_block(block.clone())));
        }
        _ => None,
    }
}

fn basic_simplify_block(mut block: Block) -> Block {
    for mut stmt in &mut block.statements {
        match &mut stmt {
            Statement::Expression(exp) => {
                if let Some(new_exp) = basic_simplify_expression(exp) {
                    *stmt = Statement::Expression(new_exp);
                }
            }
            _ => {}
        }
    }

    return block;
}

/*
makes a project ast to have less fluff like operations, whiles, etc etc
*/
pub fn basic_simplify_project(project: &mut Project) {
    for def in &mut project.root_module.definitions {
        match &mut def.kind {
            DefinitionKind::FunctionDefinition(func) => {
                func.body = basic_simplify_block(func.body.clone());
            }
            _ => {}
        }
    }
}
