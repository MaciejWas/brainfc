use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;

use inkwell::values::PointerValue;

pub struct Reset<'ctxt, 'a> {
    context: &'ctxt Context,
    module: &'a Module<'ctxt>,
    builder: &'a Builder<'ctxt>,

    tape: PointerValue<'ctxt>,
    tape_pos: PointerValue<'ctxt>,
}

impl<'ctxt, 'a> Reset<'ctxt, 'a> {
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

    pub fn build(&self) {
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

        self.builder
            .build_store(ptr_to_value, self.context.i32_type().const_int(0, false));
    }
}
