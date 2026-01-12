use super::*;
use std::collections::HashMap;

pub struct TraitBuilder {
    ident: String,
    visibility: Visibility,
    functions: HashMap<String, Function>,
}

impl TraitBuilder {
    pub fn new<S: Into<String>>(ident: S) -> Self {
        let ident = ident.into();
        return Self {
            ident,
            visibility: Visibility::Private,
            functions: HashMap::new(),
        };
    }

    pub fn build_def(self) -> Definition {
        Definition {
            kind: DefinitionKind::Trait(Trait {
                functions: self.functions,
            }),
            visibility: self.visibility,
            name: self.ident,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Trait {
    pub functions: HashMap<String, Function>,
}
