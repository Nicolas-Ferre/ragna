use crate::app::CURRENT_CTX;
use crate::operations::Operation;
use crate::{Gpu, GpuTypeDetails};
use once_cell::sync::{Lazy, OnceCell};
use std::any::Any;
use std::mem;
use std::sync::atomic::{AtomicU32, AtomicUsize, Ordering};
use std::sync::{LockResult, Mutex, MutexGuard};

pub(crate) fn next_var_id() -> u32 {
    static NEXT_ID: AtomicU32 = AtomicU32::new(0);
    NEXT_ID.fetch_add(1, Ordering::Relaxed)
}

pub(crate) fn next_static_value<'a, T: Gpu>(value: T) -> &'a T {
    static VALUES: Lazy<Vec<OnceCell<Box<dyn Any + Sync + Send>>>> =
        Lazy::new(|| (0..10_000).map(|_| OnceCell::new() as _).collect());
    static NEXT_INDEX: AtomicUsize = AtomicUsize::new(0);
    VALUES
        .get(NEXT_INDEX.fetch_add(1, Ordering::Relaxed))
        .expect("index call limit reached")
        .get_or_init(|| Box::new(value) as _)
        .downcast_ref()
        .expect("internal error: invalid static value type")
}

/// The context used to track GPU operations.
#[doc(hidden)]
#[derive(Debug, Default)]
pub struct GpuContext {
    pub(crate) operations: Vec<Operation>,
    pub(crate) types: Vec<GpuTypeDetails>,
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
