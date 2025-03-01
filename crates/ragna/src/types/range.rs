use crate::{Bool, Cpu, Gpu, GpuTypeDetails, GpuValue, GreaterThan, IndexItems, Iterable, U32};
use std::any::TypeId;
use std::ops;
use std::ops::Index;

/// A GPU type to iterate on a range of values.
#[derive(Clone, Copy)]
pub struct Range<T: Gpu> {
    /// The first value.
    pub start: T,
    /// The last value excluded.
    pub end: T,
    value: GpuValue<Self>,
    items: IndexItems<T>,
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
            items: IndexItems::default(),
        }
    }

    fn from_value(value: GpuValue<Self>) -> Self {
        Self {
            start: T::from_value(value.field(0)),
            end: T::from_value(value.field(1)),
            value,
            items: IndexItems::default(),
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

impl Index<U32> for Range<U32> {
    type Output = U32;

    fn index(&self, index: U32) -> &Self::Output {
        self.items.next(self.start + index)
    }
}

impl Iterable for Range<U32> {
    fn len(&self) -> U32 {
        select(
            0_u32.to_gpu(),
            self.end - self.start,
            GreaterThan::apply(self.end, self.start),
        )
    }
}

fn select(f: U32, t: U32, cond: Bool) -> U32 {
    crate::call_fn(
        "select",
        vec![f.value().into(), t.value().into(), cond.value().into()],
    )
}
