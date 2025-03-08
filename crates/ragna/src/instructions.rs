use crate::context::GpuContext;
use crate::operations::{
    AssignVarOperation, DeclareVarOperation, FnCallOperation, IfOperation, Operation, Value,
};
use crate::{context, Bool, Gpu, GpuValue};

#[doc(hidden)]
#[allow(clippy::trivially_copy_pass_by_ref)]
pub fn create_glob<T: Gpu>(id: &'static &'static str) -> T {
    T::from_value(GpuValue::glob(id))
}

#[doc(hidden)]
pub fn create_uninit_var<T: Gpu>() -> T {
    let id = GpuContext::run_current(|ctx| {
        ctx.register_type::<T>();
        let id = context::next_var_id();
        ctx.operations
            .push(Operation::DeclareVar(DeclareVarOperation {
                id,
                type_: T::details(),
            }));
        id
    });
    T::from_value(GpuValue::var(id))
}

#[doc(hidden)]
pub fn create_var<T: Gpu>(value: T) -> T {
    let var = create_uninit_var::<T>();
    assign(var, value);
    var
}

#[doc(hidden)]
pub fn assign<T: Gpu>(variable: T, value: T) {
    let left_value = variable.value().untyped();
    let right_value = value.value().untyped();
    GpuContext::run_current(|ctx| {
        ctx.operations
            .push(Operation::AssignVar(AssignVarOperation {
                left_value,
                right_value,
            }));
    });
}

#[doc(hidden)]
pub fn call_fn<T: Gpu>(fn_name: &'static str, args: Vec<Value>) -> T {
    let var = create_uninit_var::<T>();
    let var_value = var.value().untyped();
    GpuContext::run_current(|ctx| {
        ctx.operations.push(Operation::FnCall(FnCallOperation {
            var: var_value,
            fn_name,
            args,
        }));
    });
    var
}

#[doc(hidden)]
pub fn if_block(condition: Bool) {
    let condition = condition.value().untyped();
    GpuContext::run_current(|ctx| {
        ctx.operations
            .push(Operation::IfBlock(IfOperation { condition }));
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
