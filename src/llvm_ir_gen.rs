use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::{Linkage, Module};
use inkwell::targets::{
    CodeModel, FileType, InitializationConfig, RelocMode, Target, TargetMachine,
};

use inkwell::values::{IntValue, PointerValue};
use inkwell::AddressSpace;
use inkwell::OptimizationLevel;

mod loops;
mod modify_block;
mod move_block;
mod read_char;
mod reset;
mod write_char;

use loops::Loop;
use modify_block::ModifyBlock;
use move_block::MoveBlock;
use read_char::ReadChar;
use reset::Reset;
use write_char::WriteChar;

use crate::app::Args;
use crate::lexer::Op;
use crate::parser::{Block, Program};

struct LLVMBuilder<'ctxt, 'a> {
    context: &'ctxt Context,
    module: &'a Module<'ctxt>,
    builder: &'a Builder<'ctxt>,

    move_block: MoveBlock<'ctxt, 'a>,
    modify_block: ModifyBlock<'ctxt, 'a>,
    write_char: WriteChar<'ctxt, 'a>,
    read_char: ReadChar<'ctxt, 'a>,
    reset: Reset<'ctxt, 'a>,
    loops: Loop<'ctxt, 'a>,

    tape: PointerValue<'ctxt>,
    tape_pos: PointerValue<'ctxt>,

    args: Args,
}

impl<'ctxt, 'a> LLVMBuilder<'ctxt, 'a> {
    fn new(
        context: &'ctxt Context,
        module: &'a Module<'ctxt>,
        builder: &'a Builder<'ctxt>,
        args: Args,
    ) -> Self {
        let (tape, tape_pos) = Self::create_global_variables(context, module);

        let move_block = MoveBlock::new(context, module, builder, tape, tape_pos);
        let modify_block = ModifyBlock::new(context, module, builder, tape, tape_pos);
        let write_char = WriteChar::new(context, module, builder, tape, tape_pos);
        let reset = Reset::new(context, module, builder, tape, tape_pos);
        let read_char = ReadChar::new(context, module, builder, tape, tape_pos);
        let loops = Loop::new(context, module, builder, tape, tape_pos);

        LLVMBuilder {
            move_block,
            modify_block,
            write_char,
            reset,
            read_char,
            loops,

            context,
            module,
            builder,
            args,
            tape,
            tape_pos,
        }
    }

