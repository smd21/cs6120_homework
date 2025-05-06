use std::{
    collections::{HashMap, HashSet},
    fmt::Debug,
    hash::Hash,
};

use crate::briltypes::*;
use itertools::Itertools;

#[derive(Eq, PartialEq, Hash, Copy, Clone)]
pub struct BlockName(pub u32);
#[derive(Clone)]
pub struct Block {
    pub name: BlockName,
    pub instructions: Vec<InsnType>,
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
pub struct Cfg {
    preds_adj_list: HashMap<u32, Vec<u32>>, //ill change this later to a struct like blockidx or smthg
    succs_adj_list: HashMap<u32, Vec<u32>>,
}
impl Cfg {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn init(&mut self, blocks: &Vec<Block>) {
        for (idx, _block) in blocks.iter().enumerate() {
            let preds: Vec<u32> = Vec::new();
            let succs: Vec<u32> = Vec::new();
            self.preds_adj_list.insert(idx as u32, preds);
            self.succs_adj_list.insert(idx as u32, succs);
        }
    }
    pub fn get_predecessors(&self, idx: &u32) -> Option<&Vec<u32>> {
        self.preds_adj_list.get(idx)
    }
    pub fn get_successors(&self, idx: &u32) -> Option<&Vec<u32>> {
        self.succs_adj_list.get(idx)
    }
    /// Adds predecessor pred to block's predecessor list.
    pub fn add_predecessor(&mut self, block_idx: &u32, pred_idx: u32) {
        match self.preds_adj_list.get_mut(block_idx) {
            Some(b) => {
                b.push(pred_idx);
            }
            None => {
                let preds = vec![pred_idx];
                self.preds_adj_list.insert(*block_idx, preds);
            }
        }
    }
    pub fn add_successor(&mut self, block: &u32, succ: u32) {
        match self.succs_adj_list.get_mut(block) {
            Some(b) => {
                b.push(succ);
            }
            None => {
                let succs = vec![succ];
                self.succs_adj_list.insert(*block, succs);
            }
        }
    }
}
#[derive(Default)]
pub struct FuncContext {
    args: Option<Vec<String>>,
    name: String,
    fn_type: Option<String>,
    cfg: Cfg,
    pub blocks: Vec<Block>, //update this to be block idx or some struct
    dominators: HashMap<u32, HashSet<u32>>,
    dominance_tree: HashMap<u32, HashSet<u32>>, // this is parent: children and dominators is child: parents lol
}

impl FuncContext {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn init(&mut self, function: &Function) {
        self.name = function.name.clone();
        self.args = function.args.clone();
        self.fn_type = function.func_type.clone();
        self.make_cfg(function);
        self.construct_dominators();
    }
    // fn insert_phi_node(&mut self, blockidx: u32, insn: InsnType) {
    //     let bl = self.blocks.get_mut(blockidx as usize).unwrap();
    //     bl.phi_insns.push(insn);
    // }
    fn make_cfg(&mut self, function: &Function) {
        let mut name_counter: u32 = 0;
        let mut label_blocks: HashMap<&String, &Block> = HashMap::new();
        self.blocks = Vec::new();
        let mut block_indices: HashMap<Block, u32> = HashMap::new();
        let mut current_block: Vec<InsnType> = Vec::new();
        // mildly cursed iterator to blockify
        function
            .instrs
            .iter()
            .for_each(|instruction| match instruction {
                InsnType::Label { label: _ } => {
                    if !current_block.is_empty() {
                        let new_block: Block = Block {
                            name: BlockName(name_counter),
                            instructions: current_block.clone(),
                        };

                        self.blocks.push(new_block.clone());
                        block_indices.insert(new_block, name_counter);

                        current_block.clear();
                        name_counter += 1;
                    }
                    current_block.push(instruction.clone());
                }
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
                    self.blocks.push(new_block.clone());
                    block_indices.insert(new_block, name_counter);

                    current_block.clear();
                    name_counter += 1;
                }
                _ => current_block.push(instruction.clone()),
            });
        // push the last block
        let new_block: Block = Block {
            name: BlockName(name_counter),
            instructions: current_block.clone(),
        };
        self.blocks.push(new_block.clone());
        block_indices.insert(new_block, name_counter);

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
        self.cfg = Cfg::new();
        // create an initialize function that sets each block to an empty hashset
        self.cfg.init(&self.blocks);
        // construct cfg
        for (idx, block) in self.blocks.iter().enumerate() {
            let last_insn: &InsnType = block.instructions.last().unwrap();
            if let InsnType::Terminator {
                op,
                labels,
                args: _,
            } = last_insn
            {
                if op == "br" || op == "jmp" {
                    labels.iter().for_each(|label| {
                        let block_idx = block_indices.get(block).unwrap();

                        let succ = *(label_blocks.get(label).unwrap());
                        let succ_idx = block_indices.get(succ).unwrap();
                        // update successors
                        self.cfg.add_successor(block_idx, *succ_idx);
                        // update predecessors
                        self.cfg.add_predecessor(succ_idx, *block_idx);
                    });
                }
            }
            // if last is not a terminator + not the last block, add as successor/predecessor to next block
            else if idx < self.blocks.len() - 1 {
                // update successors
                self.cfg.add_successor(&(idx as u32), (idx + 1) as u32);
                // update predecessors
                self.cfg.add_predecessor(&((idx + 1) as u32), idx as u32);
            }
        }
    }

