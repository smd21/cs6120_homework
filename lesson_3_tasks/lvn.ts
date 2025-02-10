/// <reference lib="es2020" />

// print variables that are not used after creation
import program from "./test_optimization.json"

type Unk = string
type Val_Tup = [string, number, number]
type Value = number | Unk | Val_Tup

class Row {
  val: Value
  variable: string
}

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
  // do optimization one function at a time
  program.functions.forEach((func: Function) => {
    // collect function blocks
    var blocks: Block[] = build_blocks(func)
    // apply llvn and renaming
    var llvn_blocks = do_llvn(blocks)
    // dead code elimination pass
    var new_blocks = do_dce(llvn_blocks)
    // assign optimized blocks to function
    func.instrs = flatten_blocks(new_blocks)
  })
  var result = JSON.stringify(program)
  console.log(result)
}

function print_blocks(blocks: Block[]) {
  blocks.forEach((block) => {
    block.insns.forEach((insn) => {

    })
  })
}
function print_instruction(insn: Instruction) {
  if ("label" in insn) {
    console.log(insn.label!)
    return
  }
  switch (insn.op) {
    case "branch":
      console.log("branch " + insn.labels![0] + " " + insn.labels![1])
      break
    case "jmp":
      console.log("jmp " + insn.labels![0])
      break
    case "ret":
      var arg = ""
      if ("args" in insn) {
        arg = insn.args![0]
      }
      console.log("return " + arg)
      break
    case "nop":
      console.log("nop")
      break
    case "const":
      console.log(insn.dest! + " = const " + insn.value!)
    default:
      console.log(insn.op)
      break

  }
}
function flatten_blocks(blocks: Block[]) {
  var instrs: Instruction[] = []
  blocks.forEach((block) => {
    instrs.push.apply(instrs, block.insns) // add all instructions
  })
  return instrs
}

function do_llvn(blocks: Block[]) {
  var table: Row[] = []
  var table_idx: number = 0
  var fresh_vars = 0
  var vartorow: Map<string, number> = new Map()
  var valtorow: Map<Value, number> = new Map()


  blocks.forEach((block) => {
    // collect table 
    block.insns.forEach((insn) => {
      if ("dest" in insn) {
        // if const insn
        var row = new Row()
        var val: Value

        if ("value" in insn) {
          val = insn.value!
        }
        else { //insn has args. im sure this should be the case
          var args = insn.args!
          // ensure all args are in the table - unk any unknown args
          // these do not get added to seen values lol
          args.forEach((arg, idx) => {
            if (!vartorow.has(arg)) {
              var add_unk = new Row()
              add_unk.variable = arg
              add_unk.val = "unk"
              table.push(add_unk)
              vartorow.set(arg, table_idx)
              table_idx++
            }
            else { // canonicalize
              this[idx] = table[vartorow[arg]].variable
            }
          }, args)
          // set tuple
          var op: string = insn.op
          var arg1: number = vartorow.get(args[0])!
          var arg2: number = vartorow.get(args[1])!

          // commutativity for add + mul. arrange in order of smaller row to larger row
          if (op == "add" || op == "mul") {
            var smaller = arg1 < arg2 ? arg1 : arg2
            var larger = arg1 > arg2 ? arg1 : arg2
            arg1 = smaller
            arg2 = larger
          }

          val = [op, arg1, arg2]
          //reset args to canonicalized ones
          insn.args = args
        }

        // check if value is already in table
        if (valtorow.has(val)) {
          // if already in the table, add pointer to that row
          var r = valtorow.get(val)!
          vartorow.set(row.variable, r)

          // replace this instruction with dest = id var_in_table
          insn.op = "id"
          insn.args = [table[r].variable]

        } else {
          // add to table and add val/var mappings
          table[table_idx] = row
          vartorow.set(row.variable, table_idx)
          valtorow.set(row.val, table_idx)
          table_idx++
        }

        // check if dest already in table ("reassigned"), if so then generate fresh name
        row.variable = insn.dest!
        if (vartorow.has(insn.dest!)) {
          var fresh: string = "var" + fresh_vars
          fresh_vars++
          row.variable = fresh
          insn.dest = fresh // update instruction to match
        }
      }
    })
  })

  return blocks
}

// Constructs the blocks for a given function func
function build_blocks(func: Function) {
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
  return blocks
}

// performs dead code elimination over a list of blocks
function do_dce(blocks: Block[]) {
  var num_deletions = 1
  // iterate over insns, delete if dest is not in used instrs
  while (num_deletions != 0) {
    num_deletions = 0
    var used: Set<string> = new Set();
    // add all used arguments to set
    blocks.forEach((block) => {
      block.insns.forEach((insn) => {
        if ("args" in insn) {
          insn.args!.forEach((arg) => {
            used.add(arg)
          })
        }
      })
    })
    blocks.forEach((block, idx) => {
      var new_block: Block = new Block()
      var new_insns: Instruction[] = []
      block.insns.forEach((insn) => {
        var del: boolean = false
        if ("dest" in insn) {
          if (!used.has(insn.dest!)) { //to delete
            num_deletions++
            del = true
          }
        }
        if (!del) { //add to new block
          new_insns.push(insn)
        }
      })
      new_block.insns = new_insns
      this[idx] = new_block
    }, blocks)
  }
  return blocks
}

main()