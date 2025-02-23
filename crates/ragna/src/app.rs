use crate::operations::{AssignVarOperation, Glob, Operation, Value};
use crate::runner::Runner;
use crate::types::GpuTypeDetails;
use crate::{wgsl, Cpu, Gpu, GpuValue};
use derive_where::derive_where;
use fxhash::FxHashMap;
use std::any::TypeId;
use std::mem;
use std::sync::{LockResult, Mutex, MutexGuard};

pub(crate) static CURRENT_CTX: Mutex<Option<GpuContext>> = Mutex::new(None);

/// The entrypoint of a Ragna application.
#[derive(Default)]
#[derive_where(Debug)]
pub struct App {
    pub(crate) contexts: Vec<GpuContext>,
    pub(crate) globs: Vec<Glob>,
    #[derive_where(skip)]
    pub(crate) glob_defaults: Vec<Box<dyn Fn() -> Value>>,
    pub(crate) types: FxHashMap<TypeId, (usize, GpuTypeDetails)>,
    pub(crate) runner: Option<Runner>,
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
        let mut ctx = GpuContext::unlock_current(lock);
        for type_ in mem::take(&mut ctx.types) {
            self.add_type(type_);
        }
        self.contexts.push(ctx);
        self
    }

    #[doc(hidden)]
    pub fn with_glob<T: Gpu>(mut self, glob: T) -> Self {
        if let (GpuValue::Glob(_, _, default_value), Value::Glob(glob)) =
            (glob.value(), glob.value().into())
        {
            self.glob_defaults
                .push(Box::new(move || default_value().value().into()));
            self.globs.push(glob);
            self.add_type(T::details());
        }
        self
    }

    /// Reads a value stored on GPU side.
    ///
    /// If the passed value is not a global variable,
    pub fn read<T: Gpu>(&self, value: T) -> Option<T::Cpu> {
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
        for (glob, default_value) in self.globs.iter().zip(&self.glob_defaults) {
            let right_value = default_value();
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
            wgsl::compute_shader_code(&ctx, &self.types, &self.globs)
        )
    }

    pub(crate) fn wgsl_update_shaders(&self) -> impl Iterator<Item = String> + '_ {
        let header = wgsl::header_code(&self.types, &self.globs);
        self.contexts.iter().map(move |ctx| {
            format!(
                "{}{}",
                header,
                wgsl::compute_shader_code(ctx, &self.types, &self.globs)
            )
        })
    }

    pub(crate) fn add_type(&mut self, type_: GpuTypeDetails) {
        let type_count = self.types.len();
        self.types
            .entry(type_.type_id)
            .or_insert((type_count, type_));
    }
}

/// The context used to track GPU operations.
#[doc(hidden)]
#[derive(Debug, Default)]
pub struct GpuContext {
    pub(crate) next_var_id: u64,
    pub(crate) operations: Vec<Operation>,
    pub(crate) types: Vec<GpuTypeDetails>,
}

impl GpuContext {
    pub(crate) fn next_var_id(&mut self) -> u64 {
        let id = self.next_var_id;
        self.next_var_id += 1;
        id
    }

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
}
