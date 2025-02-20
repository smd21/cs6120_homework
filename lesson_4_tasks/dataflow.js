"use strict";
/// <reference lib="es2020" />
var __read = (this && this.__read) || function (o, n) {
    var m = typeof Symbol === "function" && o[Symbol.iterator];
    if (!m) return o;
    var i = m.call(o), r, ar = [], e;
    try {
        while ((n === void 0 || n-- > 0) && !(r = i.next()).done) ar.push(r.value);
    }
    catch (error) { e = { error: error }; }
    finally {
        try {
            if (r && !r.done && (m = i["return"])) m.call(i);
        }
        finally { if (e) throw e.error; }
    }
    return ar;
};
var __spreadArray = (this && this.__spreadArray) || function (to, from, pack) {
    if (pack || arguments.length === 2) for (var i = 0, l = from.length, ar; i < l; i++) {
        if (ar || !(i in from)) {
            if (!ar) ar = Array.prototype.slice.call(from, 0, i);
            ar[i] = from[i];
        }
    }
    return to.concat(ar || Array.prototype.slice.call(from));
};
var __values = (this && this.__values) || function(o) {
    var s = typeof Symbol === "function" && Symbol.iterator, m = s && o[s], i = 0;
    if (m) return m.call(o);
    if (o && typeof o.length === "number") return {
        next: function () {
            if (o && i >= o.length) o = void 0;
            return { value: o && o[i++], done: !o };
        }
    };
    throw new TypeError(s ? "Object is not iterable." : "Symbol.iterator is not defined.");
};
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
var gcd_json_1 = __importDefault(require("../lesson_3_tasks/gcd.json"));
var Block = /** @class */ (function () {
    function Block() {
    }
    return Block;
}());
var Definition = /** @class */ (function () {
    function Definition() {
    }
    Definition.prototype.to_string = function () {
        return this.dest + ": " + this.value;
    };
    Definition.prototype.print = function () {
        console.log(this.dest + ": " + this.value);
    };
    return Definition;
}());
// Constructs the cfg for a given function func
function build_cfg(func) {
    var label_blocks = new Map();
    var blocks = []; // store index of all blocks
    var curr_block_idx = 0;
    var current_block = [];
    func.instrs.forEach(function (insn) {
        if (insn.op == "br" || insn.op == "jmp") {
            current_block.push(insn);
            var to_push = new Block();
            to_push.insns = JSON.parse(JSON.stringify(current_block));
            blocks.push(to_push);
            curr_block_idx++;
            current_block = [];
        }
        else {
            if ("label" in insn) { // end the previous block - if there is one, add to new one - LABEL IS ALWAYS FIRST INSTRUCTION
                if (current_block.length > 0) {
                    var to_push = new Block();
                    to_push.insns = JSON.parse(JSON.stringify(current_block));
                    blocks.push(to_push);
                    curr_block_idx++;
                    current_block = [];
                }
            }
            current_block.push(insn);
        }
    });
    //push last block in function:
    var to_push = new Block();
    to_push.insns = JSON.parse(JSON.stringify(current_block));
    blocks.push(to_push);
    curr_block_idx++;
    // iterate over blocks and collect labels
    blocks.forEach(function (block) {
        if ("label" in block.insns[0]) {
            label_blocks.set(block.insns[0].label, block); // works due to how i create blocks
        }
    });
    //create block idx map
    var block_to_idx = new Map();
    for (var i = 0; i < blocks.length; i++) {
        block_to_idx.set(blocks[i], i);
    }
    // map based on block indexesz
    var cfg_sucs = new Map();
    var cfg_pres = new Map();
    //initialize to empty for all blocks
    block_to_idx.forEach(function (label, _block) {
        cfg_pres.set(label, []);
    });
    for (var i = 0; i < blocks.length; i++) {
        var block = blocks[i];
        var successors = [];
        var last = block.insns[block.insns.length - 1];
        if ("op" in last) {
            if (last.op == "jmp") {
                var to_jump = label_blocks.get(last.labels[0]);
                var tj_idx = block_to_idx.get(to_jump);
                successors.push(tj_idx);
                var preds = cfg_pres.get(tj_idx);
                preds.push(i); //idx of current block
                cfg_pres.set(tj_idx, preds);
            }
            else if (last.op == "br") {
                last.labels.forEach(function (label) {
                    var to_jump = label_blocks.get(label);
                    var tj_idx = block_to_idx.get(to_jump);
                    successors.push(tj_idx);
                    var preds = cfg_pres.get(tj_idx);
                    preds.push(i);
                    cfg_pres.set(tj_idx, preds);
                });
            }
            else {
                if (i < blocks.length - 1) {
                    var next = block_to_idx.get(blocks[i + 1]); //index of next block
                    successors.push(next);
                    // add i as predecessor to i+1
                    var preds = cfg_pres.get(next); //get predecessors of next block
                    preds.push(i); //add current block
                    cfg_pres.set(next, preds); //set in cfg
                }
            }
        }
        cfg_sucs.set(i, JSON.parse(JSON.stringify(successors)));
    }
    return { block_to_idx: block_to_idx, blocks: blocks, cfg_sucs: cfg_sucs, cfg_pres: cfg_pres };
}
//assumption: cfg maps a block to all its parents
function dataflow(blocks, cfg_pres, cfg_sucs, func) {
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
    var worklist = [];
    var inputs = new Map();
    var outputs = new Map();
    // initialize worklist with block indexes and outputs/inputs with empty sets
    for (var i = 0; i < blocks.length; i++) {
        worklist.push(i);
        outputs.set(i, new Set());
        inputs.set(i, new Set());
    }
    // initialize in[first block] to be the function arguments, if they exist. else init to new Set
    var init_in = new Set();
    if ("args" in func) {
        var arg_names = func.args.map(function (arg) { var def = new Definition(); def.dest = arg.name; def.value = "unk"; return def.to_string(); });
        init_in = new Set(arg_names);
        console.log(init_in.size);
    }
    inputs.set(0, init_in);
    while (worklist.length > 0) {
        var b_idx = worklist.pop();
        //console.log("looking at: " + block_to_string(blocks[b_idx]));
        var preds = cfg_pres.get(b_idx); //b should def be in the cfg
        //console.log("size: " + (cfg_pres.get(b_idx)!).length);
        // use the intersection to merge
        var reduction = preds.reduce(function (acc, curr) { return new Set(__spreadArray(__spreadArray([], __read(acc), false), __read((outputs.get(curr))), false)); }, new Set()); //acc.union(outputs.get(curr)!)
        //append to current input
        if (inputs.get(b_idx).size > 0) {
            reduction = new Set(__spreadArray(__spreadArray([], __read(reduction), false), __read(inputs.get(b_idx)), false));
        }
        inputs.set(b_idx, reduction);
        //apply transfer function
        var output_b = reaching_defs_transfer(blocks[b_idx], reduction);
        // check if there was a change 
        if (set_difference(output_b, outputs.get(b_idx)).size > 0 || set_difference(outputs.get(b_idx), output_b).size > 0) {
            worklist.push.apply(worklist, __spreadArray([], __read(cfg_sucs.get(b_idx)), false));
        }
        outputs.set(b_idx, output_b);
    }
    return outputs;
}
// Transfer function for reaching defs data flow analysis
function reaching_defs_transfer(block, inputs) {
    var killed = new Set();
    var defs = new Set();
    block.insns.forEach(function (insn) {
        if ("dest" in insn) {
            // add to defs
            var val = insn.op + " ";
            if ("value" in insn) {
                val += insn.value.toString();
            }
            if ("args" in insn) {
                val += insn.args.reduce(function (acc, curr) { return acc + " " + curr; }, "");
            }
            var defn = new Definition();
            defn.dest = insn.dest;
            defn.value = val;
            defs.add(defn.to_string());
            // remove anything this def kills
            inputs.forEach(function (item) {
                if (item.split(":")[0] == insn.dest) {
                    killed.add(item);
                }
            });
        }
    });
    var diff = set_difference(inputs, killed);
    return new Set(__spreadArray(__spreadArray([], __read(defs), false), __read(diff), false)); //defs.union(diff); //res = (in - killed) U defs
}
/// helper method that takes in two sets A and B and returns A - B
function set_difference(A, B) {
    var diff = new Set(__spreadArray([], __read(A), false).filter(function (x) { return !B.has(x); }));
    return diff;
}
function generic_workflow() {
}
function main() {
    gcd_json_1.default.functions.forEach(function (func) {
        var _a = build_cfg(func), block_to_idx = _a.block_to_idx, blocks = _a.blocks, cfg_sucs = _a.cfg_sucs, cfg_pres = _a.cfg_pres;
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
        outputs.forEach(function (val, key) {
            var e_1, _a;
            console.log("defs that reach the end of block " + block_to_string(blocks[key]));
            try {
                //iterate over set and print definitions
                for (var _b = __values(val.values()), _c = _b.next(); !_c.done; _c = _b.next()) {
                    var defn = _c.value;
                    console.log(defn);
                }
            }
            catch (e_1_1) { e_1 = { error: e_1_1 }; }
            finally {
                try {
                    if (_c && !_c.done && (_a = _b.return)) _a.call(_b);
                }
                finally { if (e_1) throw e_1.error; }
            }
        });
    });
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
main();
