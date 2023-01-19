use crate::parser::{Block, Program};
use crate::lexer::Token;

#[derive(Debug, PartialEq, Eq)]
pub enum Instruction {
    Move(i8), Modify(i8), Other(Token),
}

impl Instruction {
    fn is_move(&self) -> bool {
        match self {
            Self::Move(_) => true,
            _ => false
        }
    }

    fn unwrap(&self) -> i8 {
        match self {
            Self::Move(x) => *x,
            Self::Modify(x) => *x,
            _ => panic!("not move or modify")
        }
    }
}

impl From<Token> for Instruction {
    fn from(t: Token) -> Instruction {
        match t {
            Token::Plus => Instruction::Modify(1),
            Token::Minus => Instruction::Modify(-1),
            Token::Left => Instruction::Move(-1),
            Token::Right => Instruction::Move(1),
            t => Instruction::Other(t) // temporary solution
        }
    }
}

#[derive(Debug)]
pub enum OptimizedBlock {
    Simple(Vec<Instruction>),

    Loop(Vec<OptimizedBlock>),
    ResetVal,
    LoopModifyAndGoBack( Vec<(i8, i8)> ) // (difference between loop start and target pointer, change
                                         // to apply)
}

pub type OptimizedProgram = Vec<OptimizedBlock>;


fn cumsum_moves(instructions: &Vec<Instruction>) -> i8 {
    instructions.iter().filter(|i| i.is_move()).map(|i| i.unwrap()).sum()
}

fn squash_block(block: Block) -> OptimizedBlock {
    match block {
        Block::Simple(tokens) => OptimizedBlock::Simple(
            tokens.into_iter()
                  .fold(Vec::new(), |mut instructions, next_token| { 
                        match (instructions.last_mut(), next_token) {
                            (Some(Instruction::Move(x)), Token::Right) => *x += 1,
                            (Some(Instruction::Move(x)), Token::Left) => *x -= 1,
                            (Some(Instruction::Modify(x)), Token::Plus) => *x += 1,
                            (Some(Instruction::Modify(x)), Token::Minus) => *x -= 1,
                            (_, token) => instructions.push(Instruction::from(token))
                        } 
                        instructions
                  })
        ),
        Block::Loop(prog) => OptimizedBlock::Loop(squash(prog))
    }
}

fn squash(program: Program) -> OptimizedProgram {
    program.into_iter().map(squash_block).collect()
}


fn optimize_block(b: OptimizedBlock) -> OptimizedBlock {
    use OptimizedBlock::*;

    match &b {
        Loop(blocks) => {
            if blocks.len() == 1 {
                if let Simple(ref instructions) = blocks[0] {
                    if instructions.len() == 1 && instructions[0].eq(&Instruction::Modify(-1)) {
                        return ResetVal; // optimization: [-] should immediately reset current
                                         // value to zero
                    }

                    if cumsum_moves(instructions) == 0 {
                        // Todo: modify and go back
                    }
                }
            }
        }
        _ => {}
    }

    return b
}

fn optimize_optimized(p: OptimizedProgram) -> OptimizedProgram {
    p.into_iter().map(optimize_block).collect()
}

pub fn optimize(p: Program) -> OptimizedProgram {
    optimize_optimized(squash(p))
}