    pub fn workflow<T: Eq + Hash + Debug + Clone>(
        &self,
        init: fn(
            &Option<Vec<String>>,
            &Vec<Block>,
        ) -> (HashMap<u32, HashSet<T>>, HashMap<u32, HashSet<T>>),
        transfer: fn(&Block, &HashSet<T>) -> HashSet<T>,
        merge: fn(&HashSet<T>, &HashSet<T>) -> HashSet<T>,
    ) -> HashMap<u32, HashSet<T>> {
        let mut worklist: Vec<u32> = (0..((self.blocks.len() - 1) as u32)).collect_vec();
        // map blocks to sets of defs that reach the end
        let (mut inputs, mut outputs) = init(&self.args, &self.blocks);

        while let Some(b) = worklist.pop() {
            dbg!(b);
            let preds = self.cfg.get_predecessors(&b).unwrap();
            let folded = preds.iter().fold(HashSet::new(), |mut acc, el| {
                let pred_outs = outputs.get(el).unwrap();
                acc = merge(pred_outs, &acc);
                acc
            });
            let res = folded.union(inputs.get(&b).unwrap()).cloned().collect();
            dbg!(&res);
            let out_b: HashSet<T> = transfer(self.blocks.get(b as usize).unwrap(), &res);
            dbg!(&out_b);

            inputs.insert(b, res);

            let old_out = outputs.get(&b).unwrap().clone();
            if old_out.len() != out_b.len() {
                let mut sucs = self.cfg.get_successors(&b).unwrap().clone();
                worklist.append(&mut sucs);
            }
            outputs.insert(b, out_b);
        }
        outputs
    }

    pub fn construct_dominators(&mut self) {
        // worklist algorithm - copy from lesson 5
        let mut worklist: Vec<u32> = Vec::new();
        self.blocks
            .split_first()
            .unwrap()
            .1
            .iter()
            .enumerate()
            .for_each(|(idx, _block)| {
                worklist.push(idx as u32);
            });
        let mut inputs: HashMap<u32, HashSet<u32>> = HashMap::new();
        let mut outputs: HashMap<u32, HashSet<u32>> = HashMap::new();

        self.blocks.iter().enumerate().for_each(|(pos, _bl)| {
            inputs.insert(pos as u32, HashSet::new());
            outputs.insert(
                pos as u32,
                HashSet::from_iter(0..(self.blocks.len() - 1) as u32),
            );
        });
        outputs.insert(0, HashSet::from_iter([0]));

        while let Some(b) = worklist.pop() {
            let preds = self.cfg.get_predecessors(&b).unwrap();
            // apply merge function (set intersection)
            let mut folded: HashSet<u32> = HashSet::new();
            if !preds.is_empty() {
                // this is stupid why did i have to do this
                folded = preds.iter().fold(
                    HashSet::from_iter(0..(self.blocks.len() - 1) as u32),
                    |acc, el| {
                        let pred_outs = outputs.get(el).unwrap();
                        acc.intersection(pred_outs).cloned().collect()
                    },
                );
            }
            // inputs.update b to folded
            let mut new_out = folded.clone(); // do this here bc ownership funnies
            inputs.insert(b, folded);
            // transfer function
            let old_out = outputs.get(&b).unwrap().clone();
            new_out.insert(b);
            if old_out.len() != new_out.len() {
                worklist.push(b);
            }
            // insert at bottom again because borrowing funnies
            outputs.insert(b, new_out);
        }
        self.dominators = outputs;
    }

    pub fn dominance_frontier(&self, block_idx: u32) -> HashSet<u32> {
        let mut frontier: HashSet<u32> = HashSet::new();
        let mut dominated: HashSet<u32> = HashSet::new();
        self.dominators.iter().for_each(|(b, d)| {
            if d.contains(&block_idx) {
                dominated.insert(*b);
            }
        });

        self.dominators.iter().for_each(|(b, d)| {
            if !dominated.contains(b) && !d.intersection(&dominated).collect_vec().is_empty() {
                frontier.insert(*b);
            }
        });
        frontier
    }
    pub fn make_dominance_tree(&mut self) {}

    pub fn find_natural_loops(&self) -> HashMap<u32, HashSet<u32>> {
        let mut loops: HashMap<u32, HashSet<u32>> = HashMap::new();
        self.cfg.succs_adj_list.iter().for_each(|(block, sucks)| {
            let doms = self.dominators.get(block).unwrap(); //dominators of block
            for suck in sucks {
                if doms.contains(suck) {
                    // natural loop
                    let mut natty_loop: HashSet<u32> = HashSet::new();
                    // get all blocks between a - block and b - suck
                    self.get_blocks_in_loop(&mut natty_loop, *block, *suck);
                    loops.insert(*block, natty_loop);
                }
            }
        });
        loops
    }
    fn get_blocks_in_loop(&self, blocks: &mut HashSet<u32>, curr: u32, top: u32) {
        // if curr == top then do nothing
        // else add curr to blocks and investigate children
        if curr != top {
            blocks.insert(curr);
            for pre in self.cfg.preds_adj_list.get(&curr).unwrap() {
                if !blocks.contains(pre) {
                    self.get_blocks_in_loop(blocks, *pre, top);
                }
            }
        }
    }
}
