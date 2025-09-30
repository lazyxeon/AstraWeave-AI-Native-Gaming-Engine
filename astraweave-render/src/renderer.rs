#[cfg(feature = "postfx")]
use crate::post::{WGSL_SSAO, WGSL_SSGI, WGSL_SSR};
use anyhow::Result;
use glam::Vec4Swizzles;
use glam::{vec3, Mat4};
use std::borrow::Cow;
use wgpu::util::DeviceExt;

use crate::camera::Camera;
use crate::clustered::{bin_lights_cpu, ClusterDims, CpuLight, WGSL_CLUSTER_BIN};
use crate::depth::Depth;
use crate::types::SkinnedVertex;
use crate::types::{Instance, InstanceRaw, Mesh};
use astraweave_cinematics as awc;
use astraweave_materials::MaterialPackage;

const SHADER_SRC: &str = r#"
struct VSIn {
    @location(0) position: vec3<f32>,
    @location(1) normal:   vec3<f32>,
    @location(12) tangent:  vec4<f32>,
    @location(13) uv:       vec2<f32>,
  @location(2) m0: vec4<f32>,
  @location(3) m1: vec4<f32>,
  @location(4) m2: vec4<f32>,
  @location(5) m3: vec4<f32>,
  @location(6) n0: vec3<f32>,
  @location(7) n1: vec3<f32>,
  @location(8) n2: vec3<f32>,
  @location(9) color: vec4<f32>,
};

struct VSOut {
  @builtin(position) pos: vec4<f32>,
  @location(0) world_pos: vec3<f32>,
  @location(1) normal: vec3<f32>,
    @location(3) tbn0: vec3<f32>,
    @location(4) tbn1: vec3<f32>,
    @location(5) tbn2: vec3<f32>,
    @location(6) uv: vec2<f32>,
  @location(2) color: vec4<f32>,
};

struct Camera {
  view_proj: mat4x4<f32>,
  light_dir: vec3<f32>,
  _pad: f32,
};

@group(0) @binding(0) var<uniform> uCamera: Camera;

struct MaterialUbo {
    base_color: vec4<f32>,
    metallic: f32,
    roughness: f32,
    _pad: vec2<f32>,
};

@group(1) @binding(0) var<uniform> uMaterial: MaterialUbo;

struct MainLightUbo {
    view_proj0: mat4x4<f32>,
    view_proj1: mat4x4<f32>,
    splits: vec2<f32>,
    extras: vec2<f32>, // x: pcf_radius_px, y: depth_bias; z: slope_scale in skinned path extras.x reused; keep 2 vec2s for alignment
};
@group(2) @binding(0) var<uniform> uLight: MainLightUbo;
@group(2) @binding(1) var shadow_tex: texture_depth_2d_array;
@group(2) @binding(2) var shadow_sampler: sampler_comparison;

@group(3) @binding(0) var albedo_tex: texture_2d<f32>;
@group(3) @binding(1) var albedo_samp: sampler;
@group(3) @binding(2) var mr_tex: texture_2d<f32>;      // R: metallic, G: roughness
@group(3) @binding(3) var mr_samp: sampler;
@group(3) @binding(4) var normal_tex: texture_2d<f32>;  // tangent-space normal in RGB
@group(3) @binding(5) var normal_samp: sampler;



@vertex
fn vs(input: VSIn) -> VSOut {
  let model = mat4x4<f32>(input.m0, input.m1, input.m2, input.m3);
  let world = model * vec4<f32>(input.position, 1.0);
  var out: VSOut;
  out.pos = uCamera.view_proj * world;
    // normal matrix simplified (assuming uniform scale); for accuracy pass and use 3x3
    let Nw = normalize((model * vec4<f32>(input.normal, 0.0)).xyz);
    let Tw = normalize((model * vec4<f32>(input.tangent.xyz, 0.0)).xyz);
    let Bw = normalize(cross(Nw, Tw)) * input.tangent.w;
    out.normal = Nw;
  out.world_pos = world.xyz;
    out.tbn0 = Tw; out.tbn1 = Bw; out.tbn2 = Nw;
    out.uv = input.uv;
    out.color = input.color;
    return out;
}

// Simple Cook-Torrance PBR with single directional light, no IBL
fn fresnel_schlick(cos_theta: f32, F0: vec3<f32>) -> vec3<f32> {
    return F0 + (vec3<f32>(1.0,1.0,1.0) - F0) * pow(1.0 - cos_theta, 5.0);
}

fn distribution_ggx(N: vec3<f32>, H: vec3<f32>, roughness: f32) -> f32 {
    let a = roughness * roughness;
    let a2 = a * a;
    let NdotH = max(dot(N, H), 0.0);
    let NdotH2 = NdotH * NdotH;
    let denom = (NdotH2 * (a2 - 1.0) + 1.0);
    return a2 / (3.14159 * denom * denom + 1e-5);
}

fn geometry_smith(N: vec3<f32>, V: vec3<f32>, L: vec3<f32>, roughness: f32) -> f32 {
    let r = (roughness + 1.0);
    let k = (r * r) / 8.0;
    let NdotV = max(dot(N, V), 0.0);
    let NdotL = max(dot(N, L), 0.0);
    let ggx1 = NdotV / (NdotV * (1.0 - k) + k + 1e-5);
    let ggx2 = NdotL / (NdotL * (1.0 - k) + k + 1e-5);
    return ggx1 * ggx2;
}

@fragment
fn fs(input: VSOut) -> @location(0) vec4<f32> {
    let V = normalize(-input.world_pos); // fake view dir from origin camera
    let L = normalize(-uCamera.light_dir);
    let H = normalize(V + L);
    // Base normal from geometry
    var N = normalize(input.normal);
    // Normal map sample using real UVs and TBN
    let nrm_rgb = textureSample(normal_tex, normal_samp, input.uv).rgb;
    let nrm_ts = normalize(nrm_rgb * 2.0 - vec3<f32>(1.0,1.0,1.0));
    let T = input.tbn0; let B = input.tbn1; let NN = input.tbn2;
    N = normalize(T * nrm_ts.x + B * nrm_ts.y + NN * nrm_ts.z);
    let NdotL = max(dot(N, L), 0.0);

    var base_color = (uMaterial.base_color.rgb * input.color.rgb);
    let tex = textureSample(albedo_tex, albedo_samp, input.uv);
    base_color = base_color * tex.rgb;
    var metallic = clamp(uMaterial.metallic, 0.0, 1.0);
    var roughness = clamp(uMaterial.roughness, 0.04, 1.0);
    let mr = textureSample(mr_tex, mr_samp, input.uv);
    metallic = clamp(max(metallic, mr.r), 0.0, 1.0);
    roughness = clamp(min(roughness, max(mr.g, 0.04)), 0.04, 1.0);

    let F0 = mix(vec3<f32>(0.04, 0.04, 0.04), base_color, metallic);
    let F = fresnel_schlick(max(dot(H, V), 0.0), F0);
    let D = distribution_ggx(N, H, roughness);
    let G = geometry_smith(N, V, L, roughness);

    let numerator = D * G * F;
    let denom = 4.0 * max(dot(N, V), 0.0) * NdotL + 1e-5;
    let specular = numerator / denom;

    let kd = (vec3<f32>(1.0,1.0,1.0) - F) * (1.0 - metallic);
    let diffuse = kd * base_color / 3.14159;

    let radiance = vec3<f32>(1.0, 0.98, 0.9); // dir light color
        // Shadow sampling
        // Cascaded shadow mapping (2 cascades)
    let dist = length(input.world_pos);
    let use_c0 = dist < uLight.splits.x;
    var lvp: mat4x4<f32>;
    if (use_c0) { lvp = uLight.view_proj0; } else { lvp = uLight.view_proj1; }
        let lp = lvp * vec4<f32>(input.world_pos, 1.0);
    let ndc_shadow = lp.xyz / lp.w;
    let uv = ndc_shadow.xy * 0.5 + vec2<f32>(0.5, 0.5);
    let depth = ndc_shadow.z;
    let slope = max(0.0, 1.0 - dot(N, L));
    let base_bias = uLight.extras.y;
    let bias = max(base_bias /* + slope_scale * slope */ , 0.00001);
        var shadow: f32 = 1.0;
        if (uv.x >= 0.0 && uv.x <= 1.0 && uv.y >= 0.0 && uv.y <= 1.0) {
            var layer: i32;
            if (use_c0) { layer = 0; } else { layer = 1; }
            // PCF 3x3 (scaled by pcf radius in texels from extras.x)
            let dims = vec2<f32>(textureDimensions(shadow_tex).xy);
            let texel = 1.0 / dims;
            let r = max(0.0, uLight.extras.x);
            var sum = 0.0;
            for (var dx: i32 = -1; dx <= 1; dx = dx + 1) {
                for (var dy: i32 = -1; dy <= 1; dy = dy + 1) {
                    let o = vec2<f32>(f32(dx), f32(dy)) * texel * r;
                    sum = sum + textureSampleCompare(shadow_tex, shadow_sampler, uv + o, layer, depth - bias);
                }
            }
            shadow = sum / 9.0;
        }
        // Optional debug visualization: use uMaterial._pad.x > 0.5 to tint by cascade
        if (uMaterial._pad.x > 0.5) {
            var tint: vec3<f32>;
            if (use_c0) { tint = vec3<f32>(1.0, 0.3, 0.0); } else { tint = vec3<f32>(0.0, 0.2, 1.0); }
            base_color = mix(base_color, tint, 0.35);
        }
    // Add a modest ambient lift to avoid overly dark scene when sun is low
    var lit_color = (diffuse + specular) * radiance * NdotL * shadow + base_color * 0.08;
        // Clustered point lights accumulation (Lambert + simple attenuation)
    // Clustered lighting disabled for this example build; use lit_color directly
    return vec4<f32>(lit_color, uMaterial.base_color.a * input.color.a);
}
"#;

