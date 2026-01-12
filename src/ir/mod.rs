use crate::ast::expressions::Statement;
use crate::ast::traits::Trait;
use crate::ast::{DefinitionKind, Module, Project};

pub mod core;

pub struct IR<'a> {
    project: Project,
    traits: Vec<(Path, &'a Trait)>,
}

fn find_traits<'a>(cur_path: Path, traits: &mut Vec<(Path, &'a Trait)>, module: &'a Module) {
    for def in &module.definitions {
        match &def.kind {
            DefinitionKind::Trait(t) => {
                let mut path = cur_path.clone();
                path.v.push(def.name.clone());
                traits.push((path, t));
            }
            DefinitionKind::Module(m) => {
                let mut path = cur_path.clone();
                path.v.push(def.name.clone());

                find_traits(path, traits, &m);
            }
            _ => {}
        }
    }
}

impl<'a> IR<'a> {
    pub fn from_project(mut p: Project) -> Self {
        p.external_modules
            .insert("core".to_string(), create_core_module());

        let mut traits = Vec::new();
        let path = Path::from(p.name.clone());

        let ir = Self {
            project: p,
            traits: Vec::new(),
        };
        find_traits(path, &mut traits, &ir.project.root_module);
        for (name, module) in &ir.project.external_modules {
            find_traits(Path::from(name), &mut traits, module);
        }

        return ir;
    }
}

use crate::ir::core::create_core_module;
use crate::{
    ast::{
        Path,
        expressions::{Expression, ExpressionKind, block::Block, conditional::Conditional},
    },
    general::types::Type,
};

pub fn evaluate_conditional(mut condition: Conditional) -> Expression {
    condition.condition = evaluate_expression(condition.condition);
    condition.then = evaluate_expression(condition.then);
    let ret_ty = Some(condition.then.ret_ty.as_ref().unwrap().clone());

    return Expression {
        kind: Box::new(ExpressionKind::If(condition)),
        ret_ty,
    };
}

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
                    ret_ty: Some(Type::void()),
                };
            }
        }
    } else {
        return Expression {
            kind: Box::new(ExpressionKind::Block(block)),
            ret_ty: Some(Type::void()),
        };
    }
}

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
