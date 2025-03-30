use crate::runner::program::Program;
use crate::{App, GpuTypeDetails, GpuValue};
use futures::executor;
use std::sync::Arc;
use std::time::{Duration, Instant};
use wgpu::{
    Adapter, BackendOptions, Backends, BufferDescriptor, BufferUsages, Color, CommandEncoder,
    CommandEncoderDescriptor, ComputePass, ComputePassDescriptor, Device, DeviceDescriptor,
    Extent3d, Features, Instance, InstanceFlags, Limits, LoadOp, MapMode, MemoryHints, Operations,
    PowerPreference, Queue, RenderPass, RenderPassColorAttachment,
    RenderPassDepthStencilAttachment, RenderPassDescriptor, RequestAdapterOptions, StoreOp,
    Surface, SurfaceConfiguration, SurfaceTexture, TexelCopyBufferInfo, TexelCopyBufferLayout,
    Texture, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages, TextureView,
    TextureViewDescriptor,
};
use winit::dpi::PhysicalSize;
use winit::event_loop::ActiveEventLoop;
use winit::window::Window;

#[derive(Debug)]
pub(crate) struct Runner {
    pub(crate) target: Target,
    instance: Instance,
    device: Device,
    adapter: Adapter,
    queue: Queue,
    program: Program,
    is_started: bool,
    last_delta: Duration,
    last_step_end: Instant,
}

// TODO: check window size to ensure there is no crash

impl Runner {
    pub(crate) fn new_texture(app: &App, size: (u32, u32)) -> Self {
        let target = TargetConfig {
            size: (size.0.max(1), size.1.max(1)),
            ..Default::default()
        };
        let instance = Self::create_instance();
        let adapter = Self::create_adapter(&instance, None);
        let (device, queue) = Self::create_device(&adapter);
        let texture = Self::create_target_texture(&device, target.size);
        let view = texture.create_view(&TextureViewDescriptor::default());
        let depth_buffer = Self::create_depth_buffer(&device, target.size);
        let program = Program::new(app, &device);
        Self {
            target: Target {
                inner: TargetSpecialized::Texture(TextureTarget { texture, view }),
                config: target,
                depth_buffer,
            },
            instance,
            device,
            adapter,
            queue,
            program,
            is_started: false,
            last_delta: Duration::ZERO,
            last_step_end: Instant::now(),
        }
    }

    // coverage: off (window cannot be tested)
    pub(crate) fn new_window(
        app: &App,
        event_loop: &ActiveEventLoop,
        background_color: Color,
    ) -> Self {
        let target = TargetConfig {
            background_color,
            ..Default::default()
        };
        let instance = Self::create_instance();
        let window = Self::create_window(event_loop, target.size);
        let surface = Self::create_surface(&instance, window.clone());
        let adapter = Self::create_adapter(&instance, Some(&surface));
        let (device, queue) = Self::create_device(&adapter);
        let surface_config = Self::create_surface_config(&adapter, &device, &surface, target.size);
        let depth_buffer = Self::create_depth_buffer(&device, target.size);
        let program = Program::new(app, &device);
        Self {
            target: Target {
                inner: TargetSpecialized::Window(WindowTarget {
                    window,
                    surface,
                    surface_config,
                }),
                config: target,
                depth_buffer,
            },
            instance,
            device,
            adapter,
            queue,
            program,
            is_started: false,
            last_delta: Duration::ZERO,
            last_step_end: Instant::now(),
        }
    }
    // coverage: on

    #[allow(clippy::print_stdout)]
    pub(crate) fn run_step(&mut self) {
        let start = Instant::now();
        let mut encoder = self.create_encoder();
        if !self.is_started {
            let pass = Self::create_compute_pass(&mut encoder);
            self.program.run_init(pass);
            self.is_started = true;
        }
        let pass = Self::create_compute_pass(&mut encoder);
        self.program.run_update_step(pass);
        match &self.target.inner {
            // coverage: off (window cannot be tested)
            TargetSpecialized::Window(target) => {
                let texture = target.create_surface_texture();
                let view = Self::create_surface_view(&texture);
                let pass = Self::create_render_pass(
                    &mut encoder,
                    &view,
                    &self.target.depth_buffer,
                    self.target.config.background_color,
                );
                self.program.run_draw_step(pass);
                self.queue.submit(Some(encoder.finish()));
                texture.present();
            }
            // coverage: on
            TargetSpecialized::Texture(target) => {
                let pass = Self::create_render_pass(
                    &mut encoder,
                    &target.view,
                    &self.target.depth_buffer,
                    self.target.config.background_color,
                );
                self.program.run_draw_step(pass);
                self.queue.submit(Some(encoder.finish()));
            }
        }
        let end = Instant::now();
        self.last_delta = end - start;
        self.last_step_end = end;
        println!(
            "Step duration: {}µs ({}fps)",
            self.last_delta.as_micros(),
            (1. / self.last_delta.as_secs_f32()).round()
        );
    }

