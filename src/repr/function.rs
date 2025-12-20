use super::expressions::*;
use super::types::*;
use super::*;

pub struct FunctionBuilder {
    ident: String,
    ret_ty: Type,
    params: Vec<(String, Type)>,
    varidic: bool,
    constant: bool,
    body: Option<Block>,
}

impl FunctionBuilder {
    pub fn new<S: AsRef<str>>(ident: S, ret_ty: Type) -> Self {
        Self {
            constant: false,
            ident: ident.as_ref().to_string(),
            ret_ty,
            params: Vec::new(),
            varidic: false,
            body: None,
        }
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

    pub fn add_definition(self, dec: Definition) -> Self {
        self.add_statement(Statement::Definition(dec))
    }

    pub fn add_statement(mut self, stmt: Statement) -> Self {
        if let Some(body) = &mut self.body {
            body.add_statement(stmt);
        } else {
            self.body = Some(Block::new())
        }

        return self;
    }

    pub fn build_def(mut self) -> Function {
        assert!(self.body.is_none());
        let body = self.body.unwrap();
        Function {
            body,
            constant: self.constant,
            parameters: self.params,
            varidic: self.varidic,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Function {
    // TODO: add restriction to make only c functions varidic
    varidic: bool,
    constant: bool,
    // return_ty: Type,
    parameters: Vec<(String, Type)>,
    body: Block,
}
