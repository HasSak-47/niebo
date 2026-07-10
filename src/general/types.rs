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

#[derive(Debug, Clone, PartialEq, Eq)]
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
struct Params {
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
pub enum Type {
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
    pub fn int() -> Self {
        Self::Primitive(PrimitiveType::Int(32))
    }

    pub fn uint() -> Self {
        Self::Primitive(PrimitiveType::Uint(32))
    }

    pub fn int_p(prec: usize) -> Self {
        Self::Primitive(PrimitiveType::Int(prec))
    }

    pub fn uint_p(prec: usize) -> Self {
        Self::Primitive(PrimitiveType::Uint(prec))
    }

    pub fn float() -> Self {
        Self::Primitive(PrimitiveType::Float(32))
    }

    pub fn bool() -> Self {
        Self::Primitive(PrimitiveType::Bool)
    }

    pub fn string() -> Self {
        Self::Primitive(PrimitiveType::String)
    }

    pub fn void() -> Self {
        Self::Primitive(PrimitiveType::Void)
    }

    pub fn r#struct(members: Vec<(String, Type)>) -> Self {
        Self::Struct(StructType { members })
    }

    pub fn union(members: Vec<(String, Type)>) -> Self {
        Self::Union(UnionType { members })
    }

    pub fn array(element: Type) -> Self {
        Self::Array(Box::new(element))
    }

    pub fn pointer(ty: Type) -> Self {
        Self::Pointer(Box::new(ty))
    }

    pub fn reference(ty: Type) -> Self {
        Self::Reference(Box::new(ty))
    }

    pub fn function<I: Into<Params>>(params: Vec<I>, ret_ty: Type, varidic: bool) -> Self {
        Self::Function(FunctionType {
            params: params.into_iter().map(|a| a.into()).collect(),
            ret_ty: Box::new(ret_ty),
            varidic,
        })
    }

    pub fn named(path: QualifiedName) -> Self {
        Self::Named(path)
    }

    pub fn is_pointer(&self) -> bool {
        match self {
            Type::Pointer(_) => true,
            Type::MutablePointer(_) => true,
            _ => false,
        }
    }

    pub fn sizeof(&self) -> u64 {
        return 4;
    }
}

impl From<QualifiedName> for Type {
    fn from(path: QualifiedName) -> Self {
        Type::Named(path)
    }
}
