use std::{
    collections::{HashMap, HashSet},
    fmt::Debug,
};

use anyhow::{Result, bail};
use clang::Clang;

use crate::{
    ast::{
        Definition, DefinitionKind, Implementation, TraitImplementation,
        function::{FunctionDeclaration, FunctionDefinition},
        module::Module,
        project::Project,
    },
    general::{
        naming::QualifiedName,
        types::{FunctionType, Trait, Type},
    },
    lowerer::cimports::CCache,
};

pub mod expressions;
pub mod validator;
use expressions::ExpressionValidator;

#[derive(Debug, Clone)]
enum Symbol {
    Variable(Type),
    Type(Type),
    Trait(Trait),
    Function { ret_ty: Type, params: Vec<Type> },
}

#[derive(Debug)]
struct TypeData {
    ty: Type,
    traits: HashSet<QualifiedName>,
    methods: HashMap<String, FunctionDeclaration>,
}

#[derive(Debug, Default)]
pub struct Validator {
    local_scope: Vec<HashMap<QualifiedName, Symbol>>,
    global_scope: HashMap<QualifiedName, Symbol>,

    type_data: HashMap<QualifiedName, TypeData>,
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
            DefinitionKind::FunctionDefinition(func) => {
                self.register_global_symbol(
                    def.name.clone().into(),
                    Symbol::Function {
                        ret_ty: func.decl.return_ty.clone().unwrap(),
                        params: func.decl.parameters.iter().map(|a| a.1.clone()).collect(),
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
            DefinitionKind::FunctionDefinition(func) => {
                self.register_local_symbol(
                    def.name.clone().into(),
                    Symbol::Function {
                        ret_ty: func.decl.return_ty.clone().unwrap(),
                        params: func.decl.parameters.iter().map(|a| a.1.clone()).collect(),
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

    pub fn register_impl(&mut self, i: &Implementation) {
        if let Some(ty_data) = self.type_data.get_mut(&i.target) {
            for def in &i.definitions {
                if let DefinitionKind::FunctionDefinition(func) = &def.kind {
                    ty_data.methods.insert(def.name.clone(), func.decl.clone());
                }
            }
        }
    }

    pub fn register_trait_impl(&mut self, t: &TraitImplementation) -> anyhow::Result<()> {
        if let Some(ty) = self.type_data.get_mut(&t.target) {
            if ty.traits.contains(&t.trait_path) {
                bail!("type {:?} already implements {}", ty.ty, t.trait_path);
            }

            ty.traits.insert(t.trait_path.clone());
        };

        return Ok(());
    }

    // NOTE: ommit templates for now do to complexity
    pub fn process_project(&mut self, mut project: Project) -> Result<Project> {
        let clang = Clang::new().unwrap();
        let mut ccache = CCache::new(&clang)?;

        self.process_module(&mut project.root_module, &mut ccache)?;

        return Ok(project);
    }

    pub fn process_module<'a>(
        &mut self,
        module: &mut Module,
        ccache: &mut CCache<'a>,
    ) -> Result<()> {
        for import in &module.imports {
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

        for def in &mut module.definitions {
            match &def.kind {
                DefinitionKind::Type(ty) => {
                    self.type_data.insert(
                        def.name.clone().into(),
                        TypeData {
                            ty: ty.clone(),
                            traits: HashSet::new(),
                            methods: HashMap::new(),
                        },
                    );
                }
                _ => {}
            }
        }

        for def in &mut module.definitions {
            match &def.kind {
                DefinitionKind::Implementation(i) => {
                    self.register_impl(&i);
                }
                DefinitionKind::TraitImplementation(t) => {
                    self.register_trait_impl(&t)?;
                }
                DefinitionKind::Trait(_) => {}
                _ => {}
            }
        }

        for def in &mut module.definitions {
            self.validate_global_definition(def)?;
        }

        return Ok(());
    }
}
