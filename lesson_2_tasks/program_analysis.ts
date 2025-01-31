/// <reference lib="es2020" />

// print variables that are not used after creation
import program from "./contrived_cfg.json"

type Type = string | { ptr: string }
type TermInsn = { op: string, labels: string[], args?: string[] }
type Label = { label: string }

type Others = {
  op: string,
  dest?: string,
  args?: string[],
  value?: number,
  type?: Type,
  funcs?: string[]
}
type Instruction = Others | Label | TermInsn
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

function main() {
  var vars_count: Map<string, number> = new Map()// store how many times var was used
  var label_blocks: Map<string, Block> = new Map()
  var blocks: Block[] = []
  var current_block: Instruction[] = []

  // iterate over program - collect blocks and count variable usage
  program.functions.forEach((func: Function) => {
    func.instrs.forEach((insn: Instruction) => {
      if (is_term(insn)) {
        current_block.push(insn)
        var to_push = new Block()
        console.log("term block: " + current_block.length)

        to_push.insns = JSON.parse(JSON.stringify(current_block))
        blocks.push(to_push)
        current_block = []
      }
      else if ("label" in insn) { // end the previous block - if there is one, add to new one - LABEL IS ALWAYS FIRST INSTRUCTION
        if (current_block.length > 0) {
          var to_push = new Block()
          to_push.insns = JSON.parse(JSON.stringify(current_block))
          console.log("label block: " + current_block.length)
          blocks.push(to_push)
          current_block = []
        }
        current_block.push(insn)
      }
      else {
        if ("dest" in insn) {
          var curr = vars_count.has(insn.dest!) ? vars_count.get(insn.dest!)! + 1 : 1
          vars_count.set(insn.dest!, curr)

        }
        if ("args" in insn) { // this checks it exists so it must exist
          insn.args!.forEach((arg) => {
            var curr = vars_count.has(arg) ? vars_count.get(arg)! + 1 : 1
            vars_count.set(arg, curr)
          })
        }
        current_block.push(insn)
      }
    })
    //push last block in function:
    var to_push = new Block()
    console.log("last block: " + current_block.length)
    to_push.insns = JSON.parse(JSON.stringify(current_block))
    blocks.push(to_push)
    current_block = []

  })


  // iterate over blocks and collect labels
  blocks.forEach((block: Block) => {
    if ("label" in block.insns[0]) {
      label_blocks.set(block.insns[0].label, block) // works due to how i create blocks
    }
  })

  // iterate over blocks and form cfg
  var cfg: Block[][] = []
  for (let i: number = 0; i < blocks.length; i++) {
    var successors: Block[] = []
    var block: Block = blocks[i]
    var last: Instruction = block.insns[block.insns.length - 1]

    if ("op" in last) {
      if (last.op == "jmp") {
        successors.push(label_blocks.get((last as TermInsn).labels[0])!)
      } else if (last.op == "br") {
        (last as TermInsn).labels.forEach((label) => {
          successors.push(label_blocks.get(label)!)
        })
      }
      else {
        if (i < blocks.length - 1) {
          successors.push(blocks[i + 1])
        }
      }
    }
    cfg.push(JSON.parse(JSON.stringify(successors)))
  }

  console.log("variable counts: ")
  vars_count.forEach((val: number, key: string) => {
    var p = val - 1 //creating it doesn't count as a use
    console.log(key + ":" + p);
  })

  console.log("cfg graph: ")
  for (let i = 0; i < blocks.length; i++) {
    successors = cfg[i]
    console.log("block: \n" + block_to_string(blocks[i]) + "has successors: \n")
    successors.forEach((block) => {
      console.log(block_to_string(block))
    })
  }
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

function is_term(insn: Instruction) {
  if ((insn as TermInsn).op == "br" || (insn as TermInsn).op == "jmp") {
    return true
  }
  return false
}

main()