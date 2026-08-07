use std::collections::HashMap;

use crate::{
    ast::{Definition, DefinitionKind, expressions::Statement},
    general::types::Type,
};

enum SymbolKind {
    Variable,
    Function,
    Type,
}

struct Symbol {
    kind: SymbolKind,
}

#[derive(Debug)]
enum IR {
    VariableInit {
        ty: Type,
        name: String,
    },

    VariableSet {
        name: String,
        member: Vec<String>,
    },
    BlockLabel {
        name: String,
    },

    Jump {
        label: String,
    },

    Call {
        name: String,
    },

    Return {
        value: String,
    },

    VariableAccess {
        name: String,
    },

    FunctionDefinition {
        params: Vec<(String, Type)>,
        instructions: Vec<IR>,
    },
}

struct Scope {
    global: HashMap<String, Symbol>,
    local: Vec<HashMap<String, Symbol>>,
}

struct IRGen {
    scope: Scope,
    ir: Vec<IR>,
}

impl IRGen {
    fn from_definition(&mut self, irgen: IRGen, def: &Definition) {
        match &def.kind {
            DefinitionKind::Variable(var) => {
                self.ir.push(IR::VariableInit {
                    ty: var.ty.clone().unwrap(),
                    name: def.name.clone(),
                });
            }
            _ => todo!(),
        }
    }

    fn from_statement(&mut self, irgen: IRGen, stmt: &Statement) {
        match stmt {
            Statement::Definition(def) => {}
            _ => todo!(),
        }
    }
}
