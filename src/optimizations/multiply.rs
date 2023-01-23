use crate::optimizations::base::Optimization;
use crate::parser::Block;
use crate::lexer::Op;

use log::debug;

pub struct MultiplyOpt;

impl Optimization for MultiplyOpt {
    fn apply(&self, block: &Block) -> Option<Block> {
        let Block::Loop(subblocks) = block else {
            return None;
        };

        let [Block::Simple(ref ops)] = subblocks[..] else {
            return None;
        };


        let sum_of_moves = ops.iter()
            .fold(0, |acc, op| match op { Op::Move(x) => acc + x, _ => acc } );
        if sum_of_moves != 0 {
            return None;
        }

        let is_all_moves_and_mods = ops.iter()
            .all(|op| match op { Op::Move(_) => true, Op::Modify(_) => true, _ => false });
        if is_all_moves_and_mods {
            let max_cell_distance = ops
                .iter()
                .fold(
                    0, 
                    |acc, op| match op { Op::Move(x) => acc + x.abs(), _ => acc }
                );

            // Initiate an array and record all call modifications which happen in the loop
            let mut pos = max_cell_distance as usize;
            let mut init = vec![0; pos + 1];
            init.append(&mut vec![0; pos]);

            for op in ops {
                match op {
                    Op::Move(x) => { pos = ((pos as i16) + x) as usize }
                    Op::Modify(x) => { init[pos] += x } 
                    _ => unreachable!()
                }
                println!("init: {:?}", init);
            }

            if init[max_cell_distance as usize] != -1 { // loop must reset base value
                return None
            }

            let ops = init.into_iter()
                .enumerate()
                .filter( |(i, val)| *val != 0 )
                .filter( |(i, val)|  *i != max_cell_distance as usize)
                .map(|(i, val)| ((i as i32 - max_cell_distance as i32), val as i32) )
                .collect::<Vec<(i32, i32)>>();
            println!("DONE");

            let has_negative_mods = ops.iter().filter(|(i, val)| *i < 0).next().is_some();
            if has_negative_mods {
                println!("rejected");
                return None
            }
            println!("accepted");
            return Some( Block::Multiply { ops } )
        }
 
        None
    }
}