    pub(crate) fn read(&self, app: &App, value: &GpuValue) -> Vec<u8> {
        if let Some(buffer) = &self.program.buffer {
            if let Some(position) = app.globs.iter().position(|other_glob| other_glob == value) {
                let buffer_type_details = GpuTypeDetails::from_fields(&app.globs, &app.types);
                let value_size = app.types[&value.type_id].1.size();
                let tmp_buffer = self.device.create_buffer(&BufferDescriptor {
                    label: Some("ragna:glob_buffer"),
                    size: value_size,
                    usage: BufferUsages::MAP_READ | BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                });
                let mut encoder = self
                    .device
                    .create_command_encoder(&CommandEncoderDescriptor {
                        label: Some("ragna:glob_buffer_retrieval"),
                    });
                encoder.copy_buffer_to_buffer(
                    buffer,
                    buffer_type_details.field_offset(position),
                    &tmp_buffer,
                    0,
                    value_size,
                );
                let submission_index = self.queue.submit(Some(encoder.finish()));
                let slice = tmp_buffer.slice(..);
                slice.map_async(MapMode::Read, |_| ());
                self.device
                    .poll(wgpu::Maintain::WaitForSubmissionIndex(submission_index));
                let view = slice.get_mapped_range();
                let content = view.to_vec();
                drop(view);
                tmp_buffer.unmap();
                content
            } else {
                vec![]
            }
        } else {
            vec![]
        }
    }

    pub(crate) fn read_target(&self) -> Vec<u8> {
        match &self.target.inner {
            TargetSpecialized::Texture(target) => {
                let size = self.target.config.size;
                let padded_bytes_per_row = Self::calculate_padded_row_bytes(size.0);
                let padded_row_bytes = Self::calculate_padded_row_bytes(size.0);
                let tmp_buffer = self.device.create_buffer(&BufferDescriptor {
                    label: Some("ragna:texture_buffer"),
                    size: (padded_bytes_per_row * size.1).into(),
                    usage: BufferUsages::MAP_READ | BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                });
                let mut encoder = self
                    .device
                    .create_command_encoder(&CommandEncoderDescriptor {
                        label: Some("ragna:texture_buffer_retrieval"),
                    });
                encoder.copy_texture_to_buffer(
                    target.texture.as_image_copy(),
                    TexelCopyBufferInfo {
                        buffer: &tmp_buffer,
                        layout: TexelCopyBufferLayout {
                            offset: 0,
                            bytes_per_row: Some(padded_row_bytes),
                            rows_per_image: None,
                        },
                    },
                    Extent3d {
                        width: size.0,
                        height: size.1,
                        depth_or_array_layers: 1,
                    },
                );
                let submission_index = self.queue.submit(Some(encoder.finish()));
                let slice = tmp_buffer.slice(..);
                slice.map_async(MapMode::Read, |_| ());
                self.device
                    .poll(wgpu::Maintain::WaitForSubmissionIndex(submission_index));
                let view = slice.get_mapped_range();
                let padded_row_bytes = Self::calculate_padded_row_bytes(size.0);
                let unpadded_row_bytes = Self::calculate_unpadded_row_bytes(size.0);
                let content = view
                    .chunks(padded_row_bytes as usize)
                    .flat_map(|a| &a[..unpadded_row_bytes as usize])
                    .copied()
                    .collect();
                drop(view);
                tmp_buffer.unmap();
                content
            }
            TargetSpecialized::Window(_) => {
                unreachable!("internal error: cannot read window buffer")
            }
        }
    }

    // coverage: off (window cannot be tested)
    pub(crate) fn refresh_surface(&mut self) {
        match &mut self.target.inner {
            TargetSpecialized::Window(target) => {
                target.surface = Self::create_surface(&self.instance, target.window.clone());
                target.surface_config = Self::create_surface_config(
                    &self.adapter,
                    &self.device,
                    &target.surface,
                    self.target.config.size,
                );
            }
            TargetSpecialized::Texture(_) => {
                unreachable!("internal error: refreshing non-window target surface")
            }
        }
    }

    pub(crate) fn update_surface_size(&mut self, size: PhysicalSize<u32>) {
        match &mut self.target.inner {
            TargetSpecialized::Window(target) => {
                self.target.config.size = (size.width, size.height);
                self.target.depth_buffer =
                    Self::create_depth_buffer(&self.device, self.target.config.size);
                target.surface_config = Self::create_surface_config(
                    &self.adapter,
                    &self.device,
                    &target.surface,
                    self.target.config.size,
                );
            }
            TargetSpecialized::Texture(_) => {
                unreachable!("internal error: updating non-window target surface")
            }
        }
    }
    // coverage: on

    fn calculate_padded_row_bytes(width: u32) -> u32 {
        let unpadded_bytes_per_row = Self::calculate_unpadded_row_bytes(width);
        let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
        let padded_bytes_per_row_padding = (align - unpadded_bytes_per_row % align) % align;
        unpadded_bytes_per_row + padded_bytes_per_row_padding
    }

