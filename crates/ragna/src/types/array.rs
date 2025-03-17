use crate::{
    context, Cpu, Gpu, GpuTypeDetails, GpuValue, GreaterThan, Iterable, Wgsl, WgslConstructor, U32,
};
use std::any::TypeId;
use std::marker::PhantomData;
use std::ops::Index;

/// A GPU array.
#[derive(Clone, Copy)]
pub struct Array<T, const N: usize> {
    value: GpuValue,
    phantom: PhantomData<fn(T)>,
}

impl<T: Gpu, const N: usize> Array<T, N> {
    /// Creates a new array.
    pub fn new(items: [T; N]) -> Self {
        crate::call_fn("array", items.into_iter().map(Gpu::value).collect())
    }

    /// Creates a new array from a repeated item.
    #[allow(clippy::cast_possible_truncation)]
    pub fn repeated(item: T) -> Self {
        let array = crate::create_uninit_var::<Self>();
        let index = 0_u32.to_gpu();
        crate::loop_block();
        crate::if_block(GreaterThan::apply((N as u32).to_gpu(), index));
        crate::assign(array[index], item);
        crate::assign(index, index + 1_u32.to_gpu());
        crate::else_block();
        crate::break_();
        crate::end_block();
        crate::end_block();
        array
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
            size: Some(
                N as u64 * GpuTypeDetails::round_up(item_details.alignment(), item_details.size()),
            ),
            alignment: Some(item_details.alignment()),
            field_types: vec![item_details],
        }
    }

    fn value(self) -> GpuValue {
        self.value
    }

    fn from_value(value: GpuValue) -> Self {
        assert_ne!(N, 0, "arrays should not be empty");
        Self {
            value,
            phantom: PhantomData,
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

    fn to_wgsl(&self) -> Wgsl {
        Wgsl::Constructor(WgslConstructor {
            type_id: TypeId::of::<Self::Gpu>(),
            args: self.iter().map(T::to_wgsl).collect(),
        })
    }
}

impl<T: Gpu, const N: usize> Index<U32> for Array<T, N> {
    type Output = T;

    fn index(&self, index: U32) -> &Self::Output {
        let transformed_index = index % self.len();
        context::next_static_value(T::from_value(
            self.value().index::<T>(transformed_index.value().var_id()),
        ))
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
