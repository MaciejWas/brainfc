use std::cell::Cell;

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

pub struct Loop<'ctxt, 'a> {
    pub context: &'ctxt Context,
    pub module: &'a Module<'ctxt>,
    pub builder: &'a Builder<'ctxt>,
    pub tape: PointerValue<'ctxt>,
    pub tape_pos: PointerValue<'ctxt>,
    pub loop_id: Cell<u16>,
}

impl<'ctxt, 'a> Loop<'ctxt, 'a> {
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
            loop_id: Cell::new(0),
        }
    }

    pub fn build_loop_start(&self) -> (BasicBlock, BasicBlock) {
        let curr_loop_id = self.loop_id.get() + 1;
        self.loop_id.set(curr_loop_id);

        let loop_block_name = format!("loop_{}", curr_loop_id);
        let cont_block_name = format!("cont_{}", curr_loop_id);

        let main_fn = self.module.get_function("main").unwrap();
        let loop_block = self
            .context
            .append_basic_block(main_fn, loop_block_name.as_str());
        let cont_block = self
            .context
            .append_basic_block(main_fn, cont_block_name.as_str());

        let current_tape_pos = self
            .builder
            .build_load(self.tape_pos, "tape_pos")
            .into_int_value();

        let ptr_to_value = unsafe {
            self.builder.build_gep(
                self.tape,
                &[
                    self.context.i32_type().const_int(0, false),
                    current_tape_pos,
                ],
                "ptr_to_value",
            )
        };
        let value = self
            .builder
            .build_load(ptr_to_value, "value")
            .into_int_value();

        let cmp = self.builder.build_int_compare(
            inkwell::IntPredicate::NE,
            value,
            self.context.i8_type().const_zero(),
            "compare value at pointer to zero",
        );

        self.builder
            .build_conditional_branch(cmp, loop_block, cont_block);
        self.builder.position_at_end(loop_block);

        (loop_block, cont_block)
    }

    pub fn build_loop_end(&self, loop_block: BasicBlock, cont_block: BasicBlock) {
        let current_tape_pos = self
            .builder
            .build_load(self.tape_pos, "tape_pos")
            .into_int_value();

        let ptr_to_value = unsafe {
            self.builder.build_gep(
                self.tape,
                &[
                    self.context.i32_type().const_int(0, false),
                    current_tape_pos,
                ],
                "ptr_to_value",
            )
        };
        let value = self
            .builder
            .build_load(ptr_to_value, "value")
            .into_int_value();

        let cmp = self.builder.build_int_compare(
            inkwell::IntPredicate::NE,
            value,
            self.context.i8_type().const_zero(),
            "compare value at pointer to zero",
        );

        self.builder
            .build_conditional_branch(cmp, loop_block, cont_block);
        self.builder.position_at_end(cont_block);
    }
}
