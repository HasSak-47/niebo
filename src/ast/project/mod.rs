use std::{collections::HashMap, fmt::Debug};

use anyhow::{Result, anyhow};

use crate::{
    ast::{Definition, DefinitionKind, Module, expressions::Statement, typing::TypeName},
    general::types::*,
};

#[derive(Debug, Clone)]
pub struct Project {
    pub root_module: Module,
    pub external_modules: HashMap<String, Module>,
    pub name: String,
    pub version: (usize, usize, usize),
}

impl Project {
    pub fn new<S: Into<String>>(name: S, version: (usize, usize, usize)) -> Self {
        return Self {
            root_module: Module {
                imports: vec![],
                definitions: vec![],
            },
            external_modules: HashMap::new(),
            name: name.into(),
            version,
        };
    }

    // NOTE: ommit templates for now do to complexity
    pub fn generate_ir(&mut self) -> Result<()> {
        // todo
        // - generate a registry to determine what is each Identifier/Path
        // - determine type of all variables
        // for example "let i = 10;" has no type in the AST but it's type should be i32
        // and the type of "i" should be i32 and the 10? should collapse into a 10i32
        // getting the statement "let i: i32 = 10i32;"
        // - make sure that the path's taken are indeed valid objects
        // for example:
        // type TypeAlias = i32;
        //
        // fn foo(){
        //     let var = TypeAlias;
        // }
        //
        // TypeAlias is a valid Path but not an expression so it get's discarted
        // - for each expression get determine it's return type
        // - make sure that if something returns that it returns the same type

        return Ok(());
    }
}
