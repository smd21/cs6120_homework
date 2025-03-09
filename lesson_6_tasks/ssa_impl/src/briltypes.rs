use serde::{Deserialize, Serialize};

// #[derive(Serialize, Deserialize, Clone, Copy)]
// #[serde(tag = "type")]
// pub enum Types {
//     Int(i32),
//     Boolean(bool),
// }

#[derive(Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum InsnType {
    Label {
        label: String,
    },
    Terminator {
        op: String,
        labels: Vec<String>,
        args: Option<Vec<String>>,
    },
    Constant {
        op: String,
        dest: String,
        insn_type: String,
        value: String,
    },
    ValOp {
        op: String,
        insn_type: String,
        dest: String,
        args: Option<Vec<String>>,
        funcs: Option<Vec<String>>,
    },
    Effect {
        op: String,
        args: Option<Vec<String>>,
    },
}

#[derive(Serialize, Deserialize)]
pub struct Function {
    pub name: String,
    pub args: Option<Vec<String>>,
    pub instructions: Vec<InsnType>,
    pub func_type: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct Program {
    pub functions: Vec<Function>,
}
