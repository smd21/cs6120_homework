/// <reference lib="es2020" />

import program from "../lesson_3_tasks/gcd.json"

// types for making blocks 
type Type = string | { ptr: string }
type Label = { label: string }

type Others = {
  op: string,
  dest?: string,
  args?: string[],
  value?: number,
  type?: Type,
  funcs?: string[],
  labels?: string[]
}
type Instruction = Others | Label

type Param = {
  name: string,
  type: Type
}
type Function = {
  args?: Param[],
  instrs: Instruction[],
  name: string,
  type?: Type
}
class Block {
  insns: Instruction[];
}

class Definition {
  dest: string;
  value: string; // this seems easier than whatever the fuck i did for lvn 

  to_string(): string {
    return this.dest + ":" + this.value;
  }
  print(): void {
    console.log(this.dest + ":" + this.value);
  }
}

// Constructs the cfg for a given function func
function build_cfg(func: Function) {
  var label_blocks: Map<string, Block> = new Map();
  var blocks: Block[] = []; // store index of all blocks
  var curr_block_idx = 0;
  var current_block: Instruction[] = []

  func.instrs.forEach((insn: Instruction) => {
    if ((insn as Others).op == "br" || (insn as Others).op == "jmp") {
      current_block.push(insn)
      var to_push = new Block()

      to_push.insns = JSON.parse(JSON.stringify(current_block))
      blocks.push(to_push)
      curr_block_idx++;
      current_block = []
    }
    else {
      if ("label" in insn) { // end the previous block - if there is one, add to new one - LABEL IS ALWAYS FIRST INSTRUCTION
        if (current_block.length > 0) {
          var to_push = new Block()
          to_push.insns = JSON.parse(JSON.stringify(current_block))
          blocks.push(to_push)
          curr_block_idx++;
          current_block = []
        }
      }
      current_block.push(insn)
    }
  })
  //push last block in function:
  var to_push = new Block()
  to_push.insns = JSON.parse(JSON.stringify(current_block))
  blocks.push(to_push)
  curr_block_idx++;

  // iterate over blocks and collect labels
  blocks.forEach((block: Block) => {
    if ("label" in block.insns[0]) {
      label_blocks.set(block.insns[0].label, block) // works due to how i create blocks
    }
  })

  //create block idx map
  var block_to_idx: Map<Block, number> = new Map();
  for (var i: number = 0; i < blocks.length; i++) {
    block_to_idx.set(blocks[i], i);
  }

  // map based on block indexesz
  var cfg_sucs: Map<number, number[]> = new Map()
  var cfg_pres: Map<number, number[]> = new Map()

  //initialize to empty for all blocks
  block_to_idx.forEach((label: number, _block: Block) => {
    cfg_pres.set(label, [])
  })
  for (var i = 0; i < blocks.length; i++) {
    var block = blocks[i];
    var successors: number[] = []
    var last: Instruction = block.insns[block.insns.length - 1]

    if ("op" in last) {
      if (last.op == "jmp") {
        var to_jump: Block = label_blocks.get((last as Others).labels![0])!;
        var tj_idx = block_to_idx.get(to_jump)!;
        successors.push(tj_idx)
        var preds = cfg_pres.get(tj_idx)!;
        preds.push(i) //idx of current block
        cfg_pres.set(tj_idx, preds)

      } else if (last.op == "br") {
        (last as Others).labels!.forEach((label) => {
          var to_jump: Block = label_blocks.get(label)!;
          var tj_idx = block_to_idx.get(to_jump)!;
          successors.push(tj_idx)
          var preds = cfg_pres.get(tj_idx)!;
          preds.push(i)
          cfg_pres.set(tj_idx, preds)
        })
      }
      else {
        if (i < blocks.length - 1) {
          var next: number = block_to_idx.get(blocks[i + 1])! //index of next block
          successors.push(next)
          // add i as predecessor to i+1
          var preds = cfg_pres.get(next)!; //get predecessors of next block
          preds.push(i); //add current block
          cfg_pres.set(next, preds) //set in cfg
        }
      }
    }
    cfg_sucs.set(i, JSON.parse(JSON.stringify(successors)))
  }
  return { block_to_idx, blocks, cfg_sucs, cfg_pres }
}

function dataflow(blocks: Block[], cfg_pres: Map<number, number[]>, cfg_sucs: Map<number, number[]>, func: Function) {
  // reaching definitions
  // forward analysis, transition fn: f(in_b) = (in_b - killed_b) U defs_b 
  // use union to merge blocks

  var worklist: number[] = [];
  var inputs: Map<number, Set<string>> = new Map();
  var outputs: Map<number, Set<string>> = new Map();

  // initialize worklist with block indexes and outputs/inputs with empty sets
  for (var i = 0; i < blocks.length; i++) {
    worklist.push(i);
    outputs.set(i, new Set());
    inputs.set(i, new Set());
  }
  // initialize in[first block] to be the function arguments, if they exist. else init to new Set
  var init_in: Set<string> = new Set()
  if ("args" in func) {
    var arg_names: string[] = func.args!.map((arg) => { var def = new Definition(); def.dest = arg.name; def.value = "unk"; return def.to_string() })
    init_in = new Set(arg_names)
    console.log(init_in.size)
  }
  inputs.set(0, init_in);

  while (worklist.length > 0) {
    var b_idx: number = worklist.pop()!;
    //console.log("looking at: " + block_to_string(blocks[b_idx]));
    var preds: number[] = cfg_pres.get(b_idx)!; //b should def be in the cfg
    //console.log("size: " + (cfg_pres.get(b_idx)!).length);

    // use the intersection to merge
    var reduction: Set<string> = preds.reduce((acc: Set<string>, curr: number) => new Set([...acc, ...(outputs.get(curr)!)]), new Set()); //acc.union(outputs.get(curr)!)
    //append to current input
    if (inputs.get(b_idx)!.size! > 0) {
      reduction = new Set([...reduction, ...inputs.get(b_idx)!]);
    }
    inputs.set(b_idx, reduction);

    //apply transfer function
    var output_b: Set<string> = reaching_defs_transfer(blocks[b_idx], reduction);

    // check if there was a change 
    if (set_difference(output_b, outputs.get(b_idx)!).size > 0 || set_difference(outputs.get(b_idx)!, output_b).size > 0) {
      worklist.push(...cfg_sucs.get(b_idx)!);
    }
    outputs.set(b_idx, output_b);
  }
  return outputs;
}

