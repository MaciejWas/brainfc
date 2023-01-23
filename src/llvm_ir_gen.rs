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

mod move_block;
mod modify_block;
mod write_char;

use std::cell::Cell;

use crate::app::Args;
use crate::lexer::Op;
use crate::parser::{Program, Block};

const WRITE_FN_NAME: &str = "write";
const READ_FN_NAME: &str = "read";
const MODIFY_FN_NAME: &str = "modify";
const MOVE_FN_NAME: &str = "move";
const ZERO_FN_NAME: &str = "zero";

struct Libc<'ctxt> {
    getchar_fn: FunctionValue<'ctxt>,
    putchar_fn: FunctionValue<'ctxt>,
    calloc_fn: FunctionValue<'ctxt>
}

struct TapeFunctions<'a> {
    move_fn: FunctionValue<'a>,
    modify_fn: FunctionValue<'a>,
    read_fn: FunctionValue<'a>,
    write_fn: FunctionValue<'a>,
    zero_fn: FunctionValue<'a>
}

struct LLVMBuilder<'ctxt, 'a> {
    context: &'ctxt Context,
    module: &'a Module<'ctxt>,
    builder: &'a Builder<'ctxt>,

    move_block: MoveBlock<'ctxt, 'a>,
    modify_block: ModifyBlock<'ctxt, 'a>,

    loop_id: Cell<u16>,

    libc: Option<Libc<'ctxt>>,
    tape_fncs: Option<TapeFunctions<'ctxt>>,

    tape: PointerValue<'ctxt>,
    tape_pos: PointerValue<'ctxt>,

    args: Args,
}

impl <'ctxt, 'a> LLVMBuilder<'ctxt, 'a> {
    fn new(context: &'ctxt Context, module: &'a Module<'ctxt>, builder: &'a Builder<'ctxt>, args: Args) -> Self {
        let (tape, tape_pos) = Self::create_global_variables(context, module);

        let move_block = move_block::MoveBlock {  context, module, builder, tape, tape_pos };
        let modify_block = modify_block::ModifyBlock {  context, module, builder, tape, tape_pos };
        let write_char = write_char::WriteChar {  context, module, builder, tape, tape_pos };
        
        LLVMBuilder {
            move_block,
            modify_block,
            context,
            module,
            builder,
            loop_id: Cell::new(0),
            args,
            libc: None,
            tape_fncs: None,
            tape,
            tape_pos
        }
    }

    fn build_entry_block(&self, f: FunctionValue) -> BasicBlock {
        let entry_block = self.context.append_basic_block(f, "entry");
        self.builder.position_at_end(entry_block);
        entry_block
    }

    fn create_write_char_fn(&self) {
        let write_fn = self.module.add_function(
            WRITE_FN_NAME,  
            self.context.void_type().fn_type(&[], false), 
            None
        );

        self.build_entry_block(write_fn);

        let current_tape_pos = self.build_load_tape_pos();
        let ptr_to_value = unsafe {
            self.builder.build_gep(
                self.tape, 
                &[self.context.i32_type().const_int(0, false), current_tape_pos],
                "ptr_to_value"
            )
        };
        let value = self.builder.build_load(ptr_to_value, "value").into_int_value();
        self.builder.build_call(self.libc.as_ref().unwrap().putchar_fn, &[value.into()], "_");
        self.builder.build_return(None);
    }

    fn create_zero_fn(&self) {
        let zero_fn = self.module.add_function(
            ZERO_FN_NAME,  
            self.context.void_type().fn_type(&[], false), 
            None
        );

        let entry_block = self.build_entry_block(zero_fn);

        let current_tape_pos = self.build_load_tape_pos();
        let ptr_to_value = self.build_pointer_to_value(current_tape_pos);

        self.builder.build_store(ptr_to_value, self.context.i32_type().const_zero());
        self.builder.build_return(None);
    }

    fn create_read_char_fn(&self) {
        let read_fn = self.module.add_function(
            READ_FN_NAME,  
            self.context.void_type().fn_type(&[], false), 
            None
        );

        let entry_block = self.build_entry_block(read_fn);

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

        let input = self.builder
            .build_call(
                self.libc.as_ref().unwrap().getchar_fn, 
                &[], 
                "input"
            )
            .try_as_basic_value()
            .unwrap_left();
        self.builder.build_store(ptr_to_value, input);
        self.builder.build_return(None);
    }

    fn create_global_variables<'m>(context: &'m Context, module: &Module<'m>) -> (PointerValue<'m>, PointerValue<'m>) {
        let i32_type = context.i32_type();
        let i32_arr_type = i32_type.array_type(30_000);
        let addr_space = Some(AddressSpace::default());

        let tape = module
            .add_global(i32_arr_type, addr_space, "tape");
        tape.set_initializer(&i32_type.array_type(30_000).const_zero());

