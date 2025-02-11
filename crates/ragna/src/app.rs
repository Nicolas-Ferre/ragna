use crate::operations::{AssignVarOperation, FnCallOperation, Glob, Operation, Value};
use crate::runner::Runner;
use crate::types::{GpuType, GpuTypeDetails};
use crate::{wgsl, Gpu, Mut};
use fxhash::FxHashMap;
use std::any::TypeId;
use std::mem;

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
        .with_type::<i32>()
        .with_type::<u32>()
        .with_type::<f32>()
        .with_type::<bool>()
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
    pub fn with_compute(mut self, f: impl FnOnce(&mut GpuContext)) -> Self {
        let mut ctx = GpuContext::default();
        f(&mut ctx);
        self.contexts.push(ctx);
        self
    }

    #[doc(hidden)]
    pub fn with_glob<T>(mut self, glob: Gpu<T, Mut>) -> Self
    where
        T: GpuType,
    {
        if let Value::Glob(glob) = glob.value() {
            self.globs.push(glob);
        }
        self
    }

    /// Reads a value stored on GPU side.
    ///
    /// If the passed value is not a global variable,
    pub fn read<T>(&self, value: Gpu<T, Mut>) -> Option<T>
    where
        T: GpuType,
    {
        self.runner.as_ref().and_then(|runner| {
            let bytes = runner.read(self, &value.value());
            if bytes.is_empty() {
                None
            } else {
                Some(T::from_bytes(&bytes))
            }
        })
    }

    pub(crate) fn wgsl_init_shader(&self) -> String {
        let mut ctx = GpuContext::default();
        for glob in &self.globs {
            let right_value = glob.default_value.call(&mut ctx);
            ctx.operations
                .push(Operation::AssignVar(AssignVarOperation {
                    left_value: Value::Glob(glob.clone()),
                    right_value,
                }));
        }
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
        T: GpuType,
    {
        self.types.insert(TypeId::of::<T>(), T::gpu_type_details());
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
    #[doc(hidden)]
    pub fn call_fn<T>(&mut self, fn_name: &'static str, args: Vec<Value>) -> Gpu<T, Mut>
    where
        T: GpuType,
    {
        let var = Gpu::uninitialized_var(self);
        self.operations.push(Operation::FnCall(FnCallOperation {
            var: var.value(),
            fn_name,
            args,
        }));
        var
    }

    pub(crate) fn next_var_id(&mut self) -> u64 {
        let id = self.next_var_id;
        self.next_var_id += 1;
        id
    }

    fn wgsl_code(&self, all_globs: &[Glob]) -> String {
        wgsl::compute_shader_code(self, all_globs)
    }
}
