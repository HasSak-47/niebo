use crate::{ast::Path, general::types::Type};

#[derive(Debug, Clone)]
pub enum TypeName {
    Type(Type),
    Name(Path),
}

impl From<Path> for TypeName {
    fn from(path: Path) -> Self {
        return Self::Name(path);
    }
}

impl From<Type> for TypeName {
    fn from(ty: Type) -> Self {
        return Self::Type(ty);
    }
}
