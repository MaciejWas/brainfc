use crate::parser::Block;

pub trait Optimization {
    fn apply(&self, prog: &Block) -> Option<Block>;
}
