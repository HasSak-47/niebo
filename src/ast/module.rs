use crate::ast::DefinitionKind;

use super::{
    Definition, FunctionBuilder, Implementation, Import, QualifiedName, TraitBuilder,
    TraitImplementation,
};

#[derive(Debug, Clone, PartialEq)]
pub enum ModuleKind {
    InFile,
    ExFile,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Module {
    pub kind: ModuleKind,
    pub imports: Vec<Import>,
    pub definitions: Vec<Definition>,
    pub impls: Vec<Implementation>,
    pub trait_impls: Vec<TraitImplementation>,
}

impl Module {
    pub fn new() -> Self {
        return Self {
            kind: ModuleKind::InFile,
            trait_impls: vec![],
            impls: vec![],
            imports: vec![],
            definitions: vec![],
        };
    }

    pub fn add_trait(&mut self, t: TraitBuilder) {
        self.definitions.push(t.build_def());
    }

    pub fn add_function(&mut self, f: FunctionBuilder) {
        self.definitions.push(f.build_def());
    }

    pub fn add_c_import<P: Into<QualifiedName>>(&mut self, path: P) {
        let path = path.into();
        // everything in c has the format header::name
        assert!(path.v.len() == 2);

        self.imports.push(Import {
            c_import: true,
            path,
        });
    }

    pub fn add_import(&mut self, path: QualifiedName) {
        self.imports.push(Import {
            c_import: false,
            path,
        });
    }

    pub fn get_c_imports(&self) -> Vec<QualifiedName> {
        let mut v = Vec::new();
        for import in &self.imports {
            if import.c_import {
                v.push(import.path.clone());
            }
        }

        for def in &self.definitions {
            if let DefinitionKind::Module(md) = &def.kind {
                let mut other = md.get_c_imports();
                v.append(&mut other);
            }
        }

        return v;
    }
}