const POST_SHADER: &str = r#"
struct VSOut { @builtin(position) pos: vec4<f32>, @location(0) uv: vec2<f32> };
@vertex
fn vs_main(@builtin(vertex_index) vid: u32) -> VSOut {
    var pos = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -3.0),
        vec2<f32>( 3.0,  1.0),
        vec2<f32>(-1.0,  1.0)
    );
    var out: VSOut;
    out.pos = vec4<f32>(pos[vid], 0.0, 1.0);
    out.uv = (pos[vid] + vec2<f32>(1.0,1.0)) * 0.5;
    return out;
}

@group(0) @binding(0) var hdr_tex: texture_2d<f32>;
@group(0) @binding(1) var samp: sampler;

fn aces_tonemap(x: vec3<f32>) -> vec3<f32> {
    let a = 2.51;
    let b = 0.03;
    let c = 2.43;
    let d = 0.59;
    let e = 0.14;
    return clamp((x*(a*x+b))/(x*(c*x+d)+e), vec3<f32>(0.0), vec3<f32>(1.0));
}

@fragment
fn fs_main(in: VSOut) -> @location(0) vec4<f32> {
    let hdr = textureSampleLevel(hdr_tex, samp, in.uv, 0.0);
    let mapped = aces_tonemap(vec3<f32>(hdr.r, hdr.g, hdr.b));
    return vec4<f32>(mapped, 1.0);
}
"#;

#[cfg(feature = "postfx")]
const POST_SHADER_FX: &str = r#"
struct VSOut { @builtin(position) pos: vec4<f32>, @location(0) uv: vec2<f32> };
@vertex
fn vs_main(@builtin(vertex_index) vid: u32) -> VSOut {
    var pos = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -3.0),
        vec2<f32>( 3.0,  1.0),
        vec2<f32>(-1.0,  1.0)
    );
    var out: VSOut;
    out.pos = vec4<f32>(pos[vid], 0.0, 1.0);
    out.uv = (pos[vid] + vec2<f32>(1.0,1.0)) * 0.5;
    return out;
}

@group(0) @binding(0) var hdr_tex: texture_2d<f32>;
@group(0) @binding(1) var ao_tex: texture_2d<f32>;
@group(0) @binding(2) var gi_tex: texture_2d<f32>;
@group(0) @binding(3) var samp: sampler;

fn aces_tonemap(x: vec3<f32>) -> vec3<f32> {
    let a = 2.51; let b = 0.03; let c = 2.43; let d = 0.59; let e = 0.14;
    return clamp((x*(a*x+b))/(x*(c*x+d)+e), vec3<f32>(0.0), vec3<f32>(1.0));
}

@fragment
fn fs_main(in: VSOut) -> @location(0) vec4<f32> {
    let hdr = textureSampleLevel(hdr_tex, samp, in.uv, 0.0).rgb;
    let ao = textureSampleLevel(ao_tex, samp, in.uv, 0.0).r;
    let gi = textureSampleLevel(gi_tex, samp, in.uv, 0.0).rgb;
    let ao_strength = 0.6;
    let gi_strength = 0.2;
    let comp = hdr * (1.0 - ao * ao_strength) + gi * gi_strength;
    let mapped = aces_tonemap(comp);
    return vec4<f32>(mapped, 1.0);
}
"#;

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct CameraUBO {
    view_proj: [[f32; 4]; 4],
    light_dir_pad: [f32; 4],
}

pub struct Renderer {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    depth: Depth,

    #[allow(dead_code)]
    shader: wgpu::ShaderModule,
    pipeline: wgpu::RenderPipeline,
    material_buf: wgpu::Buffer,
    material_bg: wgpu::BindGroup,
    post_pipeline: wgpu::RenderPipeline,
    post_bind_group: wgpu::BindGroup,
    #[allow(dead_code)]
    post_bgl: wgpu::BindGroupLayout,
    hdr_tex: wgpu::Texture,
    hdr_view: wgpu::TextureView,
    hdr_sampler: wgpu::Sampler,
    #[allow(dead_code)]
    shadow_tex: wgpu::Texture,
    #[allow(dead_code)]
    shadow_view: wgpu::TextureView, // array view for sampling
    shadow_layer0_view: wgpu::TextureView,
    shadow_layer1_view: wgpu::TextureView,
    #[allow(dead_code)]
    shadow_sampler: wgpu::Sampler,
    shadow_pipeline: wgpu::RenderPipeline,
    light_buf: wgpu::Buffer,
    light_bg: wgpu::BindGroup,
    // Bind group used only during shadow depth passes (binds light buffer only) to avoid sampling the
    // shadow depth texture while it's being written.
    light_bg_shadow: wgpu::BindGroup,
    #[allow(dead_code)]
    shadow_bgl: wgpu::BindGroupLayout,
    // Cascade data cached on CPU for shadow passes
    cascade0: glam::Mat4,
    cascade1: glam::Mat4,
    split0: f32,
    split1: f32,
    // Tunable cascade ortho extents (half-width/height)
    cascade0_extent: f32,
    cascade1_extent: f32,
    // CSM tuning
    cascade_lambda: f32, // split distribution (0..1)
    shadow_pcf_radius_px: f32,
    shadow_depth_bias: f32,
    shadow_slope_scale: f32,

    // Albedo (base color) texture and sampler
    albedo_tex: wgpu::Texture,
    albedo_view: wgpu::TextureView,
    albedo_sampler: wgpu::Sampler,
    tex_bgl: wgpu::BindGroupLayout,
    tex_bg: wgpu::BindGroup,
    // Metallic-Roughness texture and sampler
    mr_tex: wgpu::Texture,
    mr_view: wgpu::TextureView,
    mr_sampler: wgpu::Sampler,
    // Normal map texture and sampler
    normal_tex: wgpu::Texture,
    normal_view: wgpu::TextureView,
    normal_sampler: wgpu::Sampler,
    // Extra textures bind group layout and group (for future extensibility)
    // extra texture bind group layout/bg removed; combined tex_bgl/tex_bg used
    camera_ubo: CameraUBO,
    camera_buf: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,

    #[allow(dead_code)]
    mesh_cube: Mesh,
    mesh_sphere: Mesh,
    mesh_plane: Mesh,
    mesh_external: Option<Mesh>,

    instances: Vec<Instance>,
    instance_buf: wgpu::Buffer,

    #[allow(dead_code)]
    overlay: crate::overlay::OverlayFx,
    pub overlay_params: crate::overlay::OverlayParams,
    pub weather: crate::effects::WeatherFx,
    // Environment & sky
    sky: crate::environment::SkyRenderer,

    // Skinning (v0)
    #[allow(dead_code)]
    skin_bgl: wgpu::BindGroupLayout,
    #[allow(dead_code)]
    skin_bg: wgpu::BindGroup,
    skin_palette_buf: wgpu::Buffer,
    skinned_pipeline: wgpu::RenderPipeline,
    skinned_mesh: Option<(wgpu::Buffer, wgpu::Buffer, u32)>, // (vbuf, ibuf, index_count)

    // Clustered lighting resources
    clustered_dims: ClusterDims,
    clustered_params_buf: wgpu::Buffer,
    clustered_lights_buf: wgpu::Buffer,
    clustered_offsets_buf: wgpu::Buffer,
    clustered_counts_buf: wgpu::Buffer,
    #[allow(dead_code)]
    clustered_indices_buf: wgpu::Buffer,
    #[allow(dead_code)]
    clustered_bgl: wgpu::BindGroupLayout,
    #[allow(dead_code)]
    clustered_bg: wgpu::BindGroup,
    #[allow(dead_code)]
    clustered_comp_bgl: wgpu::BindGroupLayout,
    clustered_comp_bg: wgpu::BindGroup,
    clustered_comp_pipeline: wgpu::ComputePipeline,
    point_lights: Vec<CpuLight>,
    #[cfg(feature = "gpu-tests")]
    timestamp_query_set: wgpu::QuerySet,
    #[cfg(feature = "gpu-tests")]
    timestamp_buf: wgpu::Buffer,

