/// <reference lib="es2020" />

// print variables that are not used after creation
import program from "./test_optimization.json"


type Unk = string
class Val_Tup {
  op: string
  arg1: number
  arg2: number
}

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
  var valtorow: Map<string, number> = new Map()


  blocks.forEach((block, idx) => {
    // collect table 
    var new_block: Block = new Block()
    var new_insns: Instruction[] = []
    block.insns.forEach((insn) => {
      var new_insn: Instruction = insn
      if ("dest" in insn) {
        // if const insn
        var row = new Row()
        var val: Value
        var val_map_key: string = ""

        if ("value" in insn) {
          console.log("value insn")
          val = insn.value!
          val_map_key = insn.value!.toString()
        }
        else { //insn has args. im sure this should be the case
          var argis = insn.args!
          // ensure all args are in the table - unk any unknown args
          argis.forEach((arg, idx) => {
            if (!vartorow.has(arg)) {
              var add_unk = new Row()
              add_unk.variable = arg
              add_unk.val = "unk"
              table.push(add_unk)
              vartorow.set(arg, table_idx)
              table_idx++
            }
            else { // canonicalize
              console.log("104: " + table[vartorow.get(arg)!].variable + ": " + table[vartorow.get(arg)!].val)
              argis[idx] = table[vartorow.get(arg)!].variable
            }
          }, argis)
          // set tuple
          var op: string = insn.op
          var arg1: number = vartorow.get(argis[0])!
          var arg2: number = -1 //default to handle only one argument
          if (argis.length == 2) {
            arg2 = vartorow.get(argis[1])!
          }
          console.log("arg1: " + arg1 + " arg2: " + arg2);

          // commutativity for add + mul. arrange in order of smaller row to larger row
          if (op == "add" || op == "mul") {
            console.log("115: commutative")
            var smaller = arg1 < arg2 ? arg1 : arg2
            var larger = arg1 > arg2 ? arg1 : arg2
            console.log("smaller: " + smaller + " larger: " + larger);
            arg1 = smaller
            arg2 = larger
            console.log("commurarive: arg1: " + arg1 + " arg2: " + arg2);

          }

          val = new Val_Tup();
          val.op = op;
          val.arg1 = arg1;
          val.arg2 = arg2;
          val_map_key = op + arg1.toString() + arg2.toString(); //string lel
          //reset args to canonicalized ones - argis has no -1 so we all good here
          (new_insn as Others).args = argis;
        }
        row.val = val // set value
        // check if dest already in table ("reassigned"), if so then generate fresh name
        row.variable = insn.dest!
        if (vartorow.has(insn.dest!)) {
          var fresh: string = "freshvar" + fresh_vars;
          fresh_vars++;
          row.variable = fresh;
          console.log("132: fresh: " + fresh);
          (new_insn as Others).dest = fresh; // update instruction to match
        }

        // check if value is already in table
        if (valtorow.has(val_map_key)) { // this check doesn't work properly lol
          console.log("138: val in table")
          // if already in the table, add pointer to that row
          var r = valtorow.get(val_map_key)!
          vartorow.set(row.variable, r);

          // replace this instruction with dest = id var_in_table
          (new_insn as Others).op = "id";
          (new_insn as Others).args = [table[r].variable];

        } else {
          // add to table and add val/var mappings
          console.log("149: added: " + row.variable + " to table idx " + table_idx);
          table[table_idx] = row
          vartorow.set(row.variable, table_idx)
          valtorow.set(val_map_key, table_idx)
          table_idx++
        }
      }
      new_insns.push(new_insn);
    })
    new_block.insns = new_insns
    blocks[idx] = new_block;
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
      blocks[idx] = new_block
    })
  }
  return blocks
}

main()