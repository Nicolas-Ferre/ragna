use crate::operations::{IfOperation, Operation};
use crate::{Bool, Gpu, GpuContext};

#[doc(hidden)]
pub fn if_(condition: Bool) {
    GpuContext::run_current(|ctx| {
        ctx.operations.push(Operation::If(IfOperation {
            condition: condition.value().into(),
        }));
    });
}

#[doc(hidden)]
pub fn else_() {
    GpuContext::run_current(|ctx| ctx.operations.push(Operation::Else));
}

#[doc(hidden)]
pub fn end_if() {
    GpuContext::run_current(|ctx| ctx.operations.push(Operation::EndIf));
}