    // Cinematics integration
    cin_tl: Option<awc::Timeline>,
    cin_seq: awc::Sequencer,
    cin_playing: bool,
}

impl Renderer {
    /// Compose a standalone fragment shader from a `MaterialPackage` for validation/pipeline creation.
    /// Returns a `ShaderModule` ready to be used in a pipeline (caller wires layouts/bindings).
    pub fn shader_from_material_package(&self, pkg: &MaterialPackage) -> wgpu::ShaderModule {
        // Declare group(0) bindings based on `bindings` ids collected by the compiler (tex/sampler pairs)
        let mut decls = String::new();
        let mut idx = 0u32;
        for id in pkg.bindings.iter() {
            decls.push_str(&format!(
                "@group(0) @binding({}) var tex_{}: texture_2d<f32>;\n",
                idx, id
            ));
            idx += 1;
            decls.push_str(&format!(
                "@group(0) @binding({}) var samp_{}: sampler;\n",
                idx, id
            ));
            idx += 1;
        }
        // Compose WGSL: eval_material + a tiny VS/FS pair.
        let full = format!(
            "{}\n{}\nstruct VSOut {{ @builtin(position) pos: vec4<f32>, @location(0) uv: vec2<f32> }};\n@vertex fn vs_main(@builtin(vertex_index) vid: u32) -> VSOut {{\n  var pos = array<vec2<f32>,3>(vec2<f32>(-1.0,-3.0), vec2<f32>(3.0,1.0), vec2<f32>(-1.0,1.0));\n  var o: VSOut; o.pos = vec4<f32>(pos[vid], 0.0, 1.0); o.uv = (pos[vid]+vec2<f32>(1.0,1.0))*0.5; return o; }}\n@fragment fn fs_main(i: VSOut) -> @location(0) vec4<f32> {{ let m = eval_material(i.uv); return vec4<f32>(m.base, 1.0); }}\n",
            decls, pkg.wgsl
        );
        self.device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("material composed shader"),
                source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Owned(full)),
            })
    }
    pub async fn new(window: std::sync::Arc<winit::window::Window>) -> Result<Self> {
        // WGPU init
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::default());
        let surface = instance.create_surface(window.clone())?;
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .ok_or_else(|| anyhow::anyhow!("No adapter"))?;

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("device"),
                    required_features: {
                        let f = wgpu::Features::empty();
                        #[cfg(feature = "gpu-tests")]
                        {
                            wgpu::Features::TIMESTAMP_QUERY
                        }
                        #[cfg(not(feature = "gpu-tests"))]
                        {
                            f
                        }
                    },
                    required_limits: wgpu::Limits::default(),
                },
                None,
            )
            .await?;

        // Surface config
        let caps = surface.get_capabilities(&adapter);
        let format = caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(caps.formats[0]);
        let size = window.inner_size();
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: caps.present_modes[0],
            alpha_mode: caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        #[cfg(feature = "gpu-tests")]
        let timestamp_query_set = device.create_query_set(&wgpu::QuerySetDescriptor {
            label: Some("timestamps"),
            ty: wgpu::QueryType::Timestamp,
            count: 2,
        });
        #[cfg(feature = "gpu-tests")]
        let timestamp_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("ts readback"),
            size: 16,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        // Depth
        let depth = crate::depth::Depth::create(&device, &config);

        // Shaders / pipeline
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("basic shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(SHADER_SRC)),
        });

        let camera_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("camera ubo"),
            size: std::mem::size_of::<CameraUBO>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("camera bgl"),
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

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("camera bg"),
            layout: &bind_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buf.as_entire_binding(),
            }],
        });

        // Material buffer
        let material_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("material bgl"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let material_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("material ubo"),
            size: 32, // vec4 + 2 f32 + padding
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        // Seed the material buffer with a bright-ish default so geometry renders visibly out of the box.
        let default_material: [f32; 8] = [0.85, 0.78, 0.72, 1.0, 0.05, 0.6, 0.0, 0.0];
        queue.write_buffer(&material_buf, 0, bytemuck::cast_slice(&default_material));
        let material_bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("material bg"),
            layout: &material_bgl,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: material_buf.as_entire_binding(),
            }],
        });

        // HDR color target
        let hdr_tex = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("hdr tex"),
            size: wgpu::Extent3d {
                width: config.width,
                height: config.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba16Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let hdr_view = hdr_tex.create_view(&wgpu::TextureViewDescriptor::default());
        let hdr_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("hdr sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        #[cfg(feature = "postfx")]
        let hdr_aux = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("hdr aux tex"),
            size: wgpu::Extent3d {
                width: config.width,
                height: config.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba16Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        #[cfg(feature = "postfx")]
        let hdr_aux_view = hdr_aux.create_view(&wgpu::TextureViewDescriptor::default());
        #[cfg(feature = "postfx")]
        let fx_gi = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("fx gi tex"),
            size: wgpu::Extent3d {
                width: config.width,
                height: config.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba16Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        #[cfg(feature = "postfx")]
        let fx_gi_view = fx_gi.create_view(&wgpu::TextureViewDescriptor::default());
        #[cfg(feature = "postfx")]
        let fx_ao = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("fx ao tex"),
            size: wgpu::Extent3d {
                width: config.width,
                height: config.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba16Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        #[cfg(feature = "postfx")]
        let fx_ao_view = fx_ao.create_view(&wgpu::TextureViewDescriptor::default());

        // Postprocess pipeline
        let post_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("post shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(POST_SHADER)),
        });
        let post_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("post bgl"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });
        let post_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("post bg"),
            layout: &post_bgl,
            entries: &[
                #[cfg(not(feature = "postfx"))]
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&hdr_view),
                },
                #[cfg(feature = "postfx")]
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&hdr_aux_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&hdr_sampler),
                },
            ],
        });

        // Feature-gated SSR pass (passthrough using color + depth)
        #[cfg(feature = "postfx")]
        let ssr_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("ssr shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(WGSL_SSR)),
        });
        #[cfg(feature = "postfx")]
        let ssr_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("ssr bgl"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    // color_tex
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    // depth_tex
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Depth,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    // sampler
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });
        #[cfg(feature = "postfx")]
        let ssr_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("ssr bg"),
            layout: &ssr_bgl,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&hdr_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&depth.view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&hdr_sampler),
                },
            ],
        });
        #[cfg(feature = "postfx")]
        let ssr_pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("ssr layout"),
            bind_group_layouts: &[&ssr_bgl],
            push_constant_ranges: &[],
        });
        #[cfg(feature = "postfx")]
        let ssr_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("ssr pipeline"),
            layout: Some(&ssr_pl),
            vertex: wgpu::VertexState {
                module: &ssr_shader,
                entry_point: "vs_main",
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &ssr_shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Rgba16Float,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        // SSAO
        #[cfg(feature = "postfx")]
        let ssao_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("ssao shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(WGSL_SSAO)),
        });
        #[cfg(feature = "postfx")]
        let ssao_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("ssao bgl"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Depth,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });
        #[cfg(feature = "postfx")]
        let ssao_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("ssao bg"),
            layout: &ssao_bgl,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&depth.view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&hdr_sampler),
                },
            ],
        });
        #[cfg(feature = "postfx")]
        let ssao_pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("ssao layout"),
            bind_group_layouts: &[&ssao_bgl],
            push_constant_ranges: &[],
        });
        #[cfg(feature = "postfx")]
        let ssao_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("ssao pipeline"),
            layout: Some(&ssao_pl),
            vertex: wgpu::VertexState {
                module: &ssao_shader,
                entry_point: "vs_main",
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &ssao_shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Rgba16Float,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        // SSGI
        #[cfg(feature = "postfx")]
        let ssgi_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("ssgi shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(WGSL_SSGI)),
        });
        #[cfg(feature = "postfx")]
        let ssgi_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("ssgi bgl"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Depth,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });
        #[cfg(feature = "postfx")]
        let ssgi_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("ssgi bg"),
            layout: &ssgi_bgl,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&normal_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&depth.view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&hdr_sampler),
                },
            ],
        });
        #[cfg(feature = "postfx")]
        let ssgi_pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("ssgi layout"),
            bind_group_layouts: &[&ssgi_bgl],
            push_constant_ranges: &[],
        });
        #[cfg(feature = "postfx")]
        let ssgi_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("ssgi pipeline"),
            layout: Some(&ssgi_pl),
            vertex: wgpu::VertexState {
                module: &ssgi_shader,
                entry_point: "vs_main",
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &ssgi_shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Rgba16Float,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        // Post-fx composition pipeline
        #[cfg(feature = "postfx")]
        let post_fx_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("post fx shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(POST_SHADER_FX)),
        });
        #[cfg(feature = "postfx")]
        let post_fx_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("post fx bgl"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });
        #[cfg(feature = "postfx")]
        let post_fx_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("post fx bg"),
            layout: &post_fx_bgl,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&hdr_aux_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&fx_ao_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&fx_gi_view),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Sampler(&hdr_sampler),
                },
            ],
        });
        #[cfg(feature = "postfx")]
        let post_fx_pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("post fx layout"),
            bind_group_layouts: &[&post_fx_bgl],
            push_constant_ranges: &[],
        });
        #[cfg(feature = "postfx")]
        let post_fx_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("post fx pipeline"),
            layout: Some(&post_fx_pl),
            vertex: wgpu::VertexState {
                module: &post_fx_shader,
                entry_point: "vs_main",
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &post_fx_shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        // Shadow bind group layout (declared early so we can include it in main pipeline layout)
        let shadow_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("shadow bgl"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Depth,
                        view_dimension: wgpu::TextureViewDimension::D2Array,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Comparison),
                    count: None,
                },
            ],
        });

        // Combined textures + skin bind group layout (group 3): albedo, mr, normal textures + samplers, plus optional skin storage buffer
        // bindings: 0: albedo tex, 1: albedo samp, 2: mr tex, 3: mr samp, 4: normal tex, 5: normal samp, 6: skin palette (storage)
        let tex_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("combined tex+skin bgl"),
            entries: &[
                // albedo texture + sampler
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                // metallic-roughness texture + sampler
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                // normal texture + sampler
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 5,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                // skin palette (storage) - vertex-stage visibility but kept in same group to reduce group count
                wgpu::BindGroupLayoutEntry {
                    binding: 6,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        // extra_tex_bgl is no longer needed; MR and normal are merged into tex_bgl

        // Clustered lighting bind group layout (fragment reads)
        let clustered_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("clustered bgl (frag)"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 4,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("pipeline layout"),
            // Group indices: 0: camera, 1: material, 2: shadow/light, 3: combined textures + skin
            bind_group_layouts: &[&bind_layout, &material_bgl, &shadow_bgl, &tex_bgl],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs",
                buffers: &[
                    crate::types::Vertex::layout(),
                    crate::types::InstanceRaw::layout(),
                ],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs",
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Rgba16Float,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: depth.format,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });
        // after instance_buf creation
        let weather = crate::effects::WeatherFx::new(&device, 800);

        // Sky/environment
        let mut sky = crate::environment::SkyRenderer::new(Default::default());
        sky.init_gpu_resources(&device, wgpu::TextureFormat::Rgba16Float)?;

        // Post pipeline uses surface format
        let post_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("post layout"),
            bind_group_layouts: &[&post_bgl],
            push_constant_ranges: &[],
        });
        let post_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("post pipeline"),
            layout: Some(&post_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &post_shader,
                entry_point: "vs_main",
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &post_shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        // Shadow map resources (2-layer array for CSM)
        let shadow_size: u32 = 1024;
        let shadow_tex = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("shadow map"),
            size: wgpu::Extent3d {
                width: shadow_size,
                height: shadow_size,
                depth_or_array_layers: 2,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        // Array view for sampling
        let shadow_view = shadow_tex.create_view(&wgpu::TextureViewDescriptor {
            label: Some("shadow array view"),
            format: Some(wgpu::TextureFormat::Depth32Float),
            dimension: Some(wgpu::TextureViewDimension::D2Array),
            aspect: wgpu::TextureAspect::All,
            base_mip_level: 0,
            mip_level_count: None,
            base_array_layer: 0,
            array_layer_count: None,
        });
        // Per-layer views for rendering
        let shadow_layer0_view = shadow_tex.create_view(&wgpu::TextureViewDescriptor {
            label: Some("shadow layer0 view"),
            format: Some(wgpu::TextureFormat::Depth32Float),
            dimension: Some(wgpu::TextureViewDimension::D2),
            aspect: wgpu::TextureAspect::All,
            base_mip_level: 0,
            mip_level_count: None,
            base_array_layer: 0,
            array_layer_count: Some(1),
        });
        let shadow_layer1_view = shadow_tex.create_view(&wgpu::TextureViewDescriptor {
            label: Some("shadow layer1 view"),
            format: Some(wgpu::TextureFormat::Depth32Float),
            dimension: Some(wgpu::TextureViewDimension::D2),
            aspect: wgpu::TextureAspect::All,
            base_mip_level: 0,
            mip_level_count: None,
            base_array_layer: 1,
            array_layer_count: Some(1),
        });
        let shadow_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("shadow sampler"),
            compare: Some(wgpu::CompareFunction::LessEqual),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        // shadow_bgl already created above
        let light_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("light ubo"),
            // 2 mat4 (128 bytes) + vec2 splits + pad (16 bytes) => 144; round to 160
            size: 160,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let light_bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("light bg"),
            layout: &shadow_bgl,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: light_buf.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&shadow_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&shadow_sampler),
                },
            ],
        });

        // Minimal layout for shadow-only pass: only the light uniform buffer (binding 0).
        let shadow_bgl_light = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("shadow bgl light-only"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });
        let light_bg_shadow = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("light bg shadow-only"),
            layout: &shadow_bgl_light,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: light_buf.as_entire_binding(),
            }],
        });

        // Shadow map pipeline (depth-only)
        let shadow_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("shadow shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(
                r#"
struct VSIn {
  @location(0) position: vec3<f32>,
  @location(1) normal:   vec3<f32>,
    @location(12) tangent:  vec4<f32>,
    @location(13) uv:       vec2<f32>,
  @location(2) m0: vec4<f32>,
  @location(3) m1: vec4<f32>,
  @location(4) m2: vec4<f32>,
  @location(5) m3: vec4<f32>,
};
struct VSOut { @builtin(position) pos: vec4<f32> };
struct Light { view_proj: mat4x4<f32> };
@group(0) @binding(0) var<uniform> uLight: Light;
@vertex
fn vs(input: VSIn) -> VSOut {
  let model = mat4x4<f32>(input.m0, input.m1, input.m2, input.m3);
  var out: VSOut;
  out.pos = uLight.view_proj * (model * vec4<f32>(input.position, 1.0));
  return out;
}
@fragment fn fs() { }
"#,
            )),
        });
        // Shadow-only pipeline uses a light-only bind group layout so the
        // depth-only pass doesn't require bindings for the shadow texture/sampler.
        let shadow_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("shadow layout"),
                bind_group_layouts: &[&shadow_bgl_light],
                push_constant_ranges: &[],
            });
        let shadow_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("shadow pipeline"),
            layout: Some(&shadow_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shadow_shader,
                entry_point: "vs",
                buffers: &[
                    crate::types::Vertex::layout(),
                    crate::types::InstanceRaw::layout(),
                ],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: None,
            primitive: wgpu::PrimitiveState {
                cull_mode: Some(wgpu::Face::Back),
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::LessEqual,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState {
                    constant: 2,
                    slope_scale: 2.0,
                    clamp: 0.0,
                },
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        // Default 1x1 white albedo
        let albedo_tex = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("albedo tex"),
            size: wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let albedo_view = albedo_tex.create_view(&wgpu::TextureViewDescriptor::default());
        let albedo_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("albedo sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });
        // Initialize albedo with a 1x1 white texel so sampling yields visible color
        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &albedo_tex,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &[255u8, 255u8, 255u8, 255u8],
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4),
                rows_per_image: Some(1),
            },
            wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
        );
        // Skin palette storage buffer (max 64 bones) - create before bind group so it can be referenced
        let skin_palette_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("skin palette"),
            size: (64 * 64) as u64, // 64 mat4 (16 floats * 4 bytes) = 1024 bytes; allocate 4096 (rounded)
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Default extra textures (create MR and normal before building combined bind group)
        let mr_tex = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("mr tex"),
            size: wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let mr_view = mr_tex.create_view(&wgpu::TextureViewDescriptor::default());
        let mr_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("mr samp"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
            ..Default::default()
        });
        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &mr_tex,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &[0u8, 255u8, 0u8, 255u8],
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4),
                rows_per_image: Some(1),
            },
            wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
        );
        let normal_tex = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("normal tex"),
            size: wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let normal_view = normal_tex.create_view(&wgpu::TextureViewDescriptor::default());
        let normal_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("normal samp"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
            ..Default::default()
        });
        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &normal_tex,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &[128u8, 128u8, 255u8, 255u8],
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4),
                rows_per_image: Some(1),
            },
            wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
        );

        // Combined bind group for albedo, mr, normal, and skin palette (bindings 0..6)
        let tex_bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("combined tex bg"),
            layout: &tex_bgl,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&albedo_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&albedo_sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&mr_view),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Sampler(&mr_sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::TextureView(&normal_view),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: wgpu::BindingResource::Sampler(&normal_sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 6,
                    resource: skin_palette_buf.as_entire_binding(),
                },
            ],
        });

        // Skin palette storage buffer (max 64 bones)
        let skin_palette_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("skin palette"),
            size: (64 * 64) as u64, // 64 mat4 (16 floats * 4 bytes) = 1024 bytes; allocate 4096 (rounded)
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let skin_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("skin bgl"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });
        let skin_bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("skin bg"),
            layout: &skin_bgl,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: skin_palette_buf.as_entire_binding(),
            }],
        });

        // Skinned pipeline (skin storage is now in combined tex_bgl at binding 6)
        let skinned_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("skinned pipeline layout"),
                bind_group_layouts: &[&bind_layout, &material_bgl, &shadow_bgl, &tex_bgl],
                push_constant_ranges: &[],
            });
        let skinned_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor{
            label: Some("skinned shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(r#"
struct VSIn {
  @location(0) position: vec3<f32>,
  @location(1) normal:   vec3<f32>,
    @location(12) tangent:  vec4<f32>,
  @location(10) joints:  vec4<u32>,
  @location(11) weights: vec4<f32>,
  @location(2) m0: vec4<f32>,
  @location(3) m1: vec4<f32>,
  @location(4) m2: vec4<f32>,
  @location(5) m3: vec4<f32>,
  @location(6) n0: vec3<f32>,
  @location(7) n1: vec3<f32>,
  @location(8) n2: vec3<f32>,
  @location(9) color: vec4<f32>,
};

struct VSOut {
  @builtin(position) pos: vec4<f32>,
  @location(0) world_pos: vec3<f32>,
  @location(1) normal: vec3<f32>,
    @location(3) tbn0: vec3<f32>,
    @location(4) tbn1: vec3<f32>,
    @location(5) tbn2: vec3<f32>,
  @location(2) color: vec4<f32>,
};

struct Camera { view_proj: mat4x4<f32>, light_dir: vec3<f32>, _pad: f32 };
@group(0) @binding(0) var<uniform> uCamera: Camera;

struct MaterialUbo { base_color: vec4<f32>, metallic: f32, roughness: f32, _pad: vec2<f32> };
@group(1) @binding(0) var<uniform> uMaterial: MaterialUbo;

struct MainLightUbo { view_proj0: mat4x4<f32>, view_proj1: mat4x4<f32>, splits: vec2<f32>, extras: vec2<f32> };
@group(2) @binding(0) var<uniform> uLight: MainLightUbo;
@group(2) @binding(1) var shadow_tex: texture_depth_2d_array;
@group(2) @binding(2) var shadow_sampler: sampler_comparison;

@group(3) @binding(0) var albedo_tex: texture_2d<f32>;
@group(3) @binding(1) var albedo_samp: sampler;
@group(3) @binding(2) var mr_tex: texture_2d<f32>;
@group(3) @binding(3) var mr_samp: sampler;
@group(3) @binding(4) var normal_tex: texture_2d<f32>;
@group(3) @binding(5) var normal_samp: sampler;
struct Skinning { mats: array<mat4x4<f32>> };
@group(3) @binding(6) var<storage, read> skin: Skinning;

@vertex
fn vs(input: VSIn) -> VSOut {
  // Build instance model matrix
  let model_inst = mat4x4<f32>(input.m0, input.m1, input.m2, input.m3);
  // Skinning transform
  let j = input.joints;
  let w = input.weights;
  let m0 = skin.mats[u32(j.x)];
  let m1 = skin.mats[u32(j.y)];
  let m2 = skin.mats[u32(j.z)];
  let m3 = skin.mats[u32(j.w)];
  let pos4 = vec4<f32>(input.position, 1.0);
    let nrm4 = vec4<f32>(input.normal, 0.0);
  let skinned_pos = (m0 * pos4) * w.x + (m1 * pos4) * w.y + (m2 * pos4) * w.z + (m3 * pos4) * w.w;
  let skinned_nrm = (m0 * nrm4) * w.x + (m1 * nrm4) * w.y + (m2 * nrm4) * w.z + (m3 * nrm4) * w.w;
    let tan4 = vec4<f32>(input.tangent.xyz, 0.0);
    let skinned_tan = (m0 * tan4) * w.x + (m1 * tan4) * w.y + (m2 * tan4) * w.z + (m3 * tan4) * w.w;
  let world = model_inst * skinned_pos;
  var out: VSOut;
  out.pos = uCamera.view_proj * world;
    let Nw = normalize((model_inst * skinned_nrm).xyz);
    let Tw = normalize((model_inst * skinned_tan).xyz);
    let Bw = normalize(cross(Nw, Tw)) * input.tangent.w;
    out.normal = Nw;
  out.world_pos = world.xyz;
    out.tbn0 = Tw;
    out.tbn1 = Bw;
    out.tbn2 = Nw;
  out.color = input.color;
  return out;
}

// Reuse the same fragment code as the static pipeline
fn fresnel_schlick(cos_theta: f32, F0: vec3<f32>) -> vec3<f32> {
    return F0 + (vec3<f32>(1.0,1.0,1.0) - F0) * pow(1.0 - cos_theta, 5.0);
}
fn distribution_ggx(N: vec3<f32>, H: vec3<f32>, roughness: f32) -> f32 {
    let a = roughness * roughness;
    let a2 = a * a;
    let NdotH = max(dot(N, H), 0.0);
    let NdotH2 = NdotH * NdotH;
    let denom = (NdotH2 * (a2 - 1.0) + 1.0);
    return a2 / (3.14159 * denom * denom + 1e-5);
}
fn geometry_smith(N: vec3<f32>, V: vec3<f32>, L: vec3<f32>, roughness: f32) -> f32 {
    let r = (roughness + 1.0);
    let k = (r * r) / 8.0;
    let NdotV = max(dot(N, V), 0.0);
    let NdotL = max(dot(N, L), 0.0);
    let ggx1 = NdotV / (NdotV * (1.0 - k) + k + 1e-5);
    let ggx2 = NdotL / (NdotL * (1.0 - k) + k + 1e-5);
    return ggx1 * ggx2;
}
@fragment
fn fs(input: VSOut) -> @location(0) vec4<f32> {
    let V = normalize(-input.world_pos);
    let L = normalize(-uCamera.light_dir);
    let H = normalize(V + L);
    var N = normalize(input.normal);
    // Normal mapping disabled in skinned path for now; use vertex normal transformed to world.
    let NdotL = max(dot(N, L), 0.0);
    var base_color = (uMaterial.base_color.rgb * input.color.rgb);
    var metallic = clamp(uMaterial.metallic, 0.0, 1.0);
    var roughness = clamp(uMaterial.roughness, 0.04, 1.0);
    let F0 = mix(vec3<f32>(0.04, 0.04, 0.04), base_color, metallic);
    let F = fresnel_schlick(max(dot(H, V), 0.0), F0);
    let D = distribution_ggx(N, H, roughness);
    let G = geometry_smith(N, V, L, roughness);
    let numerator = D * G * F;
    let denom = 4.0 * max(dot(N, V), 0.0) * NdotL + 1e-5;
    let specular = numerator / denom;
    let kd = (vec3<f32>(1.0,1.0,1.0) - F) * (1.0 - metallic);
    let diffuse = kd * base_color / 3.14159;
    let radiance = vec3<f32>(1.0, 0.98, 0.9);
    // Cascaded shadow sampling (same as static path)
    let dist = length(input.world_pos);
    let use_c0 = dist < uLight.splits.x;
    var lvp: mat4x4<f32>;
    if (use_c0) { lvp = uLight.view_proj0; } else { lvp = uLight.view_proj1; }
    let lp = lvp * vec4<f32>(input.world_pos, 1.0);
    let ndc = lp.xyz / lp.w;
    let uv = ndc.xy * 0.5 + vec2<f32>(0.5, 0.5);
    let depth = ndc.z;
    let slope = max(0.0, 1.0 - dot(N, L));
    let base_bias = uLight.extras.y;
    let bias = max(base_bias /* + slope_scale * slope */ , 0.00001);
    var shadow: f32 = 1.0;
    if (uv.x >= 0.0 && uv.x <= 1.0 && uv.y >= 0.0 && uv.y <= 1.0) {
        let layer = i32(select(1, 0, use_c0));
        let dims = vec2<f32>(textureDimensions(shadow_tex).xy);
        let texel = 1.0 / dims;
        var sum = 0.0;
        for (var dx: i32 = -1; dx <= 1; dx = dx + 1) {
            for (var dy: i32 = -1; dy <= 1; dy = dy + 1) {
                let o = vec2<f32>(f32(dx), f32(dy)) * texel * max(0.0, uLight.extras.x);
                sum = sum + textureSampleCompare(shadow_tex, shadow_sampler, uv + o, layer, depth - bias);
            }
        }
        shadow = sum / 9.0;
    }
    // Match ambient lift with static pipeline
    let lit_color = (diffuse + specular) * radiance * NdotL * shadow + base_color * 0.08;
    return vec4<f32>(lit_color, uMaterial.base_color.a * input.color.a);
}
"#;

const POST_SHADER: &str = r#"
struct VSOut { @builtin(position) pos: vec4<f32>, @location(0) uv: vec2<f32> };
@vertex
fn vs_main(@builtin(vertex_index) vid: u32) -> VSOut {
    var pos = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -3.0),
        vec2<f32>( 3.0,  1.0),
        vec2<f32>(-1.0,  1.0)
    );
    var out: VSOut;
    out.pos = vec4<f32>(pos[vid], 0.0, 1.0);
    out.uv = (pos[vid] + vec2<f32>(1.0,1.0)) * 0.5;
    return out;
}

