use super::*;
use std::collections::HashMap;

pub struct TraitBuilder {
    ident: String,
    functions: HashMap<String, Function>,
}

impl TraitBuilder {
    fn new<S: Into<String>>(ident: S) -> Self {
        let ident = ident.into();
        return Self {
            ident,
            functions: HashMap::new(),
        };
    }
}

#[derive(Debug, Clone)]
pub struct Trait {
    functions: HashMap<String, Function>,
}
