use crate::types::GpuTypeDetails;
use derive_where::derive_where;
use dyn_clone::DynClone;
use std::any::TypeId;

#[derive(Debug)]
#[allow(private_interfaces)]
pub enum Value {
    Glob(Glob),
    Var(Var),
}

impl Value {
    pub(crate) fn value_type_id(&self) -> TypeId {
        match self {
            Self::Glob(value) => value.type_id,
            Self::Var(value) => value.type_id,
        }
    }
}

pub(crate) trait DefaultGlobValueFn: DynClone {
    fn call(&self) -> Value;
}

impl<F> DefaultGlobValueFn for F
where
    F: Fn() -> Value + Clone,
{
    fn call(&self) -> Value {
        self()
    }
}

#[derive_where(Debug)]
pub(crate) struct Glob {
    pub(crate) module: &'static str,
    pub(crate) id: u64,
    pub(crate) type_id: TypeId,
    #[derive_where(skip)]
    pub(crate) default_value: Box<dyn DefaultGlobValueFn + Sync + Send>,
}

impl PartialEq for Glob {
    fn eq(&self, other: &Self) -> bool {
        self.module == other.module && self.id == other.id
    }
}

impl Eq for Glob {}

impl Clone for Glob {
    fn clone(&self) -> Self {
        Self {
            module: self.module,
            id: self.id,
            type_id: self.type_id,
            default_value: dyn_clone::clone_box(&*self.default_value),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct Var {
    pub(crate) id: u64,
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
