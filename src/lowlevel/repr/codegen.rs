use inkwell::{
    AddressSpace,
    module::Linkage,
    values::{
        AnyValue, AnyValueEnum, ArrayValue, BasicMetadataValueEnum, BasicValue, BasicValueEnum,
        FunctionValue, PointerValue,
    },
};

use crate::lowlevel::{compiler::ModuleCompiler, repr::registry::SymbolRegistry, types::Type};

trait Expression {
    fn get_expression_type<'a, 'ctx>(
        &self,
        symbols: &SymbolRegistry<'ctx>,
        compiler: &ModuleCompiler<'a, 'ctx>,
    ) -> Type;

    fn code_gen<'a, 'ctx, S: AsRef<str>>(
        &self,
        symbols: SymbolRegistry<'ctx>,
        compiler: &ModuleCompiler<'a, 'ctx>,
        assing_to: Option<Box<Self>>,
    ) -> AnyValueEnum<'ctx>;
}
