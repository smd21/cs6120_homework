"use strict";
/// <reference lib="es2020" />
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
// print variables that are not used after creation
var add_json_1 = __importDefault(require("./add.json"));
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
    add_json_1.default.functions.forEach(function (func) {
        func.instrs.forEach(function (insn) {
            if (is_term(insn)) {
                current_block.push(insn);
                var to_push = new Block();
                to_push.insns = JSON.parse(JSON.stringify(current_block));
                blocks.push(to_push);
                current_block = [];
            }
            else if ("label" in insn) { // end the previous block, add to new one - LABEL IS ALWAYS FIRST INSTRUCTION
                var to_push = new Block();
                to_push.insns = JSON.parse(JSON.stringify(current_block));
                blocks.push(to_push);
                current_block = [];
                current_block.push(insn);
            }
            else {
                if ("dest" in insn) {
                    console.log("has dest");
                    var curr = vars_count.has(insn.dest) ? vars_count.get(insn.dest) + 1 : 1;
                    vars_count.set(insn.dest, curr);
                }
                if ("args" in insn) { // this checks it exists so it must exist
                    console.log("has args");
                    insn.args.forEach(function (arg) {
                        var curr = vars_count.has(arg) ? vars_count.get(arg) + 1 : 1;
                        vars_count.set(arg, curr);
                    });
                }
                current_block.push(insn);
            }
        });
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
}
function block_to_string(block) {
    var result = "";
    return result;
}
function is_term(insn) {
    if (insn.op == "br" || insn.op == "jmp") {
        return true;
    }
    return false;
}
main();
