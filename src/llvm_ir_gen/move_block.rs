use inkwell::basic_block::BasicBlock;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::{Linkage, Module};
use inkwell::targets::{
    CodeModel, FileType, InitializationConfig, RelocMode, Target, TargetMachine,
};
use inkwell::types::{IntType, PointerType};
use inkwell::values::{FunctionValue, IntValue, PointerValue};
use inkwell::AddressSpace;
use inkwell::OptimizationLevel;

use crate::app::Args;
use crate::lexer::Op;
use crate::parser::{Block, Program};

pub struct MoveBlock<'ctxt, 'a> {
    context: &'ctxt Context,
    module: &'a Module<'ctxt>,
    builder: &'a Builder<'ctxt>,

    tape: PointerValue<'ctxt>,
    tape_pos: PointerValue<'ctxt>,
}

impl<'ctxt, 'a> MoveBlock<'ctxt, 'a> {
    pub fn new(
        context: &'ctxt Context,
        module: &'a Module<'ctxt>,
        builder: &'a Builder<'ctxt>,
        tape: PointerValue<'ctxt>,
        tape_pos: PointerValue<'ctxt>,
    ) -> Self {
        Self {
            context,
            module,
            builder,
            tape,
            tape_pos,
        }
    }

    pub fn build(&self, move_value: i16) {
        let current_pos = self
            .builder
            .build_load(self.tape_pos, "tape_pos")
            .into_int_value();

        let new_tape_pos = self.builder.build_int_add(
            self.context.i32_type().const_int(move_value as u64, false),
            current_pos,
            "new_value",
        );

        self.builder.build_store(self.tape_pos, new_tape_pos);
    }
}
