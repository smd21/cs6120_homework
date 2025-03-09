use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
};

use crate::briltypes::*;
use itertools::Itertools;

#[derive(Eq, PartialEq, Hash, Copy, Clone)]
struct BlockName(u32);
#[derive(Clone)]
pub struct Block {
    name: BlockName,
    instructions: Vec<InsnType>,
    phi_insns: Vec<InsnType>,
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
pub struct CFG {
    preds_adj_list: HashMap<u32, Vec<u32>>, //ill change this later to a struct like blockidx or smthg
    succs_adj_list: HashMap<u32, Vec<u32>>,
}
impl CFG {
    pub fn new() -> Self {
        Self::default()
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
                self.succs_adj_list.insert(block.clone(), succs);
            }
        }
    }
}
#[derive(Default)]
pub struct FuncContext {
    cfg: CFG,
    blocks: Vec<Block>, //update this to be block idx or some struct
    dominators: HashMap<u32, HashSet<u32>>,
    variable_assigns: HashMap<String, HashSet<u32>>,
}

impl FuncContext {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn init(&mut self, function: &Function) {
        self.make_cfg(function);
    }
    fn insert_phi_node(&mut self, blockidx: u32, insn: InsnType) {
        let bl = self.blocks.get_mut(blockidx as usize).unwrap();
        bl.phi_insns.push(insn);
    }
    fn make_cfg(&mut self, function: &Function) {
        let mut name_counter: u32 = 0;
        let mut label_blocks: HashMap<&String, &Block> = HashMap::new();
        self.blocks = Vec::new();
        let mut block_indices: HashMap<Block, u32> = HashMap::new();
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
                            phi_insns: Vec::new(),
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
                        phi_insns: Vec::new(),
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
            phi_insns: Vec::new(),
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
        });
    }

    //todo!("adjust this to use indices");

    pub fn construct_dominators(&mut self) {
        // worklist algorithm - copy from lesson 5
        let mut worklist: Vec<u32> = Vec::new();
        self.blocks
            .split_first()
            .unwrap()
            .1
            .iter()
            .enumerate()
            .for_each(|(idx, block)| {
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
            let folded = preds.iter().fold(
                HashSet::from_iter(0..(self.blocks.len() - 1) as u32),
                |acc, el| {
                    let pred_outs = outputs.get(el).unwrap();
                    acc.intersection(pred_outs).cloned().collect()
                },
            );
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

    pub fn collect_vars(&mut self) {
        self.variable_assigns = HashMap::new();
        self.blocks.iter().enumerate().for_each(|(idx, block)| {
            block.instructions.iter().for_each(|insn| match insn {
                InsnType::Constant {
                    op: _,
                    dest,
                    insn_type,
                    value: _,
                } => {
                    if self.variable_assigns.contains_key(dest) {
                        let mut defs = self.variable_assigns.get(dest).unwrap().clone();
                        defs.insert(idx as u32);
                        self.variable_assigns.insert(dest.clone(), defs);
                    } else {
                        self.variable_assigns
                            .insert(dest.clone(), HashSet::from([idx as u32]));
                    }
                }
                InsnType::ValOp {
                    op: _,
                    insn_type: _,
                    dest,
                    args: _,
                    funcs: _,
                } => {
                    if self.variable_assigns.contains_key(dest) {
                        let mut defs = self.variable_assigns.get(dest).unwrap().clone();
                        defs.insert(idx as u32);
                        self.variable_assigns.insert(dest.clone(), defs);
                    } else {
                        self.variable_assigns
                            .insert(dest.clone(), HashSet::from([idx as u32]));
                    }
                }
                _ => (),
            });
        });
    }

    pub fn into_ssa(&mut self) {
        // insert phi nodes and generate new function context
        self.construct_dominators();
        for (var, def_blocks) in self.variable_assigns.iter() {
            let mut defs: Vec<&u32> = def_blocks.iter().clone().collect();
            while let Some(d) = defs.pop() {
                // iterates over where var is defined
                // add set insn to d in the original blocks list

                // go over dom frontier, add the get insn
                let f = self.dominance_frontier(*d);
                for b_idx in f {
                    let get_insn: InsnType = InsnType::ValOp {
                        op: String::from("get"),
                        insn_type: String::from("int"),
                        dest: String::from(var),
                        args: None,
                        funcs: None,
                    };
                    let block = self.blocks.get_mut(b_idx as usize).unwrap();
                    block.phi_insns.push(get_insn);
                }
            }
        }
    }
}
