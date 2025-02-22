use crate::operations::{FnCallOperation, Operation, Value};
use crate::{Gpu, GpuContext};

#[doc(hidden)]
pub fn call_fn<T>(fn_name: &'static str, args: Vec<Value>) -> T
where
    T: Gpu,
{
    let var = T::create_uninit_var();
    GpuContext::run_current(|ctx| {
        ctx.operations.push(Operation::FnCall(FnCallOperation {
            var: var.value().into(),
            fn_name,
            args,
        }));
    });
    var
}
