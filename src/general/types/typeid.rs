use std::collections::{HashMap, HashSet};

use crate::general::naming::QualifiedName;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PrimitiveTypeID {
    Bool,
    Int(usize),
    Uint(usize),
    Float(usize),
    Void,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
pub struct StructTypeID {
    pub members: Vec<(String, TypeID)>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UnionTypeID {
    pub members: Vec<(String, TypeID)>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct VariantTypeID {
    pub members: Vec<(String, TypeID)>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Params {
    name: Option<String>,
    ty: TypeID,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FunctionType {
    pub params: Vec<Params>,
    pub ret_ty: Box<TypeID>,
    pub varidic: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TemplateID {
    pub restrictions: Vec<QualifiedName>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TypeIDKind {
    Primitive(PrimitiveTypeID),
    Struct(StructTypeID),
    Array(Box<TypeID>),
    Union(UnionTypeID),
    Pointer(Box<TypeID>),
    MutablePointer(Box<TypeID>),
    Reference(Box<TypeID>),
    MutableReference(Box<TypeID>),
    Function(FunctionType),
    Template(TemplateID),
    // alias to a type
    Alias {
        name: QualifiedName,
        to: Box<TypeID>,
    },
    // path to a type that should be resolved later
    Named(QualifiedName),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Trait {
    functions: HashMap<String, FunctionType>,
    types: HashMap<String, TypeID>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TypeID {
    pub kind: TypeIDKind,
}

impl From<(String, TypeID)> for Params {
    fn from(value: (String, TypeID)) -> Self {
        return Params {
            name: Some(value.0),
            ty: value.1,
        };
    }
}

impl From<TypeID> for Params {
    fn from(value: TypeID) -> Self {
        return Params {
            name: None,
            ty: value,
        };
    }
}

impl TypeID {
    fn no_type() -> TypeID {
        TypeID {
            kind: TypeIDKind::Primitive(PrimitiveTypeID::Void),
        }
    }

    pub fn alias(name: QualifiedName, ty: TypeID) -> Self {
        Self {
            kind: TypeIDKind::Alias {
                name,
                to: Box::new(ty),
            },
            ..TypeID::no_type()
        }
    }

    pub fn primitive(p: PrimitiveTypeID) -> Self {
        Self {
            kind: TypeIDKind::Primitive(p),
            ..TypeID::no_type()
        }
    }

    pub fn int() -> Self {
        Self {
            kind: TypeIDKind::Primitive(PrimitiveTypeID::Int(32)),
            ..TypeID::no_type()
        }
    }

    pub fn uint() -> Self {
        Self {
            kind: TypeIDKind::Primitive(PrimitiveTypeID::Uint(32)),
            ..TypeID::no_type()
        }
    }

    pub fn int_p(prec: usize) -> Self {
        Self {
            kind: TypeIDKind::Primitive(PrimitiveTypeID::Int(prec)),
            ..TypeID::no_type()
        }
    }

    pub fn uint_p(prec: usize) -> Self {
        Self {
            kind: TypeIDKind::Primitive(PrimitiveTypeID::Uint(prec)),
            ..TypeID::no_type()
        }
    }

    pub fn float() -> Self {
        Self {
            kind: TypeIDKind::Primitive(PrimitiveTypeID::Float(32)),
            ..TypeID::no_type()
        }
    }

    pub fn float_p(prec: usize) -> Self {
        Self {
            kind: TypeIDKind::Primitive(PrimitiveTypeID::Float(prec)),
            ..TypeID::no_type()
        }
    }

    pub fn bool() -> Self {
        Self {
            kind: TypeIDKind::Primitive(PrimitiveTypeID::Bool),
            ..TypeID::no_type()
        }
    }

    pub fn void() -> Self {
        Self {
            kind: TypeIDKind::Primitive(PrimitiveTypeID::Void),
            ..TypeID::no_type()
        }
    }

    pub fn struct_t(members: Vec<(String, TypeID)>) -> Self {
        Self {
            kind: TypeIDKind::Struct(StructTypeID { members }),
            ..TypeID::no_type()
        }
    }

    pub fn union(members: Vec<(String, TypeID)>) -> Self {
        Self {
            kind: TypeIDKind::Union(UnionTypeID { members }),
            ..TypeID::no_type()
        }
    }

    pub fn array(element: TypeID) -> Self {
        Self {
            kind: TypeIDKind::Array(Box::new(element)),
            ..TypeID::no_type()
        }
    }

    pub fn pointer(ty: TypeID) -> Self {
        Self {
            kind: TypeIDKind::Pointer(Box::new(ty)),
            ..TypeID::no_type()
        }
    }

    pub fn reference(ty: TypeID) -> Self {
        Self {
            kind: TypeIDKind::Reference(Box::new(ty)),
            ..TypeID::no_type()
        }
    }

    pub fn mutable_pointer(ty: TypeID) -> Self {
        Self {
            kind: TypeIDKind::MutablePointer(Box::new(ty)),
            ..TypeID::no_type()
        }
    }

    pub fn mutable_reference(ty: TypeID) -> Self {
        Self {
            kind: TypeIDKind::MutableReference(Box::new(ty)),
            ..TypeID::no_type()
        }
    }

    pub fn function<I: Into<Params>>(params: Vec<I>, ret_ty: TypeID, varidic: bool) -> Self {
        Self {
            kind: TypeIDKind::Function(FunctionType {
                params: params.into_iter().map(|a| a.into()).collect(),
                ret_ty: Box::new(ret_ty),
                varidic,
            }),
            ..TypeID::no_type()
        }
    }

    pub fn named(path: QualifiedName) -> Self {
        Self {
            kind: TypeIDKind::Named(path),
            ..TypeID::no_type()
        }
    }

    pub fn is_pointer(&self) -> bool {
        match &self.kind {
            TypeIDKind::Pointer(_) => true,
            TypeIDKind::MutablePointer(_) => true,
            _ => false,
        }
    }
}

impl From<QualifiedName> for TypeID {
    fn from(path: QualifiedName) -> Self {
        TypeID {
            kind: TypeIDKind::Named(path),
            ..TypeID::no_type()
        }
    }
}
