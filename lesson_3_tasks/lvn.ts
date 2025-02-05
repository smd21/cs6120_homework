/// <reference lib="es2020" />

// print variables that are not used after creation
import program from "./test_optimization.json"

type UNKNOWN = string
type Value = number | UNKNOWN
type Row = {
  val: Value,
  variable: string
}

// types for making blocks 
type Type = string | { ptr: string }
//type TermInsn = { op: string, labels: string[], args?: string[] }
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
type Instruction = Others | Label //| TermInsn
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
  //iterate over program and make blocks (use old algorithm)
  var blocks: Block[] = []
  var current_block: Instruction[] = []

  // iterate over program - collect blocks
  program.functions.forEach((func: Function) => {
    func.instrs.forEach((insn: Instruction) => {
      if ((insn as Others).op == "br" || (insn as Others).op == "jmp") {
        current_block.push(insn)
        var to_push = new Block()
        console.log("term block: " + current_block.length)

        to_push.insns = JSON.parse(JSON.stringify(current_block))
        blocks.push(to_push)
        current_block = []
      }
      else {
        if ("label" in insn) { // end the previous block - if there is one, add to new one - LABEL IS ALWAYS FIRST INSTRUCTION
          if (current_block.length > 0) {
            var to_push = new Block()
            to_push.insns = JSON.parse(JSON.stringify(current_block))
            console.log("label block: " + current_block.length)
            blocks.push(to_push)
            current_block = []
          }
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

}
