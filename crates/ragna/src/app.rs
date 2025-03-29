use crate::context::GpuContext;
use crate::operations::{AssignVarOperation, Operation};
use crate::runner::common::Runner;
use crate::runner::window::WindowRunner;
use crate::types::GpuTypeDetails;
use crate::{wgsl, Cpu, Glob, Gpu, GpuValue};
use derive_where::derive_where;
use fxhash::FxHashMap;
use std::any::TypeId;
use std::mem;
use std::sync::Mutex;
use winit::event_loop::EventLoop;

pub(crate) static CURRENT_CTX: Mutex<Option<GpuContext>> = Mutex::new(None);

/// The entrypoint of a Ragna application.
#[derive(Default)]
#[derive_where(Debug)]
pub struct App {
    pub(crate) contexts: Vec<GpuContext>,
    pub(crate) globs: Vec<GpuValue>,
    #[derive_where(skip)]
    pub(crate) glob_defaults: Vec<Box<dyn Fn() -> GpuValue>>,
    pub(crate) types: FxHashMap<TypeId, (usize, GpuTypeDetails)>,
}

impl App {
    /// Configure the application for testing.
    pub fn testing(self) -> TestApp {
        let runner = Runner::new(&self);
        TestApp { app: self, runner }
    }

    // coverage: off (window cannot be tested)
    /// Configure the application to run with a window.
    pub fn window(self) -> WindowApp {
        WindowApp { app: self }
    }
    // coverage: on

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
    pub fn with_glob<T: Gpu>(mut self, glob: &Glob<T>) -> Self {
        let default_value = glob.default_value;
        self.glob_defaults
            .push(Box::new(move || default_value().value()));
        self.globs.push(glob.inner.value());
        let lock = GpuContext::lock_current();
        GpuContext::run_current(GpuContext::register_type::<T>);
        let mut ctx = GpuContext::unlock_current(lock);
        for type_ in mem::take(&mut ctx.types) {
            self.add_type(type_);
        }
        self
    }

    pub(crate) fn wgsl_init_shader(&self) -> String {
        let lock = GpuContext::lock_current();
        for (glob, default_value) in self.globs.iter().zip(&self.glob_defaults) {
            let right_value = default_value();
            GpuContext::run_current(|ctx| {
                ctx.operations
                    .push(Operation::AssignVar(AssignVarOperation {
                        left_value: *glob,
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

// coverage: off (window cannot be tested)

/// An application run with a window.
#[derive(Debug)]
pub struct WindowApp {
    app: App,
}

impl WindowApp {
    /// Runs the application with a window.
    pub fn run(self) {
        let event_loop = EventLoop::builder()
            .build()
            .expect("event loop initialization failed");
        event_loop
            .run_app(&mut WindowRunner::new(self.app))
            .expect("event loop failed");
    }
}

// coverage: on

/// An application for testing purpose.
#[derive(Debug)]
pub struct TestApp {
    app: App,
    runner: Runner,
}

impl TestApp {
    /// Reads a value stored on GPU side.
    ///
    /// If the passed value is not a global variable,
    pub fn read<T: Gpu>(&self, value: T) -> Option<T::Cpu> {
        let bytes = self.runner.read(&self.app, &value.value());
        if bytes.is_empty() {
            None
        } else {
            Some(Cpu::from_gpu(&bytes))
        }
    }

    /// Runs the application during `update_count` steps.
    pub fn run(mut self, update_count: u64) -> Self {
        for _ in 0..update_count {
            self.runner.run_step();
        }
        self
    }
}
