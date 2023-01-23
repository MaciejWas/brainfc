use crate::parser::{Block, Program};
use crate::optimizations::{  ResetValOpt, MultiplyOpt };
use crate::optimizations::base::*;

pub struct Optimizer {
    opts: Vec<Box<dyn Optimization>>
} 

impl Optimizer {
    pub fn new() -> Self {
        Optimizer { opts: vec![ Box::new(ResetValOpt {}), Box::new( MultiplyOpt ) ] }
    } 

    fn optimize_block(&self, mut block: Block) -> Block {
        // first, optimize subblocks
        block = block.map_loop(|b| self.optimize_block(b));
        
        // then optimize the block itself
        for opt in self.opts.iter() {
            if let Some(optimized) = opt.apply(&block) {
                return optimized
            }
        }

        block
    } 

    pub fn optimize(&self, p: Program) -> Program {
        p.into_iter().map(|block| self.optimize_block(block)).collect()
    }
}