@group(0) @binding(0) var hdr_tex: texture_2d<f32>;
@group(0) @binding(1) var samp: sampler;

fn aces_tonemap(x: vec3<f32>) -> vec3<f32> {
    let a = 2.51;
    let b = 0.03;
    let c = 2.43;
    let d = 0.59;
    let e = 0.14;
    return clamp((x*(a*x+b))/(x*(c*x+d)+e), vec3<f32>(0.0), vec3<f32>(1.0));
}

@fragment
fn fs_main(in: VSOut) -> @location(0) vec4<f32> {
    let hdr = textureSampleLevel(hdr_tex, samp, in.uv, 0.0);
    let mapped = aces_tonemap(vec3<f32>(hdr.r, hdr.g, hdr.b));
    return vec4<f32>(mapped, 1.0);
}
"#;

#[cfg(feature = "postfx")]
const POST_SHADER_FX: &str = r#"
struct VSOut { @builtin(position) pos: vec4<f32>, @location(0) uv: vec2<f32> };
@vertex
fn vs_main(@builtin(vertex_index) vid: u32) -> VSOut {
    var pos = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -3.0),
        vec2<f32>( 3.0,  1.0),
        vec2<f32>(-1.0,  1.0)
    );
    var out: VSOut;
    out.pos = vec4<f32>(pos[vid], 0.0, 1.0);
    out.uv = (pos[vid] + vec2<f32>(1.0,1.0)) * 0.5;
    return out;
}

