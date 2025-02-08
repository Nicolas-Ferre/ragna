use crate::operations::{Operation, UnaryOperation};
use crate::types::GpuType;
use crate::{Gpu, GpuContext, Mut};
use std::any::Any;

macro_rules! impl_unary {
    ($trait_:ident, $operator:literal, $type_:ty, $out_type:ty) => {
        impl $trait_ for $type_ {
            type Output = $out_type;

            fn apply(value: Gpu<Self, impl Any>, ctx: &mut GpuContext) -> Gpu<Self, Mut> {
                let var = Gpu::var(value, ctx);
                ctx.operations.push(Operation::Unary(UnaryOperation {
                    var: var.value(),
                    value: value.value(),
                    operator: $operator,
                }));
                var
            }
        }
    };
}

/// A trait implemented for types that supports `-` unary operator on GPU side.
pub trait GpuNeg: GpuType {
    type Output: GpuType;

    fn apply(value: Gpu<Self, impl Any>, ctx: &mut GpuContext) -> Gpu<Self, Mut>;
}

impl_unary!(GpuNeg, "-", i32, i32);
impl_unary!(GpuNeg, "-", f32, f32);

/// A trait implemented for types that supports `!` unary operator on GPU side.
pub trait GpuNot: GpuType {
    type Output: GpuType;

    fn apply(value: Gpu<Self, impl Any>, ctx: &mut GpuContext) -> Gpu<Self, Mut>;
}

impl_unary!(GpuNot, "!", bool, bool);
