use super::*;
use std::collections::HashMap;

#[derive(Debug)]
pub struct SymbolRegistry<'ctx> {
    reg: HashMap<String, Symbol<'ctx>>,
    scope: SymbolScope<'ctx>,
}

impl<'ctx> SymbolRegistry<'ctx> {
    pub fn new<S: AsRef<str>>(namespace: S) -> Self {
        let mut reg = HashMap::new();
        reg.insert(
            namespace.as_ref().to_string(),
            Symbol::Registry(SymbolRegistry {
                reg: HashMap::new(),
                scope: SymbolScope::new(),
            }),
        );
        return Self {
            reg,
            scope: SymbolScope::new(),
        };
    }

    pub fn push_scope(&mut self) {
        self.scope.push_scope();
    }
    pub fn pop_scope(&mut self) {
        self.scope.pop_scope();
    }

    pub fn get_symbol<S: AsRef<str>>(&self, ident: S) -> &Symbol<'ctx> {
        let ident = ident.as_ref();
        if let Some(s) = self.scope.get_symbol(ident) {
            return s;
        }

        for (id, symbol) in &self.reg {
            if id == ident {
                return &symbol;
            }
        }
        panic!("symbol \"{ident}\" not found! {self:#?}");
    }

    pub fn register_symbol<S: AsRef<str>>(&mut self, ident: S, symbol: Symbol<'ctx>) {
        if let Some(_) = self.reg.insert(ident.as_ref().to_string(), symbol) {
            panic!("symbol redefined! {self:#?}");
        }
    }

    pub fn register_symbol_scope<S: AsRef<str>>(&mut self, ident: S, symbol: Symbol<'ctx>) {
        self.scope
            .register_symbol(ident.as_ref().to_string(), symbol);
    }
}

#[derive(Debug)]
pub struct SymbolScope<'ctx> {
    scope: Vec<HashMap<String, Symbol<'ctx>>>,
}

impl<'ctx> SymbolScope<'ctx> {
    pub fn new() -> Self {
        Self { scope: Vec::new() }
    }

    pub fn push_scope(&mut self) {
        self.scope.push(HashMap::new());
    }

    pub fn pop_scope(&mut self) {
        self.scope.pop();
    }

    pub fn register_symbol<S: AsRef<str>>(&mut self, ident: S, symbol: Symbol<'ctx>) {
        if self.scope.len() == 0 {
            self.scope.push(HashMap::new());
            self.scope[0].insert(ident.as_ref().to_string(), symbol);

            return;
        }
        let repeat = self
            .scope
            .last_mut()
            .unwrap()
            .insert(ident.as_ref().to_string(), symbol);
        if repeat.is_some() {
            panic!("symbol redefined!: {self:#?}");
        }
    }

    pub fn get_symbol<S: AsRef<str>>(&self, ident: S) -> Option<&Symbol<'ctx>> {
        let ident = ident.as_ref();
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
        pointer: FunctionValue<'ctx>,
        external: bool,
        ty: Type,
    },
    Symbol {
        ty: Type,
        pointer: PointerValue<'ctx>,
    },
    SymbolVal {
        ty: Type,
        pointer: BasicValueEnum<'ctx>,
    },
    Registry(SymbolRegistry<'ctx>),
}

impl<'ctx> Symbol<'ctx> {
    pub fn get_type(&self) -> Type {
        match self {
            Self::Function { ty, .. } => ty.clone(),
            Self::Symbol { ty, .. } => ty.clone(),
            _ => unreachable!(),
        }
    }
}
