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
    general::{naming::QualifiedName, types::Type},
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
