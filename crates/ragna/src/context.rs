use crate::app::CURRENT_CTX;
use crate::operations::{AssignVarOperation, DeclareVarOperation, Operation, Value};
use crate::types::MAX_INDEX_CALLS_PER_SHADER;
use crate::{Gpu, GpuTypeDetails, GpuValue, U32};
use fxhash::FxHashMap;
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
    pub(crate) indexes: FxHashMap<Value, Vec<U32>>,
    pub(crate) next_index_ids: FxHashMap<Value, usize>,
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

    pub(crate) fn index(&self, value: &Value, id: u8) -> U32 {
        self.indexes[value][id as usize]
    }

    pub(crate) fn next_index_id(&mut self, parent_value: Value, index_value: U32) -> usize {
        let id = *self
            .next_index_ids
            .entry(parent_value.clone())
            .and_modify(|id| {
                *id += 1;
            })
            .or_insert_with(|| {
                let indexes = (0..MAX_INDEX_CALLS_PER_SHADER)
                    .map(|_| U32::from_value(GpuValue::unregistered_var()))
                    .collect();
                self.indexes.insert(parent_value.clone(), indexes);
                0
            });
        let index = self.indexes[&parent_value]
            .get(id)
            .expect("index call limit reached");
        self.operations
            .push(Operation::DeclareVar(DeclareVarOperation {
                id: index.value().var_id(),
                type_: U32::details(),
            }));
        let left_value = index.value().untyped_with_ctx(self);
        let right_value = index_value.value().untyped_with_ctx(self);
        self.operations
            .push(Operation::AssignVar(AssignVarOperation {
                left_value,
                right_value,
            }));
        id
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
