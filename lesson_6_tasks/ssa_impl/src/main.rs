mod briltypes;
mod cfg;
use cfg::FuncContext;
mod ssa_into;
use crate::briltypes::*;
use std::env;

fn main() {
    println!("Hello, world!");
    let args: Vec<String> = env::args().collect();
    let file: &String = args.get(1).unwrap(); // i should catch error but o well

    let deserialized_program: Program = serde_json::from_str(file).unwrap();
    deserialized_program.functions.iter().for_each(|func| {
        let mut ctx = FuncContext::new();
        // construct cfg for object
        ctx.init(func);
        // do the variables thing
        ctx.collect_vars();
        ctx.to_ssa();
    });
}
