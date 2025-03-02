use crate::operations::{IndexOperation, Operation};
use crate::types::IndexItems;
use crate::{Cpu, Gpu, GpuContext, GpuTypeDetails, GpuValue, Iterable, U32};
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
            items.into_iter().map(|item| item.value().into()).collect(),
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

    fn unregistered() -> Self {
        assert_ne!(N, 0, "arrays should not be empty");
        Self {
            value: GpuValue::unregistered_var(),
            items: IndexItems::default(),
        }
    }

    fn from_value(value: GpuValue<Self>) -> Self {
        assert_ne!(N, 0, "arrays should not be empty");
        Self {
            value,
            items: IndexItems::default(),
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
        let var = crate::create_uninit_var::<T>();
        let index = index % self.len();
        GpuContext::run_current(|ctx| {
            ctx.operations.push(Operation::Index(IndexOperation {
                var: var.value().into(),
                array: self.value().into(),
                index: index.value().into(),
            }));
        });
        self.items.next(var)
    }
}

impl<T: Gpu, const N: usize> Iterable for Array<T, N> {
    #[allow(clippy::cast_possible_truncation)]
    fn len(&self) -> U32 {
        (N as u32).to_gpu()
    }
}
