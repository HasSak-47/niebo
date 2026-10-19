use super::typeid::TypeID;
use std::collections::{HashMap, HashSet};

use crate::general::naming::QualifiedName;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PrimitiveTypeShape {
    Bool,
    Int(usize),
    Uint(usize),
    Float(usize),
    Void,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct StructTypeShape {
    pub members: Vec<(String, TypeID)>,
}

impl StructTypeShape {
    pub fn get_member_type(&self, name: &String) -> Option<&TypeID> {
        for mem in &self.members {
            if mem.0 == *name {
                return Some(&mem.1);
            }
        }

        return None;
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnionTypeShape {
    pub members: Vec<(String, TypeID)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VariantTypeShape {
    pub members: Vec<(String, TypeID)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Params {
    name: Option<String>,
    ty: TypeID,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionType {
    pub params: Vec<Params>,
    pub ret_ty: Box<TypeShape>,
    pub varidic: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TemplateID {
    pub restrictions: Vec<QualifiedName>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeKind {
    Primitive(PrimitiveTypeShape),
    Struct(StructTypeShape),
    Array(Box<TypeShape>),
    Union(UnionTypeShape),
    Variant(VariantTypeShape),
    Pointer(Box<TypeShape>),
    MutablePointer(Box<TypeShape>),
    Reference(Box<TypeShape>),
    MutableReference(Box<TypeShape>),
    Function(FunctionType),
    Template(TemplateID),
    // alias to a type
    Alias {
        name: QualifiedName,
        to: Box<TypeShape>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeShape {
    pub kind: TypeKind,
}

impl TypeShape {
    fn no_type() -> TypeShape {
        TypeShape {
            kind: TypeKind::Primitive(PrimitiveTypeShape::Void),
        }
    }

    pub fn alias(name: QualifiedName, ty: TypeShape) -> Self {
        Self {
            kind: TypeKind::Alias {
                name,
                to: Box::new(ty),
            },
            ..TypeShape::no_type()
        }
    }

    pub fn primitive(p: PrimitiveTypeShape) -> Self {
        Self {
            kind: TypeKind::Primitive(p),
            ..TypeShape::no_type()
        }
    }

    pub fn int() -> Self {
        Self {
            kind: TypeKind::Primitive(PrimitiveTypeShape::Int(32)),
            ..TypeShape::no_type()
        }
    }

    pub fn uint() -> Self {
        Self {
            kind: TypeKind::Primitive(PrimitiveTypeShape::Uint(32)),
            ..TypeShape::no_type()
        }
    }

    pub fn int_p(prec: usize) -> Self {
        Self {
            kind: TypeKind::Primitive(PrimitiveTypeShape::Int(prec)),
            ..TypeShape::no_type()
        }
    }

    pub fn uint_p(prec: usize) -> Self {
        Self {
            kind: TypeKind::Primitive(PrimitiveTypeShape::Uint(prec)),
            ..TypeShape::no_type()
        }
    }

    pub fn float() -> Self {
        Self {
            kind: TypeKind::Primitive(PrimitiveTypeShape::Float(32)),
            ..TypeShape::no_type()
        }
    }

    pub fn float_p(prec: usize) -> Self {
        Self {
            kind: TypeKind::Primitive(PrimitiveTypeShape::Float(prec)),
            ..TypeShape::no_type()
        }
    }

    pub fn bool() -> Self {
        Self {
            kind: TypeKind::Primitive(PrimitiveTypeShape::Bool),
            ..TypeShape::no_type()
        }
    }

    pub fn void() -> Self {
        Self {
            kind: TypeKind::Primitive(PrimitiveTypeShape::Void),
            ..TypeShape::no_type()
        }
    }

    pub fn struct_t(members: Vec<(String, TypeID)>) -> Self {
        Self {
            kind: TypeKind::Struct(StructTypeShape { members }),
            ..TypeShape::no_type()
        }
    }

    pub fn union(members: Vec<(String, TypeID)>) -> Self {
        Self {
            kind: TypeKind::Union(UnionTypeShape { members }),
            ..TypeShape::no_type()
        }
    }

    pub fn array(element: TypeShape) -> Self {
        Self {
            kind: TypeKind::Array(Box::new(element)),
            ..TypeShape::no_type()
        }
    }

    pub fn pointer(ty: TypeShape) -> Self {
        Self {
            kind: TypeKind::Pointer(Box::new(ty)),
            ..TypeShape::no_type()
        }
    }

    pub fn reference(ty: TypeShape) -> Self {
        Self {
            kind: TypeKind::Reference(Box::new(ty)),
            ..TypeShape::no_type()
        }
    }

    pub fn mutable_pointer(ty: TypeShape) -> Self {
        Self {
            kind: TypeKind::MutablePointer(Box::new(ty)),
            ..TypeShape::no_type()
        }
    }

    pub fn mutable_reference(ty: TypeShape) -> Self {
        Self {
            kind: TypeKind::MutableReference(Box::new(ty)),
            ..TypeShape::no_type()
        }
    }

    pub fn function<I: Into<Params>>(params: Vec<I>, ret_ty: TypeShape, varidic: bool) -> Self {
        Self {
            kind: TypeKind::Function(FunctionType {
                params: params.into_iter().map(|a| a.into()).collect(),
                ret_ty: Box::new(ret_ty),
                varidic,
            }),
            ..TypeShape::no_type()
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
