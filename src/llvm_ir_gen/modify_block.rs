use inkwell::context::{Context, };
use inkwell::types::{IntType, PointerType};
use inkwell::builder::{Builder, };
use inkwell::basic_block::BasicBlock;
use inkwell::module::{Linkage, Module, };
use inkwell::values::{PointerValue, FunctionValue, IntValue};
use inkwell::targets::{
    CodeModel, FileType, InitializationConfig, RelocMode, Target, TargetMachine,
};
use inkwell::AddressSpace;
use inkwell::OptimizationLevel;


use crate::app::Args;
use crate::lexer::Op;
use crate::parser::{Program, Block};

pub struct ModifyBlock<'ctxt, 'a> {
    context: &'ctxt Context, module: &'a Module<'ctxt>, builder: &'a Builder<'ctxt>, 

    tape: PointerValue<'ctxt>,
    tape_pos: PointerValue<'ctxt>,

}

impl<'ctxt, 'a> ModifyBlock<'ctxt, 'a> {

    pub fn build(&self, modify_val: i16) {

        let current_tape_pos = self.builder
            .build_load(self.tape_pos, "tape_pos")
            .into_int_value();

        let ptr_to_value = unsafe {
            self.builder.build_gep(
                self.tape, 
                &[self.context.i32_type().const_int(0, false), current_tape_pos],
                "ptr_to_value"
            )
        };
        
        let old_value = self.builder.build_load(ptr_to_value, "value").into_int_value();
        let new_value = self.builder.build_int_add(
            old_value, 
            self.context.i32_type().const_int(modify_val as u64, false),
            "new_value"
        );

        self.builder.build_store(ptr_to_value, new_value);
        self.builder.build_return(None);
    }
}




