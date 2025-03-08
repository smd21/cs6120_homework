use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
};

use crate::briltypes::*;
use itertools::Itertools;

#[derive(Eq, PartialEq, Hash, Copy, Clone)]
struct BlockName(u32);
#[derive(Clone)]
struct Block {
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

#[derive(Default)]
struct CFG {
    preds_adj_list: HashMap<Block, Vec<Block>>,
    succs_adj_list: HashMap<Block, Vec<Block>>,
}
impl CFG {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn get_predecessors(&self, block: &Block) -> Option<&Vec<Block>> {
        self.preds_adj_list.get(block)
    }
    pub fn get_successors(&self, block: &Block) -> Option<&Vec<Block>> {
        self.succs_adj_list.get(block)
    }
    /// Adds predecessor pred to block's predecessor list.
    pub fn add_predecessor(&mut self, block: &Block, pred: Block) {
        match self.preds_adj_list.get_mut(block) {
            Some(b) => {
                b.push(pred);
            }
            None => {
                let preds = vec![pred];
                self.preds_adj_list.insert(block.clone(), preds);
            }
        }
    }
    pub fn add_successor(&mut self, block: &Block, succ: Block) {
        match self.succs_adj_list.get_mut(block) {
            Some(b) => {
                b.push(succ);
            }
            None => {
                let succs = vec![succ];
                self.succs_adj_list.insert(block.clone(), succs);
            }
        }
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
        self.cfg = CFG::new();
        self.blocks.iter().for_each(|block| {
            let last_insn: &InsnType = block.instructions.last().unwrap();
            if let InsnType::Terminator {
                op,
                labels,
                args: _,
            } = last_insn
            {
                if op == "br" {
                    labels.iter().for_each(|label| {
                        let succ = *(label_blocks.get(label).unwrap());
                        // update successors
                        self.cfg.add_successor(block, succ.clone());
                        // update predecessors
                        self.cfg.add_predecessor(succ, block.clone());
                    });
                }
            }
        });
    }

    pub fn construct_dominators(&self) -> HashMap<&Block, HashSet<&Block>> {
        // worklist algorithm - copy from lesson 5
        let mut worklist: Vec<&Block> = Vec::new();
        self.blocks
            .split_first()
            .unwrap()
            .1
            .iter()
            .for_each(|block| {
                worklist.push(block);
            });
        let mut inputs: HashMap<&Block, HashSet<&Block>> = HashMap::new();
        let mut outputs: HashMap<&Block, HashSet<&Block>> = HashMap::new();
        self.blocks.iter().for_each(|block| {
            inputs.insert(block, HashSet::new());
            outputs.insert(block, HashSet::from_iter(self.blocks.iter())); // argh this is bad how do i not do thisssss
        });
        outputs.insert(
            self.blocks.first().unwrap(),
            HashSet::from_iter([self.blocks.first().unwrap()]),
        );

        while let Some(b) = worklist.pop() {
            let preds = self.cfg.get_predecessors(b).unwrap();
            // apply merge function (set intersection)
            let folded = preds
                .iter()
                .fold(HashSet::from_iter(self.blocks.iter()), |acc, el| {
                    let pred_outs = outputs.get(el).unwrap();
                    acc.intersection(pred_outs).cloned().collect()
                });
            // inputs.update b to folded
            let mut new_out = folded.clone(); // do this here bc ownership funnies
            inputs.insert(b, folded);

            // transfer function
            let old_out = outputs.get(b).unwrap().clone();
            new_out.insert(b);
            if old_out.len() != new_out.len() {
                worklist.push(b);
            }
            // insert at bottom again because borrowing funnies
            outputs.insert(b, new_out);
        }
        outputs
    }
    pub fn dominance_frontier<'a>(
        &self,
        block: &Block,
        dominators: HashMap<&'a Block, HashSet<&Block>>,
    ) -> HashSet<&'a Block> {
        let mut frontier: HashSet<&Block> = HashSet::new();
        let mut dominated: HashSet<&Block> = HashSet::new();
        dominators.iter().for_each(|(b, d)| {
            if d.contains(block) {
                dominated.insert(*b);
            }
        });

        dominators.iter().for_each(|(b, d)| {
            if !dominated.contains(*b) && !d.intersection(&dominated).collect_vec().is_empty() {
                frontier.insert(*b);
            }
        });
        frontier
    }
}
