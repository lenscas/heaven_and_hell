#![allow(dead_code)]
use rand::Rng;
use crate::Block;

fn node_to_block(value: usize) -> usize {
    value * 2 + 1
}
enum Dir {
    Down,
    Up,
    Left,
    Right,
}
/// resulting size is (X * 2 + 1, Y * 2 + 1)
pub fn generate_maze(node_size: (usize, usize)) -> Vec<Vec<Block>> {
    let size_blocks = (node_size.0 * 2 + 1, node_size.1 * 2 + 1);
    let mut result_blocks = vec![vec![Block::Dirt; size_blocks.1]; size_blocks.0];
    let mut position_stack = Vec::new();
    let mut point_free = vec![vec![true; size_blocks.1]; size_blocks.0];
    let mut rnd = rand::thread_rng();
    //Start
    let node = (rnd.gen_range(0, node_size.0), rnd.gen_range(0, node_size.1));
    position_stack.push(node);
    point_free[node.0][node.1] = false;
    result_blocks[node_to_block(node.0)][node_to_block(node.1)] = Block::PlayerStart;
    //Fill
    while position_stack.len() > 0 {
        let pos = *position_stack.last().unwrap();
        let mut choices = Vec::<Dir>::new();
        //Checking
        let pos_new = (pos.0, pos.1.wrapping_sub(1));
        if pos_new.1 != usize::MAX && point_free[pos_new.0][pos_new.1] {
            choices.push(Dir::Down);
        }
        let pos_new = (pos.0, pos.1 + 1);
        if pos_new.1 != node_size.1 && point_free[pos_new.0][pos_new.1] {
            choices.push(Dir::Up);
        }
        let pos_new = (pos.0.wrapping_sub(1), pos.1);
        if pos_new.0 != usize::MAX && point_free[pos_new.0][pos_new.1] {
            choices.push(Dir::Left);
        }
        let pos_new = (pos.0 + 1, pos.1);
        if pos_new.0 != node_size.0 && point_free[pos_new.0][pos_new.1] {
            choices.push(Dir::Right);
        }
        //Acting
        if choices.len() > 0 {
            let node = {
                match &choices[rnd.gen_range(0, choices.len())] {
                    Dir::Down => {
                        let result_node = (pos.0, pos.1 - 1);
                        result_blocks[node_to_block(result_node.0)][node_to_block(result_node.1) + 1] = Block::Air;
                        result_node
                    },
                    Dir::Up => {
                        let result_node = (pos.0, pos.1 + 1);
                        result_blocks[node_to_block(result_node.0)][node_to_block(result_node.1) - 1] = Block::Air;
                        result_node
                    },
                    Dir::Left => {
                        let result_node = (pos.0 - 1, pos.1);
                        result_blocks[node_to_block(result_node.0) + 1][node_to_block(result_node.1)] = Block::Air;
                        result_node
                    },
                    Dir::Right => {
                        let result_node = (pos.0 + 1, pos.1);
                        result_blocks[node_to_block(result_node.0) - 1][node_to_block(result_node.1)] = Block::Air;
                        result_node
                    },
                }
            };
            position_stack.push(node);
            point_free[node.0][node.1] = false;
            result_blocks[node_to_block(node.0)][node_to_block(node.1)] = Block::Air;
        } else {
            position_stack.pop();
        }
    }
    result_blocks
}