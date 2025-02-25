use crate::{Cpu, Gpu, GpuContext, GpuTypeDetails, GpuValue, U32};
use std::any::TypeId;
use std::ops;
use std::ops::{Add, Index};

/// A GPU type to iterate on a range of values.
#[derive(Clone, Copy)]
pub struct Range<T: Gpu> {
    /// The first value.
    pub start: T,
    /// The last value excluded.
    pub end: T,
    value: GpuValue<Self>,
    item: T,
}

impl<T: Gpu> Range<T> {
    /// Creates a new range
    pub fn new(start: T, end: T) -> Self {
        let var = crate::create_uninit_var::<Self>();
        crate::assign(var.start, start);
        crate::assign(var.end, end);
        var
    }
}

impl<T: Gpu> Gpu for Range<T> {
    type Cpu = ops::Range<T::Cpu>;

    fn details() -> GpuTypeDetails {
        GpuTypeDetails {
            type_id: TypeId::of::<Self>(),
            name: None,
            size: None,
            field_types: vec![T::details(), T::details()],
        }
    }

    fn value(self) -> GpuValue<Self> {
        self.value
    }

    fn unregistered() -> Self {
        Self {
            start: T::unregistered(),
            end: T::unregistered(),
            value: GpuValue::unregistered_var(),
            item: T::unregistered(),
        }
    }

    fn from_value(value: GpuValue<Self>) -> Self {
        Self {
            start: T::from_value(value.field(0)),
            end: T::from_value(value.field(1)),
            value,
            item: T::unregistered(),
        }
    }
}

impl<T: Cpu> Cpu for ops::Range<T> {
    type Gpu = Range<T::Gpu>;

    #[allow(clippy::cast_possible_truncation)]
    fn from_gpu(bytes: &[u8]) -> Self {
        let size = <T as Cpu>::Gpu::details().size() as usize;
        T::from_gpu(&bytes[..size])..T::from_gpu(&bytes[size..])
    }

    fn to_wgsl(self) -> String {
        format!("<name>({}, {})", self.start.to_wgsl(), self.end.to_wgsl())
    }
}

impl<T> Index<U32> for Range<T>
where
    T: Gpu + Add<U32, Output = T>,
{
    type Output = T;

    fn index(&self, index: U32) -> &Self::Output {
        GpuContext::run_current(|ctx| ctx.register_var(self.item));
        crate::assign(self.item, self.start + index);
        &self.item
    }
}
