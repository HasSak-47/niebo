use std::{
    collections::{HashMap, HashSet},
    fmt::Debug,
};

use anyhow::Result;
use clang::Clang;

use crate::{
    ast::{Definition, DefinitionKind, project::Project},
    general::{naming::QualifiedName, types::Type},
    lowerer::cimports::CCache,
};

mod validator;
use validator::ExpressionValidator;

#[derive(Debug, Clone)]
enum Symbol {
    Variable(Type),
    Type(Type),
    Function { ret_ty: Type, params: Vec<Type> },
}

#[derive(Debug, Default)]
pub struct Validator {
    local_scope: Vec<HashMap<QualifiedName, Symbol>>,
    global_scope: HashMap<QualifiedName, Symbol>,
}

impl Validator {
    fn push_scope(&mut self) {
        self.local_scope.push(HashMap::new());
    }

    fn pop_scope(&mut self) {
        self.local_scope.pop();
    }

    fn register_local_symbol(&mut self, path: QualifiedName, kind: Symbol) {
        self.local_scope.last_mut().unwrap().insert(path, kind);
    }

    fn register_global_symbol(&mut self, path: QualifiedName, kind: Symbol) {
        self.global_scope.insert(path, kind);
    }

    fn find_symbol(&self, path: QualifiedName) -> Option<Symbol> {
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

impl Validator {
    pub fn new() -> Self {
        let mut p = Validator::default();
        p.global_scope.insert(
            "nullptr".into(),
            Symbol::Variable(Type::pointer(Type::void())),
        );
        return p;
    }

    pub fn validate_global_definition(&mut self, def: &mut Definition) -> Result<()> {
        match &mut def.kind {
            DefinitionKind::Function(func) => {
                self.register_global_symbol(
                    def.name.clone().into(),
                    Symbol::Function {
                        ret_ty: func.return_ty.clone().unwrap(),
                        params: func.parameters.iter().map(|a| a.1.clone()).collect(),
                    },
                );
            }
            DefinitionKind::Variable(var) => {
                var.value.validate(self)?;
                self.register_global_symbol(
                    def.name.clone().into(),
                    Symbol::Variable(var.ty.clone().unwrap()),
                );
            }
            DefinitionKind::Type(ty) => {
                self.register_global_symbol(def.name.clone().into(), Symbol::Type(ty.clone()));
            }
            _ => todo!(),
        };
        return Ok(());
    }

    pub fn validate_local_definition(&mut self, def: &mut Definition) -> Result<()> {
        match &mut def.kind {
            DefinitionKind::Function(func) => {
                self.register_local_symbol(
                    def.name.clone().into(),
                    Symbol::Function {
                        ret_ty: func.return_ty.clone().unwrap(),
                        params: func.parameters.iter().map(|a| a.1.clone()).collect(),
                    },
                );
            }
            DefinitionKind::Variable(var) => {
                var.value.validate(self)?;
                self.register_local_symbol(
                    def.name.clone().into(),
                    Symbol::Variable(var.ty.clone().unwrap()),
                );
            }
            DefinitionKind::Type(ty) => {
                self.register_local_symbol(def.name.clone().into(), Symbol::Type(ty.clone()));
            }
            _ => todo!(),
        };
        return Ok(());
    }

    // NOTE: ommit templates for now do to complexity
    pub fn process_project(&mut self, mut project: Project) -> Result<Project> {
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
        let clang = Clang::new().unwrap();
        let mut ccache = CCache::new(&clang)?;

        for import in &project.root_module.imports {
            if import.c_import {
                ccache.resolve_c_definitions(&import.path.get(0).ident)?;
                let mut name = QualifiedName::new();
                let mut header_path = QualifiedName::new();
                header_path.add_segment(&import.path.get(0).ident);
                header_path.add_segment(&import.path.get(1).ident);
                name.add_segment(&import.path.get(1).ident);

                let func = ccache.get_definition(&header_path)?;
                match &func.kind {
                    crate::ast::DefinitionKind::FunctionC(f) => self.register_global_symbol(
                        name,
                        Symbol::Function {
                            ret_ty: f.return_ty.clone().unwrap(),
                            params: f.parameters.iter().map(|f| f.1.clone()).collect(),
                        },
                    ),
                    td => unreachable!("{td:?}"),
                }
            }
        }

        for def in &mut project.root_module.definitions {
            self.validate_global_definition(def)?;
        }

        return Ok(project);
    }
}
