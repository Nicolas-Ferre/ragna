use crate::types::GpuTypeDetails;
use crate::{GpuValue, Wgsl};

#[derive(Debug)]
#[allow(clippy::large_enum_variant)]
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
    pub(crate) left_value: GpuValue,
    pub(crate) right_value: GpuValue,
}

#[derive(Debug)]
pub(crate) struct ConstantAssignVarOperation {
    pub(crate) left_value: GpuValue,
    pub(crate) right_value: Wgsl,
}

#[derive(Debug)]
pub(crate) struct UnaryOperation {
    pub(crate) var: GpuValue,
    pub(crate) value: GpuValue,
    pub(crate) operator: &'static str,
}

#[derive(Debug)]
pub(crate) struct BinaryOperation {
    pub(crate) var: GpuValue,
    pub(crate) left_value: GpuValue,
    pub(crate) right_value: GpuValue,
    pub(crate) operator: &'static str,
}

#[derive(Debug)]
pub(crate) struct FnCallOperation {
    pub(crate) var: GpuValue,
    pub(crate) fn_name: &'static str,
    pub(crate) args: Vec<GpuValue>,
    // Whether the WGSL function accepts WGSL `bool` (i.e. boolean `u32` values should converted).
    pub(crate) is_supporting_bool: bool,
}

#[derive(Debug)]
pub(crate) struct IfOperation {
    pub(crate) condition: GpuValue,
}
