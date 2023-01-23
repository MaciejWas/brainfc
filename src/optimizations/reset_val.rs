use crate::lexer::Op;
use crate::optimizations::base::Optimization;
use crate::parser::{Block, Program};

pub struct ResetValOpt;
impl Optimization for ResetValOpt {
    fn apply(&self, block: &Block) -> Option<Block> {
        let Block::Loop(subblocks) = block else {
            return None
        };

        if subblocks.len() != 1 {
            return None;
        };

        let Block::Simple(ref ops) = subblocks[0] else {
            return None
        };

        if let [Op::Modify(-1)] = ops[..] {
            return Some(Block::Reset { offset: 0 });
        }

        return None;
    }
}
