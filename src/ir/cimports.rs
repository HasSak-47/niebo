use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
};

use anyhow::bail;
use clang::{Clang, Index};

use crate::{
    ast::{
        Definition, DefinitionKind, Visibility, expressions::Statement, function::FunctionBuilder,
    },
    general::{
        path::Path,
        types::{PrimitiveType, Type},
    },
};

fn rec_from<'a>(
    value: &clang::Type<'a>,
    parent: &str,
    depth: usize,
    visiting: &mut HashSet<String>,
    deps: &mut HashSet<String>,
) -> Type {
    let int_from_size = |signed: bool| {
        let bits = value.get_sizeof().ok().map(|s| s * 8).unwrap_or(32);
        if signed {
            Type::Primitive(PrimitiveType::Int(bits))
        } else {
            Type::Primitive(PrimitiveType::Uint(bits))
        }
    };

    let float_from_size = || {
        let bits = value.get_sizeof().ok().map(|s| s * 8).unwrap_or(32);
        Type::Primitive(PrimitiveType::Float(bits))
    };

    let kind = match value.get_kind() {
        // what the fuck clang
        clang::TypeKind::Void => Type::void(),
        clang::TypeKind::Bool => Type::bool(),
        clang::TypeKind::SChar | clang::TypeKind::CharS => Type::int_p(8),
        clang::TypeKind::CharU | clang::TypeKind::UChar => Type::uint_p(8),
        clang::TypeKind::WChar => int_from_size(true),
        clang::TypeKind::Char16 | clang::TypeKind::Char32 => int_from_size(false),
        clang::TypeKind::Short
        | clang::TypeKind::Int
        | clang::TypeKind::Long
        | clang::TypeKind::LongLong
        | clang::TypeKind::Int128 => int_from_size(true),
        clang::TypeKind::UShort
        | clang::TypeKind::UInt
        | clang::TypeKind::ULong
        | clang::TypeKind::ULongLong
        | clang::TypeKind::UInt128 => int_from_size(false),
        clang::TypeKind::Half
        | clang::TypeKind::Float16
        | clang::TypeKind::Float
        | clang::TypeKind::Double
        | clang::TypeKind::LongDouble
        | clang::TypeKind::Float128 => float_from_size(),
        clang::TypeKind::Pointer => {
            let ty = value.get_pointee_type().unwrap();
            Type::pointer(rec_from(&ty, parent, depth + 1, visiting, deps))
        }
        clang::TypeKind::BlockPointer | clang::TypeKind::MemberPointer => {
            let ty = value
                .get_pointee_type()
                .unwrap_or_else(|| value.get_canonical_type());
            Type::pointer(rec_from(&ty, parent, depth + 1, visiting, deps))
        }
        clang::TypeKind::LValueReference => {
            let ty = value.get_pointee_type().unwrap();
            Type::reference(rec_from(&ty, parent, depth + 1, visiting, deps))
        }
        clang::TypeKind::RValueReference => {
            let ty = value.get_pointee_type().unwrap();
            Type::reference(rec_from(&ty, parent, depth + 1, visiting, deps))
        }
        clang::TypeKind::Enum => int_from_size(true),
        clang::TypeKind::Typedef
        | clang::TypeKind::Elaborated
        | clang::TypeKind::Auto
        | clang::TypeKind::Unexposed => rec_from(
            &value.get_canonical_type(),
            parent,
            depth + 1,
            visiting,
            deps,
        ),
        clang::TypeKind::FunctionPrototype | clang::TypeKind::FunctionNoPrototype => {
            let ret = value
                .get_result_type()
                .as_ref()
                .map(|t| rec_from(t, parent, depth + 1, visiting, deps))
                .unwrap_or_else(Type::void);
            let params = value
                .get_argument_types()
                .unwrap_or_default()
                .into_iter()
                .map(|ty| {
                    (
                        "".to_string(),
                        rec_from(&ty, parent, depth + 1, visiting, deps),
                    )
                })
                .collect();
            Type::function(params, ret, value.is_variadic())
        }
        clang::TypeKind::ConstantArray
        | clang::TypeKind::DependentSizedArray
        | clang::TypeKind::IncompleteArray
        | clang::TypeKind::VariableArray
        | clang::TypeKind::Vector => {
            let el = value.get_element_type().unwrap();
            Type::array(rec_from(&el, parent, depth + 1, visiting, deps))
        }
        clang::TypeKind::Record => {
            if let Some(decl) = value.get_declaration() {
                let record_name = decl.get_name().unwrap_or_else(|| value.get_display_name());
                let record_kind = match decl.get_kind() {
                    clang::EntityKind::UnionDecl => "union",
                    _ => "struct",
                };
                let record_name = if record_name.is_empty() {
                    "<anonymous>".to_string()
                } else {
                    record_name
                };
                if record_name != "<anonymous>" {
                    deps.insert(record_name.clone());
                    return Type::named(Path::from(record_name));
                }
                let record_key = decl
                    .get_usr()
                    .map(|usr| usr.0)
                    .unwrap_or_else(|| format!("{record_kind}:{record_name}"));
                if visiting.contains(&record_key) {
                    return Type::r#struct(Vec::new());
                }
                visiting.insert(record_key.clone());
                let record_path = if parent.is_empty() {
                    format!("{record_kind} {record_name}")
                } else {
                    format!("{parent}.{record_kind} {record_name}")
                };
                println!("[{depth:3}] {record_path}");
                let mut members = Vec::new();
                for child in decl
                    .get_children()
                    .into_iter()
                    .filter(|c| c.get_kind() == clang::EntityKind::FieldDecl)
                    .filter(|ty| ty.get_type().is_some())
                {
                    let name = child.get_name().unwrap_or_default();
                    println!(
                        "[{depth:3}] {record_path}.{name}: {}",
                        child.get_type().as_ref().unwrap().get_display_name()
                    );
                    members.push((
                        name,
                        rec_from(
                            &child.get_type().unwrap(),
                            &record_path,
                            depth + 1,
                            visiting,
                            deps,
                        ),
                    ));
                }
                visiting.remove(&record_key);
                return match decl.get_kind() {
                    clang::EntityKind::UnionDecl => Type::union(members),
                    _ => Type::r#struct(members),
                };
            }
            Type::r#struct(Vec::new())
        }
        td => todo!("todo: {td:?}"),
    };
    return kind;
}

