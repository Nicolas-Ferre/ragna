use crate::context::GpuContext;
use crate::operations::{BinaryOperation, Operation, UnaryOperation};
use crate::{
    Bool, F32x2, F32x3, F32x4, Gpu, I32x2, I32x3, I32x4, U32x2, U32x3, U32x4, F32, I32, U32,
};
use std::ops::{Add, Div, Mul, Neg, Not, Rem, Sub};

pub(crate) fn apply_unary_op<I: Gpu, O: Gpu>(input: I, operator: &'static str) -> O {
    let var = crate::create_uninit_var::<O>();
    GpuContext::run_current(|ctx| {
        ctx.operations.push(Operation::Unary(UnaryOperation {
            var: var.value(),
            value: input.value(),
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
            var: var.value(),
            left_value: left.value(),
            right_value: right.value(),
            operator,
        }));
    });
    var
}

macro_rules! unary_impl {
    ($trait_:ident, $method:ident, $type_:ty) => {
        impl $trait_ for $type_ {
            type Output = $type_;

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
    ($trait_:ident, $method:ident, $type_:ty) => {
        #[allow(clippy::use_self)]
        impl $trait_<$type_> for $type_ {
            fn $method(self, right_value: $type_) -> Bool {
                apply_binary_op(self, right_value, operator!($trait_))
            }
        }
    };
}

macro_rules! binary_impl {
    ($trait_:ident, $method:ident, $type_:ty) => {
        #[allow(clippy::use_self)]
        impl $trait_<$type_> for $type_ {
            type Output = $type_;

            fn $method(self, right_value: $type_) -> Self::Output {
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

unary_impl!(Neg, neg, I32);
unary_impl!(Neg, neg, F32);
unary_impl!(Neg, neg, I32x2);
unary_impl!(Neg, neg, F32x2);
unary_impl!(Neg, neg, I32x3);
unary_impl!(Neg, neg, F32x3);
unary_impl!(Neg, neg, I32x4);
unary_impl!(Neg, neg, F32x4);
unary_impl!(Not, not, Bool);
binary_impl!(Add, add, U32);
binary_impl!(Add, add, I32);
binary_impl!(Add, add, F32);
binary_impl!(Add, add, U32x2);
binary_impl!(Add, add, I32x2);
binary_impl!(Add, add, F32x2);
binary_impl!(Add, add, U32x3);
binary_impl!(Add, add, I32x3);
binary_impl!(Add, add, F32x3);
binary_impl!(Add, add, U32x4);
binary_impl!(Add, add, I32x4);
binary_impl!(Add, add, F32x4);
binary_impl!(Sub, sub, U32);
binary_impl!(Sub, sub, I32);
binary_impl!(Sub, sub, F32);
binary_impl!(Sub, sub, U32x2);
binary_impl!(Sub, sub, I32x2);
binary_impl!(Sub, sub, F32x2);
binary_impl!(Sub, sub, U32x3);
binary_impl!(Sub, sub, I32x3);
binary_impl!(Sub, sub, F32x3);
binary_impl!(Sub, sub, U32x4);
binary_impl!(Sub, sub, I32x4);
binary_impl!(Sub, sub, F32x4);
binary_impl!(Mul, mul, U32);
binary_impl!(Mul, mul, I32);
binary_impl!(Mul, mul, F32);
binary_impl!(Mul, mul, U32x2);
binary_impl!(Mul, mul, I32x2);
binary_impl!(Mul, mul, F32x2);
binary_impl!(Mul, mul, U32x3);
binary_impl!(Mul, mul, I32x3);
binary_impl!(Mul, mul, F32x3);
binary_impl!(Mul, mul, U32x4);
binary_impl!(Mul, mul, I32x4);
binary_impl!(Mul, mul, F32x4);
binary_impl!(Div, div, U32);
binary_impl!(Div, div, I32);
binary_impl!(Div, div, F32);
binary_impl!(Div, div, U32x2);
binary_impl!(Div, div, I32x2);
binary_impl!(Div, div, F32x2);
binary_impl!(Div, div, U32x3);
binary_impl!(Div, div, I32x3);
binary_impl!(Div, div, F32x3);
binary_impl!(Div, div, U32x4);
binary_impl!(Div, div, I32x4);
binary_impl!(Div, div, F32x4);
binary_impl!(Rem, rem, U32);
binary_impl!(Rem, rem, I32);
binary_impl!(Rem, rem, U32x2);
binary_impl!(Rem, rem, I32x2);
binary_impl!(Rem, rem, F32x2);
binary_impl!(Rem, rem, U32x3);
binary_impl!(Rem, rem, I32x3);
binary_impl!(Rem, rem, F32x3);
binary_impl!(Rem, rem, U32x4);
binary_impl!(Rem, rem, I32x4);
binary_impl!(Rem, rem, F32x4);
bool_binary_impl!(Equal, apply, I32);
bool_binary_impl!(Equal, apply, U32);
bool_binary_impl!(Equal, apply, F32);
bool_binary_impl!(Equal, apply, Bool);
bool_binary_impl!(GreaterThan, apply, I32);
bool_binary_impl!(GreaterThan, apply, U32);
bool_binary_impl!(GreaterThan, apply, F32);
