use crate::app::CURRENT_CTX;
use crate::operations::{DeclareVarOperation, Operation, Value};
use crate::{Gpu, GpuTypeDetails, GpuValue};
use fxhash::{FxHashMap, FxHashSet};
use std::mem;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{LockResult, Mutex, MutexGuard};

pub(crate) fn next_var_id() -> u32 {
    static NEXT_ID: AtomicU32 = AtomicU32::new(0);
    NEXT_ID.fetch_add(1, Ordering::Relaxed)
}

/// The context used to track GPU operations.
#[doc(hidden)]
#[derive(Debug, Default)]
pub struct GpuContext {
    pub(crate) operations: Vec<Operation>,
    pub(crate) types: Vec<GpuTypeDetails>,
    registered_var_ids: FxHashSet<u32>,
    next_indexes: FxHashMap<Value, usize>,
}

impl GpuContext {
    pub(crate) fn register_type<T: Gpu>(&mut self) {
        let mut types_to_register = vec![T::details()];
        while !types_to_register.is_empty() {
            let types = mem::take(&mut types_to_register);
            for type_ in &types {
                types_to_register.extend(type_.field_types.clone());
            }
            self.types.extend(types);
        }
    }

    pub(crate) fn register_var<T: Gpu>(&mut self, value: T) {
        if let GpuValue::Var(id) = value.value() {
            if self.registered_var_ids.insert(id) {
                self.operations
                    .push(Operation::DeclareVar(DeclareVarOperation {
                        id,
                        type_: T::details(),
                    }));
            }
        } else {
            unreachable!("internal error: register non-variable value")
        }
    }

    pub(crate) fn next_index<T: Gpu>(&mut self, value: T) -> usize {
        *self
            .next_indexes
            .entry(value.value().into())
            .and_modify(|i| *i += 1)
            .or_insert(0)
    }

    pub(crate) fn run_current<O>(f: impl FnOnce(&mut Self) -> O) -> O {
        f(CURRENT_CTX
            .try_lock()
            .as_mut()
            .expect("cannot lock GPU context")
            .as_mut()
            .expect("internal error: missing GPU context"))
    }

    pub(crate) fn lock_current<'a>() -> LockResult<MutexGuard<'a, ()>> {
        static CTX_LOCK: Mutex<()> = Mutex::new(());
        let lock = CTX_LOCK.lock();
        **CURRENT_CTX
            .try_lock()
            .as_mut()
            .expect("cannot lock GPU context") = Some(Self::default());
        lock
    }

    pub(crate) fn unlock_current(lock: LockResult<MutexGuard<'_, ()>>) -> Self {
        let ctx = CURRENT_CTX
            .try_lock()
            .as_mut()
            .expect("cannot lock GPU context")
            .take()
            .expect("internal error: missing GPU context");
        drop(lock);
        ctx
    }
}
