// SPDX-License: MIT
use either::{Either, Left, Right};

use llvm_plugin::{LlvmModulePass, PassBuilder, PipelineParsing, PreservedAnalyses};

#[llvm_plugin::plugin(name = "SkeletonPass", version = "0.1")]
fn plugin_registrar(builder: &mut PassBuilder) {
    builder.add_module_pipeline_parsing_callback(|name, manager| {
        if name == "skeleton-pass" {
            manager.add_pass(SkeletonPass);
            PipelineParsing::Parsed
        } else {
            PipelineParsing::NotParsed
        }
    });
}

struct SkeletonPass;

impl LlvmModulePass for SkeletonPass {
    fn run_pass(
        &self,
        module: &mut llvm_plugin::inkwell::module::Module<'_>,
        _manager: &llvm_plugin::ModuleAnalysisManager,
    ) -> PreservedAnalyses {
        for function in module.get_functions() {
            for basic_block in function.get_basic_blocks() {
                for instr in basic_block.get_instructions() {
                    match instr.get_opcode() {
                        llvm_plugin::inkwell::values::InstructionOpcode::Br => {
                            let insn_labels = instr.get_operands();
                            let mapped = insn_labels.into_iter().map(|x| {
                                let y = x.unwrap().left().unwrap().get_name().to_str();
                                if let Ok(y_2) = y {
                                    y_2
                                } else {
                                    let new_str = "";
                                    new_str
                                }
                            });
                            let folded = mapped.fold(String::new(), |acc, el| acc.push_str(el));
                            println!("tree branch {}", folded);
                        }
                        llvm_plugin::inkwell::values::InstructionOpcode::Call => {
                            let first = instr.get_operands().next().unwrap();
                            let bb = first.unwrap().right().unwrap().get_name().to_str();
                            if let Ok(b) = bb {
                                println!("you have called a function: {}", b);
                            }
                        }
                        llvm_plugin::inkwell::values::InstructionOpcode::CallBr => {
                            println!("you have called a function - branch ver");
                        }
                        _ => (),
                    }
                }
            }
        }
        PreservedAnalyses::All
    }
}
