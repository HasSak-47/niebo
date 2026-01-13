use std::collections::HashMap;

use super::expressions::{block::Block, *};
use super::*;
use crate::general::types::*;

pub struct FunctionBuilder {
    pub ident: String,
    pub ret_ty: Option<Type>,
    pub params: Vec<(String, Type)>,
    pub varidic: bool,
    pub constant: bool,
    pub body: Option<Block>,
    pub visibility: Visibility,
}

impl FunctionBuilder {
    pub fn new<S: AsRef<str>>(ident: S) -> Self {
        Self {
            visibility: Visibility::Private,
            constant: false,
            ident: ident.as_ref().to_string(),
            ret_ty: None,
            params: Vec::new(),
            varidic: false,
            body: None,
        }
    }

    pub fn set_ret_ty(mut self, ty: Type) -> Self {
        self.ret_ty = Some(ty);
        return self;
    }

    pub fn set_constant(mut self) -> Self {
        self.constant = true;
        return self;
    }

    pub fn varidic(mut self) -> Self {
        self.varidic = true;
        return self;
    }

    pub fn add_param<S: AsRef<str>>(mut self, ident: S, ty: Type) -> Self {
        self.params.push((ident.as_ref().to_string(), ty));
        return self;
    }

    /**
    adds a definition to the function body e.j.
    let x = 10;
    */
    pub fn add_definition(self, dec: Definition) -> Self {
        self.add_statement(Statement::Definition(dec))
    }

    pub fn add_statement(mut self, stmt: Statement) -> Self {
        if let Some(body) = &mut self.body {
            body.add_statement(stmt);
        } else {
            let mut body = Block::new();
            body.add_statement(stmt);
            self.body = Some(body);
        }

        return self;
    }

    pub fn build_def(self) -> Definition {
        assert!(self.body.is_some());
        let body = self.body.unwrap();
        let f = Function {
            body,
            constant: self.constant,
            parameters: self.params,
            varidic: self.varidic,
            return_ty: self.ret_ty.unwrap_or(Type::void()),
        };

        return Definition {
            kind: DefinitionKind::Function(f),
            visibility: self.visibility,
            name: self.ident,
        };
    }
}

#[derive(Debug, Clone)]
pub struct Function {
    // TODO: add restriction to make only c functions varidic
    pub varidic: bool,
    pub constant: bool,
    pub return_ty: Type,
    pub parameters: Vec<(String, Type)>,
    pub body: Block,
}
