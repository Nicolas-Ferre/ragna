use crate::{
    Bool, Cpu, Gpu, GpuTypeDetails, GpuValue, GreaterThan, Iterable, Wgsl, WgslConstructor, U32,
};
use std::any::TypeId;
use std::ops;

/// A GPU type to iterate on a range of values.
#[derive(Clone, Copy)]
pub struct Range<T: Gpu> {
    /// The first value.
    pub start: T,
    /// The last value excluded.
    pub end: T,
    value: GpuValue,
}

impl<T: Gpu> Range<T> {
    /// Creates a new range.
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
            array_generics: None,
            size: None,
            alignment: None,
            field_types: vec![T::details(), T::details()],
        }
    }

    fn value(self) -> GpuValue {
        self.value
    }

    fn from_value(value: GpuValue) -> Self {
        Self {
            start: T::from_value(value.field::<T>(0)),
            end: T::from_value(value.field::<T>(1)),
            value,
        }
    }
}

impl<T: Cpu> Cpu for ops::Range<T> {
    type Gpu = Range<T::Gpu>;

    #[allow(clippy::cast_possible_truncation)]
    fn from_gpu(bytes: &[u8]) -> Self {
        let end_offset = Self::Gpu::details().field_offset(1) as usize;
        T::from_gpu(&bytes[..end_offset])..T::from_gpu(&bytes[end_offset..])
    }

    fn to_wgsl(&self) -> Wgsl {
        Wgsl::Constructor(WgslConstructor {
            type_id: TypeId::of::<Self::Gpu>(),
            args: vec![self.start.to_wgsl(), self.end.to_wgsl()],
        })
    }
}

impl Iterable for Range<U32> {
    type Item<'a> = U32;

    fn next(&self, index: U32) -> Self::Item<'_> {
        self.start + index
    }

    fn len(&self) -> U32 {
        select(
            0_u32.to_gpu(),
            self.end - self.start,
            GreaterThan::apply(self.end, self.start),
        )
    }
}

fn select(f: U32, t: U32, cond: Bool) -> U32 {
    crate::call_fn("select", vec![f.value(), t.value(), cond.value()], true)
}
