use super::*;
use std::collections::HashMap;

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
        panic!("symbol {ident} not found!");
    }

    pub fn register_symbol<S: AsRef<str>>(&mut self, ident: S, symbol: Symbol<'ctx>) {
        if let Some(_) = self.reg.insert(ident.as_ref().to_string(), symbol) {
            panic!("symbol redefined!");
        }
    }
}

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
        let repeat = self
            .scope
            .last_mut()
            .unwrap()
            .insert(ident.as_ref().to_string(), symbol);
        if repeat.is_some() {
            panic!("symbol redefined!");
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
