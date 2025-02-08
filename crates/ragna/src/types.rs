use std::any;
use std::any::Any;

/// A trait implemented for types that can be used on GPU side.
pub trait GpuType: Any + Copy {
    fn gpu_type_details() -> GpuTypeDetails {
        GpuTypeDetails {
            name: any::type_name::<Self>(),
        }
    }

    fn into_wgsl(self) -> String;

    fn from_bytes(bytes: &[u8]) -> Self;
}

impl GpuType for i32 {
    fn into_wgsl(self) -> String {
        ToString::to_string(&self)
    }

    fn from_bytes(bytes: &[u8]) -> Self {
        Self::from_ne_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])
    }
}

impl GpuType for u32 {
    fn into_wgsl(self) -> String {
        ToString::to_string(&self)
    }

    fn from_bytes(bytes: &[u8]) -> Self {
        Self::from_ne_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])
    }
}

impl GpuType for f32 {
    fn into_wgsl(self) -> String {
        ToString::to_string(&self)
    }

    fn from_bytes(bytes: &[u8]) -> Self {
        Self::from_ne_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])
    }
}

impl GpuType for bool {
    fn gpu_type_details() -> GpuTypeDetails {
        u32::gpu_type_details()
    }

    fn into_wgsl(self) -> String {
        ToString::to_string(&u32::from(self))
    }

    fn from_bytes(bytes: &[u8]) -> Self {
        u32::from_ne_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) != 0
    }
}

/// Details about a GPU type on WGSL side.
#[derive(Debug, Clone)]
pub struct GpuTypeDetails {
    pub(crate) name: &'static str,
}
