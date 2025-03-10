use crate::operations::Value;
use crate::{App, GpuTypeDetails};
use futures::executor;
use std::time::{Duration, Instant};
use wgpu::{
    Adapter, BackendOptions, Backends, BindGroup, BindGroupEntry, Buffer, BufferDescriptor,
    BufferUsages, CommandEncoder, CommandEncoderDescriptor, ComputePass, ComputePassDescriptor,
    ComputePipeline, ComputePipelineDescriptor, Device, DeviceDescriptor, Features, Instance,
    InstanceFlags, Limits, MapMode, MemoryHints, PowerPreference, Queue, RequestAdapterOptions,
    ShaderModuleDescriptor,
};

#[derive(Debug)]
pub(crate) struct Runner {
    device: Device,
    queue: Queue,
    program: Program,
    is_started: bool,
    last_delta: Duration,
    last_step_end: Instant,
}

impl Runner {
    pub(crate) fn new(app: &App) -> Self {
        let instance = Self::create_instance();
        let adapter = Self::create_adapter(&instance);
        let (device, queue) = Self::create_device(&adapter);
        let program = Program::new(app, &device);
        Self {
            device,
            queue,
            program,
            is_started: false,
            last_delta: Duration::ZERO,
            last_step_end: Instant::now(),
        }
    }

    pub(crate) fn run_step(&mut self) {
        let start = Instant::now();
        if !self.is_started {
            self.program.run_init(&self.device, &self.queue);
            self.is_started = true;
        }
        self.program.run_step(&self.device, &self.queue);
        let end = Instant::now();
        self.last_delta = end - start;
        self.last_step_end = end;
    }

    pub(crate) fn delta(&self) -> Duration {
        self.last_delta
    }

    pub(crate) fn read(&self, app: &App, value: &Value) -> Vec<u8> {
        if let Some(buffer) = &self.program.buffer {
            if let Some(position) = app.globs.iter().position(|other_glob| other_glob == value) {
                let buffer_type_details = GpuTypeDetails::from_fields(&app.globs, &app.types);
                let value_size = app.types[&value.type_id].1.size();
                let tmp_buffer = self.device.create_buffer(&BufferDescriptor {
                    label: Some("ragna:texture_buffer"),
                    size: value_size,
                    usage: BufferUsages::MAP_READ | BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                });
                let mut encoder = self
                    .device
                    .create_command_encoder(&CommandEncoderDescriptor {
                        label: Some("ragna:buffer_retrieval"),
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

    fn create_instance() -> Instance {
        Instance::new(&wgpu::InstanceDescriptor {
            backends: Backends::from_env().unwrap_or_else(Backends::all),
            flags: InstanceFlags::default(),
            backend_options: BackendOptions::default(),
        })
    }

    fn create_adapter(instance: &Instance) -> Adapter {
        let adapter_request = RequestAdapterOptions {
            power_preference: PowerPreference::default(),
            force_fallback_adapter: false,
            compatible_surface: None,
        };
        executor::block_on(instance.request_adapter(&adapter_request))
            .expect("no supported graphic adapter found")
    }

    fn create_device(adapter: &Adapter) -> (Device, Queue) {
        let device_descriptor = DeviceDescriptor {
            label: None,
            required_features: Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES,
            required_limits: Limits::default(),
            memory_hints: MemoryHints::Performance,
        };
        executor::block_on(adapter.request_device(&device_descriptor, None))
            .expect("error when retrieving graphic device")
    }
}

#[derive(Debug)]
struct Program {
    init_shader: ComputeShader,
    step_shaders: Vec<ComputeShader>,
    buffer: Option<Buffer>,
}

impl Program {
    fn new(app: &App, device: &Device) -> Self {
        let buffer = Self::create_buffer(app, device);
        let bind_group_entry = Self::create_bing_group_entry(buffer.as_ref());
        Self {
            init_shader: ComputeShader::new(
                app.wgsl_init_shader(),
                device,
                bind_group_entry.clone(),
            ),
            step_shaders: app
                .wgsl_update_shaders()
                .map(|code| ComputeShader::new(code, device, bind_group_entry.clone()))
                .collect(),
            buffer,
        }
    }

    fn run_init(&self, device: &Device, queue: &Queue) {
        let mut encoder = Self::create_encoder(device);
        let mut pass = Self::start_compute_pass(&mut encoder);
        pass.set_pipeline(&self.init_shader.pipeline);
        pass.set_bind_group(0, &self.init_shader.bind_group, &[]);
        pass.dispatch_workgroups(1, 1, 1);
        drop(pass);
        queue.submit(Some(encoder.finish()));
    }

    fn run_step(&self, device: &Device, queue: &Queue) {
        let mut encoder = Self::create_encoder(device);
        let mut pass = Self::start_compute_pass(&mut encoder);
        for shader in &self.step_shaders {
            pass.set_pipeline(&shader.pipeline);
            pass.set_bind_group(0, &shader.bind_group, &[]);
            pass.dispatch_workgroups(1, 1, 1);
        }
        drop(pass);
        queue.submit(Some(encoder.finish()));
    }

    fn create_buffer(app: &App, device: &Device) -> Option<Buffer> {
        if app.globs.is_empty() {
            None
        } else {
            Some(device.create_buffer(&BufferDescriptor {
                label: Some("ragna:buffer"),
                size: GpuTypeDetails::from_fields(&app.globs, &app.types).size(),
                usage: BufferUsages::STORAGE | BufferUsages::COPY_SRC,
                mapped_at_creation: false,
            }))
        }
    }

    fn create_bing_group_entry(buffer: Option<&Buffer>) -> Option<BindGroupEntry<'_>> {
        buffer.map(|buffer| BindGroupEntry {
            binding: 0,
            resource: buffer.as_entire_binding(),
        })
    }

    fn create_encoder(device: &Device) -> CommandEncoder {
        device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("ragna:encoder"),
        })
    }

    fn start_compute_pass(encoder: &mut CommandEncoder) -> ComputePass<'_> {
        encoder.begin_compute_pass(&ComputePassDescriptor {
            label: None,
            timestamp_writes: None,
        })
    }
}

#[derive(Debug)]
struct ComputeShader {
    pipeline: ComputePipeline,
    bind_group: Option<BindGroup>,
}

impl ComputeShader {
    fn new(code: String, device: &Device, bind_group_entry: Option<BindGroupEntry<'_>>) -> Self {
        let module = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("ragna:compute_shader:module"),
            source: wgpu::ShaderSource::Wgsl(code.into()),
        });
        let pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: Some("ragna:compute_shader:pipeline"),
            layout: None,
            module: &module,
            entry_point: None,
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            cache: None,
        });
        let bind_group = bind_group_entry.map(|entry| {
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("ragna:compute_shader:bind_group"),
                layout: &pipeline.get_bind_group_layout(0),
                entries: &[entry],
            })
        });
        Self {
            pipeline,
            bind_group,
        }
    }
}
