use crate::{App, GpuTypeDetails};
use wgpu::{
    BindGroup, BindGroupEntry, Buffer, BufferDescriptor, BufferUsages, ComputePass,
    ComputePipeline, ComputePipelineDescriptor, Device, RenderPass, ShaderModuleDescriptor,
};

#[derive(Debug)]
pub(crate) struct Program {
    init_shader: ComputeShader,
    step_shaders: Vec<ComputeShader>,
    pub(crate) buffer: Option<Buffer>,
}

impl Program {
    pub(crate) fn new(app: &App, device: &Device) -> Self {
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

    pub(crate) fn run_init(&self, mut pass: ComputePass<'_>) {
        pass.set_pipeline(&self.init_shader.pipeline);
        pass.set_bind_group(0, &self.init_shader.bind_group, &[]);
        pass.dispatch_workgroups(1, 1, 1);
    }

    pub(crate) fn run_update_step(&self, mut pass: ComputePass<'_>) {
        for shader in &self.step_shaders {
            pass.set_pipeline(&shader.pipeline);
            pass.set_bind_group(0, &shader.bind_group, &[]);
            pass.dispatch_workgroups(1, 1, 1);
        }
    }

    // coverage: off (window cannot be tested)
    #[allow(clippy::unused_self)]
    pub(crate) fn run_draw_step(&self, _pass: RenderPass<'_>) {
        // do nothing for the moment
    }
    // coverage: on

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
