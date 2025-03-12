mod briltypes;
mod cfg;
use cfg::FuncContext;
use itertools::Itertools;
mod ssa_into;
use crate::briltypes::*;
use std::io::Write;
use std::{env, fs::File};

fn main() -> Result<(), std::io::Error> {
    println!("Hello, world!");
    let args: Vec<String> = env::args().collect();
    let file = File::open(args.get(1).unwrap()).expect("er file didn't open send help"); // i should catch error but o well

    let deserialized_program: Program = serde_json::from_reader(file).unwrap();
    let ssa_functions = deserialized_program
        .functions
        .iter()
        .map(|func| {
            let mut ctx = FuncContext::new();
            // construct cfg for object
            ctx.init(func);
            // do the variables thing
            ctx.collect_vars();
            ctx.ssa_convert()
        })
        .collect_vec();
    let ssa_program = Program {
        functions: ssa_functions,
    };
    // reserialize and print to file
    let output = match serde_json::to_string(&ssa_program) {
        Ok(reserialized) => reserialized,
        Err(e) => e.to_string(),
    };
    let mut out_file = File::create("output_ssa.json")?;
    write!(out_file, "{output}")
}
