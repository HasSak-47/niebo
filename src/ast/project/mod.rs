use std::{collections::HashMap, fmt::Debug, fs::File, io::Read, path::PathBuf};

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
    fn qualified_name(parts: &[&str]) -> QualifiedName {
        let mut name = QualifiedName::new();
        for part in parts {
            name.add_segment(*part);
        }
        name
    }

    fn add_core_imports(module: &mut Module) {
        const CORE_TRAITS: &[&str] = &[
            "Add", "Neg", "Sub", "Mul", "Div", "Eq", "NEq", "LEq", "GEq", "Les", "Gre", "Copy",
        ];

        module.add_import(Self::qualified_name(&["core", "operations"]));
        for trait_name in CORE_TRAITS {
            module.add_import(Self::qualified_name(&["core", "traits", trait_name]));
        }
    }

    fn core_project_path() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("core")
    }

    fn with_core_dependency(mut project: Project) -> Result<Project> {
        if project.name == "core" {
            return Ok(project);
        }

        Self::add_core_imports(&mut project.root_module);

        let core_path = Self::core_project_path();
        if core_path.exists() {
            let core = Self::load_inner(core_path, false)?;
            project.add_external_project("core", core);
        }

        Ok(project)
    }

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

                let mut file = File::open(path)?;
                let mut buffer = String::new();
                file.read_to_string(&mut buffer)?;

                *md = parse_module(buffer)?;
                Self::load_modules(md, cur_path)?
            }
        }

        return Ok(());
    }
    pub fn load_script<P: AsRef<std::path::Path>>(path: P) -> Result<Self> {
        let mut buffer = String::new();
        let mut file = File::open(path)?;
        file.read_to_string(&mut buffer)?;

        let mut root = parse_module(buffer)?;
        Self::add_core_imports(&mut root);

        let mut project = Project {
            root_module: root,
            external_projects: HashMap::new(),
            name: "".into(),
            version: (0, 1, 0),
            edition: (0, 1, 0),
        };

        let core_path = Self::core_project_path();
        if core_path.exists() {
            let core = Self::load_inner(core_path, false)?;
            project.add_external_project("core", core);
        }

        return Ok(project);
    }

    pub fn load<P: AsRef<std::path::Path>>(path: P) -> Result<Self> {
        Self::load_inner(path, true)
    }

    fn load_inner<P: AsRef<std::path::Path>>(path: P, include_core: bool) -> Result<Self> {
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
        let project = Project {
            root_module: root,
            external_projects: HashMap::new(),
            name: chmura.project.name,
            version: (0, 1, 0),
            edition: (0, 1, 0),
        };

        if include_core {
            return Self::with_core_dependency(project);
        }

        return Ok(project);
    }

    pub fn add_external_project<S: Into<String>>(&mut self, name: S, project: Project) {
        self.external_projects.insert(name.into(), project);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn project_load_adds_core_dependency_and_operator_imports() -> Result<()> {
        let project = Project::load("test")?;

        assert!(project.external_projects.contains_key("core"));
        assert!(
            project
                .root_module
                .imports
                .iter()
                .any(|import| !import.c_import && format!("{:?}", import.path) == "core::operations")
        );
        assert!(
            project
                .root_module
                .imports
                .iter()
                .any(|import| !import.c_import && format!("{:?}", import.path) == "core::traits::Add")
        );

        Ok(())
    }

    #[test]
    fn core_project_load_does_not_depend_on_itself() -> Result<()> {
        let project = Project::load("core")?;

        assert_eq!(project.name, "core");
        assert!(!project.external_projects.contains_key("core"));

        Ok(())
    }
}
