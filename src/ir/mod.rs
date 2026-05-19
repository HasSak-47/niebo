use std::collections::HashMap;

use crate::general::{path::Path, types::Type};

struct Binary {
    symbols: HashMap<Path, Symbol>,
    types: HashMap<Type, Symbol>,
}

pub enum LinkKind {
    External,
    Internal,
}

pub enum SymbolKind {
    Registry,
    Function,
    Variable,
}

pub struct Symbol {
    kind: SymbolKind,
    link: LinkKind,
    ident: Path,
}
