use std::{collections::HashMap, fmt::Debug, fs::File, io::Read};

use anyhow::{Result, bail};

use crate::{
    ast::{
        Definition, DefinitionKind, Module, ModuleKind,
        expressions::{
            loops::{LoopExpression, WhileLoop},
            operations::BinaryOperation,
        },
    },
    general::{path::Path, types::Type},
    parser::parse_module,
};

mod chmura {
    use serde::Deserialize;

    #[derive(Deserialize)]
    #[serde(rename_all = "snake_case")]
    pub enum LibKinds {
        NieboLib,
        LibC,
    }

    #[derive(Deserialize)]
    pub struct Lib {
        pub project_type: LibKinds,
    }

    #[derive(Deserialize)]
    pub struct Project {
        pub name: String,
        pub version: String,
        pub edition: String,
    }

    #[derive(Deserialize)]
    pub struct Chmura {
        pub project: Project,
        pub lib: Option<Lib>,
    }
}

#[derive(Debug, Clone)]
pub struct Project {
    pub root_module: Module,
    pub external_projects: HashMap<String, Project>,
    pub name: String,
    pub version: (usize, usize, usize),
    // edition of the compiler
    pub edition: (usize, usize, usize),
}

impl Project {
    #[must_use]
    fn load_modules<P: AsRef<std::path::Path>>(module: &mut Module, cur_path: P) -> Result<()> {
        let cur_path = cur_path.as_ref();
        for def in &mut module.definitions {
            if let Definition {
                kind: DefinitionKind::Module(md),
                name,
                ..
            } = def
            {
                if md.kind == ModuleKind::InFile {
                    continue;
                }
                let mut path = cur_path.to_path_buf();
                path.push(name);
                path.set_extension("nb");

                println!("loading module: {}", path.display());
                let mut file = File::open(path)?;
                let mut buffer = String::new();
                file.read_to_string(&mut buffer)?;

                *md = parse_module(buffer)?;
                println!("module: {md:?}");
                Self::load_modules(md, cur_path)?
            }
        }

        return Ok(());
    }
    pub fn load_script<P: AsRef<std::path::Path>>(path: P) -> Result<Self> {
        let mut buffer = String::new();
        let mut file = File::open(path)?;
        file.read_to_string(&mut buffer)?;

        let root = parse_module(buffer)?;

        return Ok(Project {
            root_module: root,
            external_projects: HashMap::new(),
            name: "".into(),
            version: (0, 1, 0),
            edition: (0, 1, 0),
        });
    }
    pub fn load<P: AsRef<std::path::Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        if !path.is_dir() {
            bail!("Niebo: Path is not a directory");
        }
        let mut chmura_path = path.to_path_buf();
        chmura_path.push("chmura");
        chmura_path.set_extension("toml");
        let mut file = File::open(chmura_path)?;

        let mut buffer = String::new();
        file.read_to_string(&mut buffer)?;

        let chmura: chmura::Chmura = toml::from_str(buffer.as_str())?;
        let src_path = {
            let mut b = path.to_path_buf();
            b.push("src");
            b
        };

        let mut entry_path = src_path.clone();
        entry_path.push(if chmura.lib.is_some() { "lib" } else { "main" });
        entry_path.set_extension("nb");

        let mut file = File::open(entry_path)?;
        let mut buffer = String::new();
        file.read_to_string(&mut buffer)?;

        let mut root = parse_module(buffer)?;

        Self::load_modules(&mut root, src_path)?;
        return Ok(Project {
            root_module: root,
            external_projects: HashMap::new(),
            name: chmura.project.name,
            version: (0, 1, 0),
            edition: (0, 1, 0),
        });
    }

    pub fn add_external_project<S: Into<String>>(&mut self, name: S, project: Project) {
        self.external_projects.insert(name.into(), project);
    }
}

#[derive(Debug, Clone)]
enum Symbol {
    Variable(Type),
    Type(Type),
    Function { ret_ty: Type, params: Vec<Type> },
}

#[derive(Debug, Default)]
pub struct ProjectPreprocessor {
    local_scope: Vec<HashMap<Path, Symbol>>,
    global_scope: HashMap<Path, Symbol>,
}

trait ExpressionValidator {
    fn validate(&mut self, procesor: &mut ProjectPreprocessor) -> anyhow::Result<()>;
    fn resolve_ret_ty(&mut self, procesor: &mut ProjectPreprocessor) -> anyhow::Result<Type>;
}

