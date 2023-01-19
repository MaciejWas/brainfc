use inkwell::context::{Context, };
use inkwell::types::{IntType, PointerType};
use inkwell::builder::{Builder, };
use inkwell::basic_block::BasicBlock;
use inkwell::module::{Linkage, Module, };
use inkwell::values::{PointerValue, FunctionValue};
use inkwell::targets::{
    CodeModel, FileType, InitializationConfig, RelocMode, Target, TargetMachine,
};
use inkwell::AddressSpace;
use inkwell::OptimizationLevel;




struct TapeFunctions<'a> {
    move_fn: FunctionValue<'a>,
    modify_fn: FunctionValue<'a>,
    read_fn: FunctionValue<'a>,
    write_fn: FunctionValue<'a>,
    zero_fn: FunctionValue<'a>
}




impl<'ctxt> TapeFunctions<'ctxt> {
    pub fn create<'a>(context: &'ctxt Context, module: &'a Module<'ctxt>, builder: &'a Builder<'ctxt>) {


    }

}
