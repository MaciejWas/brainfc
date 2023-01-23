use log::{debug, info};
use super::lexer::Op;

#[derive(Debug)]
pub enum Block {
    // Blocks coming from parser
    Simple(Vec<Op>), Loop(Program),

    // Blocks coming from optimizer
    Reset { offset: i32 },
    JmpLoop { jmp_size: i8 },
    Multiply { ops: Vec<(i32, i32)> }
}
impl Block {
    fn empty() -> Block {
        Block::Simple(Vec::new())
    }

    pub fn is_loop(&self) -> bool {
        match self {
            Block::Loop(_) => true,
            _ => false
        } 
    }

    pub fn map_loop(self, f: impl Fn(Block) -> Block) -> Block {
        if let Block::Loop( subblocks ) = self {
            return Block::Loop( subblocks.into_iter().map( f ).collect() )
        }

        self
    }
}

pub type Program = Vec<Block>;

struct ProgramBuilder {
    parsing_stack: Vec<Program>,
    err: Option<String>
}

impl ProgramBuilder {
    fn new() -> ProgramBuilder {
        ProgramBuilder { parsing_stack: vec![ Program::new() ], err: None }
    }

    fn add(&mut self, operation: Op) {
        use Op::*;

        if self.err.is_some() {
            return;
        }

        match operation {
            LBr => self.start_loop(),
            RBr => self.finish_loop(),
            t => self.add_to_latest_block(t)
        }
    }

    fn add_to_latest_block(&mut self, t: Op) {
        debug!("add_to_latest_block: {:?}", t);

        let Some(latest_program) = self.parsing_stack.last_mut() else {
            self.err = Some("Unexpected empty parsing stack".to_string());
            return;
        };

        if latest_program.is_empty() || latest_program.last().unwrap().is_loop() {
            latest_program.push(Block::empty())
        }

        let Block::Simple(latest_block) = latest_program.last_mut().unwrap() else {
            self.err = Some("Failed to handle loop".to_string());
            return;
        };

        latest_block.push(t);
    }

    fn finish_loop(&mut self) {
        let finished_loop: Program = self.parsing_stack.pop().unwrap();
        debug!("finished loop: {:?}", finished_loop);
        self.parsing_stack.last_mut().unwrap().push(Block::Loop(finished_loop))
    }

    fn start_loop(&mut self) {
        debug!("started loop");
        self.parsing_stack.push(Program::new());
    }

    fn finalize(mut self) -> Result<Program, String> {
        if let Some(err) = self.err {
            return Err(err);
        }
        if self.parsing_stack.len() == 1 {
            return self.parsing_stack.pop().ok_or("Unexpected empty parsing stack".to_string())
        } else {
            return Err("Parsing finalized before clearing parsing stack.".to_string())
        }
    }
}



pub fn parse(tokens: Vec<Op>) -> Result<Program, String> {
    let mut builder = ProgramBuilder::new();
    tokens.into_iter().for_each( |t| builder.add(t) );
    builder.finalize()
}
