pub mod expressions;
pub mod function;
pub mod project;
pub mod traits;

use std::collections::HashMap;

use anyhow::{Result, anyhow};

use crate::general::{path::{Path, PathIdent}, types::*};
use function::*;

use expressions::Expression;

use traits::{Trait, TraitBuilder};

#[derive(Debug, Clone)]
pub enum Visibility {
    Private,
    Module,
    Public,
}

#[derive(Debug, Clone)]
pub struct Variable {
    mutable: bool,
    value: Expression,
    ty: Option<Type>,
}

#[derive(Debug, Clone)]
pub struct Implementation {
    inner_definitions: Vec<Definition>,
}

macro_rules! into_definition {
    ($ty: tt) => {
        impl From<$ty> for DefinitionKind {
            fn from(value: $ty) -> Self {
                Self::$ty(value)
            }
        }
    };
}

into_definition!(Function);
into_definition!(Variable);
into_definition!(Module);
into_definition!(Trait);

#[derive(Debug, Clone)]
pub enum DefinitionKind {
    Variable(Variable),
    Type(Type),
    TypeAlias(Path),
    Function(Function),
    Module(Module),
    Trait(Trait),

    // hidden definitions used for c symbol resolution
    FunctionC(FunctionC),
    VarC(Variable),
    // MacroC(Variable),
}

#[derive(Debug, Clone)]
pub struct Definition {
    pub kind: DefinitionKind,
    pub visibility: Visibility,
    pub name: String,
}

impl Definition {
    pub fn type_def<S: Into<String>>(ident: S, path: Path, visibility: Visibility) -> Self {
        return Self {
            name: ident.into(),
            kind: DefinitionKind::TypeAlias(path),
            visibility,
        };
    }
    pub fn variable<S: Into<String>, T: Into<Type>>(
        ident: S,
        value: Expression,
        mutable: bool,
        ty: Option<T>,
    ) -> Result<Definition> {
        return Ok(Self {
            kind: DefinitionKind::Variable(Variable {
                mutable: mutable,
                value: value,
                ty: ty.and_then(|k| Some(k.into())),
            }),
            visibility: Visibility::Private,
            name: ident.into(),
        });
    }

    pub fn variable_with_mut<S: Into<String>, T: Into<Type>>(
        ident: S,
        value: Expression,
        ty: Option<T>,
        mutable: bool,
    ) -> Result<Definition> {
        let ty = ty.and_then(|t| Some(t.into()));

        return Ok(Self {
            kind: DefinitionKind::Variable(Variable { mutable, value, ty }),
            visibility: Visibility::Private,
            name: ident.into(),
        });
    }
}

#[derive(Debug, Default, Clone)]
pub struct Import {
    pub c_import: bool,
    pub path: Path,
}

impl Import {
    pub fn c_import(path: Path) -> Self {
        // c imports only have 2 path members
        assert!(path.v.len() == 2);
        return Self {
            c_import: true,
            path,
        };
    }
}

#[derive(Debug, Clone)]
pub struct Module {
    pub imports: Vec<Import>,
    pub definitions: Vec<Definition>,
}

impl Module {
    pub fn new() -> Self {
        return Self {
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

    pub fn add_c_import<P: Into<Path>>(&mut self, path: P) {
        let path = path.into();
        // everything in c has the format header::name
        assert!(path.v.len() == 2);

        self.imports.push(Import {
            c_import: true,
            path,
        });
    }

    pub fn add_import(&mut self, path: Path) {
        self.imports.push(Import {
            c_import: false,
            path,
        });
    }
}