// Transfer function for reaching defs data flow analysis
function reaching_defs_transfer(block: Block, inputs: Set<string>) {
  var killed: Set<string> = new Set();
  var defs: Set<string> = new Set();
  block.insns.forEach((insn) => {
    if ("dest" in insn) {
      // add to defs
      var val: string = insn.op + " ";
      if ("value" in insn) {
        val += insn.value!.toString()
      }
      if ("args" in insn) {
        val += insn.args!.reduce((acc: string, curr: string) => acc + " " + curr, "");
      }
      defs.add(insn.dest! + ":" + val);
      // remove anything this def kills
      inputs.forEach((item) => {
        if (item.split(":")[0] == insn.dest!) {
          killed.add(item)
        }
      })
    }
  })
  var diff: Set<string> = set_difference(inputs, killed);
  return new Set([...defs, ...diff]) //defs.union(diff); //res = (in - killed) U defs
}

/// helper method that takes in two sets A and B and returns A - B
function set_difference<T>(A: Set<T>, B: Set<T>): Set<T> {
  var diff: Set<T> = new Set([...A].filter((x) => !B.has(x)));
  return diff;
}

/// wowie this takes in many things: transfer function, merge function, list of blocks in fn, 
// cfg of blocks to predecessors, cfg of blocks to successors, the bril function, and a boolean indicating analysis direction
function generic_workflow(transfer_fn: (b: Block, inp: Set<string>) => Set<string>, merge: (acc: Set<string>, curr: number) => Set<string>, blocks: Block[], cfg_preds: Map<number, number[]>, cfg_succs: Map<number, number[]>, func: Function, backwards: boolean) {
  // worklist algorithm pseudocode:
  // in[entry] = init 
  //out[*] = init
  // worklist = all blocks
  // while worklist is not empty:
  // b = pick any block from worklist
  // in[b] = merge(out[p] for every predecessor p of b)
  // out[b] = transfer(b, in[b])
  // if out[b] changed:
  // worklist += successors of b
  var worklist: number[] = [];
  var inputs: Map<number, Set<string>> = new Map();
  var outputs: Map<number, Set<string>> = new Map();

  // initialize worklist with block indexes and outputs/inputs with empty sets
  for (var i = 0; i < blocks.length; i++) {
    worklist.push(i);
    outputs.set(i, new Set());
    inputs.set(i, new Set());
  }

  while (worklist.length > 0) {
    var b_idx: number = worklist.pop()!;
    var preds: number[] = cfg_preds.get(b_idx)!; //b should def be in the cfg

    // merge with merge function
    var reduction: Set<string> = preds.reduce(merge, new Set());

    inputs.set(b_idx, reduction);

    //apply transfer function
    var output_b: Set<string> = transfer_fn(blocks[b_idx], reduction);

    // check if there was a change 
    if (set_difference(output_b, outputs.get(b_idx)!).size > 0 || set_difference(outputs.get(b_idx)!, output_b).size > 0) {
      worklist.push(...cfg_succs.get(b_idx)!);
    }
    outputs.set(b_idx, output_b);

    //add if backwards
  }
  return outputs; //if backwards return inputs

}

function main() {
  program.functions.forEach((func) => {
    var { block_to_idx, blocks, cfg_sucs, cfg_pres } = build_cfg(func)
    // test build cfg - appears to be correct for gcd.bril 
    // console.log("successors cfg graph: ")

    // for (let i = 0; i < blocks.length; i++) {
    //   var block: Block = blocks[i];
    //   var successors = cfg_sucs.get(i)!;
    //   console.log("block: \n" + block_to_string(block) + "has successors: \n")
    //   successors.forEach((idx) => {
    //     console.log(block_to_string(blocks[idx]))
    //   })
    // }

    // console.log("predecessors cfg graph: ")
    // for (let i = 0; i < blocks.length; i++) {
    //   var bl: Block = blocks[i];
    //   var preds = cfg_pres.get(i)!;
    //   console.log("block: \n" + block_to_string(bl) + "has predecessors: \n")
    //   preds.forEach((idx) => {
    //     console.log(block_to_string(blocks[idx]))
    //   })
    // }

    var outputs = dataflow(blocks, cfg_pres, cfg_sucs, func);
    // print outputs and ensure they are correct
    outputs.forEach((val, key) => {
      console.log("defs that reach the end of block " + block_to_string(blocks[key]));
      //iterate over set and print definitions
      for (const defn of val.values()) {
        console.log(defn);
      }

    });
  })

}

function block_to_string(block: Block) {
  var result: string = ""
  block.insns.forEach((instr) => {
    if ("label" in instr) {
      result = result + instr.label + "\n"
    }
    else {
      result = result + instr.op
      if ("labels" in instr) {
        result = result + instr.labels!.toString()
      }
      result = result + "\n"
    }
  })
  return result
}
main()