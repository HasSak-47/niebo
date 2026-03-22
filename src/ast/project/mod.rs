mod cimports;

use std::{
    collections::HashMap,
    fmt::Debug,
    fs::File,
    io::{BufReader, Read},
    path::PathBuf,
};

use anyhow::{Result, anyhow, bail};
use clang::{Clang, Index, TranslationUnit};
use serde::Deserialize;

use crate::{
    ast::{
        Definition, DefinitionKind, Module, ModuleKind, Variable,
        expressions::Statement,
        function::{Function, FunctionBuilder, FunctionC},
        project::cimports::CCache,
    },
    general::{path::Path, types::*},
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

    fn get_non_local_definition_module<P1, P2>(md: &Module, cur_path: P1, name: P2) -> &Definition
    where
        P1: AsRef<Path>,
        P2: AsRef<Path>,
    {
        let name = name.as_ref();
        let cur_path = cur_path.as_ref();
        let root = name.get(0);
        if root.is_template() {
            todo!()
        }

        for def in &md.definitions {
            println!("{def:?}");
            if def.name == root.ident {
                match &def.kind {
                    DefinitionKind::Module(md) => {
                        let mut new_path = cur_path.clone();
                        new_path.pop_front();
                        let mut new_name = name.clone();
                        new_name.pop_front();
                        Project::get_non_local_definition_module(&md, new_path, new_name);
                    }
                    _ => return def,
                }
            }
        }

        unreachable!();
    }

    pub fn get_non_local_definition<P1, P2>(&self, cur_path: P1, name: P2) -> &Definition
    where
        P1: AsRef<Path>,
        P2: AsRef<Path>,
    {
        let name = name.as_ref();
        let root = name.get(0);
        if root.is_template() {
            todo!()
        }

        if self.external_projects.contains_key(&root.ident) {
            let p = &self.external_projects[&root.ident];
            let mut new_name = name.clone();
            new_name.pop_front();

            return p.get_non_local_definition(cur_path, new_name);
        }

        return Project::get_non_local_definition_module(&self.root_module, cur_path, name);
    }
}

#[derive(Debug)]
enum Symbol {
    FunctionC(FunctionC),
    Function(Function),
    Variable(Variable),
}

#[derive(Debug, Default)]
struct Scope {
    symbols: HashMap<Path, Symbol>,
    types: HashMap<Path, Type>,
}

#[derive(Debug, Default)]
pub struct IRGenerator {
    global: Scope,

    scope: Vec<Scope>,
}

impl IRGenerator {
    fn get_symbol(&self, path: &Path) -> Result<&Symbol> {
        if self.global.symbols.contains_key(path) {
            return Ok(&self.global.symbols[path]);
        }

        for scope in self.scope.iter().rev() {
            if scope.symbols.contains_key(path) {
                return Ok(&scope.symbols[path]);
            }
        }

        bail!("not found");
    }

    // NOTE: ommit templates for now do to complexity
    pub fn generate_ir(&mut self, mut project: Project) -> Result<()> {
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
        let c_imports = project.root_module.get_c_imports();

        let mut import_registry = HashMap::<Path, &Definition>::new();
        // TODO: load libs to clang
        let clang = Clang::new().map_err(|s| anyhow!("clang: {s}"))?;
        let mut ccache = CCache::new(&clang)?;

        for import in &project.root_module.imports {
            if import.c_import {
                ccache.resolve_c_definitions(&import.path.get(0).ident)?;
                let mut path = Path::new();
                path.add_segment(&import.path.get(1).ident);
                // self.global.symbols.insert(path, );
            }
        }

        todo!("{:#?}", self);
    }
}
