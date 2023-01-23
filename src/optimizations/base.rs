use crate::parser::{Block, Program};

pub trait Optimization {
    fn apply(&self, prog: &Block) -> Option<Block>;
}

