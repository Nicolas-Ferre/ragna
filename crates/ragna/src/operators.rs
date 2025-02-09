use crate::operations::{BinaryOperation, Operation, UnaryOperation};
use crate::types::GpuType;
use crate::{Gpu, GpuContext, Mut};
use std::any::Any;

macro_rules! unary_trait {
    ($name:ident) => {
        #[doc = concat!("A trait implemented for types that support `", operator!($name), "` unary operator on GPU side.")]
        pub trait $name: GpuType {
            /// The resulting type after applying the operator.
            type Output: GpuType;

            /// Applies the operator.
            fn apply(value: Gpu<Self, impl Any>, ctx: &mut GpuContext) -> Gpu<Self::Output, Mut>;
        }
    };
}

macro_rules! unary_impl {
    ($trait_:ident, $type_:ty, $out_type:ty) => {
        impl $trait_ for $type_ {
            type Output = $out_type;

            fn apply(value: Gpu<Self, impl Any>, ctx: &mut GpuContext) -> Gpu<Self::Output, Mut> {
                let var = Gpu::uninitialized_var(ctx);
                ctx.operations.push(Operation::Unary(UnaryOperation {
                    var: var.value(),
                    value: value.value(),
                    operator: operator!($trait_),
                }));
                var
            }
        }
    };
}

macro_rules! binary_trait {
    ($name:ident) => {
        #[doc = concat!("A trait implemented for types that support `", operator!($name), "` binary operator on GPU side.")]
        pub trait $name<R: GpuType>: GpuType {
            /// The resulting type after applying the operator.
            type Output: GpuType;

            /// Applies the operator.
            fn apply(
                left_value: Gpu<Self, impl Any>,
                right_value: Gpu<R, impl Any>,
                ctx: &mut GpuContext,
            ) -> Gpu<Self::Output, Mut>;
        }
    };
}

macro_rules! binary_impl {
    ($trait_:ident, $left_type:ty, $right_type:ty, $out_type:ty) => {
        #[allow(clippy::use_self)]
        impl $trait_<$right_type> for $left_type {
            type Output = $out_type;

            fn apply(
                left_value: Gpu<Self, impl Any>,
                right_value: Gpu<$right_type, impl Any>,
                ctx: &mut GpuContext,
            ) -> Gpu<Self::Output, Mut> {
                let var = Gpu::uninitialized_var(ctx);
                ctx.operations.push(Operation::Binary(BinaryOperation {
                    var: var.value(),
                    left_value: left_value.value(),
                    right_value: right_value.value(),
                    operator: operator!($trait_),
                }));
                var
            }
        }
    };
}

macro_rules! operator {
    (GpuNeg) => {
        "-"
    };
    (GpuNot) => {
        "!"
    };
    (GpuAdd) => {
        "+"
    };
    (GpuSub) => {
        "-"
    };
    (GpuMul) => {
        "*"
    };
    (GpuDiv) => {
        "/"
    };
    (GpuRem) => {
        "%"
    };
    (GpuEq) => {
        "=="
    };
    (GpuGreaterThan) => {
        ">"
    };
    (GpuAnd) => {
        "&&"
    };
    (GpuOr) => {
        "||"
    };
}

unary_trait!(GpuNeg);
unary_trait!(GpuNot);
binary_trait!(GpuAdd);
binary_trait!(GpuSub);
binary_trait!(GpuMul);
binary_trait!(GpuDiv);
binary_trait!(GpuRem);
binary_trait!(GpuEq);
binary_trait!(GpuGreaterThan);
binary_trait!(GpuAnd);
binary_trait!(GpuOr);

unary_impl!(GpuNeg, i32, i32);
unary_impl!(GpuNeg, f32, f32);
unary_impl!(GpuNot, bool, bool);
binary_impl!(GpuAdd, u32, u32, u32);
binary_impl!(GpuAdd, u32, i32, i32);
binary_impl!(GpuAdd, i32, i32, i32);
binary_impl!(GpuAdd, i32, u32, i32);
binary_impl!(GpuAdd, f32, f32, f32);
binary_impl!(GpuSub, u32, u32, u32);
binary_impl!(GpuSub, u32, i32, i32);
binary_impl!(GpuSub, i32, i32, i32);
binary_impl!(GpuSub, i32, u32, i32);
binary_impl!(GpuSub, f32, f32, f32);
binary_impl!(GpuMul, u32, u32, u32);
binary_impl!(GpuMul, u32, i32, i32);
binary_impl!(GpuMul, i32, i32, i32);
binary_impl!(GpuMul, i32, u32, i32);
binary_impl!(GpuMul, f32, f32, f32);
binary_impl!(GpuDiv, u32, u32, u32);
binary_impl!(GpuDiv, u32, i32, i32);
binary_impl!(GpuDiv, i32, i32, i32);
binary_impl!(GpuDiv, i32, u32, i32);
binary_impl!(GpuDiv, f32, f32, f32);
binary_impl!(GpuRem, u32, u32, u32);
binary_impl!(GpuRem, u32, i32, i32);
binary_impl!(GpuRem, i32, i32, i32);
binary_impl!(GpuRem, i32, u32, i32);
binary_impl!(GpuEq, i32, i32, bool);
binary_impl!(GpuEq, i32, u32, bool);
binary_impl!(GpuEq, u32, u32, bool);
binary_impl!(GpuEq, u32, i32, bool);
binary_impl!(GpuEq, f32, f32, bool);
binary_impl!(GpuEq, bool, bool, bool);
binary_impl!(GpuGreaterThan, i32, i32, bool);
binary_impl!(GpuGreaterThan, i32, u32, bool);
binary_impl!(GpuGreaterThan, u32, u32, bool);
binary_impl!(GpuGreaterThan, u32, i32, bool);
binary_impl!(GpuGreaterThan, f32, f32, bool);
binary_impl!(GpuAnd, bool, bool, bool);
binary_impl!(GpuOr, bool, bool, bool);
