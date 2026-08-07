use std::{
    collections::{HashMap, HashSet},
    fmt::Debug,
};

use anyhow::Result;
use clang::Clang;

use crate::{
    ast::{
        Definition, DefinitionKind,
        expressions::{Statement, block::Block},
        project::Project,
    },
    general::{naming::QualifiedName, types::Type},
};

fn simplify_expression(block: &mut Block) {
    for mut stmt in &mut block.statements {
        match &mut stmt {
            Statement::Definition(func) => {}
            Statement::Expression(func) => {}
            Statement::Value(func) => {}
            _ => todo!(),
        }
    }
}

fn simplify_block(block: &mut Block) {
    for mut stmt in &mut block.statements {
        match &mut stmt {
            Statement::Definition(func) => {}
            Statement::Expression(func) => {}
            Statement::Value(func) => {}
            _ => todo!(),
        }
    }
}

pub fn simplify_project(project: &mut Project) {
    for def in &mut project.root_module.definitions {
        match &mut def.kind {
            DefinitionKind::Function(func) => {}
            _ => todo!(),
        }
    }
}
