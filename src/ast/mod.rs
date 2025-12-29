pub mod expressions;
pub mod function;
pub mod traits;
pub mod types;

use anyhow::{Result, anyhow};

use function::*;
use types::*;

use expressions::{
    Expression,
    operations::{BinaryOperation, UnaryOperation},
};

use traits::{Trait, TraitBuilder};

#[derive(Debug, Clone)]
pub struct Path {
    v: Vec<String>,
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
    kind: DefinitionKind,
    visibility: Visibility,
    name: String,
}

impl Definition {
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
}

#[derive(Debug, Clone)]
pub struct Module {
    pub imports: Vec<Path>,
    pub definitions: Vec<Definition>,
}

impl Module {
    pub fn add_trait(&mut self, t: TraitBuilder) {
        self.definitions.push(t.build_def());
    }
    pub fn add_function(&mut self, f: FunctionBuilder) {
        self.definitions.push(f.build_def());
    }
}

#[derive(Debug, Clone)]
pub struct Registry {}

#[derive(Debug, Clone)]
pub struct Project {
    pub root_module: Module,
    pub registry: Registry,
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
            registry: Registry {},
            name: name.into(),
            version,
        };
    }
}
