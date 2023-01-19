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


use std::cell::Cell;

use crate::app::Args;
use crate::lexer::Token;
use crate::optimizer::{OptimizedBlock, OptimizedProgram, Instruction};

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
        LLVMBuilder {
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

    fn create_move_fn(&self) {
        let move_fn = self.module.add_function(
            MOVE_FN_NAME,  
            self.context.void_type().fn_type(&[self.context.i32_type().into()], false), 
            None
        );
        let move_value = move_fn.get_first_param().unwrap().into_int_value();

        let entry_block = self.context.append_basic_block(move_fn, "entry");
        self.builder.position_at_end(entry_block);

        let current_tape_pos = self.builder
            .build_load(self.tape_pos, "tape_pos")
            .into_int_value();
        let new_tape_pos = self.builder
            .build_int_add(move_value, current_tape_pos, "new_value");
        self.builder.build_store(self.tape_pos, new_tape_pos);
        self.builder.build_return(None);
    }

    fn create_modify_fn(&self) {
        let modify_fn = self.module.add_function(
            MODIFY_FN_NAME,  
            self.context.void_type().fn_type(&[self.context.i32_type().into()], false), 
            None
        );
        let diff = modify_fn.get_first_param().unwrap().into_int_value();

        let entry_block = self.context.append_basic_block(modify_fn, "entry");
        self.builder.position_at_end(entry_block);

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
        let new_value = self.builder.build_int_add(value, diff, "new_value");
        self.builder.build_store(ptr_to_value, new_value);
        self.builder.build_return(None);
    }

    fn create_write_char_fn(&self) {
        let write_fn = self.module.add_function(
            WRITE_FN_NAME,  
            self.context.void_type().fn_type(&[], false), 
            None
        );

        let entry_block = self.context.append_basic_block(write_fn, "entry");
        self.builder.position_at_end(entry_block);

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
        self.builder.build_call(self.libc.as_ref().unwrap().putchar_fn, &[value.into()], "_");
        self.builder.build_return(None);
    }

    fn create_zero_fn(&self) {
        let zero_fn = self.module.add_function(
            ZERO_FN_NAME,  
            self.context.void_type().fn_type(&[], false), 
            None
        );

        let entry_block = self.context.append_basic_block(zero_fn, "entry");
        self.builder.position_at_end(entry_block);

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
        self.builder.build_store(ptr_to_value, self.context.i32_type().const_zero());
        self.builder.build_return(None);
    }

    fn create_read_char_fn(&self) {
        let read_fn = self.module.add_function(
            READ_FN_NAME,  
            self.context.void_type().fn_type(&[], false), 
            None
        );

        let entry_block = self.context.append_basic_block(read_fn, "entry");
        self.builder.position_at_end(entry_block);

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

    fn compile(&self, program: &OptimizedProgram) {
        for block in program {
            self.compile_block(&block);
        }
    }

    fn compile_instructions(&self, instrs: &Vec<Instruction>) {
        for i in instrs {
            self.compile_instruction(&i)
        }
    }

    fn compile_instruction(&self, i: &Instruction) {
        use Instruction::*;
        
        let tape_fncs = self.tape_fncs.as_ref().unwrap();

        match &i {
            Move(x) => {self.builder.build_call(tape_fncs.move_fn, &[ self.context.i32_type().const_int(*x as u64, false).into() ], "_");},
            Modify(x) => {self.builder.build_call(tape_fncs.modify_fn, &[ self.context.i32_type().const_int(*x as u64, false).into() ], "_");},
            Other(token) => match token {
                Token::In => { self.builder.build_call(tape_fncs.read_fn, &[], "_"); },
                Token::Out => { self.builder.build_call(tape_fncs.write_fn, &[], "_"); },
                _ => {}
            }
        }
    }

    fn compile_block(&self, block: &OptimizedBlock) {
        use OptimizedBlock::*;

        match &block {
            Simple(ref instrs) => self.compile_instructions(instrs),
            Loop(ref program) => self.compile_loop(program),
            ResetVal => { self.builder.build_call(self.tape_fncs.as_ref().unwrap().zero_fn, &[], "_" ); }
            _ => {}
        }
    }

    fn compile_loop(&self, program: &OptimizedProgram) {
        let (loop_block, cont_block) = self.build_loop_start();

        self.compile(program);

        self.build_loop_end(loop_block, cont_block);
    }
}

pub fn compile(program: OptimizedProgram, args: Args) {
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
