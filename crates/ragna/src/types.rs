use crate::operations::{
    AssignVarOperation, ConstantAssignVarOperation, DeclareVarOperation, Glob, Operation, Value,
    Var,
};
use crate::{operators, GpuContext};
use std::any::TypeId;

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
        let var = Self::Gpu::create_uninit_var();
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

    /// Creates an initialized GPU variable.
    fn create_var(value: Self) -> Self;

    /// Creates an uninitialized GPU variable.
    fn create_uninit_var() -> Self;

    /// Assigns a value.
    fn assign(self, value: Self);

    /// Retrieves the GPU value.
    fn value(self) -> GpuValue<Self>;
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

macro_rules! native_gpu_type {
    ($name:ident, $cpu_name:ident, $wgsl_name:literal) => {
        #[doc = concat!("The `", stringify!($cpu_name), "` type on GPU side.")]
        #[derive(Copy, Clone, Debug)]
        pub struct $name {
            __value: GpuValue<Self>,
        }

        impl $name {
            #[doc(hidden)]
            pub const fn define_glob(
                module: &'static str,
                id: u64,
                default_value: fn() -> Self,
            ) -> Self {
                Self {
                    __value: GpuValue::Glob(module, id, default_value),
                }
            }
        }

        impl Gpu for $name {
            type Cpu = $cpu_name;

            fn details() -> GpuTypeDetails {
                GpuTypeDetails { name: $wgsl_name }
            }

            fn create_var(value: Self) -> Self {
                let var = Self::create_uninit_var();
                GpuContext::run_current(|ctx| {
                    ctx.operations
                        .push(Operation::AssignVar(AssignVarOperation {
                            left_value: var.__value.into(),
                            right_value: value.__value.into(),
                        }));
                });
                var
            }

            fn create_uninit_var() -> Self {
                let id = GpuContext::run_current(|ctx| {
                    let id = ctx.next_var_id();
                    ctx.operations
                        .push(Operation::DeclareVar(DeclareVarOperation {
                            id,
                            type_: Self::details(),
                        }));
                    id
                });
                Self {
                    __value: GpuValue::Var(id),
                }
            }

            fn assign(self, value: Self) {
                GpuContext::run_current(|ctx| {
                    ctx.operations
                        .push(Operation::AssignVar(AssignVarOperation {
                            left_value: self.__value.into(),
                            right_value: value.__value.into(),
                        }));
                })
            }

            fn value(self) -> GpuValue<Self> {
                self.__value
            }
        }
    };
}

native_gpu_type!(I32, i32, "i32");
native_gpu_type!(U32, u32, "u32");
native_gpu_type!(F32, f32, "f32");
native_gpu_type!(Bool, bool, "u32");

impl Cpu for i32 {
    type Gpu = I32;

    fn from_gpu(bytes: &[u8]) -> Self {
        Self::from_ne_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])
    }

    fn to_wgsl(self) -> String {
        ToString::to_string(&self)
    }
}

impl Cpu for u32 {
    type Gpu = U32;

    fn from_gpu(bytes: &[u8]) -> Self {
        Self::from_ne_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])
    }

    fn to_wgsl(self) -> String {
        ToString::to_string(&self)
    }
}

impl Cpu for f32 {
    type Gpu = F32;

    fn from_gpu(bytes: &[u8]) -> Self {
        Self::from_ne_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])
    }

    fn to_wgsl(self) -> String {
        let value = ToString::to_string(&self);
        if value.contains('.') {
            value
        } else {
            format!("{value}.")
        }
    }
}

impl Cpu for bool {
    type Gpu = Bool;

    fn from_gpu(bytes: &[u8]) -> Self {
        u32::from_ne_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) != 0
    }

    fn to_wgsl(self) -> String {
        ToString::to_string(&u32::from(self))
    }
}

impl Bool {
    /// Applies AND logical operator.
    pub fn and(self, other: Self) -> Self {
        operators::apply_binary_op(self, other, "&&")
    }

    /// Applies OR logical operator.
    pub fn or(self, other: Self) -> Self {
        operators::apply_binary_op(self, other, "||")
    }
}