        let tape_pos = module.add_global(i32_type, addr_space, "tape_block");
        tape_pos.set_initializer(&i32_type.const_zero());

        (tape.as_pointer_value(), tape_pos.as_pointer_value())
    }

    fn build_load_tape_pos(&self) -> IntValue<'ctxt> {
        self.builder
            .build_load(self.tape_pos, "tape_pos")
            .into_int_value()
    }

    fn build_load_value(&self) -> IntValue<'ctxt> {
        let current_tape_pos = self.build_load_tape_pos();
        let ptr = self.build_pointer_to_value(current_tape_pos);
        self.build_load_value_from_pointer(ptr)
    }

    fn build_pointer_to_value(&self, current_tape_pos: IntValue) -> PointerValue<'ctxt> {
        unsafe {
            self.builder.build_gep(
                self.tape, 
                &[self.context.i32_type().const_int(0, false), current_tape_pos],
                "ptr_to_value"
            )
        }
    }

    fn build_load_value_from_pointer(&self, ptr: PointerValue) -> IntValue<'ctxt> {
        self.builder.build_load(ptr, "value").into_int_value()
    }

    fn compare_with_zero(&self, val: IntValue<'ctxt>) -> IntValue<'ctxt> {
        self.builder.build_int_compare(
            inkwell::IntPredicate::NE,
            val,
            self.context.i8_type().const_zero(),
            "compare value at pointer to zero",
        )
    }

    fn build_jump(&self, jump_size: i32) {
        let loop_block_name = format!("jump_loop");
        let cont_block_name = format!("jump_cont");

        let main_fn = self.module.get_function("main").unwrap();
        let loop_block = self.context.append_basic_block(main_fn, loop_block_name.as_str());
        let cont_block = self.context.append_basic_block(main_fn, cont_block_name.as_str());

        let value = self.build_load_value();
        let cmp = self.compare_with_zero(value);
        
        self.builder.block

        self.builder.build_conditional_branch(cmp, loop_block, cont_block);
        self.builder.position_at_end(loop_block);

        let phi = self.builder.build_phi(self.context.i32_type(), "next_val")
            .add_incoming([  ]);
        

    }

    fn build_multiply(&self, ops: Vec<(i32, i32)>) {
        let current_tape_pos = self.builder
            .build_load(self.tape_pos, "tape_pos")
            .into_int_value();

        let ptr_to_base_value = unsafe {
            self.builder.build_gep(
                self.tape, 
                &[self.context.i32_type().const_int(0, false), current_tape_pos],
                "ptr_to_value"
            )
        };

        let base_value = self.builder.build_load(ptr_to_base_value, "base_value").into_int_value();


        for (diff, multiply_val) in ops {

            // Load value to be modified
            let modification_pos = self.builder.build_int_add(
                current_tape_pos, 
                self.context.i32_type().const_int(diff as u64, false), 
                "modification_pos"
            );
            let ptr_to_value = unsafe {
                self.builder.build_gep(
                    self.tape, 
                    &[self.context.i32_type().const_int(0, false), modification_pos],
                    "ptr_to_value"
                )
            };
            let value = self.builder
                .build_load(ptr_to_value, "value")
                .into_int_value(); 

            // Calculate the new value
            let multipled = self.builder.build_int_mul(
                base_value,
                self.context.i32_type().const_int(multiply_val as u64, false),
                "multipled"
            );
            let new_value = self.builder.build_int_add(
                value,
                multipled,
                "new_value"
            );

            // Store new value
            self.builder.build_store(ptr_to_value, new_value);
        }

        self.builder.build_store(ptr_to_base_value, self.context.i32_type().const_int(0, false));

    }

    fn build_loop_start(&self) -> (BasicBlock, BasicBlock) {
        let curr_loop_id = self.loop_id.get() + 1;
        self.loop_id.set(curr_loop_id);

        let loop_block_name = format!("loop_{}", curr_loop_id);
        let cont_block_name = format!("cont_{}", curr_loop_id);

        let main_fn = self.module.get_function("main").unwrap();
        let loop_block = self.context.append_basic_block(main_fn, loop_block_name.as_str());
        let cont_block = self.context.append_basic_block(main_fn, cont_block_name.as_str());


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
        let value = self.builder.build_load(ptr_to_value, "value").into_int_value();


        let cmp = self.builder.build_int_compare(
            inkwell::IntPredicate::NE,
            value,
            self.context.i8_type().const_zero(),
            "compare value at pointer to zero",
        );

        self.builder.build_conditional_branch(cmp, loop_block, cont_block);
        self.builder.position_at_end(loop_block);


        (loop_block, cont_block)
    }

    fn build_loop_end(&self, loop_block: BasicBlock, cont_block: BasicBlock) {
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
        let value = self.builder.build_load(ptr_to_value, "value").into_int_value();

        let cmp = self.builder.build_int_compare(
            inkwell::IntPredicate::NE,
            value,
            self.context.i8_type().const_zero(),
            "compare value at pointer to zero",
        );

        self.builder.build_conditional_branch(cmp, loop_block, cont_block);
        self.builder.position_at_end(cont_block);
    }


    fn create_main_fn(&self) {
        let main_fn_type = self.context.i32_type().fn_type(&[], false);
        let main_fn = self.module.add_function("main", main_fn_type, Some(Linkage::External));
        let main_entry = self.context.append_basic_block(main_fn, "entry");

        self.builder.position_at_end(main_entry);
    }

    fn load_libc(&mut self) {
        let calloc_fn_type = self.context
            .i8_type()
            .ptr_type(AddressSpace::default())
            .fn_type(
                &[self.context.i64_type().into(), self.context.i64_type().into()], 
                false
            );
        let calloc_fn = self.module
            .add_function("calloc", calloc_fn_type, Some(Linkage::External));
        let getchar_fn_type = self.context.i32_type().fn_type(&[], false);
        let getchar_fn = self.module
            .add_function("getchar", getchar_fn_type, Some(Linkage::External));

        let putchar_fn_type = self.context.i32_type().fn_type(&[self.context.i32_type().into()], false);
        let putchar_fn = self.module
            .add_function("putchar", putchar_fn_type, Some(Linkage::External));

        self.libc = Some(Libc { getchar_fn, putchar_fn, calloc_fn } )

    }

    fn load_tape_functions(&mut self) {
        self.tape_fncs = Some(TapeFunctions {
            move_fn: self.module.get_function(MOVE_FN_NAME).unwrap(),
            modify_fn: self.module.get_function(MODIFY_FN_NAME).unwrap(),
            read_fn: self.module.get_function(READ_FN_NAME).unwrap(),
            write_fn: self.module.get_function(WRITE_FN_NAME).unwrap(),
            zero_fn: self.module.get_function(ZERO_FN_NAME).unwrap(),
        })
    }


    fn finalize(&self) {
        self.builder.build_return(Some(&self.context.i32_type().const_int(0, false)));
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
            .ok_or_else(|| "Unable to create target machine!".to_string()).unwrap();
        
        let output = self.args.output.as_ref()
            .unwrap_or(&std::path::PathBuf::new().with_file_name("out")).with_extension("o");
        
        target_machine
            .write_to_file(&self.module, FileType::Object, output.as_path())
            .unwrap();

    }

    fn compile(&self, program: &Program) {
        for block in program {
            self.compile_block(&block);
        }
    }

    fn compile_instructions(&self, instrs: &Vec<Op>) {
        for i in instrs {
            self.compile_instruction(&i)
        }
    }

    fn compile_instruction(&self, i: &Op) {
        let tape_fncs = self.tape_fncs.as_ref().unwrap();

        match &i {
            Op::Move(x) => { self.move_block.build(*x) },
            Op::Modify(x) => {self.builder.build_call(tape_fncs.modify_fn, &[ self.context.i32_type().const_int(*x as u64, false).into() ], "_");},
            Op::Inp(x) => { for _ in 0..(*x) {self.builder.build_call(tape_fncs.read_fn, &[], "_"); }},
            Op::Outp(x) => { for _ in 0..(*x) {self.builder.build_call(tape_fncs.write_fn, &[], "_"); }},
             _ => {}
        }
    }

    fn compile_block(&self, block: &Block) {
        use Block::*;
        match &block {
            Simple(ref instrs) => self.compile_instructions(instrs),
            Loop(ref program) => self.compile_loop(program),
            Reset { .. } => { self.builder.build_call(self.tape_fncs.as_ref().unwrap().zero_fn, &[], "_" ); }
            Multiply { ops } => { self.build_multiply(ops.clone()) }
            _ => {}
        }
    }

    fn compile_loop(&self, program: &Program) {
        let (loop_block, cont_block) = self.build_loop_start();

        self.compile(program);

        self.build_loop_end(loop_block, cont_block);
    }
}

pub fn compile(program: Program, args: Args) {
    let context = Context::create();
    let module = context.create_module("brainf");
    let builder = context.create_builder();

    let mut llvm_builder = LLVMBuilder::new(&context, &module, &builder, args);
    
    llvm_builder.load_libc();

    llvm_builder.create_move_fn();
    llvm_builder.create_modify_fn();
    llvm_builder.create_read_char_fn();
    llvm_builder.create_write_char_fn();
    llvm_builder.create_zero_fn();
    llvm_builder.create_main_fn();
    llvm_builder.load_tape_functions();
    
    llvm_builder.compile(&program);

    llvm_builder.finalize();
    llvm_builder.create_binary();
}
