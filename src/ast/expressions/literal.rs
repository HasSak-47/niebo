use std::fmt::{Debug, Display};

#[derive(Debug, Clone, PartialEq)]
pub enum LiteralInfo {
    Integer {
        signed: Option<bool>,
        precision: Option<u64>,
    },

    String,

    Float {
        precision: Option<u64>,
    },
}

#[derive(Clone, PartialEq)]
pub struct Literal {
    pub info: LiteralInfo,
    pub data: String,
}

impl Display for Literal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.data)
    }
}

impl Debug for Literal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "lit: ({})", self.data)
    }
}

impl Literal {
    pub fn string<S: Into<String>>(value: S) -> Self {
        let value = value.into();
        return Literal {
            info: LiteralInfo::String,
            data: value,
        };
    }

    pub fn float<S: Into<String>>(value: S, precision: Option<u64>) -> Self {
        let value = value.into();
        return Literal {
            info: LiteralInfo::Float { precision },
            data: value,
        };
    }

    pub fn integer<S: Into<String>>(
        value: S,
        signed: Option<bool>,
        precision: Option<u64>,
    ) -> Self {
        let value = value.into();
        return Literal {
            info: LiteralInfo::Integer { signed, precision },
            data: value,
        };
    }
}

impl<T> From<T> for Literal
where
    T: Into<String>,
{
    fn from(value: T) -> Self {
        let value = value.into();
        return Self {
            info: LiteralInfo::String,
            data: value,
        };
    }
}
