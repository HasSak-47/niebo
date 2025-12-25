pub mod expressions;
pub mod function;
pub mod types;

use anyhow::{Result, anyhow};

use function::*;
use types::*;

use expressions::{
    ExpressionTrait,
    operations::{BinaryOperation, UnaryOperation},
};

#[derive(Debug, Clone)]
pub struct Path {
    v: Vec<String>,
}

#[derive(Debug, Clone)]
enum Statement {
    // DefinitionKind::Module and DefinitionKind::Trait not allowed
    Definition(Definition),
    Expression(Expression),
    Use(Path),
}

#[derive(Debug, Clone)]
pub enum Literal {
    Integer {
        signed: Option<bool>,
        precision: Option<u64>,
        negative: bool,
        value: u64,
    },

    Float {
        precision: Option<u64>,
        value: f64,
    },
}

#[derive(Debug, Clone)]
pub struct Block {
    pub statements: Vec<Statement>,
}

impl Block {
    // TODO: validate statement
    pub fn add_statement(&mut self, stmt: Statement) {
        self.statements.push(stmt);
    }

    pub fn new() -> Self {
        return Self { statements: vec![] };
    }
}

#[derive(Debug, Clone)]
pub enum ExpressionKind {
    Block(Block),
    If {
        condition: Expression,
        then: Expression,
        else_: Option<Expression>,
    },
    While {
        condition: Expression,
        then: Expression,
    },

    Literal(Literal),
    Identifier(Path),
    BinaryOperation(BinaryOperation),
    UnaryOperation(UnaryOperation),
    Call {
        called: Expression,
        parameters: Vec<Expression>,
    },
    Return(Expression),
}

#[derive(Debug, Clone)]
pub struct Expression {
    pub kind: Box<ExpressionKind>,
}

impl ExpressionTrait for Expression {
    fn get_return_type(&self) -> Type {
        todo!()
    }

    fn resolve_and_validate(&mut self) -> Result<()> {
        use ExpressionKind as EK;
        match &mut *self.kind {
            EK::BinaryOperation(oper) => oper.resolve_and_validate(),
            _ => todo!(),
        }
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
    ty: Type,
}

#[derive(Debug, Clone)]
pub struct Trait {
    // DefinitionKind::Module and DefinitionKind::Trait not allowed
    inner_definitions: Vec<Definition>,
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
        let ty = if let Some(ty) = ty {
            if value.get_return_type() != ty {
                return Err(anyhow!("ty and expression return are of diff types"));
            }
            ty
        } else {
            value.get_return_type()
        };

        return Ok(Self {
            kind: DefinitionKind::Variable(Variable {
                mutable: false,
                value: value,
                ty,
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
    pub fn add_function<S: Into<String>>(&mut self, name: S, f: Function) {
        let d = Definition {
            kind: f.into(),
            name: name.into(),
            visibility: Visibility::Public,
        };

        self.definitions.push(d);
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
