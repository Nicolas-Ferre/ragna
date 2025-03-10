use crate::{operators, Cpu, Gpu, GpuTypeDetails, GpuValue};
use std::any::TypeId;

macro_rules! native_gpu_type {
    ($name:ident, $cpu_name:ident, $wgsl_name:literal) => {
        #[doc = concat!("The `", stringify!($cpu_name), "` type on GPU side.")]
        #[derive(Copy, Clone, Debug)]
        pub struct $name {
            __value: GpuValue<Self>,
        }

        impl Gpu for $name {
            type Cpu = $cpu_name;

            fn details() -> GpuTypeDetails {
                GpuTypeDetails {
                    type_id: TypeId::of::<Self>(),
                    name: Some($wgsl_name),
                    array_generics: None,
                    size: Some(4),
                    alignment: Some(4),
                    field_types: vec![],
                }
            }

            fn value(self) -> GpuValue<Self> {
                self.__value
            }

            fn from_value(value: GpuValue<Self>) -> Self {
                Self { __value: value }
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

    fn to_wgsl(&self) -> String {
        ToString::to_string(self)
    }
}

impl Cpu for u32 {
    type Gpu = U32;

    fn from_gpu(bytes: &[u8]) -> Self {
        Self::from_ne_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])
    }

    fn to_wgsl(&self) -> String {
        ToString::to_string(self)
    }
}

impl Cpu for f32 {
    type Gpu = F32;

    fn from_gpu(bytes: &[u8]) -> Self {
        Self::from_ne_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])
    }

    fn to_wgsl(&self) -> String {
        let value = ToString::to_string(self);
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

    fn to_wgsl(&self) -> String {
        ToString::to_string(&u32::from(*self))
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
