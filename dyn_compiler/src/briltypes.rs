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
    ValOp {
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        args: Vec<String>,
        dest: String,
        op: ValOps,
        #[serde(rename = "type")]
        insn_type: String,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        funcs: Vec<String>,
    },
    Effect {
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        funcs: Vec<String>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        args: Vec<String>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        labels: Vec<String>,
        op: EffectOps,
    },
    Constant {
        dest: String,
        op: ConstOp,
        #[serde(rename = "type")]
        insn_type: String,
        value: Value,
    },
}
#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
#[serde(untagged)]
pub enum Value {
    Int(i32),
    Bools,
}
#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum Bools {
    True,
    False,
}
#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum ValOps {
    Add,
    Sub,
    Mul,
    Div,
    Lt,
    Gt,
    Le,
    Ge,
    Eq,
    Not,
    And,
    Or,
    Id,
    Call,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum EffectOps {
    #[serde(rename = "jmp")]
    Jmp,
    #[serde(rename = "br")]
    Branch,
    #[serde(rename = "call")]
    Call,
    #[serde(rename = "ret")]
    Return,
    #[serde(rename = "print")]
    Print,
    #[serde(rename = "speculate")]
    Speculate,
    #[serde(rename = "guard")]
    Guard,
    #[serde(rename = "commit")]
    Commit,
    #[serde(rename = "nop")]
    Nop,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub enum ConstOp {
    #[serde(rename = "const")]
    Const,
}
#[derive(Serialize, Deserialize)]
pub struct Argument {
    name: String,
    #[serde(rename = "type")]
    arg_type: String,
}
#[derive(Serialize, Deserialize)]
pub struct Function {
    pub name: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub args: Vec<Argument>,
    pub instrs: Vec<InsnType>,
    #[serde(rename = "type")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub func_type: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct Program {
    pub functions: Vec<Function>,
}
