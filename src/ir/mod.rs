use crate::general::types::Type;

pub enum ScopeNode {
    Scope(Box<ScopeNode>),
}

pub enum Instruction {
    Init(String, Type),
    If,
    Break,
    Continue,
    Loop,
}
