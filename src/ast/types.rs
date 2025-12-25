#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PrimitiveType {
    Bool,
    Int,
    Uint,
    Float,
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
pub struct FunctionType {
    pub params: Vec<(String, Type)>,
    pub ret_ty: Box<Type>,
    pub varidic: bool,
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
}

impl Type {
    pub fn int() -> Self {
        Self::Primitive(PrimitiveType::Int)
    }

    pub fn uint() -> Self {
        Self::Primitive(PrimitiveType::Uint)
    }

    pub fn float() -> Self {
        Self::Primitive(PrimitiveType::Float)
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

    pub fn function(params: Vec<(String, Type)>, ret_ty: Type, varidic: bool) -> Self {
        Self::Function(FunctionType {
            params,
            ret_ty: Box::new(ret_ty),
            varidic,
        })
    }
}
