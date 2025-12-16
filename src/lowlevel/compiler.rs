use inkwell::{basic_block::BasicBlock, builder::Builder, context::Context, module::Module};

use crate::lowlevel::repr::ir::Statement;

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

    pub fn new_module(&self, ident: String) -> ModuleCompiler<'a, 'ctx> {
        let module = ModuleCompiler {
            current_block: Vec::new(),
            context: self.context,
            module: self.context.create_module(&ident),
            builder: self.context.create_builder(),
            ident,
        };

        return module;
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
    pub current_block: Vec<BasicBlock<'ctx>>,
}

impl<'a, 'ctx> ModuleCompiler<'a, 'ctx> {
    pub fn compile(&mut self, repr: &Repr) {
        repr.code_gen(self);
    }

    pub fn get_ll_code(&self) -> String {
        return self.module.to_string();
    }

    pub fn add_block(&mut self, block: BasicBlock<'ctx>) {
        self.current_block.push(block);
    }
    pub fn remove_block(&mut self) {
        self.current_block.pop();
    }
}
