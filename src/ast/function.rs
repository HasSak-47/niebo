use std::collections::HashMap;

use super::expressions::{block::Block, *};
use super::*;
use crate::general::types::*;

#[derive(Debug, PartialEq)]
pub struct FunctionBuilder {
    pub ident: String,
    pub ret_ty: Option<Type>,
    pub params: Vec<(Option<String>, Type)>,
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

    pub fn set_body(mut self, body: Block) -> Self {
        self.body = Some(body);
        return self;
    }

    pub fn set_ret_ty<Ty: Into<Type>>(mut self, ty: Ty) -> Self {
        self.ret_ty = Some(ty.into());
        return self;
    }

    pub fn set_constant(mut self) -> Self {
        self.constant = true;
        return self;
    }

    pub fn set_varidic(mut self, varidic: bool) -> Self {
        self.varidic = varidic;
        return self;
    }

    pub fn varidic(mut self) -> Self {
        self.varidic = true;
        return self;
    }

    pub fn add_param<S: Into<String>, ITy: Into<Type>>(mut self, ident: S, ty: ITy) -> Self {
        let ident = ident.into();
        self.params.push((Some(ident), ty.into()));
        return self;
    }

    pub fn add_anon_param<ITy: Into<Type>>(mut self, ty: ITy) -> Self {
        self.params.push((None, ty.into()));

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

    pub fn build_c_function(self) -> Definition {
        let fun_c = FunctionC {
            varidic: self.varidic,
            return_ty: self.ret_ty,
            constant: false,
            parameters: self.params,
        };
        return Definition {
            kind: DefinitionKind::FunctionC(fun_c),
            visibility: Visibility::Public,
            name: self.ident,
        };
    }

    pub fn build_def(self) -> Definition {
        assert!(self.body.is_some());
        let body = self.body.unwrap();
        let f = FunctionDefinition {
            body,
            decl: FunctionDeclaration {
                constant: self.constant,
                parameters: self
                    .params
                    .into_iter()
                    .map(|(n, t)| (n.unwrap(), t))
                    .collect(),
                return_ty: self.ret_ty,
            },
        };

        return Definition {
            kind: DefinitionKind::FunctionDefinition(f),
            visibility: self.visibility,
            name: self.ident,
        };
    }
}

// since it lives in the AST the types are not yet resolved and are treated as paths
#[derive(Debug, Clone, PartialEq)]
pub struct FunctionDefinition {
    pub decl: FunctionDeclaration,
    pub body: Block,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionDeclaration {
    pub constant: bool,
    pub return_ty: Option<Type>,
    pub parameters: Vec<(String, Type)>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionC {
    pub varidic: bool,
    pub constant: bool,
    pub return_ty: Option<Type>,
    pub parameters: Vec<(Option<String>, Type)>,
}
