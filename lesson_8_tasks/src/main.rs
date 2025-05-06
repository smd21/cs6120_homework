mod briltypes;
mod cfg;
mod licm;
use crate::briltypes::*;
use cfg::FuncContext;
use itertools::Itertools;
use licm::Licm;
use std::{env, fs::File};

fn main() -> Result<(), std::io::Error> {
    println!("Hello, world!");
    let args: Vec<String> = env::args().collect();
    let file = File::open(args.get(1).unwrap()).expect("er file didn't open send help"); // i should catch error but o well

    let deserialized_program: Program = serde_json::from_reader(file).unwrap();
    let new_fns = deserialized_program
        .functions
        .iter()
        .map(|func| {
            let mut ctx = FuncContext::new();
            ctx.init(func);
            let mut licm = Licm::new(&ctx);
            licm.do_licm();
            let output_insns = licm.construct_new_program();
            Function {
                instrs: output_insns,
                name: func.name.clone(),
                args: func.args.clone(),
                func_type: func.func_type.clone(),
            }
        })
        .collect_vec();
    let new_program = Program { functions: new_fns };
    let string_prog = serde_json::to_string(&new_program)?;
    println!("{}", string_prog);
    Ok(())
}
