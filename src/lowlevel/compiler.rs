use inkwell::{builder::Builder, context::Context, module::Module};

use super::repr::Repr;

struct Compiler {
    pub context: Context,
}

impl Compiler {
    pub fn new() -> Self {
        Self {
            context: Context::create(),
        }
    }

    pub fn new_module(&self, repr: Repr, ident: String) -> ModuleCompiler {
        ModuleCompiler {
            context: &self.context,
            module: self.context.create_module(&ident),
            builder: self.context.create_builder(),
            ident,
            repr,
        }
    }
}

pub struct ModuleCompiler<'ctx> {
    pub ident: String,
    pub context: &'ctx Context,
    pub module: Module<'ctx>,
    pub builder: Builder<'ctx>,
    pub repr: Repr,
}

impl<'ctx> ModuleCompiler<'ctx> {
    pub fn compile(&self) {}
}
