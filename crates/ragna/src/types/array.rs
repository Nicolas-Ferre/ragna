use crate::types::IndexItems;
use crate::{Cpu, Gpu, GpuTypeDetails, GpuValue, Iterable, U32};
use itertools::Itertools;
use std::any::TypeId;
use std::ops::Index;

/// A GPU array.
#[derive(Clone, Copy)]
pub struct Array<T, const N: usize> {
    value: GpuValue<Self>,
    items: IndexItems<T>,
}

impl<T: Gpu, const N: usize> Array<T, N> {
    /// Creates a new array.
    pub fn new(items: [T; N]) -> Self {
        crate::call_fn(
            "array",
            items
                .into_iter()
                .map(|item| item.value().untyped())
                .collect(),
        )
    }
}

impl<T: Gpu, const N: usize> Gpu for Array<T, N> {
    type Cpu = [T::Cpu; N];

    fn details() -> GpuTypeDetails {
        let item_details = T::details();
        GpuTypeDetails {
            type_id: TypeId::of::<Self>(),
            name: Some("array"),
            array_generics: Some((item_details.clone().into(), N)),
            size: Some(item_details.size() * N as u64),
            field_types: vec![item_details],
        }
    }

    fn value(self) -> GpuValue<Self> {
        self.value
    }

    fn from_value(value: GpuValue<Self>) -> Self {
        assert_ne!(N, 0, "arrays should not be empty");
        Self {
            value,
            items: IndexItems::new(value),
        }
    }
}

impl<T: Cpu, const N: usize> Cpu for [T; N] {
    type Gpu = Array<T::Gpu, N>;

    fn from_gpu(bytes: &[u8]) -> Self {
        bytes
            .chunks(bytes.len().div_euclid(N))
            .map(T::from_gpu)
            .collect::<Vec<_>>()
            .try_into()
            .ok()
            .expect("internal error: invalid GPU array")
    }

    fn to_wgsl(&self) -> String {
        format!("<name>({})", self.iter().map(T::to_wgsl).join(", "))
    }
}

impl<T: Gpu, const N: usize> Index<U32> for Array<T, N> {
    type Output = T;

    fn index(&self, index: U32) -> &Self::Output {
        self.items.next(*self, index % self.len())
    }
}

impl<T: Gpu, const N: usize> Iterable for Array<T, N> {
    type Item<'a> = &'a T;

    fn next(&self, index: U32) -> Self::Item<'_> {
        &self[index]
    }

    #[allow(clippy::cast_possible_truncation)]
    fn len(&self) -> U32 {
        (N as u32).to_gpu()
    }
}
