use crate::operations::{IfOperation, Operation};
use crate::{Bool, Gpu, GpuContext};

#[doc(hidden)]
pub fn if_block(condition: Bool) {
    GpuContext::run_current(|ctx| {
        ctx.operations.push(Operation::IfBlock(IfOperation {
            condition: condition.value().into(),
        }));
    });
}

#[doc(hidden)]
pub fn else_block() {
    GpuContext::run_current(|ctx| ctx.operations.push(Operation::ElseBlock));
}

#[doc(hidden)]
pub fn loop_block() {
    GpuContext::run_current(|ctx| ctx.operations.push(Operation::LoopBlock));
}

#[doc(hidden)]
pub fn end_block() {
    GpuContext::run_current(|ctx| ctx.operations.push(Operation::EndBlock));
}

#[doc(hidden)]
pub fn break_() {
    GpuContext::run_current(|ctx| ctx.operations.push(Operation::Break));
}

#[doc(hidden)]
pub fn continue_() {
    GpuContext::run_current(|ctx| ctx.operations.push(Operation::Continue));
}
