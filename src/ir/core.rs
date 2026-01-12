use std::collections::HashMap;

use crate::ast::{Definition, DefinitionKind, Module, traits::Trait};

fn create_operation_module() -> Module {
    let mut definitions = Vec::new();
    definitions.push(Definition {
        name: "Add".to_string(),
        visibility: crate::ast::Visibility::Public,
        kind: DefinitionKind::Trait(Trait {
            functions: HashMap::new(),
        }),
    });
    let m = Module {
        imports: vec![],
        definitions,
    };

    return m;
}

pub fn create_core_module() -> Module {
    let mut definitions = Vec::new();
    definitions.push(Definition {
        name: "op".to_string(),
        visibility: crate::ast::Visibility::Public,
        kind: DefinitionKind::Module(create_operation_module()),
    });
    let m = Module {
        imports: vec![],
        definitions,
    };

    return m;
}
