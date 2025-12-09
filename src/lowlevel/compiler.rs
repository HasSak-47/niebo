use inkwell::{builder::Builder, context::Context, module::Module};

use super::repr::Repr;

pub struct Compiler<'a, 'ctx> {
    pub context: &'a Context,
    pub modules: Vec<ModuleCompiler<'a, 'ctx>>,
}

impl<'a, 'ctx> Compiler<'a, 'ctx> {
    pub fn new(context: &'a Context) -> Self {
        Self {
            context,
            modules: Vec::new(),
        }
    }

    pub fn new_module(&self, repr: Repr, ident: String) -> ModuleCompiler {
        return ModuleCompiler {
            context: self.context,
            module: self.context.create_module(&ident),
            builder: self.context.create_builder(),
            ident,
            repr,
        };
    }
}

pub struct ModuleCompiler<'a, 'ctx>
where
    'a: 'ctx,
{
    pub ident: String,
    pub context: &'a Context,
    pub module: Module<'ctx>,
    pub builder: Builder<'ctx>,
    pub repr: Repr,
}

impl<'a, 'ctx> ModuleCompiler<'a, 'ctx> {
    pub fn compile(&self) {
        self.repr.code_gen(self);
    }
}
