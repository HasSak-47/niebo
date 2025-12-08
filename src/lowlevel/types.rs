pub enum PrimitiveType {
    Int,
    Uint,
    Float,
    String,
    Void,
}

pub struct StructType {
    ident: String,
    members: Vec<(String, Type)>,
}

pub struct UnionType {
    ident: String,
    members: Vec<(String, Type)>,
}

pub struct FunctionType {
    ident: String,
    params: Vec<(String, Type)>,
    ret_ty: Box<Type>,
}

pub enum Type {
    Primitive(PrimitiveType),
    Struct(StructType),
    Union(UnionType),
    Pointer(Box<Type>),
    Function(FunctionType),
}

impl Type {
    pub fn get_function_type()
}
