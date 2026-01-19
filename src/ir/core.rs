use std::collections::HashMap;

use crate::ast::{Definition, DefinitionKind, Module, project::Project, traits::Trait};

fn create_operation_module() -> Module {
    let mut definitions = Vec::new();
    definitions.push(Definition {
        name: "Add".to_string(),
        visibility: crate::ast::Visibility::Public,
        kind: DefinitionKind::Trait(Trait {
            functions: HashMap::new(),
        }),
    });

    definitions.push(Definition {
        name: "Mul".to_string(),
        visibility: crate::ast::Visibility::Public,
        kind: DefinitionKind::Trait(Trait {
            functions: HashMap::new(),
        }),
    });

    definitions.push(Definition {
        name: "Sub".to_string(),
        visibility: crate::ast::Visibility::Public,
        kind: DefinitionKind::Trait(Trait {
            functions: HashMap::new(),
        }),
    });

    definitions.push(Definition {
        name: "Div".to_string(),
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

pub fn create_core_project() -> Project {
    let mut definitions = Vec::new();
    definitions.push(Definition {
        name: "op".to_string(),
        visibility: crate::ast::Visibility::Public,
        kind: DefinitionKind::Module(create_operation_module()),
    });
    let mut core = Project::new("core", (0, 1, 0));
    core.root_module = Module {
        imports: vec![],
        definitions,
    };

    return core;
}