    fn create_global_variables<'m>(
        context: &'m Context,
        module: &Module<'m>,
    ) -> (PointerValue<'m>, PointerValue<'m>) {
        let i32_type = context.i32_type();
        let i32_arr_type = i32_type.array_type(30_000);
        let addr_space = Some(AddressSpace::default());

        let tape = module.add_global(i32_arr_type, addr_space, "tape");
        tape.set_initializer(&i32_type.array_type(30_000).const_zero());

        let tape_pos = module.add_global(i32_type, addr_space, "tape_block");
        tape_pos.set_initializer(&i32_type.const_zero());

        (tape.as_pointer_value(), tape_pos.as_pointer_value())
    }

    fn compare_with_zero(&self, val: IntValue<'ctxt>) -> IntValue<'ctxt> {
        self.builder.build_int_compare(
            inkwell::IntPredicate::NE,
            val,
            self.context.i8_type().const_zero(),
            "compare value at pointer to zero",
        )
    }

    fn build_multiply(&self, ops: Vec<(i32, i32)>) {
        let current_tape_pos = self
            .builder
            .build_load(self.tape_pos, "tape_pos")
            .into_int_value();

        let ptr_to_base_value = unsafe {
            self.builder.build_gep(
                self.tape,
                &[
                    self.context.i32_type().const_int(0, false),
                    current_tape_pos,
                ],
                "ptr_to_value",
            )
        };

        let base_value = self
            .builder
            .build_load(ptr_to_base_value, "base_value")
            .into_int_value();

        for (diff, multiply_val) in ops {
            // Load value to be modified
            let modification_pos = self.builder.build_int_add(
                current_tape_pos,
                self.context.i32_type().const_int(diff as u64, false),
                "modification_pos",
            );
            let ptr_to_value = unsafe {
                self.builder.build_gep(
                    self.tape,
                    &[
                        self.context.i32_type().const_int(0, false),
                        modification_pos,
                    ],
                    "ptr_to_value",
                )
            };
            let value = self
                .builder
                .build_load(ptr_to_value, "value")
                .into_int_value();

            // Calculate the new value
            let multipled = self.builder.build_int_mul(
                base_value,
                self.context
                    .i32_type()
                    .const_int(multiply_val as u64, false),
                "multipled",
            );
            let new_value = self.builder.build_int_add(value, multipled, "new_value");

            // Store new value
            self.builder.build_store(ptr_to_value, new_value);
        }

        self.builder.build_store(
            ptr_to_base_value,
            self.context.i32_type().const_int(0, false),
        );
    }

    fn create_main_fn(&self) {
        let main_fn_type = self.context.i32_type().fn_type(&[], false);
        let main_fn = self
            .module
            .add_function("main", main_fn_type, Some(Linkage::External));
        let main_entry = self.context.append_basic_block(main_fn, "entry");

        self.builder.position_at_end(main_entry);
    }

    fn load_libc(&mut self) {
        let calloc_fn_type = self
            .context
            .i8_type()
            .ptr_type(AddressSpace::default())
            .fn_type(
                &[
                    self.context.i64_type().into(),
                    self.context.i64_type().into(),
                ],
                false,
            );
        self.module
            .add_function("calloc", calloc_fn_type, Some(Linkage::External));
        let getchar_fn_type = self.context.i32_type().fn_type(&[], false);
        self.module
            .add_function("getchar", getchar_fn_type, Some(Linkage::External));

        let putchar_fn_type = self
            .context
            .i32_type()
            .fn_type(&[self.context.i32_type().into()], false);
        self.module
            .add_function("putchar", putchar_fn_type, Some(Linkage::External));
    }

    fn finalize(&self) {
        self.builder
            .build_return(Some(&self.context.i32_type().const_int(0, false)));
    }

    fn create_binary(&self) {
        if self.args.show_llvm_ir {
            self.module.print_to_stderr();
        }

        Target::initialize_all(&InitializationConfig::default());

        let target_triple = TargetMachine::get_default_triple();
        let cpu = TargetMachine::get_host_cpu_name().to_string();
        let features = TargetMachine::get_host_cpu_features().to_string();

        let target = Target::from_triple(&target_triple).unwrap();

        let target_machine = target
            .create_target_machine(
                &target_triple,
                &cpu,
                &features,
                OptimizationLevel::Default,
                RelocMode::Default,
                CodeModel::Default,
            )
            .ok_or_else(|| "Unable to create target machine!".to_string())
            .unwrap();

        let output = self
            .args
            .output
            .as_ref()
            .unwrap_or(&std::path::PathBuf::new().with_file_name("out"))
            .with_extension("o");

        target_machine
            .write_to_file(self.module, FileType::Object, output.as_path())
            .unwrap();
    }

    fn compile(&self, program: &Program) {
        for block in program {
            self.compile_block(block);
        }
    }

    fn compile_instructions(&self, instrs: &Vec<Op>) {
        for i in instrs {
            self.compile_instruction(i)
        }
    }

    fn compile_instruction(&self, op: &Op) {
        match &op {
            Op::Move(x) => self.move_block.build(*x),
            Op::Modify(x) => self.modify_block.build(*x),
            Op::Inp(x) => {
                for _ in 0..(*x) {
                    self.read_char.build()
                }
            }
            Op::Outp(x) => {
                for _ in 0..(*x) {
                    self.write_char.build()
                }
            }
            _ => unreachable!(),
        }
    }

    fn compile_block(&self, block: &Block) {
        use Block::*;
        match &block {
            Simple(ref instrs) => self.compile_instructions(instrs),
            Loop(ref program) => self.compile_loop(program),
            Reset { .. } => self.reset.build(),
            Multiply { ops } => self.build_multiply(ops.clone()),
            _ => {}
        }
    }

    fn compile_loop(&self, program: &Program) {
        let (loop_block, cont_block) = self.loops.build_loop_start();

        self.compile(program);

        self.loops.build_loop_end(loop_block, cont_block);
    }
}

pub fn compile(program: Program, args: Args) {
    let context = Context::create();
    let module = context.create_module("brainf");
    let builder = context.create_builder();

    let mut llvm_builder = LLVMBuilder::new(&context, &module, &builder, args);

    llvm_builder.load_libc();

    llvm_builder.compile(&program);

    llvm_builder.finalize();
    llvm_builder.create_binary();
}
