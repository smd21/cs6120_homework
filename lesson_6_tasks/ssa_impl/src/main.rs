mod briltypes;
mod cfg;
use crate::briltypes::*;
use std::env;

fn main() {
    println!("Hello, world!");
    let args: Vec<String> = env::args().collect();
    let file: &String = args.get(1).unwrap(); // i should catch error but o well

    let deserialized: Program = serde_json::from_str(file).unwrap();
}
