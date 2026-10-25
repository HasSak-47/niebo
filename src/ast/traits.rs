use super::*;
use std::collections::HashMap;

pub struct TraitBuilder {
    ident: String,
    visibility: Visibility,
    functions: HashMap<String, FunctionDeclaration>,
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

/**
 * simple trait definition
```niebo
inter Foo {
    // allowed
    fn bar(...) -> TypeBar;
    fn baz(...) -> TypeBaz;
    ...
    // disallowed
    // shit to annoying to handle
    fn foo(...) -> TypeFoo {
        ...
    }
```
}

 */
#[derive(Debug, Clone, PartialEq)]
pub struct Trait {
    pub functions: HashMap<String, FunctionDeclaration>,
}
