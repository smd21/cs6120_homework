"use strict";
/// <reference lib="es2020" />
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
// print variables that are not used after creation
var contrived_cfg_json_1 = __importDefault(require("./contrived_cfg.json"));
var Block = /** @class */ (function () {
    function Block() {
    }
    return Block;
}());
function main() {
    var vars_count = new Map(); // store how many times var was used
    var label_blocks = new Map();
    var blocks = [];
    var current_block = [];
    // iterate over program - collect blocks and count variable usage
    contrived_cfg_json_1.default.functions.forEach(function (func) {
        func.instrs.forEach(function (insn) {
            if (is_term(insn)) {
                current_block.push(insn);
                var to_push = new Block();
                console.log("term block: " + current_block.length);
                to_push.insns = JSON.parse(JSON.stringify(current_block));
                blocks.push(to_push);
                current_block = [];
            }
            else if ("label" in insn) { // end the previous block - if there is one, add to new one - LABEL IS ALWAYS FIRST INSTRUCTION
                if (current_block.length > 0) {
                    var to_push = new Block();
                    to_push.insns = JSON.parse(JSON.stringify(current_block));
                    console.log("label block: " + current_block.length);
                    blocks.push(to_push);
                    current_block = [];
                }
                current_block.push(insn);
            }
            else {
                if ("dest" in insn) {
                    var curr = vars_count.has(insn.dest) ? vars_count.get(insn.dest) + 1 : 1;
                    vars_count.set(insn.dest, curr);
                }
                if ("args" in insn) { // this checks it exists so it must exist
                    insn.args.forEach(function (arg) {
                        var curr = vars_count.has(arg) ? vars_count.get(arg) + 1 : 1;
                        vars_count.set(arg, curr);
                    });
                }
                current_block.push(insn);
            }
        });
        //push last block in function:
        var to_push = new Block();
        console.log("last block: " + current_block.length);
        to_push.insns = JSON.parse(JSON.stringify(current_block));
        blocks.push(to_push);
        current_block = [];
    });
    // iterate over blocks and collect labels
    blocks.forEach(function (block) {
        if ("label" in block.insns[0]) {
            label_blocks.set(block.insns[0].label, block); // works due to how i create blocks
        }
    });
    // iterate over blocks and form cfg
    var cfg = [];
    for (var i = 0; i < blocks.length; i++) {
        var successors = [];
        var block = blocks[i];
        var last = block.insns[block.insns.length - 1];
        if ("op" in last) {
            if (last.op == "jmp") {
                successors.push(label_blocks.get(last.labels[0]));
            }
            else if (last.op == "br") {
                last.labels.forEach(function (label) {
                    successors.push(label_blocks.get(label));
                });
            }
            else {
                if (i < blocks.length - 1) {
                    successors.push(blocks[i + 1]);
                }
            }
        }
        cfg.push(JSON.parse(JSON.stringify(successors)));
    }
    console.log("variable counts: ");
    vars_count.forEach(function (val, key) {
        var p = val - 1; //creating it doesn't count as a use
        console.log(key + ":" + p);
    });
    console.log("cfg graph: ");
    for (var i = 0; i < blocks.length; i++) {
        successors = cfg[i];
        console.log("block: \n" + block_to_string(blocks[i]) + "has successors: \n");
        successors.forEach(function (block) {
            console.log(block_to_string(block));
        });
    }
}
function block_to_string(block) {
    var result = "";
    block.insns.forEach(function (instr) {
        if ("label" in instr) {
            result = result + instr.label + "\n";
        }
        else {
            result = result + instr.op;
            if ("labels" in instr) {
                result = result + instr.labels.toString();
            }
            result = result + "\n";
        }
    });
    return result;
}
function is_term(insn) {
    if (insn.op == "br" || insn.op == "jmp") {
        return true;
    }
    return false;
}
main();
