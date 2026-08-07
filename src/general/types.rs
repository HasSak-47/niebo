use super::naming::QualifiedName;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PrimitiveType {
    Bool,
    Int(usize),
    Uint(usize),
    Float(usize),
    String,
    Void,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct StructType {
    pub members: Vec<(String, Type)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnionType {
    pub members: Vec<(String, Type)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VariantType {
    pub members: Vec<(String, Type)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Params {
    name: Option<String>,
    ty: Type,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionType {
    pub params: Vec<Params>,
    pub ret_ty: Box<Type>,
    pub varidic: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Template {
    pub restrictions: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeKind {
    Primitive(PrimitiveType),
    Struct(StructType),
    Array(Box<Type>),
    Union(UnionType),
    Variant(VariantType),
    Pointer(Box<Type>),
    MutablePointer(Box<Type>),
    Reference(Box<Type>),
    MutableReference(Box<Type>),
    Function(FunctionType),
    Template(Template),
    // alias to a type
    Alias(Box<Type>),
    // path to a type that should be resolved later
    Named(QualifiedName),
}

pub enum Trait {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Type {
    pub kind: TypeKind,
}

impl From<(String, Type)> for Params {
    fn from(value: (String, Type)) -> Self {
        return Params {
            name: Some(value.0),
            ty: value.1,
        };
    }
}

impl From<Type> for Params {
    fn from(value: Type) -> Self {
        return Params {
            name: None,
            ty: value,
        };
    }
}

impl Type {
    pub fn alias(ty: Type) -> Self {
        Self {
            kind: TypeKind::Alias(Box::new(ty)),
        }
    }

    pub fn primitive(p: PrimitiveType) -> Self {
        Self {
            kind: TypeKind::Primitive(p),
        }
    }

    pub fn int() -> Self {
        Self {
            kind: TypeKind::Primitive(PrimitiveType::Int(32)),
        }
    }

    pub fn uint() -> Self {
        Self {
            kind: TypeKind::Primitive(PrimitiveType::Uint(32)),
        }
    }

    pub fn int_p(prec: usize) -> Self {
        Self {
            kind: TypeKind::Primitive(PrimitiveType::Int(prec)),
        }
    }

    pub fn uint_p(prec: usize) -> Self {
        Self {
            kind: TypeKind::Primitive(PrimitiveType::Uint(prec)),
        }
    }

    pub fn float() -> Self {
        Self {
            kind: TypeKind::Primitive(PrimitiveType::Float(32)),
        }
    }

    pub fn float_p(prec: usize) -> Self {
        Self {
            kind: TypeKind::Primitive(PrimitiveType::Float(prec)),
        }
    }

    pub fn bool() -> Self {
        Self {
            kind: TypeKind::Primitive(PrimitiveType::Bool),
        }
    }

    pub fn string() -> Self {
        Self {
            kind: TypeKind::Primitive(PrimitiveType::String),
        }
    }

    pub fn void() -> Self {
        Self {
            kind: TypeKind::Primitive(PrimitiveType::Void),
        }
    }

    pub fn struct_t(members: Vec<(String, Type)>) -> Self {
        Self {
            kind: TypeKind::Struct(StructType { members }),
        }
    }

    pub fn union(members: Vec<(String, Type)>) -> Self {
        Self {
            kind: TypeKind::Union(UnionType { members }),
        }
    }

    pub fn array(element: Type) -> Self {
        Self {
            kind: TypeKind::Array(Box::new(element)),
        }
    }

    pub fn pointer(ty: Type) -> Self {
        Self {
            kind: TypeKind::Pointer(Box::new(ty)),
        }
    }

    pub fn reference(ty: Type) -> Self {
        Self {
            kind: TypeKind::Reference(Box::new(ty)),
        }
    }

    pub fn mutable_pointer(ty: Type) -> Self {
        Self {
            kind: TypeKind::MutablePointer(Box::new(ty)),
        }
    }

    pub fn mutable_reference(ty: Type) -> Self {
        Self {
            kind: TypeKind::MutableReference(Box::new(ty)),
        }
    }

    pub fn function<I: Into<Params>>(params: Vec<I>, ret_ty: Type, varidic: bool) -> Self {
        Self {
            kind: TypeKind::Function(FunctionType {
                params: params.into_iter().map(|a| a.into()).collect(),
                ret_ty: Box::new(ret_ty),
                varidic,
            }),
        }
    }

    pub fn named(path: QualifiedName) -> Self {
        Self {
            kind: TypeKind::Named(path),
        }
    }

    pub fn is_pointer(&self) -> bool {
        match &self.kind {
            TypeKind::Pointer(_) => true,
            TypeKind::MutablePointer(_) => true,
            _ => false,
        }
    }
}

impl From<QualifiedName> for Type {
    fn from(path: QualifiedName) -> Self {
        Type {
            kind: TypeKind::Named(path),
        }
    }
}
