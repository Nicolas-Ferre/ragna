use crate::types::{GpuTypeDetails, GpuValueRoot};
use derive_where::derive_where;
use std::any::TypeId;

#[derive(Debug, Clone)]
#[derive_where(PartialEq, Eq, Hash)]
pub struct Value {
    #[derive_where(skip)]
    pub(crate) type_id: TypeId,
    pub(crate) root: GpuValueRoot,
    pub(crate) extensions: Vec<ValueExt>,
}

impl Value {
    pub(crate) fn root_value<'a>(&self, globs: &'a [Self]) -> &'a Self {
        globs
            .iter()
            .find(|glob| glob.root == self.root)
            .expect("internal error: root value should be a glob")
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum ValueExt {
    FieldPosition(u8),
    FieldName(&'static str),
    IndexVarId(u32),
}

#[derive(Debug)]
pub(crate) enum Operation {
    DeclareVar(DeclareVarOperation),
    AssignVar(AssignVarOperation),
    ConstantAssignVar(ConstantAssignVarOperation),
    Unary(UnaryOperation),
    Binary(BinaryOperation),
    FnCall(FnCallOperation),
    IfBlock(IfOperation),
    ElseBlock,
    LoopBlock,
    EndBlock,
    Break,
    Continue,
}

#[derive(Debug)]
pub(crate) struct DeclareVarOperation {
    pub(crate) id: u32,
    pub(crate) type_: GpuTypeDetails,
}

#[derive(Debug)]
pub(crate) struct AssignVarOperation {
    pub(crate) left_value: Value,
    pub(crate) right_value: Value,
}

#[derive(Debug)]
pub(crate) struct ConstantAssignVarOperation {
    pub(crate) left_value: Value,
    pub(crate) right_value: String,
}

#[derive(Debug)]
pub(crate) struct UnaryOperation {
    pub(crate) var: Value,
    pub(crate) value: Value,
    pub(crate) operator: &'static str,
}

#[derive(Debug)]
pub(crate) struct BinaryOperation {
    pub(crate) var: Value,
    pub(crate) left_value: Value,
    pub(crate) right_value: Value,
    pub(crate) operator: &'static str,
}

#[derive(Debug)]
pub(crate) struct FnCallOperation {
    pub(crate) var: Value,
    pub(crate) fn_name: &'static str,
    pub(crate) args: Vec<Value>,
}

#[derive(Debug)]
pub(crate) struct IfOperation {
    pub(crate) condition: Value,
}
