use std::{collections::HashMap, hash::Hash};

use crate::briltypes::*;
use itertools::Itertools;

#[derive(Eq, PartialEq, Hash)]
struct BlockName(u32);
struct Block {
    label: Option<String>,
    name: BlockName,
    instructions: Vec<InsnType>,
}
impl PartialEq for Block {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}
impl Eq for Block {}
impl Hash for Block {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

struct CFG {
    preds_adj_list: HashMap<Block, Vec<Block>>,
    succs_adj_list: HashMap<Block, Vec<Block>>,
}
impl CFG {
    pub fn init(&mut self) {
        self.preds_adj_list = HashMap::new();
        self.succs_adj_list = HashMap::new();
    }
    pub fn get_predecessors(&self, block: &Block) -> Option<&Vec<Block>> {
        self.preds_adj_list.get(block)
    }
    pub fn get_successors(&self, block: &Block) -> Option<&Vec<Block>> {
        self.succs_adj_list.get(block)
    }
}
struct FuncContext {
    cfg: CFG,
    blocks: Vec<Block>,
}

impl FuncContext {
    pub fn make_cfg(&mut self, function: Function) {
        let mut name_counter: u32 = 0;
        let mut label_blocks: HashMap<&String, &Block> = HashMap::new();
        self.blocks = Vec::new();
        let mut current_block: Vec<InsnType> = Vec::new();
        // mildly cursed iterator to blockify
        function
            .instructions
            .iter()
            .for_each(|instruction| match instruction {
                InsnType::Label { label: _ } => {
                    if !current_block.is_empty() {
                        let new_block: Block = Block {
                            label: None,
                            name: BlockName(name_counter),
                            instructions: current_block.clone(),
                        };
                        self.blocks.push(new_block);
                        current_block.clear();
                        name_counter += 1;
                    }
                    current_block.push(instruction.clone());
                }
                InsnType::Constant {
                    op: _,
                    dest: _,
                    insn_type: _,
                    value: _,
                } => current_block.push(instruction.clone()),
                InsnType::Terminator {
                    op,
                    labels: _,
                    args: _,
                } => {
                    dbg!(op);
                    current_block.push(instruction.clone());
                    let new_block: Block = Block {
                        label: None,
                        name: BlockName(name_counter),
                        instructions: current_block.clone(),
                    };
                    self.blocks.push(new_block);
                    current_block.clear();
                    name_counter += 1;
                }
                InsnType::ValOp {
                    op: _,
                    insn_type: _,
                    dest: _,
                    args: _,
                    funcs: _,
                } => current_block.push(instruction.clone()),
            });
        // push the last block
        let new_block: Block = Block {
            label: None,
            name: BlockName(name_counter),
            instructions: current_block.clone(),
        };
        self.blocks.push(new_block);
        current_block.clear();

        // collect labeled blocks
        self.blocks.iter().for_each(|block| {
            let first: &InsnType = block.instructions.first().unwrap();
            // clippy told me to use if let and who am i to question it
            if let InsnType::Label { label } = first {
                label_blocks.insert(label, block);
            }
        });

        // construct cfg
    }
    pub fn construct_dominators() {
        // worklist algorithm - copy from lesson 5
    }
}
