use crate::context::GpuContext;
use crate::operations::{
    AssignVarOperation, DeclareVarOperation, FnCallOperation, IfOperation, Operation,
};
use crate::{context, Bool, Gpu, GpuValue};

#[doc(hidden)]
pub fn create_glob<T: Gpu>(id: &'static &'static str) -> T {
    T::from_value(GpuValue::glob::<T>(id))
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
    T::from_value(GpuValue::var::<T>(id))
}

#[doc(hidden)]
pub fn create_var<T: Gpu>(value: T) -> T {
    let var = create_uninit_var::<T>();
    assign(var, value);
    var
}

#[doc(hidden)]
pub fn assign<T: Gpu>(variable: T, value: T) {
    GpuContext::run_current(|ctx| {
        ctx.operations
            .push(Operation::AssignVar(AssignVarOperation {
                left_value: variable.value(),
                right_value: value.value(),
            }));
    });
}

#[doc(hidden)]
pub fn call_fn<T: Gpu>(fn_name: &'static str, args: Vec<GpuValue>) -> T {
    let var = create_uninit_var::<T>();
    GpuContext::run_current(|ctx| {
        ctx.operations.push(Operation::FnCall(FnCallOperation {
            var: var.value(),
            fn_name,
            args,
        }));
    });
    var
}

#[doc(hidden)]
pub fn if_block(condition: Bool) {
    GpuContext::run_current(|ctx| {
        ctx.operations.push(Operation::IfBlock(IfOperation {
            condition: condition.value(),
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
