pub mod expressions;
pub mod function;
pub mod module;
pub mod project;
pub mod traits;

use module::*;
use std::collections::HashMap;

use anyhow::{Result, anyhow};

use crate::general::{
    path::{Path, PathIdent},
    types::*,
};
use function::*;

use expressions::Expression;

use traits::{Trait, TraitBuilder};

// WARN: bad default!
#[derive(Debug, Default, Clone)]
pub enum Visibility {
    Private,
    Module,
    #[default]
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
    pub target: Path,
    pub definitions: Vec<Definition>,
}

#[derive(Debug, Clone)]
pub struct TraitImplementation {
    pub trait_path: Path,
    pub target: Path,
    pub definitions: Vec<Definition>,
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
into_definition!(Implementation);
into_definition!(TraitImplementation);

#[derive(Debug, Clone)]
pub enum DefinitionKind {
    Variable(Variable),
    Type(Type),
    TypeAlias(Path),
    Function(Function),
    Module(Module),
    Trait(Trait),
    Implementation(Implementation),
    TraitImplementation(TraitImplementation),

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
    pub fn module<S: Into<String>>(ident: S, module: Module) -> Self {
        return Self {
            kind: DefinitionKind::Module(module),
            name: ident.into(),
            visibility: Visibility::Public,
        };
    }
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
            visibility: Visibility::Public,
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
            visibility: Visibility::Public,
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
