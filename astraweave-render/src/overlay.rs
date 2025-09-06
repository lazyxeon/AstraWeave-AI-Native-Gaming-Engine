#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct OverlayParams {
    pub fade: f32,      // 0..1 black fade
    pub letterbox: f32, // 0..0.45 fraction of screen height for bars
    pub _pad: [f32; 2],
}

pub struct OverlayFx {
    buf: wgpu::Buffer,
    pipeline: wgpu::RenderPipeline,
    bind: wgpu::BindGroup,
}

const SHADER: &str = r#"
struct Params { fade: f32, letterbox: f32, _pad: vec2<f32> };
@group(0) @binding(0) var<uniform> U: Params;

struct VSOut { @builtin(position) pos: vec4<f32>, @location(0) ndc: vec2<f32> };
@vertex
fn vs_main(@builtin(vertex_index) vid: u32) -> VSOut {
  var pos = array<vec2<f32>, 3>(
    vec2<f32>(-1.0, -3.0), vec2<f32>(3.0, -1.0), vec2<f32>(-1.0, 1.0)
  );
  var out: VSOut;
  out.pos = vec4<f32>(pos[vid], 0.0, 1.0);
  out.ndc = pos[vid];
  return out;
}
@fragment
fn fs_main(inf: VSOut) -> @location(0) vec4<f32> {
  let y = (inf.ndc.y * 0.5 + 0.5);
  let lb = U.letterbox;
  var col = vec4<f32>(0.0, 0.0, 0.0, U.fade);
  if (y < lb || y > (1.0 - lb)) {
    // letterbox bar: opaque black
    col = vec4<f32>(0.0, 0.0, 0.0, 1.0);
  }
  return col;
}
"#;

impl OverlayFx {
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("overlay params"),
            size: std::mem::size_of::<OverlayParams>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("overlay bgl"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });
        let bind = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("overlay bind"),
            layout: &bgl,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buf.as_entire_binding(),
            }],
        });
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("overlay shader"),
            source: wgpu::ShaderSource::Wgsl(SHADER.into()),
        });
        let pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("overlay pl"),
            bind_group_layouts: &[&bgl],
            push_constant_ranges: &[],
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("overlay pipe"),
            layout: Some(&pl),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });
        Self {
            buf,
            pipeline,
            bind,
        }
    }

    pub fn update(&self, queue: &wgpu::Queue, p: &OverlayParams) {
        queue.write_buffer(&self.buf, 0, bytemuck::bytes_of(p));
    }

    pub fn draw<'a>(&'a self, rpass: &mut wgpu::RenderPass<'a>) {
        rpass.set_pipeline(&self.pipeline);
        rpass.set_bind_group(0, &self.bind, &[]);
        rpass.draw(0..3, 0..1);
    }
}
