use inkwell::{context::Context, types::BasicType, AddressSpace};

use crate::lowlevel::compiler::ModuleCompiler;

#[derive(Debug, Clone)]
pub enum PrimitiveType {
    Int,
    Uint,
    Float,
    String,
    Void,
}

#[derive(Debug, Clone)]
pub struct StructType {
    pub members: Vec<(String, Type)>,
}

#[derive(Debug, Clone)]
pub struct UnionType {
    pub members: Vec<(String, Type)>,
}

#[derive(Debug, Clone)]
pub struct FunctionType {
    pub params: Vec<(String, Type)>,
    pub ret_ty: Box<Type>,
    pub varidic: bool,
}

impl FunctionType {
    pub fn build_parameter_type<'a>(
        ty: &Type,
        context: &'a Context,
    ) -> inkwell::types::BasicMetadataTypeEnum<'a> {
        match ty {
            Type::Primitive(primitive) => match primitive {
                PrimitiveType::Void => panic!("cannot use voids as params"),
                PrimitiveType::Int => context.i32_type().as_basic_type_enum(),
                PrimitiveType::String => context
                    .ptr_type(AddressSpace::default())
                    .as_basic_type_enum(),
                _ => todo!(),
            },
            _ => todo!(),
        }
        .into()
    }
    pub fn build_fn_type<'ctx>(
        &self,
        context: &'ctx Context,
    ) -> inkwell::types::FunctionType<'ctx> {
        let params: Vec<_> = self
            .params
            .iter()
            .map(|(_, ty)| Self::build_parameter_type(ty, context))
            .collect();

        match &*self.ret_ty {
            Type::Primitive(p) => match p {
                PrimitiveType::Void => {
                    return context
                        .void_type()
                        .fn_type(params.as_slice(), self.varidic.clone());
                }
                PrimitiveType::Int => {
                    return context
                        .i32_type()
                        .fn_type(params.as_slice(), self.varidic.clone());
                }
                _ => todo!(),
            },
            _ => todo!(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct AliasType {
    pub ident: String,
    pub ty: Box<Type>,
}

#[derive(Debug, Clone)]
pub enum Type {
    Primitive(PrimitiveType),
    Struct(StructType),
    Array(Box<Type>),
    Union(UnionType),
    Alias(AliasType),
    Pointer(Box<Type>),
    Reference(Box<Type>),
    Function(FunctionType),
}

impl Type {
    pub fn to_llvm_basic_type<'a, 'ctx>(
        &self,
        compiler: &ModuleCompiler<'a, 'ctx>,
    ) -> inkwell::types::BasicTypeEnum<'ctx> {
        match self {
            Self::Primitive(PrimitiveType::Int) => compiler.context.i32_type().as_basic_type_enum(),
            _ => todo!(),
        }
    }

    pub fn int() -> Self {
        Self::Primitive(PrimitiveType::Int)
    }

    pub fn uint() -> Self {
        Self::Primitive(PrimitiveType::Uint)
    }

    pub fn float() -> Self {
        Self::Primitive(PrimitiveType::Float)
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

    pub fn alias(ident: impl Into<String>, ty: Type) -> Self {
        Self::Alias(AliasType {
            ident: ident.into(),
            ty: Box::new(ty),
        })
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
