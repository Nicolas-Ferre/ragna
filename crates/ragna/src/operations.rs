use crate::types::GpuTypeDetails;
use crate::GpuContext;
use derive_where::derive_where;
use dyn_clone::DynClone;
use std::any::TypeId;

#[derive(Debug)]
#[allow(private_interfaces)]
pub enum Value {
    Constant(Constant),
    Glob(Glob),
    Var(Var),
}

impl Value {
    pub(crate) fn value_type_id(&self) -> TypeId {
        match self {
            Self::Constant(value) => value.type_id,
            Self::Glob(value) => value.type_id,
            Self::Var(value) => value.type_id,
        }
    }
}

#[derive(Debug)]
pub(crate) struct Constant {
    pub(crate) value: String,
    pub(crate) type_id: TypeId,
    pub(crate) gpu_type: GpuTypeDetails,
}

pub(crate) trait DefaultGlobValueFn: DynClone {
    fn call(&self, ctx: &mut GpuContext) -> Value;
}

impl<F> DefaultGlobValueFn for F
where
    F: Fn(&mut GpuContext) -> Value + Clone,
{
    fn call(&self, ctx: &mut GpuContext) -> Value {
        self(ctx)
    }
}

#[derive_where(Debug)]
pub(crate) struct Glob {
    pub(crate) module: &'static str,
    pub(crate) id: u64,
    pub(crate) type_id: TypeId,
    #[derive_where(skip)]
    pub(crate) default_value: Box<dyn DefaultGlobValueFn>,
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
    Unary(UnaryOperation),
    Binary(BinaryOperation),
    FnCall(FnCallOperation),
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
