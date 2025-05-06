use crate::briltypes::*;
use crate::cfg;
use crate::cfg::Block;
use crate::cfg::BlockName;
use std::collections::HashMap;
use std::collections::HashSet;
use std::hash::Hash;

#[derive(PartialEq, Eq, Clone)]
pub struct Definition(String, Vec<BlockName>);
impl Hash for Definition {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}
pub struct Licm<'a> {
    original_ctx: &'a cfg::FuncContext,
    preheaders: HashMap<u32, cfg::Block>, //maps the new block to what it comes before
}
impl<'a> Licm<'a> {
    pub fn new(orig: &'a cfg::FuncContext) -> Self {
        Self {
            original_ctx: orig,
            preheaders: HashMap::new(),
        }
    }
    fn reaching_defs_transfer(block: &Block, inputs: &HashSet<Definition>) -> HashSet<Definition> {
        dbg!(block.instructions.len());
        let mut killed: HashSet<String> = HashSet::new();
        let mut defs: HashSet<String> = HashSet::new();
        block.instructions.iter().for_each(|insn| match insn {
            InsnType::Constant {
                op: _,
                dest,
                insn_type: _,
                value: _,
            } => {
                if inputs.contains(&def) {
                    killed.insert(def);
                    println!("killed {}", dest);
                }
                defs.insert(dest.clone());
                println!("defined {}", dest);
            }
            InsnType::ValOp {
                op: _,
                insn_type: _,
                dest,
                args: _,
                funcs: _,
            } => {
                if inputs.contains(dest) {
                    killed.insert(dest.clone());
                    println!("killed {}", dest);
                }
                defs.insert(dest.clone());
                println!("defined {}", dest);
            }
            InsnType::Effect { op: _, args: _ } => println!("effect"),
            InsnType::Label { label: _ } => println!("label"),
            InsnType::Terminator {
                op: _,
                labels: _,
                args: _,
            } => println!("term"),
        });
        let diff: HashSet<String> = inputs.difference(&killed).cloned().collect();
        let res: HashSet<String> = defs.union(&diff).cloned().collect();
        res
    }

    fn reaching_defs_merge(acc: &HashSet<String>, el: &HashSet<String>) -> HashSet<String> {
        acc.union(el).cloned().collect()
    }

    fn reaching_defs_init(
        fn_args: &Option<Vec<String>>,
        blocks: &Vec<Block>,
    ) -> (HashMap<u32, HashSet<String>>, HashMap<u32, HashSet<String>>) {
        let mut inputs: HashMap<u32, HashSet<String>> = HashMap::new();
        let mut outputs: HashMap<u32, HashSet<String>> = HashMap::new();
        for (idx, _blk) in blocks.iter().enumerate() {
            inputs.insert(idx as u32, HashSet::new());
            outputs.insert(idx as u32, HashSet::new());
        }
        if let Some(argos) = fn_args {
            inputs.insert(0, HashSet::from_iter(argos.iter().cloned()));
        }
        (inputs, outputs)
    }

    pub fn do_licm(&mut self) {
        let mut prehead_ctr: u32 = self.original_ctx.blocks.len() as u32;
        let loops = self.original_ctx.find_natural_loops();
        // get reaching defs
        let reaching_defs = self.original_ctx.workflow(
            Self::reaching_defs_init,
            Self::reaching_defs_transfer,
            Self::reaching_defs_merge,
        );
        dbg!(&reaching_defs);
        for (top, loop_blocks) in loops {
            let mut inv_insns: Vec<InsnType> = Vec::new();
            let mut inv_defs: HashSet<String> = HashSet::new();
            for block in loop_blocks {
                self.original_ctx
                    .blocks
                    .get(block as usize)
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
                            inv_defs.insert(dest.clone());
                            inv_insns.push(insn.clone());
                            println!("added const {} to inv insns", dest);
                        }
                        InsnType::ValOp {
                            op: _,
                            insn_type: _,
                            dest,
                            args: Some(argos),
                            funcs: None,
                        } => {
                            let mut check = true;
                            for arg in argos {
                                if !reaching_defs.get(&block).unwrap().contains(arg)
                                    || !inv_defs.contains(arg)
                                {
                                    check = false;
                                    break;
                                }
                            }
                            // all args are invariant, so add to invariant instructions + add dest to invariant definitions
                            if check {
                                inv_defs.insert(dest.clone());
                                inv_insns.push(insn.clone());
                            }
                            println!("added val {} to inv insns", dest);
                        }
                        _ => (), // we do not want to move termiator instructions, effects (prints, jumps, etc), labels, or function calls
                    });
            }
            let prehead = Block {
                name: BlockName(prehead_ctr), //idk why this makes everything need to be pub
                instructions: inv_insns.clone(),
            };
            self.preheaders.insert(top, prehead);
            prehead_ctr += 1;
            inv_defs.clear();
            inv_insns.clear();
        }
    }

    pub fn construct_new_program(&self) -> Vec<InsnType> {
        let mut result: Vec<InsnType> = Vec::new();
        for (idx, block) in self.original_ctx.blocks.iter().enumerate() {
            if self.preheaders.contains_key(&(idx as u32)) {
                dbg!(idx);
                let mut preheader = self.preheaders.get(&(idx as u32)).unwrap().clone();
                result.append(&mut preheader.instructions);
            }
            let mut cloned = block.clone();
            result.append(&mut cloned.instructions);
        }
        result
    }
}
