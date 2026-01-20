mod cimports;

use std::{collections::HashMap, fmt::Debug, path::PathBuf};

use anyhow::{Result, anyhow, bail};
use clang::{Clang, Index, TranslationUnit};

use crate::{
    ast::{
        Definition, DefinitionKind, Module, expressions::Statement, function::FunctionBuilder,
        project::cimports::CCache,
    },
    general::path::Path,
    general::types::*,
};

#[derive(Debug, Clone)]
pub struct Project {
    pub root_module: Module,
    pub external_projects: HashMap<String, Project>,
    pub name: String,
    pub version: (usize, usize, usize),
}

impl Project {
    pub fn new<S: Into<String>>(name: S, version: (usize, usize, usize)) -> Self {
        return Self {
            root_module: Module::new(),
            // todo load core into external projects for core::* resolution
            external_projects: HashMap::new(),
            name: name.into(),
            version,
        };
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

    // NOTE: ommit templates for now do to complexity
    pub fn generate_ir(&mut self) -> Result<()> {
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
        let mut c_definitions = HashMap::<String, Definition>::new();
        let mut import_registry = HashMap::<Path, &Definition>::new();
        let clang = Clang::new().map_err(|s| anyhow!("clang: {s}"))?;
        let mut ccache = CCache::new(&clang)?;

        for import in &self.root_module.imports {
            if import.c_import {
                ccache.resolve_c_definition(&import.path)?;
                continue;
            }
            import_registry.insert(
                import.path.clone(),
                self.get_non_local_definition(Path::new(), &import.path),
            );
        }

        // validate that all typenames in the module are indeed names/alias of a type
        //

        todo!()
    }
}