fn convert_clang_type<'a>(value: &clang::Type<'a>) -> Type {
    let mut visiting = HashSet::new();
    let mut deps = HashSet::new();
    rec_from(value, "", 0, &mut visiting, &mut deps)
}

fn convert_clang_type_with_deps<'a>(value: &clang::Type<'a>, deps: &mut HashSet<String>) -> Type {
    let mut visiting = HashSet::new();
    rec_from(value, "", 0, &mut visiting, deps)
}

impl<'a> From<&clang::Type<'a>> for Type {
    fn from(value: &clang::Type<'a>) -> Self {
        convert_clang_type(value)
    }
}

// keeps the precompiled translation units and a map of searched definitions
struct CCacheEntry {
    symbols: HashMap<String, Definition>,
}

// handles the cache of c headers
pub struct CCache<'c> {
    index: Index<'c>,
    map: HashMap<String, CCacheEntry>,
}

impl<'c> CCache<'c> {
    pub fn new(clang: &'c Clang) -> anyhow::Result<CCache<'c>> {
        let index = Index::new(&clang, false, false);

        return Ok(CCache {
            index,
            map: HashMap::new(),
        });
    }

    pub fn get_definition(&self, path: &Path) -> anyhow::Result<&Definition> {
        let header = &path.get(0).ident;
        let symbol = &path.get(1).ident;

        return Ok(&self.map[header].symbols[symbol]);
    }

    // loads all definitions in c header into a cache
    pub fn resolve_c_definitions<S: AsRef<str>>(&mut self, header: S) -> anyhow::Result<()> {
        let header = header.as_ref();

        if !self.map.contains_key(header) {
            // quick and dirty stdlib c handler
            let mut path = PathBuf::from(format!("/usr/include/{header}"));
            path.set_extension("h");
            if !path.exists() {
                bail!("header not found");
            }
            let tu = self.index.parser(path).parse()?;
            let mut entry = CCacheEntry {
                symbols: HashMap::new(),
            };
            let mut deps = HashSet::new();
            let childs = tu.get_entity().get_children();
            for child in childs.iter().filter(|child| child.get_name().is_some()) {
                let s = child.get_name().unwrap();
                match child.get_kind() {
                    clang::EntityKind::FunctionDecl => {
                        let mut builder = FunctionBuilder::new(&s)
                            .set_varidic(child.is_variadic())
                            .set_ret_ty(convert_clang_type_with_deps(
                                &child.get_result_type().unwrap(),
                                &mut deps,
                            ));
                        let params = child.get_children();

                        for param in params.iter().filter(|f| f.get_type().is_some()) {
                            let name = param.get_name();
                            let ty: Type = convert_clang_type_with_deps(
                                param.get_type().as_ref().unwrap(),
                                &mut deps,
                            );
                            if let Some(ident) = name {
                                builder = builder.add_param(ident, ty)
                            } else {
                                builder = builder.add_anon_param(ty)
                            }
                        }

                        entry.symbols.insert(s.clone(), builder.build_c_function());
                    }
                    _ => {}
                }
            }
            for dep in deps {
                entry.symbols.entry(dep.clone()).or_insert(Definition {
                    kind: DefinitionKind::Type(Type::named(Path::from(dep.clone()))),
                    visibility: Visibility::Public,
                    name: dep,
                });
            }
            self.map.insert(header.to_string(), entry);
        }

        return Ok(());
    }
}
