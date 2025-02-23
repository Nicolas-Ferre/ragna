use crate::operations::{ConstantAssignVarOperation, Glob, Operation, Value, Var};
use crate::GpuContext;
use std::any::TypeId;

pub(crate) mod native;

/// A trait implemented for Rust types that have a corresponding CPU type.
pub trait Cpu: Sized {
    /// The GPU type.
    type Gpu: Gpu;

    /// Converts bytes from GPU to a CPU instance of the type.
    fn from_gpu(bytes: &[u8]) -> Self;

    /// Converts a value to WGSL code.
    fn to_wgsl(self) -> String;

    /// Converts a value to a GPU value.
    fn to_gpu(self) -> Self::Gpu {
        let var = crate::create_uninit_var::<Self::Gpu>();
        GpuContext::run_current(|ctx| {
            ctx.operations
                .push(Operation::ConstantAssignVar(ConstantAssignVarOperation {
                    left_value: var.value().into(),
                    right_value: self.to_wgsl(),
                }));
        });
        var
    }
}

/// A trait implemented for GPU types that have a corresponding CPU type.
pub trait Gpu: 'static + Copy {
    /// The CPU type.
    type Cpu: Cpu;

    /// Returns details about the GPU type.
    fn details() -> GpuTypeDetails;

    /// Retrieves the GPU value.
    fn value(self) -> GpuValue<Self>;

    /// Creates a variable from a GPU value.
    fn from_value(value: GpuValue<Self>) -> Self;
}

/// A GPU value.
#[derive(Debug, Clone, Copy)]
pub enum GpuValue<T>
where
    T: Gpu,
{
    /// A global variable.
    Glob(&'static str, u64, fn() -> T),
    /// A local variable.
    Var(u64),
}

impl<T> From<GpuValue<T>> for Value
where
    T: Gpu,
{
    fn from(value: GpuValue<T>) -> Self {
        match value {
            GpuValue::Glob(module, id, default_value) => Self::Glob(Glob {
                module,
                id,
                type_id: TypeId::of::<T>(),
                default_value: Box::new(move || default_value().value().into()),
            }),
            GpuValue::Var(id) => Self::Var(Var {
                id,
                type_id: TypeId::of::<T>(),
            }),
        }
    }
}

/// Details about a GPU type on WGSL side.
#[derive(Debug, Clone, Copy)]
pub struct GpuTypeDetails {
    pub(crate) name: &'static str,
}
