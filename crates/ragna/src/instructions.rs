use crate::context::GpuContext;
use crate::operations::{
    AssignVarOperation, DeclareVarOperation, FnCallOperation, IfOperation, Operation, Value,
};
use crate::{context, Bool, Gpu, GpuValue};

#[doc(hidden)]
pub fn create_glob<T: Gpu>(module: &'static str, id: u64, default_value: fn() -> T) -> T {
    T::from_value(GpuValue::Glob(module, id, default_value))
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
    T::from_value(GpuValue::Var(id))
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
                left_value: variable.value().into(),
                right_value: value.value().into(),
            }));
    });
}

#[doc(hidden)]
pub fn call_fn<T: Gpu>(fn_name: &'static str, args: Vec<Value>) -> T {
    let var = create_uninit_var::<T>();
    GpuContext::run_current(|ctx| {
        ctx.operations.push(Operation::FnCall(FnCallOperation {
            var: var.value().into(),
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
