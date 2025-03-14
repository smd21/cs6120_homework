use std::{
    collections::{HashMap, HashSet},
    fs::rename,
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
    set_instrs: Vec<InsnType>,
    get_instrs: Vec<InsnType>,
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
    args: Option<Vec<String>>,
    name: String,
    fn_type: Option<String>,
    cfg: Cfg,
    blocks: Vec<Block>, //update this to be block idx or some struct
    dominators: HashMap<u32, HashSet<u32>>,
    dominance_tree: HashMap<u32, HashSet<u32>>, // this is parent: children and dominators is child: parents lol
    variable_assigns: HashMap<String, (String, HashSet<u32>)>, //the tuple is insn type and block
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
            .instructions
            .iter()
            .for_each(|instruction| match instruction {
                InsnType::Label { label: _ } => {
                    if !current_block.is_empty() {
                        let new_block: Block = Block {
                            name: BlockName(name_counter),
                            instructions: current_block.clone(),
                            set_instrs: Vec::new(),
                            get_instrs: Vec::new(),
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
                        set_instrs: Vec::new(),
                        get_instrs: Vec::new(),
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
            set_instrs: Vec::new(),
            get_instrs: Vec::new(),
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

    fn defined_insns_transfer(&self, block_idx: u32, input: &HashSet<String>) -> HashSet<String> {
        let mut defined: HashSet<String> = HashSet::new();
        self.blocks
            .get(block_idx as usize)
            .unwrap()
            .instructions
            .iter()
            .for_each(|insn| match insn {
                InsnType::Constant {
                    op: _,
                    dest,
                    insn_type: _,
                    value: _,
                } => {
                    defined.insert(dest.clone());
                }
                InsnType::ValOp {
                    op: _,
                    insn_type: _,
                    dest,
                    args: _,
                    funcs: _,
                } => {
                    defined.insert(dest.clone());
                }
                _ => (),
            });
        input.union(&defined).cloned().collect()
    }

    fn undefined_vars(&self) -> HashMap<u32, HashSet<String>> {
        let mut worklist: Vec<u32> = Vec::new();
        // map blocks to sets of defs that reach the end
        let mut inputs: HashMap<u32, HashSet<String>> = HashMap::new();
        let mut outputs: HashMap<u32, HashSet<String>> = HashMap::new();

        self.blocks
            .split_first()
            .unwrap()
            .1
            .iter()
            .enumerate()
            .for_each(|(idx, _block)| {
                worklist.push(idx as u32);
                inputs.insert(idx as u32, HashSet::new());
                outputs.insert(idx as u32, HashSet::new());
            });
        if let Some(args) = &self.args {
            inputs.insert(0, HashSet::from_iter(args.clone()));
        }

        while let Some(b) = worklist.pop() {
            let preds = self.cfg.get_predecessors(&b).unwrap();
            // apply merge function - symmetric difference - need things that are not defined along every path
            let folded = preds.iter().fold(HashSet::new(), |acc, el| {
                let pred_outs = outputs.get(el).unwrap();
                acc.symmetric_difference(pred_outs).cloned().collect()
            });
            let res = folded.union(inputs.get(&b).unwrap()).cloned().collect();
            let out_b: HashSet<String> = self.defined_insns_transfer(b, &res);

            inputs.insert(b, res);

            let old_out = outputs.get(&b).unwrap().clone();
            if old_out.len() != out_b.len() {
                worklist.push(b);
            }
            outputs.insert(b, out_b);
        }
        outputs
        // todo!("change this to workflow algorithm and refactor");
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
                        let mut defs = self.variable_assigns.get(dest).unwrap().1.clone();
                        defs.insert(idx as u32);
                        self.variable_assigns
                            .insert(dest.clone(), (insn_type.clone(), defs));
                    } else {
                        self.variable_assigns.insert(
                            dest.clone(),
                            (insn_type.clone(), HashSet::from([idx as u32])),
                        );
                    }
                }
                InsnType::ValOp {
                    op: _,
                    insn_type,
                    dest,
                    args: _,
                    funcs: _,
                } => {
                    if self.variable_assigns.contains_key(dest) {
                        let mut defs = self.variable_assigns.get(dest).unwrap().1.clone();
                        defs.insert(idx as u32);
                        self.variable_assigns
                            .insert(dest.clone(), (insn_type.clone(), defs));
                    } else {
                        self.variable_assigns.insert(
                            dest.clone(),
                            (insn_type.clone(), HashSet::from([idx as u32])),
                        );
                    }
                }
                _ => (),
            });
        });
    }

    pub fn ssa_convert(&mut self) -> Function {
        // insert phi nodes
        self.construct_dominators();
        let mut stacks: HashMap<String, Vec<String>> = HashMap::new();
        for (var, (insn_type, def_blocks)) in self.variable_assigns.iter() {
            stacks.insert(var.clone(), vec![var.clone()]);
            let mut defs: Vec<u32> = def_blocks.iter().cloned().collect();
            let mut seen: Vec<u32> = Vec::new();
            while let Some(d) = defs.pop() {
                seen.push(d);
                // iterates over where var is defined
                // add set insn to d in the original blocks list
                let set_insn: InsnType = InsnType::Effect {
                    op: String::from("set"),
                    args: Some(vec![var.clone(), var.clone()]),
                };
                let block = self.blocks.get_mut(d as usize).unwrap();
                block.set_instrs.push(set_insn);

                // go over dom frontier, add the get insn
                let f = self.dominance_frontier(d);
                for b_idx in f {
                    let get_insn: InsnType = InsnType::ValOp {
                        op: String::from("get"),
                        insn_type: insn_type.clone(),
                        dest: String::from(var),
                        args: None,
                        funcs: None,
                    };
                    let block = self.blocks.get_mut(b_idx as usize).unwrap();
                    block.get_instrs.push(get_insn);

                    if (seen.contains(&b_idx)) && !defs.contains(&b_idx) {
                        defs.push(b_idx);
                    }
                }
            }
        }

        // handle function arguments
        // i have no clue what to do with these if imma be so fr

        // handle undefined instructions - praying this works
        let undefs = self.undefined_vars().into_iter().last().unwrap().1;
        let undefs_insns: Vec<InsnType> = undefs
            .iter()
            .map(|var| {
                let var_type = self.variable_assigns.get(var).unwrap().0.clone();
                InsnType::ValOp {
                    op: String::from("undef"),
                    insn_type: var_type,
                    dest: var.clone(),
                    args: None,
                    funcs: None,
                }
            })
            .collect_vec();
        // add all of them to a block at the top - since this is block one i dont think it ever gets renamed
        let new_block = Block {
            instructions: undefs_insns,
            name: BlockName(self.blocks.len() as u32),
            set_instrs: Vec::new(),
            get_instrs: Vec::new(),
        };

        // call rename

        let mut ssa_blocks = self.rename(0, stacks, self.blocks.clone());
        ssa_blocks.push(new_block);
        // unblockify
        let mut just_instrs = ssa_blocks
            .iter_mut()
            .map(|block| {
                let mut temp: Vec<InsnType> = Vec::new();
                temp.append(&mut block.get_instrs);
                temp.append(&mut block.instructions);
                temp.append(&mut block.set_instrs);
                temp
            })
            .collect_vec();
        let final_instrs = just_instrs.iter_mut().fold(Vec::new(), |mut a, el| {
            a.append(el);
            a
        });
        Function {
            args: self.args.clone(),
            name: self.name.clone(),
            instructions: final_instrs,
            func_type: self.fn_type.clone(),
        }
    }

    fn rename(
        &self,
        block_idx: u32,
        mut stacks: HashMap<String, Vec<String>>,
        mut ssa_blocks: Vec<Block>,
    ) -> Vec<Block> {
        // for renaming phi instructions:
        // for set instructions: arg0 is changed when calling rename on predecessor
        // set arg1 reads from top of stack at the END of the block it is in
        // get: reads from top of stack (like an arg) in block it is in
        let ssa_block = ssa_blocks.get_mut(block_idx as usize).unwrap();

        // do get instructions here before stack is changed - read from top of stack
        ssa_block.get_instrs = ssa_block
            .get_instrs
            .iter()
            .map(|insn| match insn {
                InsnType::ValOp {
                    op,
                    insn_type,
                    dest,
                    args,
                    funcs,
                } => InsnType::ValOp {
                    op: op.clone(),
                    insn_type: insn_type.clone(),
                    dest: stacks.get(dest).unwrap().first().unwrap().clone(),
                    args: args.clone(),
                    funcs: funcs.clone(),
                },
                _ => InsnType::Label {
                    label: String::from("this shouldn't run my types are just bad"),
                }, // i'll fix types later lol
            })
            .collect_vec();

        // rename rest of instructions
        ssa_block.instructions = ssa_block
            .instructions
            .iter()
            .map(|insn| match insn {
                InsnType::Constant {
                    op,
                    dest,
                    insn_type,
                    value,
                } => {
                    let mut new_dest = dest.clone();
                    new_dest.push_str(".ssa.");
                    let stack_len = (stacks.get(dest).unwrap().len() as u32).to_string();
                    new_dest.push_str(&stack_len);
                    stacks.get_mut(dest).unwrap().push(new_dest.clone());
                    InsnType::Constant {
                        op: op.clone(),
                        dest: new_dest,
                        insn_type: insn_type.clone(),
                        value: value.clone(),
                    }
                }
                InsnType::Effect { op, args } => {
                    let mut new_args = None;
                    if let Some(argos) = args {
                        let temp = argos
                            .iter()
                            .map(|arg| stacks.get(arg).unwrap().first().unwrap().clone())
                            .collect_vec();
                        new_args = Some(temp);
                    }
                    InsnType::Effect {
                        op: op.clone(),
                        args: new_args,
                    }
                }
                InsnType::ValOp {
                    op,
                    insn_type,
                    dest,
                    args,
                    funcs,
                } => {
                    let mut new_dest = dest.clone();
                    new_dest.push_str(".ssa.");
                    let stack_len = (stacks.get(dest).unwrap().len() as u32).to_string();
                    new_dest.push_str(&stack_len);
                    let mut new_args = None;
                    if let Some(argos) = args {
                        let temp = argos
                            .iter()
                            .map(|arg| stacks.get(arg).unwrap().first().unwrap().clone())
                            .collect_vec();
                        new_args = Some(temp);
                    }
                    InsnType::ValOp {
                        op: op.clone(),
                        insn_type: insn_type.clone(),
                        dest: new_dest,
                        args: new_args,
                        funcs: funcs.clone(),
                    }
                }
                InsnType::Terminator { op, labels, args } => {
                    let mut new_args = None;
                    if let Some(argos) = args {
                        let temp = argos
                            .iter()
                            .map(|arg| stacks.get(arg).unwrap().first().unwrap().clone())
                            .collect_vec();
                        new_args = Some(temp);
                    }
                    InsnType::Terminator {
                        op: op.clone(),
                        labels: labels.clone(),
                        args: new_args,
                    }
                }
                InsnType::Label { label } => InsnType::Label {
                    label: label.clone(),
                },
            })
            .collect_vec();

        // do set instructions arg1 here
        let new_set_instrs = ssa_block.set_instrs.iter_mut();
        // adjust set arg0 in successors
        // recurse on children in dominance tree

        ssa_blocks
    }
}