impl ExpressionValidator for crate::ast::Expression {
    fn validate(&mut self, procesor: &mut ProjectPreprocessor) -> anyhow::Result<()> {
        use crate::ast::expressions::ExpressionKind;

        match self.kind.as_mut() {
            ExpressionKind::BinaryOperation(b_exp) => b_exp.validate(procesor),

            _ => todo!(),
        }
    }

    fn resolve_ret_ty(&mut self, procesor: &mut ProjectPreprocessor) -> anyhow::Result<Type> {
        use crate::ast::expressions::ExpressionKind;

        return match self.kind.as_mut() {
            ExpressionKind::BinaryOperation(b_exp) => b_exp.resolve_ret_ty(procesor),
            _ => todo!(),
        };
    }
}

impl ExpressionValidator for crate::ast::expressions::block::Block {
    fn validate(&mut self, procesor: &mut ProjectPreprocessor) -> anyhow::Result<()> {
        use crate::ast::{Definition, DefinitionKind, expressions::Statement};

        procesor.push_scope();
        for stmt in &self.statements {
            match stmt {
                Statement::Definition(Definition { kind, name, .. }) => match kind {
                    DefinitionKind::Function(func) => {
                        procesor.register_local_symbol(
                            name.into(),
                            Symbol::Function {
                                ret_ty: func.return_ty.clone().unwrap(),
                                params: func.parameters.iter().map(|a| a.1.clone()).collect(),
                            },
                        );
                    }
                    DefinitionKind::Variable(var) => {
                        procesor.register_local_symbol(
                            name.into(),
                            Symbol::Variable(var.ty.clone().unwrap()),
                        );
                    }
                    _ => todo!(),
                },
                _ => todo!(),
            }
        }
        todo!()
    }

    fn resolve_ret_ty(&mut self, procesor: &mut ProjectPreprocessor) -> anyhow::Result<Type> {
        todo!()
    }
}

impl ExpressionValidator for BinaryOperation {
    fn validate(&mut self, procesor: &mut ProjectPreprocessor) -> anyhow::Result<()> {
        self.operands[0].validate(procesor)?;
        self.operands[1].validate(procesor)?;

        let a_ty = self.operands[0].resolve_ret_ty(procesor)?;
        let b_ty = self.operands[1].resolve_ret_ty(procesor)?;

        if a_ty != b_ty {
            anyhow::bail!("{a_ty:?} and {b_ty:?} are not the same type");
        }
        return Ok(());
    }

    fn resolve_ret_ty(&mut self, procesor: &mut ProjectPreprocessor) -> anyhow::Result<Type> {
        use crate::ast::expressions::operations::BinaryOperator;
        self.validate(procesor)?;
        return Ok(match self.operator {
            BinaryOperator::Greater
            | BinaryOperator::Lesser
            | BinaryOperator::GreaterOrEqual
            | BinaryOperator::LesserOrEqual
            | BinaryOperator::Equal
            | BinaryOperator::NotEqual => Type::bool(),
            _ => self.operands[0].ret_ty.clone().unwrap(),
        });
    }
}

impl ProjectPreprocessor {
    fn push_scope(&mut self) {
        self.local_scope.push(HashMap::new());
    }

    fn pop_scope(&mut self) {
        self.local_scope.pop();
    }

    fn register_local_symbol(&mut self, path: Path, kind: Symbol) {
        self.local_scope.last_mut().unwrap().insert(path, kind);
    }

    fn register_global_symbol(&self, path: Path, kind: Symbol) -> Option<Symbol> {
        for (s_path, s_kind) in &self.global_scope {
            if *s_path == path {
                return Some(s_kind.clone());
            }
        }

        return None;
    }

    fn find_symbol(&self, path: Path) -> Option<Symbol> {
        for (s_path, s_kind) in &self.global_scope {
            if *s_path == path {
                return Some(s_kind.clone());
            }
        }

        for local_scope in self.local_scope.iter().rev() {
            for (s_path, s_kind) in local_scope {
                if *s_path == path {
                    return Some(s_kind.clone());
                }
            }
        }

        return None;
    }
}

impl ProjectPreprocessor {
    // NOTE: ommit templates for now do to complexity
    pub fn process_project(&mut self, project: Project) -> Result<Project> {
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
        // - convert operations into their equivalent core::op::OP

        // convert each path identifier/path into it's full path
        // loading module imports
        // let mut res = Resolver::default();

        return Ok(project);
    }
}
