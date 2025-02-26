use crate::types::GpuTypeDetails;
use derive_where::derive_where;
use std::any::TypeId;

#[derive(Debug)]
#[allow(private_interfaces)]
pub enum Value {
    Glob(Glob),
    Var(Var),
    Field(Field),
}

impl Value {
    pub(crate) fn value_type_id(&self) -> TypeId {
        match self {
            Self::Glob(value) => value.type_id,
            Self::Var(value) => value.type_id,
            Self::Field(value) => value.type_id,
        }
    }
}

#[derive(Debug, Clone)]
#[derive_where(PartialEq, Eq)]
pub(crate) struct Glob {
    pub(crate) module: &'static str,
    pub(crate) id: u64,
    #[derive_where(skip)]
    pub(crate) type_id: TypeId,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct Var {
    pub(crate) id: u64,
    pub(crate) type_id: TypeId,
}

#[derive(Debug)]
pub(crate) struct Field {
    pub(crate) source: Box<Value>,
    pub(crate) indexes: Vec<usize>,
    pub(crate) type_id: TypeId,
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
    pub(crate) id: u64,
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
