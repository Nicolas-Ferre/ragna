use crate::GpuContext;
use derive_where::derive_where;
use dyn_clone::DynClone;
use std::any::TypeId;
use std::hash::{Hash, Hasher};

#[derive(Debug)]
pub(crate) enum GpuValue {
    Constant(GpuConstant),
    Glob(GpuGlob),
    Var(GpuVar),
}

impl GpuValue {
    pub(crate) fn value_type_id(&self) -> TypeId {
        match self {
            Self::Constant(_) | Self::Glob(_) => unreachable!("get `TypeId` from constant/glob"),
            Self::Var(value) => value.type_id,
        }
    }

    fn glob(&self) -> Option<&GpuGlob> {
        if let Self::Glob(glob) = self {
            Some(glob)
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub(crate) struct GpuConstant {
    pub(crate) value: String,
    pub(crate) type_id: TypeId,
}

pub(crate) trait DefaultGlobValueFn: DynClone {
    fn call(&self, ctx: &mut GpuContext) -> GpuValue;
}

impl<F> DefaultGlobValueFn for F
where
    F: Fn(&mut GpuContext) -> GpuValue + Clone,
{
    fn call(&self, ctx: &mut GpuContext) -> GpuValue {
        self(ctx)
    }
}

#[derive_where(Debug)]
pub(crate) struct GpuGlob {
    pub(crate) module: &'static str,
    pub(crate) id: u64,
    pub(crate) type_id: TypeId,
    #[derive_where(skip)]
    pub(crate) default_value: Box<dyn DefaultGlobValueFn>,
}

impl PartialEq for GpuGlob {
    fn eq(&self, other: &Self) -> bool {
        self.module == other.module && self.id == other.id
    }
}

impl Eq for GpuGlob {}

impl Hash for GpuGlob {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.module.hash(state);
        self.id.hash(state);
    }
}

impl Clone for GpuGlob {
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
pub(crate) struct GpuVar {
    pub(crate) id: u64,
    pub(crate) type_id: TypeId,
}

#[derive(Debug)]
pub(crate) enum Operation {
    CreateVar(CreateVarOperation),
    AssignVar(AssignVarOperation),
    UnaryOperator(UnaryOperatorOperation),
}

impl Operation {
    pub(crate) fn glob(&self) -> Vec<&GpuGlob> {
        self.values()
            .into_iter()
            .filter_map(|value| value.glob())
            .collect()
    }

    pub(crate) fn values(&self) -> Vec<&GpuValue> {
        match self {
            Self::CreateVar(op) => vec![&op.value],
            Self::AssignVar(op) => vec![&op.left_value, &op.right_value],
            Self::UnaryOperator(op) => vec![&op.value],
        }
    }
}

#[derive(Debug)]
pub(crate) struct CreateVarOperation {
    pub(crate) id: u64,
    pub(crate) value: GpuValue,
}

#[derive(Debug)]
pub(crate) struct AssignVarOperation {
    pub(crate) left_value: GpuValue,
    pub(crate) right_value: GpuValue,
}

#[derive(Debug)]
pub(crate) struct UnaryOperatorOperation {
    pub(crate) value: GpuValue,
    pub(crate) operator: &'static str,
}
