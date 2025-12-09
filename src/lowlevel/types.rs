#[derive(Clone)]
pub enum PrimitiveType {
    Int,
    Uint,
    Float,
    String,
    Void,
}

#[derive(Clone)]
pub struct StructType {
    pub members: Vec<(String, Type)>,
}

#[derive(Clone)]
pub struct UnionType {
    pub members: Vec<(String, Type)>,
}

#[derive(Clone)]
pub struct FunctionType {
    pub params: Vec<(String, Type)>,
    pub ret_ty: Box<Type>,
}

#[derive(Clone)]
pub struct AliasType {
    pub ident: String,
    pub ty: Box<Type>,
}

#[derive(Clone)]
pub enum Type {
    Primitive(PrimitiveType),
    Struct(StructType),
    Union(UnionType),
    Alias(AliasType),
    Pointer(Box<Type>),
    Reference(Box<Type>),
    Function(FunctionType),
}

impl Type {}
