/// <reference lib="esnext" />

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
}

// Constructs the cfg for a given function func
function build_cfg(func: Function) {
  // TODO add labeled blocks
  var label_blocks: Map<string, Block> = new Map();
  var blocks: Block[] = []
  var current_block: Instruction[] = []

  func.instrs.forEach((insn: Instruction) => {
    if ((insn as Others).op == "br" || (insn as Others).op == "jmp") {
      current_block.push(insn)
      var to_push = new Block()

      to_push.insns = JSON.parse(JSON.stringify(current_block))
      blocks.push(to_push)
      current_block = []
    }
    else {
      if ("label" in insn) { // end the previous block - if there is one, add to new one - LABEL IS ALWAYS FIRST INSTRUCTION
        if (current_block.length > 0) {
          var to_push = new Block()
          to_push.insns = JSON.parse(JSON.stringify(current_block))
          blocks.push(to_push)
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

  // iterate over blocks and collect labels
  blocks.forEach((block: Block) => {
    if ("label" in block.insns[0]) {
      label_blocks.set(block.insns[0].label, block) // works due to how i create blocks
    }
  })

  var cfg_sucs: Map<Block, Block[]> = new Map()
  var cfg_pres: Map<Block, Block[]> = new Map()

  //initialize to empty for all blocks
  blocks.forEach((block) => {
    cfg_pres.set(block, [])
  })
  for (let i: number = 0; i < blocks.length; i++) {
    var successors: Block[] = []
    var block: Block = blocks[i]
    var last: Instruction = block.insns[block.insns.length - 1]

    if ("op" in last) {
      if (last.op == "jmp") {
        var to_jump: Block = label_blocks.get((last as Others).labels![0])!;
        successors.push(to_jump)
        var preds = cfg_pres.get(to_jump)!;
        preds.push(block)
        cfg_pres.set(to_jump, preds)

      } else if (last.op == "br") {
        (last as Others).labels!.forEach((label) => {
          var to_jump: Block = label_blocks.get(label)!;
          successors.push(to_jump)
          var preds = cfg_pres.get(to_jump)!;
          preds.push(block)
          cfg_pres.set(to_jump, preds)
        })
      }
      else {
        if (i < blocks.length - 1) {
          successors.push(blocks[i + 1])
          // add i as predecessor to i+1
          var preds = cfg_pres.get(blocks[i + 1])!;
          preds.push(block);
          cfg_pres.set(blocks[i + 1], preds)
        }
      }
    }
    cfg_sucs.set(block, JSON.parse(JSON.stringify(successors)))
  }
  return { blocks, cfg_sucs, cfg_pres }
}

//assumption: cfg maps a block to all its parents
function dataflow(blocks: Block[], cfg_pres: Map<Block, Block[]>, cfg_sucs: Map<Block, Block[]>, func: Function) {
  // reaching definitions
  // forward analysis, transition fn: f(in_b) = (in_b - killed_b) U defs_b 
  // use union to merge blocks

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

  var worklist: Block[] = blocks;
  var inputs: Map<Block, Set<Definition>> = new Map();
  var outputs: Map<Block, Set<Definition>> = new Map();

  // initialize in[first block] to be the function arguments, if they exist. else init to new Set
  var init_in: Set<Definition> = new Set()
  if ("args" in func) {
    var arg_names = func.args!.map((arg) => { var def = new Definition(); def.dest = arg.name; def.value = "unk"; return def })
    init_in = new Set(arg_names)
  }
  inputs.set(blocks[0], init_in)

  blocks.forEach((block) => {
    outputs.set(block, new Set())
  })

  while (worklist.length > 0) {
    var b: Block = worklist.pop()!;
    var preds: Block[] = cfg_pres.get(b)!; //b should def be in the cfg

    // use the intersection to merge
    var reduction: Set<Definition> = preds.reduce((acc: Set<Definition>, curr: Block) => acc.union(outputs.get(curr)!), new Set())
    inputs.set(b, reduction);

    //apply transfer function
    var output_b: Set<Definition> = reaching_defs_transfer(b, reduction)

    // check if there was a change 
    if (output_b.difference(outputs.get(b)!).size > 0 || (outputs.get(b)!).difference(output_b).size > 0) {
      worklist.push(...cfg_sucs.get(b)!);
    }
    outputs.set(b, output_b);
  }

}

// Transfer function for reaching defs data flow analysis
function reaching_defs_transfer(block: Block, inputs: Set<Definition>) {
  var killed: Set<Definition> = new Set();
  var defs: Set<Definition> = new Set();
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
      var defn: Definition = new Definition();
      defn.dest = insn.dest!;
      defn.value = val;
      defs.add(defn);
      // remove anything this def kills
      inputs.forEach((item) => {
        if (item.dest == insn.dest!) {
          killed.add(item)
        }
      })
    }
  })
  return defs.union(inputs.difference(killed)); //res = (in - killed) U defs
}

function generic_workflow() {

}

function main() {
  program.functions.forEach((func) => {
    var { blocks, cfg_sucs, cfg_pres } = build_cfg(func)
    // test build cfg
    console.log("successors cfg graph: ")
    for (let i = 0; i < blocks.length; i++) {
      var block: Block = blocks[i];
      var successors = cfg_sucs.get(block)!;
      console.log("block: \n" + block_to_string(blocks[i]) + "has successors: \n")
      successors.forEach((block) => {
        console.log(block_to_string(block))
      })
    }

    console.log("predecessors cfg graph: ")
    for (let i = 0; i < blocks.length; i++) {
      var block: Block = blocks[i];
      var preds = cfg_pres.get(block)!;
      console.log("block: \n" + block_to_string(blocks[i]) + "has predecessors: \n")
      preds.forEach((block) => {
        console.log(block_to_string(block))
      })
    }
    //dataflow(blocks, cfg_pres, cfg_sucs, func)
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