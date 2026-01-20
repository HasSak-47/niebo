use crate::ast::expressions::operations::BinaryOperator;
use crate::ir::core::create_core_project;

use crate::{
    ast::{
        DefinitionKind, Module,
        expressions::{
            Expression, ExpressionKind, Statement, block::Block, conditional::Conditional,
            loops::LoopExpression, operations::UnaryOperator,
        },
        project::Project,
        traits::Trait,
    },
    general::{path::Path, types::Type},
};

pub mod core;

#[derive(Debug)]
pub struct IR<'a> {
    project: Project,
    traits: Vec<(Path, &'a Trait)>,
}

fn find_traits<'a>(cur_path: Path, traits: &mut Vec<(Path, &'a Trait)>, module: &'a Module) {
    for def in &module.definitions {
        match &def.kind {
            DefinitionKind::Trait(t) => {
                let mut path = cur_path.clone();
                path.v.push(def.name.clone().into());
                traits.push((path, t));
            }
            DefinitionKind::Module(m) => {
                let mut path = cur_path.clone();
                path.v.push(def.name.clone().into());

                find_traits(path, traits, &m);
            }
            _ => {}
        }
    }
}

impl<'a> IR<'a> {
    pub fn from_project(mut p: Project) -> Self {
        p.external_projects
            .insert("core".to_string(), create_core_project());

        let mut traits = Vec::new();
        let path = Path::from(p.name.clone());

        let mut ir = Self {
            project: p,
            traits: Vec::new(),
        };
        find_traits(path, &mut traits, &ir.project.root_module);
        for (name, module) in &ir.project.external_projects {
            // find_traits(Path::from(name), &mut traits, module);
            todo!()
        }

        // de sugar_module
        for def in &mut ir.project.root_module.definitions {
            match &mut def.kind {
                DefinitionKind::Function(func) => desugar_block(&mut func.body),
                _ => {}
            }
        }

        // evaluate all expressions
        for def in &mut ir.project.root_module.definitions {
            match &mut def.kind {
                DefinitionKind::Function(func) => {
                    let bk = evaluate_block(func.body.clone());
                    // assert_eq!(func.return_ty, bk.ret_ty.unwrap_or(Type::void()));
                    todo!()
                }
                _ => {}
            }
        }

        return ir;
    }
}

pub fn desugar_block(exp: &mut Block) {
    for stmt in &mut exp.statements {
        match stmt {
            Statement::Expression(exp) => desugar_expression(exp),
            _ => {}
        }
    }
}
pub fn desugar_expression(exp: &mut Expression) {
    match &mut *exp.kind {
        ExpressionKind::Block(blk) => desugar_block(blk),
        ExpressionKind::While(wloop) => {
            let negated_condition =
                Expression::unary_operation(UnaryOperator::Negation, wloop.condition.clone());

            let mut break_block = Block::new();
            break_block.add_statement(Statement::Break);
            let oposite_if = Conditional::new(negated_condition, Expression::block(break_block));

            let mut body = Block::new();
            body.add_statement(Statement::Expression(Expression::if_(oposite_if)));
            body.add_statement(Statement::Expression(wloop.then.clone()));

            let lp = LoopExpression::new(Expression::block(body));

            *exp = Expression::loop_(lp);
            desugar_expression(exp);
        }
        ExpressionKind::BinaryOperation(oper) => {
            let mut operan_path = Path::new();
            operan_path.add_segment("core");
            operan_path.add_segment("op");
            let (tr, func) = match &oper.operator {
                BinaryOperator::Addition => ("Add", "add"),
                BinaryOperator::Lesser => ("Less", "less"),
                oper => todo!("{oper:?}"),
            };

            operan_path.add_segment(tr);
            operan_path.add_segment(func);

            *exp = Expression::call(
                Expression::identifier(operan_path),
                vec![oper.operands[0].clone(), oper.operands[1].clone()],
            );
        }

        ExpressionKind::UnaryOperation(oper) => {
            let mut operan_path = Path::new();
            operan_path.add_segment("core");
            operan_path.add_segment("op");
            let (tr, func) = match oper.operator {
                UnaryOperator::Negation => ("Neg", "neg"),
                _ => todo!(),
            };

            operan_path.add_segment(tr);
            operan_path.add_segment(func);

            *exp = Expression::call(
                Expression::identifier(operan_path),
                vec![oper.operand.clone()],
            );
        }
        ExpressionKind::If(if_) => {
            desugar_expression(&mut if_.condition);
            desugar_expression(&mut if_.then);
        }

        ExpressionKind::Call(call) => {
            desugar_expression(&mut call.called);
            for param in &mut call.parameters {
                desugar_expression(param);
            }
        }

        _ => {}
    };
}

pub fn desugar_module(md: &mut Module) {
    for def in &mut md.definitions {
        match &mut def.kind {
            DefinitionKind::Function(func) => desugar_block(&mut func.body),
            _ => {}
        }
    }
}

/* determine return type for conditional expression */
pub fn evaluate_conditional(mut condition: Conditional) -> Expression {
    condition.condition = evaluate_expression(condition.condition);
    condition.then = evaluate_expression(condition.then);
    let ret_ty = Some(condition.then.ret_ty.as_ref().unwrap().clone());

    return Expression {
        kind: Box::new(ExpressionKind::If(condition)),
        ret_ty,
    };
}

/* determine return type for block expression */
pub fn evaluate_block(mut block: Block) -> Expression {
    if let Some(s) = block.statements.pop() {
        match s {
            Statement::Expression(e) => {
                let e = evaluate_expression(e);
                let ret_ty = e.ret_ty.clone();

                block.statements.push(Statement::Expression(e));
                return Expression {
                    kind: Box::new(ExpressionKind::Block(block)),
                    ret_ty,
                };
            }
            _ => {
                return Expression {
                    kind: Box::new(ExpressionKind::Block(block)),
                    ret_ty: None,
                };
            }
        }
    } else {
        return Expression {
            kind: Box::new(ExpressionKind::Block(block)),
            ret_ty: None,
        };
    }
}

/* determine return type for any valid expression */
fn evaluate_expression(expression: Expression) -> Expression {
    let new_expr = match *expression.kind {
        ExpressionKind::Block(block) => evaluate_block(block),
        ExpressionKind::If(condition) => evaluate_conditional(condition),
        ExpressionKind::Loop(_) => {
            todo!()
        }
        ExpressionKind::Literal(_) => {
            todo!()
        }
        ExpressionKind::Identifier(_) => {
            todo!()
        }
        ExpressionKind::Call(_) => {
            todo!()
        }
        ExpressionKind::Return(_) => {
            todo!()
        }
        // expressions should be converted into their "non sintax sugar" equivalents
        _ => unreachable!(),
    };

    assert!(new_expr.ret_ty.is_some());

    return new_expr;
}
