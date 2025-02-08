use crate::operations::{Operation, UnaryOperatorOperation};
use crate::types::GpuType;
use crate::{Gpu, GpuContext, Mut};
use std::any::Any;

/// A trait implemented for types that supports `-` unary operator on GPU side.
pub trait GpuNeg: GpuType {
    fn neg(value: Gpu<Self, impl Any>, ctx: &mut GpuContext) -> Gpu<Self, Mut>;
}

impl GpuNeg for i32 {
    fn neg(value: Gpu<Self, impl Any>, ctx: &mut GpuContext) -> Gpu<Self, Mut> {
        let var = Gpu::var(ctx, value);
        ctx.operations
            .push(Operation::UnaryOperator(UnaryOperatorOperation {
                value: var.value(),
                operator: "-",
            }));
        var
    }
}

impl GpuNeg for f32 {
    fn neg(value: Gpu<Self, impl Any>, ctx: &mut GpuContext) -> Gpu<Self, Mut> {
        let var = Gpu::var(ctx, value);
        ctx.operations
            .push(Operation::UnaryOperator(UnaryOperatorOperation {
                value: var.value(),
                operator: "-",
            }));
        var
    }
}

/// A trait implemented for types that supports `!` unary operator on GPU side.
pub trait GpuNot: GpuType {
    fn not(value: Gpu<Self, impl Any>, ctx: &mut GpuContext) -> Gpu<Self, Mut>;
}

impl GpuNot for bool {
    fn not(value: Gpu<Self, impl Any>, ctx: &mut GpuContext) -> Gpu<Self, Mut> {
        let var = Gpu::var(ctx, value);
        ctx.operations
            .push(Operation::UnaryOperator(UnaryOperatorOperation {
                value: var.value(),
                operator: "!",
            }));
        var
    }
}