@group(0) @binding(0) var hdr_tex: texture_2d<f32>;
@group(0) @binding(1) var ao_tex: texture_2d<f32>;
@group(0) @binding(2) var gi_tex: texture_2d<f32>;
@group(0) @binding(3) var samp: sampler;

fn aces_tonemap(x: vec3<f32>) -> vec3<f32> {
    let a = 2.51; let b = 0.03; let c = 2.43; let d = 0.59; let e = 0.14;
    return clamp((x*(a*x+b))/(x*(c*x+d)+e), vec3<f32>(0.0), vec3<f32>(1.0));
}

@fragment
fn fs_main(in: VSOut) -> @location(0) vec4<f32> {
    let hdr = textureSampleLevel(hdr_tex, samp, in.uv, 0.0).rgb;
    let ao = textureSampleLevel(ao_tex, samp, in.uv, 0.0).r;
    let gi = textureSampleLevel(gi_tex, samp, in.uv, 0.0).rgb;
    let ao_strength = 0.6;
    let gi_strength = 0.2;
    let comp = hdr * (1.0 - ao * ao_strength) + gi * gi_strength;
    let mapped = aces_tonemap(comp);
    return vec4<f32>(mapped, 1.0);
}
"#;

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct CameraUBO {
    view_proj: [[f32; 4]; 4],
    light_dir_pad: [f32; 4],
}

