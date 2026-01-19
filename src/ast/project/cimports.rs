use std::{borrow::Borrow, collections::HashMap, path::PathBuf};

use anyhow::bail;
use clang::{Clang, Index, TranslationUnit};

use crate::{
    ast::{Definition, Path, function::FunctionBuilder},
    general::types::Type,
};

impl<'a, T: Borrow<clang::Type<'a>>> From<T> for Type {
    fn from(value: T) -> Self {
        let value = value.borrow();
        return match value.get_kind() {
            // what the fuck clang
            clang::TypeKind::SChar | clang::TypeKind::CharS => Self::int_p(8),
            clang::TypeKind::Pointer => {
                let ty = value.get_pointee_type().unwrap();
                return Self::pointer(Self::from(&ty));
            }
            td => todo!("todo: {td:?}"),
        };
    }
}

// keeps the precompiled translation units and a map of searched definitions
struct CCacheEntry<'c> {
    tu: TranslationUnit<'c>,
    symbols: HashMap<String, Definition>,
}

// handles the cache of c headers
pub struct CCache<'c> {
    index: Index<'c>,
    map: HashMap<String, CCacheEntry<'c>>,
}

impl<'c> CCache<'c> {
    pub fn new(clang: &'c Clang) -> anyhow::Result<CCache<'c>> {
        let index = Index::new(&clang, false, false);

        return Ok(CCache {
            index,
            map: HashMap::new(),
        });
    }
    pub fn resolve_c_definition(&'c mut self, path: &Path) -> anyhow::Result<()> {
        let header = &path.get(0).ident;
        let symbol = &path.get(1).ident;

        if !self.map.contains_key(header) {
            // quick and dirty stdlib c handler
            let mut path = PathBuf::from(format!("/usr/include/{header}"));
            path.set_extension("h");
            if !path.exists() {
                bail!("header not found");
            }
            let tu = self.index.parser(path).parse()?;
            let entry = CCacheEntry {
                tu: tu,
                symbols: HashMap::new(),
            };
            self.map.insert(header.clone(), entry);
        }

        let entry = self.map.get_mut(header).unwrap();
        if entry.symbols.contains_key(symbol) {
            return Ok(());
        }

        let unit = &entry.tu;
        let childs = unit.get_entity().get_children();
        for child in childs.iter().filter(|child| child.get_name().is_some()) {
            let s = child.get_name().unwrap();
            if s == *symbol {
                match child.get_kind() {
                    clang::EntityKind::FunctionDecl => {
                        let mut builder = FunctionBuilder::new(&s).set_varidic(child.is_variadic());
                        let params = child.get_children();

                        for param in params.iter().filter(|f| f.get_type().is_some()) {
                            let name = param.get_name();
                            let ty: Type = param.get_type().unwrap().into();
                            if let Some(ident) = name {
                                builder = builder.add_param(ident, ty)
                            } else {
                                builder = builder.add_anon_param(ty)
                            }
                        }

                        entry.symbols.insert(s.clone(), builder.build_c_function());
                        return Ok(());
                    }
                    _ => {}
                }
            }
        }

        todo!()
    }
}
