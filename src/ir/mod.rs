use std::{collections::HashMap, fmt::Debug};

use crate::{
    ast,
    general::{
        path::{Path, PathIdent},
        types::Type,
    },
};

#[derive(Debug)]
pub struct Registry<T> {
    pub entries: HashMap<PathIdent, T>,
}

struct FunctionDef {
    local_registry: Resolver,
    stmts: Vec<ast::expressions::Statement>,
    return_ty: Type,
}

impl FunctionDef {
    fn new(func: ast::function::Function) -> anyhow::Result<Self> {
        let expected_ret = func.return_ty;
        for stmt in func.body.statements {}
        todo!()
    }
}

#[derive(Debug)]
pub struct Resolver {
    pub ty_reg: Registry<Type>,
}

impl Default for Resolver {
    fn default() -> Self {
        Self {
            ty_reg: Registry {
                entries: HashMap::new(),
            },
        }
    }
}
