pub mod validator;

use crate::ast::{DefinitionKind, project::Project};

pub fn simple_desugar(p: Project) -> Project {
    for def in p.root_module.definitions {
        match def.kind {
            DefinitionKind::FunctionDefinition(func) => {
                for stmt in func.body.statements{
                    match stmt {
                        crate::ast::statements::Statement::Expression(exp) => 
                            _ => 
                    }
                }},
            _ => {}
        }
    }
    todo!();
}
