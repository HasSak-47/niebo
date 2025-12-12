use super::*;
use inkwell::values::BasicValueEnum;
use std::collections::HashMap;

#[derive(Debug)]
pub struct Registry<'ctx> {
    root: HashMap<String, Symbol<'ctx>>,
    scope: Scope<'ctx>,
}

impl<'ctx> Registry<'ctx> {
    pub fn new<S: Into<String>>(namespace: S) -> Self {
        let mut reg = HashMap::new();
        reg.insert(
            namespace.into(),
            Symbol::Module(Registry {
                root: HashMap::new(),
                scope: Scope::new(),
            }),
        );
        return Self {
            root: reg,
            scope: Scope::new(),
        };
    }

    pub fn push_scope(&mut self) {
        self.scope.push_scope();
    }
    pub fn pop_scope(&mut self) {
        self.scope.pop_scope();
    }

    pub fn get_symbol(&self, ident: &Identifier) -> &Symbol<'ctx> {
        let mut path = &self.root;
        for p in ident.path.iter() {
            if let Symbol::Module(m) = &path[p] {
                path = &m.root;
            }
        }

        if path.contains_key(&ident.name) {
            return &path[&ident.name];
        }
        if let Some(s) = self.scope.get_symbol(ident) {
            return s;
        }

        panic!("symbol \"{ident:?}\" not found! {self:#?}");
    }

    pub fn register_symbol<S: Into<String>>(&mut self, ident: S, symbol: Symbol<'ctx>) {
        if let Some(_) = self.root.insert(ident.into(), symbol) {
            panic!("symbol redefined! {self:#?}");
        }
    }

    pub fn register_symbol_scope<S: Into<String>>(&mut self, ident: S, symbol: Symbol<'ctx>) {
        self.scope.register_symbol(ident.into(), symbol);
    }
}

#[derive(Debug)]
pub struct Scope<'ctx> {
    scope: Vec<HashMap<String, Symbol<'ctx>>>,
}

impl<'ctx> Scope<'ctx> {
    pub fn new() -> Self {
        Self { scope: Vec::new() }
    }

    pub fn push_scope(&mut self) {
        self.scope.push(HashMap::new());
    }

    pub fn pop_scope(&mut self) {
        self.scope.pop();
    }

    pub fn register_symbol<S: Into<String>>(&mut self, ident: S, symbol: Symbol<'ctx>) {
        if self.scope.len() == 0 {
            self.scope.push(HashMap::new());
            self.scope[0].insert(ident.into(), symbol);

            return;
        }
        let repeat = self.scope.last_mut().unwrap().insert(ident.into(), symbol);
        if repeat.is_some() {
            panic!("symbol redefined!: {self:#?}");
        }
    }

    pub fn get_symbol(&self, ident: &Identifier) -> Option<&Symbol<'ctx>> {
        let ident = &ident.name;
        for symbols in self.scope.iter().rev() {
            if symbols.contains_key(ident) {
                return Some(&symbols[ident]);
            }
        }

        return None;
    }
}

#[derive(Debug)]
pub enum Symbol<'ctx> {
    Function {
        pointer: Option<FunctionValue<'ctx>>,
        external: bool,
        ty: Type,
    },
    Label {
        ty: Type,
        pointer: Option<PointerValue<'ctx>>,
    },
    Value {
        ty: Type,
        pointer: Option<BasicValueEnum<'ctx>>,
    },
    Module(Registry<'ctx>),
}

impl<'ctx> Symbol<'ctx> {
    pub fn get_type(&self) -> Type {
        match self {
            Self::Function { ty, .. } => ty.clone(),
            Self::Label { ty, .. } => ty.clone(),
            _ => unreachable!("registry has no type"),
        }
    }
}