    #[allow(clippy::cast_possible_truncation)]
    fn calculate_unpadded_row_bytes(width: u32) -> u32 {
        let bytes_per_pixel = size_of::<u32>() as u32;
        width * bytes_per_pixel
    }

    fn create_instance() -> Instance {
        Instance::new(&wgpu::InstanceDescriptor {
            backends: Backends::from_env().unwrap_or_else(Backends::all),
            flags: InstanceFlags::default(),
            backend_options: BackendOptions::default(),
        })
    }

    fn create_adapter(instance: &Instance, surface: Option<&Surface<'_>>) -> Adapter {
        let adapter_request = RequestAdapterOptions {
            power_preference: PowerPreference::default(),
            force_fallback_adapter: false,
            compatible_surface: surface,
        };
        executor::block_on(instance.request_adapter(&adapter_request))
            .expect("no supported graphic adapter found")
    }

    fn create_device(adapter: &Adapter) -> (Device, Queue) {
        let device_descriptor = DeviceDescriptor {
            label: None,
            required_features: Features::default(),
            required_limits: Limits::default(),
            memory_hints: MemoryHints::Performance,
        };
        executor::block_on(adapter.request_device(&device_descriptor, None))
            .expect("error when retrieving graphic device")
    }

    // coverage: off (window cannot be tested)
    fn create_window(event_loop: &ActiveEventLoop, size: (u32, u32)) -> Arc<Window> {
        let size = PhysicalSize::new(size.0, size.1);
        let window = event_loop
            .create_window(Window::default_attributes().with_inner_size(size))
            .expect("cannot create window");
        Arc::new(window)
    }

    fn create_surface(instance: &Instance, window: Arc<Window>) -> Surface<'static> {
        instance
            .create_surface(window)
            .expect("cannot create surface")
    }

    fn create_surface_config(
        adapter: &Adapter,
        device: &Device,
        surface: &Surface<'_>,
        size: (u32, u32),
    ) -> SurfaceConfiguration {
        let config = surface
            .get_default_config(adapter, size.0, size.1)
            .expect("not supported surface");
        surface.configure(device, &config);
        config
    }

    fn create_surface_view(texture: &SurfaceTexture) -> TextureView {
        texture
            .texture
            .create_view(&TextureViewDescriptor::default())
    }
    // coverage: on

    fn create_target_texture(device: &Device, size: (u32, u32)) -> Texture {
        device.create_texture(&TextureDescriptor {
            label: Some("ragna:target_texture"),
            size: Extent3d {
                width: size.0,
                height: size.1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8UnormSrgb,
            usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::COPY_SRC,
            view_formats: &[],
        })
    }

    fn create_depth_buffer(device: &Device, size: (u32, u32)) -> TextureView {
        let texture = device.create_texture(&TextureDescriptor {
            label: Some("ragna:depth_texture"),
            size: Extent3d {
                width: size.0,
                height: size.1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Depth32Float,
            usage: TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        texture.create_view(&TextureViewDescriptor::default())
    }

    fn create_encoder(&self) -> CommandEncoder {
        self.device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("ragna:encoder"),
            })
    }

    fn create_compute_pass(encoder: &mut CommandEncoder) -> ComputePass<'_> {
        encoder.begin_compute_pass(&ComputePassDescriptor {
            label: None,
            timestamp_writes: None,
        })
    }

    fn create_render_pass<'a>(
        encoder: &'a mut CommandEncoder,
        view: &'a TextureView,
        depth_buffer: &'a TextureView,
        background_color: Color,
    ) -> RenderPass<'a> {
        encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("ragna:render_pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Clear(background_color),
                    store: StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                view: depth_buffer,
                depth_ops: Some(Operations {
                    load: LoadOp::Clear(1.0),
                    store: StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
        })
    }
}

#[derive(Debug)]
pub(crate) struct Target {
    pub(crate) inner: TargetSpecialized,
    pub(crate) config: TargetConfig,
    depth_buffer: TextureView,
}

#[derive(Debug)]
pub(crate) enum TargetSpecialized {
    Window(WindowTarget),
    Texture(TextureTarget),
}

#[derive(Debug)]
pub(crate) struct TextureTarget {
    texture: Texture,
    view: TextureView,
}

// coverage: off (window cannot be tested)

#[derive(Debug)]
pub(crate) struct WindowTarget {
    pub(crate) window: Arc<Window>,
    surface: Surface<'static>,
    surface_config: SurfaceConfiguration,
}

impl WindowTarget {
    fn create_surface_texture(&self) -> SurfaceTexture {
        self.surface
            .get_current_texture()
            .expect("internal error: cannot retrieve surface texture")
    }
}

// coverage: on

#[derive(Debug)]
pub(crate) struct TargetConfig {
    pub(crate) size: (u32, u32),
    pub(crate) background_color: Color,
}

impl Default for TargetConfig {
    fn default() -> Self {
        Self {
            size: (800, 600),
            background_color: Color::BLACK,
        }
    }
}
