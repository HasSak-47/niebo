pub mod expressions;
pub mod function;
pub mod traits;
pub mod typing;

use std::collections::HashMap;

use anyhow::{Result, anyhow};

use crate::general::types::*;
use function::*;

use expressions::Expression;

use traits::{Trait, TraitBuilder};

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct PathIdent {
    pub ident: String,
    pub template_spec: Vec<Path>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Path {
    pub v: Vec<PathIdent>,
}

impl<I> From<I> for PathIdent
where
    I: Into<String>,
{
    fn from(value: I) -> Self {
        let ident = value.into();
        return PathIdent {
            ident,
            template_spec: vec![],
        };
    }
}

impl Path {
    pub fn add_segment<I: Into<PathIdent>>(&mut self, s: I) {
        let ident = s.into();
        self.v.push(ident);
    }

    pub fn new() -> Self {
        Self { v: vec![] }
    }
}

impl<T> From<T> for Path
where
    T: Into<PathIdent>,
{
    fn from(value: T) -> Self {
        let s = value.into();
        return Self { v: vec![s] };
    }
}

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
    pub fn variable<S: Into<String>>(
        ident: S,
        value: Expression,
        ty: Option<Type>,
    ) -> Result<Definition> {
        return Ok(Self {
            kind: DefinitionKind::Variable(Variable {
                mutable: false,
                value: value,
                ty: ty,
            }),
            visibility: Visibility::Private,
            name: ident.into(),
        });
    }

    pub fn variable_with_mut<S: Into<String>>(
        ident: S,
        value: Expression,
        ty: Option<Type>,
        mutable: bool,
    ) -> Result<Definition> {
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
}
