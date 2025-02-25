use crate::operations::{BinaryOperation, Operation, UnaryOperation};
use crate::{Bool, Gpu, F32, I32, U32};
use std::ops::{Add, Div, Mul, Neg, Not, Rem, Sub};
use crate::context::GpuContext;

pub(crate) fn apply_unary_op<I: Gpu, O: Gpu>(input: I, operator: &'static str) -> O {
    let var = crate::create_uninit_var::<O>();
    GpuContext::run_current(|ctx| {
        ctx.operations.push(Operation::Unary(UnaryOperation {
            var: var.value().into(),
            value: input.value().into(),
            operator,
        }));
    });
    var
}

pub(crate) fn apply_binary_op<L: Gpu, R: Gpu, O: Gpu>(
    left: L,
    right: R,
    operator: &'static str,
) -> O {
    let var = crate::create_uninit_var::<O>();
    GpuContext::run_current(|ctx| {
        ctx.operations.push(Operation::Binary(BinaryOperation {
            var: var.value().into(),
            left_value: left.value().into(),
            right_value: right.value().into(),
            operator,
        }));
    });
    var
}

macro_rules! unary_impl {
    ($trait_:ident, $method:ident, $type_:ty, $out_type:ty) => {
        impl $trait_ for $type_ {
            type Output = $out_type;

            fn $method(self) -> Self::Output {
                apply_unary_op(self, operator!($trait_))
            }
        }
    };
}

macro_rules! bool_binary_trait {
    ($name:ident) => {
        #[doc = concat!("A trait implemented for types that support `", operator!($name), "` binary operator on GPU side.")]
        pub trait $name<R> {
            /// Applies the operator.
            fn apply(self, right_value: R) -> Bool;
        }
    };
}

macro_rules! bool_binary_impl {
    ($trait_:ident, $method:ident, $left_type:ty, $right_type:ty) => {
        #[allow(clippy::use_self)]
        impl $trait_<$right_type> for $left_type {
            fn $method(self, right_value: $right_type) -> Bool {
                apply_binary_op(self, right_value, operator!($trait_))
            }
        }
    };
}

macro_rules! binary_impl {
    ($trait_:ident, $method:ident, $left_type:ty, $right_type:ty, $out_type:ty) => {
        #[allow(clippy::use_self)]
        impl $trait_<$right_type> for $left_type {
            type Output = $out_type;

            fn $method(self, right_value: $right_type) -> Self::Output {
                apply_binary_op(self, right_value, operator!($trait_))
            }
        }
    };
}

macro_rules! operator {
    (Neg) => {
        "-"
    };
    (Not) => {
        "!"
    };
    (Add) => {
        "+"
    };
    (Sub) => {
        "-"
    };
    (Mul) => {
        "*"
    };
    (Div) => {
        "/"
    };
    (Rem) => {
        "%"
    };
    (Equal) => {
        "=="
    };
    (GreaterThan) => {
        ">"
    };
}

bool_binary_trait!(Equal);
bool_binary_trait!(GreaterThan);

unary_impl!(Neg, neg, I32, I32);
unary_impl!(Neg, neg, F32, F32);
unary_impl!(Not, not, Bool, Bool);
binary_impl!(Add, add, U32, U32, U32);
binary_impl!(Add, add, I32, I32, I32);
binary_impl!(Add, add, F32, F32, F32);
binary_impl!(Sub, sub, U32, U32, U32);
binary_impl!(Sub, sub, I32, I32, I32);
binary_impl!(Sub, sub, F32, F32, F32);
binary_impl!(Mul, mul, U32, U32, U32);
binary_impl!(Mul, mul, I32, I32, I32);
binary_impl!(Mul, mul, F32, F32, F32);
binary_impl!(Div, div, U32, U32, U32);
binary_impl!(Div, div, I32, I32, I32);
binary_impl!(Div, div, F32, F32, F32);
binary_impl!(Rem, rem, U32, U32, U32);
binary_impl!(Rem, rem, I32, I32, I32);
bool_binary_impl!(Equal, apply, I32, I32);
bool_binary_impl!(Equal, apply, U32, U32);
bool_binary_impl!(Equal, apply, F32, F32);
bool_binary_impl!(Equal, apply, Bool, Bool);
bool_binary_impl!(GreaterThan, apply, I32, I32);
bool_binary_impl!(GreaterThan, apply, U32, U32);
bool_binary_impl!(GreaterThan, apply, F32, F32);