pub struct Renderer {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    depth: Depth,

    #[allow(dead_code)]
    shader: wgpu::ShaderModule,
    pipeline: wgpu::RenderPipeline,
    material_buf: wgpu::Buffer,
    material_bg: wgpu::BindGroup,
    post_pipeline: wgpu::RenderPipeline,
    post_bind_group: wgpu::BindGroup,
    #[allow(dead_code)]
    post_bgl: wgpu::BindGroupLayout,
    hdr_tex: wgpu::Texture,
    hdr_view: wgpu::TextureView,
    hdr_sampler: wgpu::Sampler,
    #[allow(dead_code)]
    shadow_tex: wgpu::Texture,
    #[allow(dead_code)]
    shadow_view: wgpu::TextureView, // array view for sampling
    shadow_layer0_view: wgpu::TextureView,
    shadow_layer1_view: wgpu::TextureView,
    #[allow(dead_code)]
    shadow_sampler: wgpu::Sampler,
    shadow_pipeline: wgpu::RenderPipeline,
    light_buf: wgpu::Buffer,
    light_bg: wgpu::BindGroup,
    // Bind group used only during shadow depth passes (binds light buffer only) to avoid sampling the
    // shadow depth texture while it's being written.
    light_bg_shadow: wgpu::BindGroup,
    #[allow(dead_code)]
    shadow_bgl: wgpu::BindGroupLayout,
    // Cascade data cached on CPU for shadow passes
    cascade0: glam::Mat4,
    cascade1: glam::Mat4,
    split0: f32,
    split1: f32,
    // Tunable cascade ortho extents (half-width/height)
    cascade0_extent: f32,
    cascade1_extent: f32,
    // CSM tuning
    cascade_lambda: f32, // split distribution (0..1)
    shadow_pcf_radius_px: f32,
    shadow_depth_bias: f32,
    shadow_slope_scale: f32,

    // Albedo (base color) texture and sampler
    albedo_tex: wgpu::Texture,
    albedo_view: wgpu::TextureView,
    albedo_sampler: wgpu::Sampler,
    tex_bgl: wgpu::BindGroupLayout,
    tex_bg: wgpu::BindGroup,
    // Metallic-Roughness texture and sampler
    mr_tex: wgpu::Texture,
    mr_view: wgpu::TextureView,
    mr_sampler: wgpu::Sampler,
    // Normal map texture and sampler
    normal_tex: wgpu::Texture,
    normal_view: wgpu::TextureView,
    normal_sampler: wgpu::Sampler,
    // Extra textures bind group layout and group (for future extensibility)
    // extra texture bind group layout/bg removed; combined tex_bgl/tex_bg used
    camera_ubo: CameraUBO,
    camera_buf: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,

    #[allow(dead_code)]
    mesh_cube: Mesh,
    mesh_sphere: Mesh,
    mesh_plane: Mesh,
    mesh_external: Option<Mesh>,

    instances: Vec<Instance>,
    instance_buf: wgpu::Buffer,

    #[allow(dead_code)]
    overlay: crate::overlay::OverlayFx,
    pub overlay_params: crate::overlay::OverlayParams,
    pub weather: crate::effects::WeatherFx,
    // Environment & sky
    sky: crate::environment::SkyRenderer,

    // Skinning (v0)
    #[allow(dead_code)]
    skin_bgl: wgpu::BindGroupLayout,
    #[allow(dead_code)]
    skin_bg: wgpu::BindGroup,
    skin_palette_buf: wgpu::Buffer,
    skinned_pipeline: wgpu::RenderPipeline,
    skinned_mesh: Option<(wgpu::Buffer, wgpu::Buffer, u32)>, // (vbuf, ibuf, index_count)

    // Clustered lighting resources
    clustered_dims: ClusterDims,
    clustered_params_buf: wgpu::Buffer,
    clustered_lights_buf: wgpu::Buffer,
    clustered_offsets_buf: wgpu::Buffer,
    clustered_counts_buf: wgpu::Buffer,
    #[allow(dead_code)]
    clustered_indices_buf: wgpu::Buffer,
    #[allow(dead_code)]
    clustered_bgl: wgpu::BindGroupLayout,
    #[allow(dead_code)]
    clustered_bg: wgpu::BindGroup,
    #[allow(dead_code)]
    clustered_comp_bgl: wgpu::BindGroupLayout,
    clustered_comp_bg: wgpu::BindGroup,
    clustered_comp_pipeline: wgpu::ComputePipeline,
    point_lights: Vec<CpuLight>,
    #[cfg(feature = "gpu-tests")]
    timestamp_query_set: wgpu::QuerySet,
    #[cfg(feature = "gpu-tests")]
    timestamp_buf: wgpu::Buffer,

    // Cinematics integration
    cin_tl: Option<awc::Timeline>,
    cin_seq: awc::Sequencer,
    cin_playing: bool,
}

