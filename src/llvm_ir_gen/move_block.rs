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

pub struct MoveBlock<'ctxt, 'a> {
    context: &'ctxt Context, module: &'a Module<'ctxt>, builder: &'a Builder<'ctxt>, 

    tape: PointerValue<'ctxt>,
    tape_pos: PointerValue<'ctxt>,

}

impl<'ctxt, 'a> MoveBlock<'ctxt, 'a> {

    pub fn build(&self, move_value: i16) {

        let current_pos = self.builder
            .build_load(self.tape_pos, "tape_pos")
            .into_int_value();

        let new_tape_pos = self.builder.build_int_add(
            self.context.i32_type().const_int(move_value as u64, false), 
            current_pos, 
            "new_value"
        );

        self.builder.build_store(self.tape_pos, new_tape_pos);
    }
}




