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
use wgpu::Color;
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
    /// Configure the application to run with a texture target.
    ///
    /// `size` corresponds to the width and height of the texture target.
    pub fn texture(self, size: (u32, u32)) -> TextureApp {
        let runner = Runner::new_texture(&self, size);
        TextureApp { app: self, runner }
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
    ///
    /// `background_color` corresponds to RGBA components between `0.0` and `1.0`
    /// of the applied background color.
    pub fn run(self, background_color: (f64, f64, f64, f64)) {
        let event_loop = EventLoop::builder()
            .build()
            .expect("event loop initialization failed");
        event_loop
            .run_app(&mut WindowRunner::new(
                self.app,
                Color {
                    r: background_color.0,
                    g: background_color.1,
                    b: background_color.2,
                    a: background_color.3,
                },
            ))
            .expect("event loop failed");
    }
}

// coverage: on

/// An application run with a texture target.
#[derive(Debug)]
pub struct TextureApp {
    app: App,
    runner: Runner,
}

impl TextureApp {
    /// Sets the background color of the texture target.
    pub fn with_background_color(mut self, background_color: (f64, f64, f64, f64)) -> Self {
        self.runner.target.config.background_color = Color {
            r: background_color.0,
            g: background_color.1,
            b: background_color.2,
            a: background_color.3,
        };
        self
    }

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

    /// Reads texture target stored on GPU side.
    pub fn read_target(&self) -> TextureData {
        TextureData {
            buffer: self.runner.read_target(),
            size: self.runner.target.config.size,
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

/// Texture data retrieved from GPU.
#[non_exhaustive]
pub struct TextureData {
    /// RGBA buffer of the texture.
    pub buffer: Vec<u8>,
    /// Width and height of the texture.
    pub size: (u32, u32),
}