impl Renderer {
    /// Compose a standalone fragment shader from a `MaterialPackage` for validation/pipeline creation.
    /// Returns a `ShaderModule` ready to be used in a pipeline (caller wires layouts/bindings).
    pub fn shader_from_material_package(&self, pkg: &MaterialPackage) -> wgpu::ShaderModule {
        // Declare group(0) bindings based on `bindings` ids collected by the compiler (tex/sampler pairs)
        let mut decls = String::new();
        let mut idx = 0u32;
        for id in pkg.bindings.iter() {
            decls.push_str(&format!(
                "@group(0) @binding({}) var tex_{}: texture_2d<f32>;\n",
                idx, id
            ));
            idx += 1;
            decls.push_str(&format!(
                "@group(0) @binding({}) var samp_{}: sampler;\n",
                idx, id
            ));
            idx += 1;
        }
        // Compose WGSL: eval_material + a tiny VS/FS pair.
        let full = format!(
            "{}\n{}\nstruct VSOut {{ @builtin(position) pos: vec4<f32>, @location(0) uv: vec2<f32> }};\n@vertex fn vs_main(@builtin(vertex_index) vid: u32) -> VSOut {{\n  var pos = array<vec2<f32>,3>(vec2<f32>(-1.0,-3.0), vec2<f32>(3.0,1.0), vec2<f32>(-1.0,1.0));\n  var o: VSOut; o.pos = vec4<f32>(pos[vid], 0.0, 1.0); o.uv = (pos[vid]+vec2<f32>(1.0,1.0))*0.5; return o; }}\n@fragment fn fs_main(i: VSOut) -> @location(0) vec4<f32> {{ let m = eval_material(i.uv); return vec4<f32>(m.base, 1.0); }}\n",
            decls, pkg.wgsl
        );
        self.device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("material composed shader"),
                source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Owned(full)),
            })
    }
    pub async fn new(window: std::sync::Arc<winit::window::Window>) -> Result<Self> {
        // WGPU init
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::default());
        let surface = instance.create_surface(window.clone())?;
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .ok_or_else(|| anyhow::anyhow!("No adapter"))?;

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("device"),
                    required_features: {
                        let f = wgpu::Features::empty();
                        #[cfg(feature = "gpu-tests")]
                        {
                            wgpu::Features::TIMESTAMP_QUERY
                        }
                        #[cfg(not(feature = "gpu-tests"))]
                        {
                            f
                        }
                    },
                    required_limits: wgpu::Limits::default(),
                },
                None,
            )
            .await?;

        // Surface config
        let caps = surface.get_capabilities(&adapter);
        let format = caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(caps.formats[0]);
        let size = window.inner_size();
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: caps.present_modes[0],
            alpha_mode: caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        #[cfg(feature = "gpu-tests")]
        let timestamp_query_set = device.create_query_set(&wgpu::QuerySetDescriptor {
            label: Some("timestamps"),
            ty: wgpu::QueryType::Timestamp,
            count: 2,
        });
        #[cfg(feature = "gpu-tests")]
        let timestamp_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("ts readback"),
            size: 16,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        // Depth
        let depth = crate::depth::Depth::create(&device, &config);

        // Shaders / pipeline
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("basic shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(SHADER_SRC)),
        });

        let camera_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("camera ubo"),
            size: std::mem::size_of::<CameraUBO>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("camera bgl"),
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

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("camera bg"),
            layout: &bind_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buf.as_entire_binding(),
            }],
        });

        // Material buffer
        let material_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("material bgl"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let material_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("material ubo"),
            size: 32, // vec4 + 2 f32 + padding
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        // Seed the material buffer with a bright-ish default so geometry renders visibly out of the box.
        let default_material: [f32; 8] = [0.85, 0.78, 0.72, 1.0, 0.05, 0.6, 0.0, 0.0];
        queue.write_buffer(&material_buf, 0, bytemuck::cast_slice(&default_material));
        let material_bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("material bg"),
            layout: &material_bgl,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: material_buf.as_entire_binding(),
            }],
        });

        // HDR color target
        let hdr_tex = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("hdr tex"),
            size: wgpu::Extent3d {
                width: config.width,
                height: config.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba16Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let hdr_view = hdr_tex.create_view(&wgpu::TextureViewDescriptor::default());
        let hdr_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("hdr sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        #[cfg(feature = "postfx")]
        let hdr_aux = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("hdr aux tex"),
            size: wgpu::Extent3d {
                width: config.width,
                height: config.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba16Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        #[cfg(feature = "postfx")]
        let hdr_aux_view = hdr_aux.create_view(&wgpu::TextureViewDescriptor::default());
        #[cfg(feature = "postfx")]
        let fx_gi = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("fx gi tex"),
            size: wgpu::Extent3d {
                width: config.width,
                height: config.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba16Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        #[cfg(feature = "postfx")]
        let fx_gi_view = fx_gi.create_view(&wgpu::TextureViewDescriptor::default());
        #[cfg(feature = "postfx")]
        let fx_ao = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("fx ao tex"),
            size: wgpu::Extent3d {
                width: config.width,
                height: config.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba16Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        #[cfg(feature = "postfx")]
        let fx_ao_view = fx_ao.create_view(&wgpu::TextureViewDescriptor::default());

        // Postprocess pipeline
        let post_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("post shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(POST_SHADER)),
        });
        let post_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("post bgl"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });
        let post_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("post bg"),
            layout: &post_bgl,
            entries: &[
                #[cfg(not(feature = "postfx"))]
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&hdr_view),
                },
                #[cfg(feature = "postfx")]
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&hdr_aux_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&hdr_sampler),
                },
            ],
        });

        // Feature-gated SSR pass (passthrough using color + depth)
        #[cfg(feature = "postfx")]
        let ssr_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("ssr shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(WGSL_SSR)),
        });
        #[cfg(feature = "postfx")]
        let ssr_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("ssr bgl"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    // color_tex
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    // depth_tex
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Depth,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    // sampler
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });
        #[cfg(feature = "postfx")]
        let ssr_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("ssr bg"),
            layout: &ssr_bgl,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&hdr_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&depth.view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&hdr_sampler),
                },
            ],
        });
        #[cfg(feature = "postfx")]
        let ssr_pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("ssr layout"),
            bind_group_layouts: &[&ssr_bgl],
            push_constant_ranges: &[],
        });
        #[cfg(feature = "postfx")]
        let ssr_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("ssr pipeline"),
            layout: Some(&ssr_pl),
            vertex: wgpu::VertexState {
                module: &ssr_shader,
                entry_point: "vs_main",
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &ssr_shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Rgba16Float,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        // SSAO
        #[cfg(feature = "postfx")]
        let ssao_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("ssao shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(WGSL_SSAO)),
        });
        #[cfg(feature = "postfx")]
        let ssao_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("ssao bgl"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Depth,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });
        #[cfg(feature = "postfx")]
        let ssao_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("ssao bg"),
            layout: &ssao_bgl,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&depth.view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&hdr_sampler),
                },
            ],
        });
        #[cfg(feature = "postfx")]
        let ssao_pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("ssao layout"),
            bind_group_layouts: &[&ssao_bgl],
            push_constant_ranges: &[],
        });
        #[cfg(feature = "postfx")]
        let ssao_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("ssao pipeline"),
            layout: Some(&ssao_pl),
            vertex: wgpu::VertexState {
                module: &ssao_shader,
                entry_point: "vs_main",
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &ssao_shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Rgba16Float,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        // SSGI
        #[cfg(feature = "postfx")]
        let ssgi_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("ssgi shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(WGSL_SSGI)),
        });
        #[cfg(feature = "postfx")]
        let ssgi_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("ssgi bgl"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Depth,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });
        #[cfg(feature = "postfx")]
        let ssgi_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("ssgi bg"),
            layout: &ssgi_bgl,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&normal_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&depth.view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&hdr_sampler),
                },
            ],
        });
        #[cfg(feature = "postfx")]
        let ssgi_pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("ssgi layout"),
            bind_group_layouts: &[&ssgi_bgl],
            push_constant_ranges: &[],
        });
        #[cfg(feature = "postfx")]
        let ssgi_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("ssgi pipeline"),
            layout: Some(&ssgi_pl),
            vertex: wgpu::VertexState {
                module: &ssgi_shader,
                entry_point: "vs_main",
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &ssgi_shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Rgba16Float,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        // Post-fx composition pipeline
        #[cfg(feature = "postfx")]
        let post_fx_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("post fx shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(POST_SHADER_FX)),
        });
        #[cfg(feature = "postfx")]
        let post_fx_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("post fx bgl"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });
        #[cfg(feature = "postfx")]
        let post_fx_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("post fx bg"),
            layout: &post_fx_bgl,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&hdr_aux_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&fx_ao_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&fx_gi_view),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Sampler(&hdr_sampler),
                },
            ],
        });
        #[cfg(feature = "postfx")]
        let post_fx_pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("post fx layout"),
            bind_group_layouts: &[&post_fx_bgl],
            push_constant_ranges: &[],
        });
        #[cfg(feature = "postfx")]
        let post_fx_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("post fx pipeline"),
            layout: Some(&post_fx_pl),
            vertex: wgpu::VertexState {
                module: &post_fx_shader,
                entry_point: "vs_main",
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &post_fx_shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        // Shadow bind group layout (declared early so we can include it in main pipeline layout)
        let shadow_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("shadow bgl"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Depth,
                        view_dimension: wgpu::TextureViewDimension::D2Array,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Comparison),
                    count: None,
                },
            ],
        });

        // Combined textures + skin bind group layout (group 3): albedo, mr, normal textures + samplers, plus optional skin storage buffer
        // bindings: 0: albedo tex, 1: albedo samp, 2: mr tex, 3: mr samp, 4: normal tex, 5: normal samp, 6: skin palette (storage)
        let tex_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("combined tex+skin bgl"),
            entries: &[
                // albedo texture + sampler
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                // metallic-roughness texture + sampler
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                // normal texture + sampler
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 5,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                // skin palette (storage) - vertex-stage visibility but kept in same group to reduce group count
                wgpu::BindGroupLayoutEntry {
                    binding: 6,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        // extra_tex_bgl is no longer needed; MR and normal are merged into tex_bgl

        // Clustered lighting bind group layout (fragment reads)
        let clustered_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("clustered bgl (frag)"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 4,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("pipeline layout"),
            // Group indices: 0: camera, 1: material, 2: shadow/light, 3: combined textures + skin
            bind_group_layouts: &[&bind_layout, &material_bgl, &shadow_bgl, &tex_bgl],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs",
                buffers: &[
                    crate::types::Vertex::layout(),
                    crate::types::InstanceRaw::layout(),
                ],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs",
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Rgba16Float,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: depth.format,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });
        // after instance_buf creation
        let weather = crate::effects::WeatherFx::new(&device, 800);

        // Sky/environment
        let mut sky = crate::environment::SkyRenderer::new(Default::default());
        sky.init_gpu_resources(&device, wgpu::TextureFormat::Rgba16Float)?;

        // Post pipeline uses surface format
        let post_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("post layout"),
            bind_group_layouts: &[&post_bgl],
            push_constant_ranges: &[],
        });
        let post_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("post pipeline"),
            layout: Some(&post_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &post_shader,
                entry_point: "vs_main",
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &post_shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        // Shadow map resources (2-layer array for CSM)
        let shadow_size: u32 = 1024;
        let shadow_tex = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("shadow map"),
            size: wgpu::Extent3d {
                width: shadow_size,
                height: shadow_size,
                depth_or_array_layers: 2,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        // Array view for sampling
        let shadow_view = shadow_tex.create_view(&wgpu::TextureViewDescriptor {
            label: Some("shadow array view"),
            format: Some(wgpu::TextureFormat::Depth32Float),
            dimension: Some(wgpu::TextureViewDimension::D2Array),
            aspect: wgpu::TextureAspect::All,
            base_mip_level: 0,
            mip_level_count: None,
            base_array_layer: 0,
            array_layer_count: None,
        });
        // Per-layer views for rendering
        let shadow_layer0_view = shadow_tex.create_view(&wgpu::TextureViewDescriptor {
            label: Some("shadow layer0 view"),
            format: Some(wgpu::TextureFormat::Depth32Float),
            dimension: Some(wgpu::TextureViewDimension::D2),
            aspect: wgpu::TextureAspect::All,
            base_mip_level: 0,
            mip_level_count: None,
            base_array_layer: 0,
            array_layer_count: Some(1),
        });
        let shadow_layer1_view = shadow_tex.create_view(&wgpu::TextureViewDescriptor {
            label: Some("shadow layer1 view"),
            format: Some(wgpu::TextureFormat::Depth32Float),
            dimension: Some(wgpu::TextureViewDimension::D2),
            aspect: wgpu::TextureAspect::All,
            base_mip_level: 0,
            mip_level_count: None,
            base_array_layer: 1,
            array_layer_count: Some(1),
        });
        let shadow_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("shadow sampler"),
            compare: Some(wgpu::CompareFunction::LessEqual),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        // shadow_bgl already created above
        let light_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("light ubo"),
            // 2 mat4 (128 bytes) + vec2 splits + pad (16 bytes) => 144; round to 160
            size: 160,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let light_bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("light bg"),
            layout: &shadow_bgl,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: light_buf.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&shadow_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&shadow_sampler),
                },
            ],
        });

        // Minimal layout for shadow-only pass: only the light uniform buffer (binding 0).
        let shadow_bgl_light = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("shadow bgl light-only"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });
        let light_bg_shadow = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("light bg shadow-only"),
            layout: &shadow_bgl_light,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: light_buf.as_entire_binding(),
            }],
        });

        // Shadow map pipeline (depth-only)
        let shadow_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("shadow shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(
                r#"
struct VSIn {
  @location(0) position: vec3<f32>,
  @location(1) normal:   vec3<f32>,
    @location(12) tangent:  vec4<f32>,
    @location(13) uv:       vec2<f32>,
  @location(2) m0: vec4<f32>,
  @location(3) m1: vec4<f32>,
  @location(4) m2: vec4<f32>,
  @location(5) m3: vec4<f32>,
};
struct VSOut { @builtin(position) pos: vec4<f32> };
struct Light { view_proj: mat4x4<f32> };
@group(0) @binding(0) var<uniform> uLight: Light;
@vertex
fn vs(input: VSIn) -> VSOut {
  let model = mat4x4<f32>(input.m0, input.m1, input.m2, input.m3);
  var out: VSOut;
  out.pos = uLight.view_proj * (model * vec4<f32>(input.position, 1.0));
  return out;
}
@fragment fn fs() { }
"#,
            )),
        });
        // Shadow-only pipeline uses a light-only bind group layout so the
        // depth-only pass doesn't require bindings for the shadow texture/sampler.
        let shadow_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("shadow layout"),
                bind_group_layouts: &[&shadow_bgl_light],
                push_constant_ranges: &[],
            });
        let shadow_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("shadow pipeline"),
            layout: Some(&shadow_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shadow_shader,
                entry_point: "vs",
                buffers: &[
                    crate::types::Vertex::layout(),
                    crate::types::InstanceRaw::layout(),
                ],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: None,
            primitive: wgpu::PrimitiveState {
                cull_mode: Some(wgpu::Face::Back),
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::LessEqual,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState {
                    constant: 2,
                    slope_scale: 2.0,
                    clamp: 0.0,
                },
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        // Default 1x1 white albedo
        let albedo_tex = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("albedo tex"),
            size: wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let albedo_view = albedo_tex.create_view(&wgpu::TextureViewDescriptor::default());
        let albedo_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("albedo sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });
        // Initialize albedo with a 1x1 white texel so sampling yields visible color
        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &albedo_tex,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &[255u8, 255u8, 255u8, 255u8],
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4),
                rows_per_image: Some(1),
            },
            wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
        );
        // Skin palette storage buffer (max 64 bones) - create before bind group so it can be referenced
        let skin_palette_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("skin palette"),
            size: (64 * 64) as u64, // 64 mat4 (16 floats * 4 bytes) = 1024 bytes; allocate 4096 (rounded)
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Default extra textures (create MR and normal before building combined bind group)
        let mr_tex = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("mr tex"),
            size: wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let mr_view = mr_tex.create_view(&wgpu::TextureViewDescriptor::default());
        let mr_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("mr samp"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
            ..Default::default()
        });
        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &mr_tex,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &[0u8, 255u8, 0u8, 255u8],
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4),
                rows_per_image: Some(1),
            },
            wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
        );
        let normal_tex = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("normal tex"),
            size: wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let normal_view = normal_tex.create_view(&wgpu::TextureViewDescriptor::default());
        let normal_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("normal samp"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
            ..Default::default()
        });
        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &normal_tex,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &[128u8, 128u8, 255u8, 255u8],
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4),
                rows_per_image: Some(1),
            },
            wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
        );

        // Combined bind group for albedo, mr, normal, and skin palette (bindings 0..6)
        let tex_bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("combined tex bg"),
            layout: &tex_bgl,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&albedo_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&albedo_sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&mr_view),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Sampler(&mr_sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::TextureView(&normal_view),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: wgpu::BindingResource::Sampler(&normal_sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 6,
                    resource: skin_palette_buf.as_entire_binding(),
                },
            ],
        });

        // Skin palette storage buffer (max 64 bones)
        let skin_palette_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("skin palette"),
            size: (64 * 64) as u64, // 64 mat4 (16 floats * 4 bytes) = 1024 bytes; allocate 4096 (rounded)
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let skin_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("skin bgl"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });
        let skin_bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("skin bg"),
            layout: &skin_bgl,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: skin_palette_buf.as_entire_binding(),
            }],
        });

