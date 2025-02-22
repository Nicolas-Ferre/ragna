use crate::operations::{AssignVarOperation, Glob, Operation, Value};
use crate::runner::Runner;
use crate::types::GpuTypeDetails;
use crate::{wgsl, Bool, Cpu, Gpu, F32, I32, U32};
use fxhash::FxHashMap;
use std::any::TypeId;
use std::mem;
use std::sync::{LockResult, Mutex, MutexGuard};

pub(crate) static CURRENT_CTX: Mutex<Option<GpuContext>> = Mutex::new(None);

/// The entrypoint of a Ragna application.
#[derive(Debug)]
pub struct App {
    pub(crate) contexts: Vec<GpuContext>,
    pub(crate) globs: Vec<Glob>,
    pub(crate) types: FxHashMap<TypeId, GpuTypeDetails>,
    pub(crate) runner: Option<Runner>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            contexts: vec![],
            globs: vec![],
            types: FxHashMap::default(),
            runner: None,
        }
        .with_type::<I32>()
        .with_type::<U32>()
        .with_type::<F32>()
        .with_type::<Bool>()
    }
}

impl App {
    /// Runs the application during `update_count` steps.
    #[allow(clippy::print_stdout)]
    pub fn run(mut self, update_count: u64) -> Self {
        let runner = if let Some(runner) = &mut self.runner {
            runner
        } else {
            self.runner.insert(Runner::new(&self))
        };
        for _ in 0..update_count {
            runner.run_step();
            println!("Step duration: {}µs", runner.delta().as_micros());
        }
        self
    }

    /// Registers a GPU module.
    pub fn with_module(mut self, f: impl FnOnce(Self) -> Self) -> Self {
        f(mem::take(&mut self))
    }

    #[doc(hidden)]
    pub fn with_compute(mut self, f: impl FnOnce()) -> Self {
        let lock = GpuContext::lock_current();
        f();
        self.contexts.push(GpuContext::unlock_current(lock));
        self
    }

    #[doc(hidden)]
    pub fn with_glob<T>(mut self, glob: T) -> Self
    where
        T: Gpu,
    {
        if let Value::Glob(glob) = glob.value().into() {
            self.globs.push(glob);
        }
        self
    }

    /// Reads a value stored on GPU side.
    ///
    /// If the passed value is not a global variable,
    pub fn read<T>(&self, value: T) -> Option<T::Cpu>
    where
        T: Gpu,
    {
        self.runner.as_ref().and_then(|runner| {
            let bytes = runner.read(self, &value.value().into());
            if bytes.is_empty() {
                None
            } else {
                Some(Cpu::from_gpu(&bytes))
            }
        })
    }

    pub(crate) fn wgsl_init_shader(&self) -> String {
        let lock = GpuContext::lock_current();
        for glob in &self.globs {
            let right_value = glob.default_value.call();
            GpuContext::run_current(|ctx| {
                ctx.operations
                    .push(Operation::AssignVar(AssignVarOperation {
                        left_value: Value::Glob(glob.clone()),
                        right_value,
                    }));
            });
        }
        let ctx = GpuContext::unlock_current(lock);
        format!(
            "{}{}",
            wgsl::header_code(&self.types, &self.globs),
            ctx.wgsl_code(&self.globs)
        )
    }

    pub(crate) fn wgsl_update_shaders(&self) -> impl Iterator<Item = String> + '_ {
        let header = wgsl::header_code(&self.types, &self.globs);
        self.contexts
            .iter()
            .map(move |ctx| format!("{}{}", header, ctx.wgsl_code(&self.globs)))
    }

    fn with_type<T>(mut self) -> Self
    where
        T: Gpu,
    {
        self.types.insert(TypeId::of::<T>(), T::details());
        self
    }
}

/// The context used to track GPU operations.
#[derive(Debug, Default)]
pub struct GpuContext {
    pub(crate) next_var_id: u64,
    pub(crate) operations: Vec<Operation>,
}

impl GpuContext {
    pub(crate) fn next_var_id(&mut self) -> u64 {
        let id = self.next_var_id;
        self.next_var_id += 1;
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

    fn lock_current<'a>() -> LockResult<MutexGuard<'a, ()>> {
        static CTX_LOCK: Mutex<()> = Mutex::new(());
        let lock = CTX_LOCK.lock();
        **CURRENT_CTX
            .try_lock()
            .as_mut()
            .expect("cannot lock GPU context") = Some(Self::default());
        lock
    }

    fn unlock_current(lock: LockResult<MutexGuard<'_, ()>>) -> Self {
        let ctx = CURRENT_CTX
            .try_lock()
            .as_mut()
            .expect("cannot lock GPU context")
            .take()
            .expect("internal error: missing GPU context");
        drop(lock);
        ctx
    }

    fn wgsl_code(&self, all_globs: &[Glob]) -> String {
        wgsl::compute_shader_code(self, all_globs)
    }
}
