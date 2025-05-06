mod briltypes;

use crate::briltypes::*;
use std::{env, fs::File, io::Write};

fn main() -> Result<(), std::io::Error> {
    let args: Vec<String> = env::args().collect();
    let original = File::open(args.get(1).unwrap()).expect("prgoram didnt open");
    let trace = File::open(args.get(2).unwrap()).expect("er trace didn't open send help"); // i should catch error but o well

    // program with one function containing instructions from the trace
    let mut original_program: Program = serde_json::from_reader(original).unwrap();
    let deserialized_trace: Program = serde_json::from_reader(trace).unwrap();

    let mut modified_insns: Vec<InsnType> = Vec::new();
    // iterate over trace
    // add a speculate before all guards. in_spec=true
    // if inspec=true add a commit before the next guard/print instruction
    // add a new speculate after the commit instruction (before the guard, after the print)
    let mut in_spec = false;
    for instruction in deserialized_trace.functions[0].instrs.iter() {
        match instruction {
            InsnType::Effect {
                op,
                labels: _,
                args: _,
                funcs: _,
            } => {
                if *op == EffectOps::Guard {
                    if in_spec {
                        modified_insns.push(InsnType::Effect {
                            op: EffectOps::Commit,
                            labels: Vec::new(),
                            args: Vec::new(),
                            funcs: Vec::new(),
                        });
                    }
                    modified_insns.push(InsnType::Effect {
                        op: EffectOps::Speculate,
                        labels: Vec::new(),
                        args: Vec::new(),
                        funcs: Vec::new(),
                    });
                    in_spec = true;
                    modified_insns.push(instruction.clone());
                } else if *op == EffectOps::Print && in_spec {
                    modified_insns.push(InsnType::Effect {
                        op: EffectOps::Commit,
                        labels: Vec::new(),
                        args: Vec::new(),
                        funcs: Vec::new(),
                    });
                    modified_insns.push(instruction.clone());
                    modified_insns.push(InsnType::Effect {
                        op: EffectOps::Speculate,
                        labels: Vec::new(),
                        args: Vec::new(),
                        funcs: Vec::new(),
                    });
                } else {
                    modified_insns.push(instruction.clone())
                }
            }
            _ => modified_insns.push(instruction.clone()),
        }
    }
    // insert a commit before we exit the trace

    if in_spec {
        modified_insns.insert(
            modified_insns.len() - 1,
            InsnType::Effect {
                op: EffectOps::Commit,
                labels: Vec::new(),
                args: Vec::new(),
                funcs: Vec::new(),
            },
        );
    }
    let last_instr = modified_insns.last().unwrap().clone();

    // i dont know a better way to do this
    let mut main_idx = 0;
    for (idx, func) in original_program.functions.iter().enumerate() {
        if func.name == "main" {
            main_idx = idx;
        }
    }
    let func = original_program.functions.get_mut(main_idx).unwrap();
    modified_insns.append(&mut func.instrs);
    if let InsnType::Effect {
        funcs: _,
        args: _,
        labels: l,
        op: EffectOps::Jmp,
    } = last_instr
    {
        if l.first().unwrap() == ".trace_end" {
            let temp_instr = InsnType::Label {
                label: String::from(".trace_end"),
            };
            modified_insns.push(temp_instr);
        }
    }

    func.instrs = modified_insns;

    let updated_program = serde_json::to_string(&original_program).unwrap();
    let mut outputfile = File::create("output.json").unwrap();
    let _ = File::write(&mut outputfile, updated_program.as_bytes());
    dbg!("done!");

    print!("{}", &updated_program);
    Ok(())
}
