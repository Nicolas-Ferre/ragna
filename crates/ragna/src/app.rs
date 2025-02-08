use crate::operations::{AssignVarOperation, Glob, Operation, Value};
use crate::runner::Runner;
use crate::types::{GpuType, GpuTypeDetails};
use crate::{wgsl, Gpu, Mut};
use fxhash::FxHashMap;
use itertools::Itertools;
use std::any::TypeId;
use std::mem;

/// The entrypoint of a Ragna application.
#[derive(Debug, Default)]
pub struct App {
    pub(crate) contexts: Vec<GpuContext>,
    pub(crate) globs: Vec<Glob>,
    pub(crate) types: FxHashMap<TypeId, GpuTypeDetails>,
    pub(crate) runner: Option<Runner>,
}

impl App {
    /// Registers a GPU module.
    pub fn with_module(mut self, f: impl FnOnce(Self) -> Self) -> Self {
        f(mem::take(&mut self))
    }

    /// Registers a compute shader.
    pub fn with_compute(mut self, f: impl FnOnce(&mut GpuContext)) -> Self {
        let mut ctx = GpuContext::default();
        f(&mut ctx);
        self.globs = mem::take(&mut self.globs)
            .into_iter()
            .chain(ctx.globs().cloned())
            .unique()
            .collect();
        self.types = mem::take(&mut self.types)
            .into_iter()
            .chain(ctx.types.clone())
            .collect();
        self.contexts.push(ctx);
        self
    }

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
            wgsl::header_code(&self.globs, &self.types),
            ctx.wgsl_code()
        )
    }

    pub(crate) fn wgsl_update_shaders(&self) -> impl Iterator<Item = String> + '_ {
        let header = wgsl::header_code(&self.globs, &self.types);
        self.contexts
            .iter()
            .map(move |ctx| format!("{}{}", header, ctx.wgsl_code()))
    }
}

/// The context used to track GPU operations.
#[derive(Debug, Default)]
pub struct GpuContext {
    pub(crate) next_var_id: u64,
    pub(crate) types: FxHashMap<TypeId, GpuTypeDetails>,
    pub(crate) operations: Vec<Operation>,
}

impl GpuContext {
    pub(crate) fn register_type<T>(&mut self)
    where
        T: GpuType,
    {
        self.types.insert(TypeId::of::<T>(), T::gpu_type_details());
    }

    pub(crate) fn next_var_id(&mut self) -> u64 {
        let id = self.next_var_id;
        self.next_var_id += 1;
        id
    }

    fn globs(&self) -> impl Iterator<Item = &Glob> {
        self.operations.iter().flat_map(|op| op.glob())
    }

    fn wgsl_code(&self) -> String {
        wgsl::compute_shader_code(self)
    }
}
