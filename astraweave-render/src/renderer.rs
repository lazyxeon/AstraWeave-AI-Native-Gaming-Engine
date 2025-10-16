#[cfg(feature = "postfx")]
use crate::post::{
    BLOOM_COMPOSITE_WGSL, BLOOM_DOWNSAMPLE_WGSL, BLOOM_THRESHOLD_WGSL, BLOOM_UPSAMPLE_WGSL,
    WGSL_SSAO, WGSL_SSR,
};
use anyhow::Result;
use glam::Vec4Swizzles;
use glam::{vec3, Mat4};
use std::borrow::Cow;
use wgpu::util::DeviceExt;

#[cfg(feature = "profiling")]
use astraweave_profiling::{span, plot};

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
    // Post-FX auxiliary textures and pipeline
    #[cfg(feature = "postfx")]
    hdr_aux_tex: wgpu::Texture,
    #[cfg(feature = "postfx")]
    hdr_aux_view: wgpu::TextureView,
    #[cfg(feature = "postfx")]
    fx_gi_tex: wgpu::Texture,
    #[cfg(feature = "postfx")]
    fx_gi_view: wgpu::TextureView,
    #[cfg(feature = "postfx")]
    fx_ao_tex: wgpu::Texture,
    #[cfg(feature = "postfx")]
    fx_ao_view: wgpu::TextureView,
    #[cfg(feature = "postfx")]
    post_fx_bgl: wgpu::BindGroupLayout,
    #[cfg(feature = "postfx")]
    post_fx_bind_group: wgpu::BindGroup,
    #[cfg(feature = "postfx")]
    post_fx_pipeline: wgpu::RenderPipeline,
    // Bloom pipeline infrastructure (5-mip pyramid)
    #[cfg(feature = "postfx")]
    bloom_threshold_pipeline: wgpu::RenderPipeline,
    #[cfg(feature = "postfx")]
    bloom_downsample_pipeline: wgpu::RenderPipeline,
    #[cfg(feature = "postfx")]
    bloom_upsample_pipeline: wgpu::RenderPipeline,
    #[cfg(feature = "postfx")]
    bloom_composite_pipeline: wgpu::RenderPipeline,
    #[cfg(feature = "postfx")]
    bloom_mip_textures: Vec<wgpu::Texture>,
    #[cfg(feature = "postfx")]
    bloom_mip_views: Vec<wgpu::TextureView>,
    #[cfg(feature = "postfx")]
    bloom_threshold_buf: wgpu::Buffer,
    #[cfg(feature = "postfx")]
    bloom_intensity_buf: wgpu::Buffer,
    #[cfg(feature = "postfx")]
    bloom_bgl: wgpu::BindGroupLayout,
    #[cfg(feature = "postfx")]
    bloom_bind_groups_down: Vec<wgpu::BindGroup>,
    #[cfg(feature = "postfx")]
    bloom_bind_groups_up: Vec<wgpu::BindGroup>,
    #[cfg(feature = "postfx")]
    bloom_threshold_bg: wgpu::BindGroup,
    #[cfg(feature = "postfx")]
    bloom_composite_bg: wgpu::BindGroup,
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

    // Asset streaming and residency management
    _residency_manager: crate::residency::ResidencyManager,
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
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());
        let surface = instance.create_surface(window.clone())?;
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await?;

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("device"),
                required_features: {
                    let _f = wgpu::Features::empty();
                    #[cfg(feature = "gpu-tests")]
                    {
                        wgpu::Features::TIMESTAMP_QUERY
                    }
                    #[cfg(not(feature = "gpu-tests"))]
                    {
                        _f
                    }
                },
                required_limits: wgpu::Limits::default(),
                memory_hints: wgpu::MemoryHints::default(),
                trace: wgpu::Trace::Off,
            })
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
        let _ssr_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
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
        let _ssr_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("ssr pipeline"),
            layout: Some(&ssr_pl),
            vertex: wgpu::VertexState {
                module: &ssr_shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &ssr_shader,
                entry_point: Some("fs_main"),
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
            cache: None,
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
        let _ssao_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
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
        let _ssao_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("ssao pipeline"),
            layout: Some(&ssao_pl),
            vertex: wgpu::VertexState {
                module: &ssao_shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &ssao_shader,
                entry_point: Some("fs_main"),
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
            cache: None,
        });

        // SSGI - requires normal_view which isn't created yet at this point
        // TODO: Move this creation after normal_tex is created, or refactor postfx init
        /*
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
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &ssgi_shader,
                entry_point: Some("fs_main"),
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
            cache: None,
        });
        */

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
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &post_fx_shader,
                entry_point: Some("fs_main"),
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
            cache: None,
        });

        // ========================================
        // Bloom Pipeline Infrastructure (5-mip pyramid)
        // ========================================
        #[cfg(feature = "postfx")]
        let (
            bloom_threshold_pipeline,
            bloom_downsample_pipeline,
            bloom_upsample_pipeline,
            bloom_composite_pipeline,
            bloom_mip_textures,
            bloom_mip_views,
            bloom_threshold_buf,
            bloom_intensity_buf,
            bloom_bgl,
            bloom_bind_groups_down,
            bloom_bind_groups_up,
            bloom_threshold_bg,
            bloom_composite_bg,
        ) = {
            // Bloom configuration
            const MIP_COUNT: u32 = 5;
            let bloom_threshold = 1.0f32;
            let bloom_intensity = 0.1f32;

            // Create uniform buffers
            let bloom_threshold_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("bloom threshold buf"),
                contents: bytemuck::cast_slice(&[bloom_threshold, 0.0f32, 0.0f32, 0.0f32]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

            let bloom_intensity_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("bloom intensity buf"),
                contents: bytemuck::cast_slice(&[bloom_intensity, 0.0f32, 0.0f32, 0.0f32]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

            // Create mip chain textures (half resolution each level)
            let mut mip_textures = Vec::with_capacity(MIP_COUNT as usize);
            let mut mip_views = Vec::with_capacity(MIP_COUNT as usize);

            for i in 0..MIP_COUNT {
                let divisor = 1 << (i + 1); // 2, 4, 8, 16, 32
                let mip_width = (config.width / divisor).max(1);
                let mip_height = (config.height / divisor).max(1);

                let tex = device.create_texture(&wgpu::TextureDescriptor {
                    label: Some(&format!("bloom mip {}", i)),
                    size: wgpu::Extent3d {
                        width: mip_width,
                        height: mip_height,
                        depth_or_array_layers: 1,
                    },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: wgpu::TextureFormat::Rgba16Float,
                    usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
                    view_formats: &[],
                });

                let view = tex.create_view(&wgpu::TextureViewDescriptor::default());
                mip_textures.push(tex);
                mip_views.push(view);
            }

            // Bind group layout for bloom passes
            let bloom_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("bloom bgl"),
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

            // Threshold bind group layout (adds uniform buffer)
            let bloom_threshold_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("bloom threshold bgl"),
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
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            });

            // Composite bind group layout (2 textures + sampler + intensity uniform)
            let bloom_composite_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("bloom composite bgl"),
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
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 3,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            });

            // Create shader modules
            let threshold_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("bloom threshold shader"),
                source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(BLOOM_THRESHOLD_WGSL)),
            });

            let downsample_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("bloom downsample shader"),
                source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(BLOOM_DOWNSAMPLE_WGSL)),
            });

            let upsample_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("bloom upsample shader"),
                source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(BLOOM_UPSAMPLE_WGSL)),
            });

            let composite_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("bloom composite shader"),
                source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(BLOOM_COMPOSITE_WGSL)),
            });

            // Create pipelines
            let threshold_pl_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("bloom threshold layout"),
                bind_group_layouts: &[&bloom_threshold_bgl],
                push_constant_ranges: &[],
            });

            let bloom_threshold_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("bloom threshold pipeline"),
                layout: Some(&threshold_pl_layout),
                vertex: wgpu::VertexState {
                    module: &threshold_shader,
                    entry_point: Some("vs"),
                    buffers: &[],
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &threshold_shader,
                    entry_point: Some("fs"),
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
                cache: None,
            });

            let downsample_pl_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("bloom downsample layout"),
                bind_group_layouts: &[&bloom_bgl],
                push_constant_ranges: &[],
            });

            let bloom_downsample_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("bloom downsample pipeline"),
                layout: Some(&downsample_pl_layout),
                vertex: wgpu::VertexState {
                    module: &downsample_shader,
                    entry_point: Some("vs"),
                    buffers: &[],
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &downsample_shader,
                    entry_point: Some("fs"),
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
                cache: None,
            });

            let upsample_pl_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("bloom upsample layout"),
                bind_group_layouts: &[&bloom_bgl],
                push_constant_ranges: &[],
            });

            let bloom_upsample_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("bloom upsample pipeline"),
                layout: Some(&upsample_pl_layout),
                vertex: wgpu::VertexState {
                    module: &upsample_shader,
                    entry_point: Some("vs"),
                    buffers: &[],
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &upsample_shader,
                    entry_point: Some("fs"),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: wgpu::TextureFormat::Rgba16Float,
                        blend: Some(wgpu::BlendState::ALPHA_BLENDING), // Additive blending for upsampling
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                }),
                primitive: wgpu::PrimitiveState::default(),
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                multiview: None,
                cache: None,
            });

            let composite_pl_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("bloom composite layout"),
                bind_group_layouts: &[&bloom_composite_bgl],
                push_constant_ranges: &[],
            });

            let bloom_composite_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("bloom composite pipeline"),
                layout: Some(&composite_pl_layout),
                vertex: wgpu::VertexState {
                    module: &composite_shader,
                    entry_point: Some("vs"),
                    buffers: &[],
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &composite_shader,
                    entry_point: Some("fs"),
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
                cache: None,
            });

            // Create bind groups
            // Threshold bind group: HDR input + sampler + threshold uniform
            let bloom_threshold_bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("bloom threshold bg"),
                layout: &bloom_threshold_bgl,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&hdr_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&hdr_sampler),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: bloom_threshold_buf.as_entire_binding(),
                    },
                ],
            });

            // Downsample bind groups (chain: mip0 -> mip1 -> mip2 -> mip3 -> mip4)
            let mut bloom_bind_groups_down = Vec::with_capacity(MIP_COUNT as usize);
            for i in 0..MIP_COUNT {
                let src_view = if i == 0 {
                    &mip_views[0] // First downsample from threshold output (mip0)
                } else {
                    &mip_views[i as usize - 1]
                };

                let bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some(&format!("bloom downsample bg {}", i)),
                    layout: &bloom_bgl,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(src_view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::Sampler(&hdr_sampler),
                        },
                    ],
                });
                bloom_bind_groups_down.push(bg);
            }

            // Upsample bind groups (chain: mip4 -> mip3 -> mip2 -> mip1 -> mip0)
            let mut bloom_bind_groups_up = Vec::with_capacity(MIP_COUNT as usize);
            for i in (0..MIP_COUNT).rev() {
                let src_view = &mip_views[i as usize];

                let bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some(&format!("bloom upsample bg {}", i)),
                    layout: &bloom_bgl,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(src_view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::Sampler(&hdr_sampler),
                        },
                    ],
                });
                bloom_bind_groups_up.push(bg);
            }

            // Composite bind group: original HDR + bloom mip0 + sampler + intensity uniform
            let bloom_composite_bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("bloom composite bg"),
                layout: &bloom_composite_bgl,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&hdr_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(&mip_views[0]),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::Sampler(&hdr_sampler),
                    },
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: bloom_intensity_buf.as_entire_binding(),
                    },
                ],
            });

            (
                bloom_threshold_pipeline,
                bloom_downsample_pipeline,
                bloom_upsample_pipeline,
                bloom_composite_pipeline,
                mip_textures,
                mip_views,
                bloom_threshold_buf,
                bloom_intensity_buf,
                bloom_bgl,
                bloom_bind_groups_down,
                bloom_bind_groups_up,
                bloom_threshold_bg,
                bloom_composite_bg,
            )
        };

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
                entry_point: Some("vs"),
                buffers: &[
                    crate::types::Vertex::layout(),
                    crate::types::InstanceRaw::layout(),
                ],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs"),
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
            cache: None,
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
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &post_shader,
                entry_point: Some("fs_main"),
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
            cache: None,
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
            usage: None,
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
            usage: None,
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
            usage: None,
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
                entry_point: Some("vs"),
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
            cache: None,
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
            wgpu::TexelCopyTextureInfo {
                texture: &albedo_tex,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &[255u8, 255u8, 255u8, 255u8],
            wgpu::TexelCopyBufferLayout {
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
            wgpu::TexelCopyTextureInfo {
                texture: &mr_tex,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &[0u8, 255u8, 0u8, 255u8],
            wgpu::TexelCopyBufferLayout {
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
            wgpu::TexelCopyTextureInfo {
                texture: &normal_tex,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &[128u8, 128u8, 255u8, 255u8],
            wgpu::TexelCopyBufferLayout {
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
"#)),
        });
        let skinned_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("skinned pipeline"),
            layout: Some(&skinned_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &skinned_shader,
                entry_point: Some("vs"),
                buffers: &[
                    crate::types::SkinnedVertex::layout(),
                    crate::types::InstanceRaw::layout(),
                ],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &skinned_shader,
                entry_point: Some("fs"),
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
            cache: None,
        });

        // Default extra textures
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
            wgpu::TexelCopyTextureInfo {
                texture: &mr_tex,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &[0u8, 255u8, 0u8, 255u8],
            wgpu::TexelCopyBufferLayout {
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
            wgpu::TexelCopyTextureInfo {
                texture: &normal_tex,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &[128u8, 128u8, 255u8, 255u8],
            wgpu::TexelCopyBufferLayout {
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
        // extra_tex_bg removed; MR/normal are in combined tex_bg

        // Clustered resources default allocs
        // Create real meshes from built-in primitives
        let (cube_v, cube_i) = crate::primitives::cube();
        let (sphere_v, sphere_i) = crate::primitives::sphere(24, 24, 1.0);
        let cube_vb = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("mesh_cube vertex_buf"),
            contents: bytemuck::cast_slice(&cube_v),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let cube_ib = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("mesh_cube index_buf"),
            contents: bytemuck::cast_slice(&cube_i),
            usage: wgpu::BufferUsages::INDEX,
        });
        let mesh_cube = Mesh {
            vertex_buf: cube_vb,
            index_buf: cube_ib,
            index_count: cube_i.len() as u32,
        };
        let sphere_vb = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("mesh_sphere vertex_buf"),
            contents: bytemuck::cast_slice(&sphere_v),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let sphere_ib = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("mesh_sphere index_buf"),
            contents: bytemuck::cast_slice(&sphere_i),
            usage: wgpu::BufferUsages::INDEX,
        });
        let mesh_sphere = Mesh {
            vertex_buf: sphere_vb,
            index_buf: sphere_ib,
            index_count: sphere_i.len() as u32,
        };

        let (plane_v, plane_i) = crate::primitives::plane();
        let plane_vb = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("mesh_plane vertex_buf"),
            contents: bytemuck::cast_slice(&plane_v),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let plane_ib = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("mesh_plane index_buf"),
            contents: bytemuck::cast_slice(&plane_i),
            usage: wgpu::BufferUsages::INDEX,
        });
        let mesh_plane = Mesh {
            vertex_buf: plane_vb,
            index_buf: plane_ib,
            index_count: plane_i.len() as u32,
        };
        // Dummy instance buffer
        let instance_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("dummy instance_buf"),
            size: 256,
            usage: wgpu::BufferUsages::VERTEX,
            mapped_at_creation: false,
        });
        let clustered_dims = ClusterDims { x: 8, y: 4, z: 8 };
        #[repr(C)]
        #[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
        // Use explicit 16-byte slots to match WGSL uniform layout: three vec4-sized slots = 48 bytes
        struct CParams {
            screen: [u32; 4],
            clusters: [u32; 4],
            params: [f32; 4],
        }
        let cparams_init = CParams {
            screen: [config.width.max(1), config.height.max(1), 0, 0],
            clusters: [clustered_dims.x, clustered_dims.y, clustered_dims.z, 0],
            params: [0.1, 200.0, std::f32::consts::FRAC_PI_3, 0.0],
        };
        let clustered_params_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("cparams"),
            contents: bytemuck::bytes_of(&cparams_init),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let clustered_lights_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("clights"),
            size: 64 * 16,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let clusters_total = (clustered_dims.x * clustered_dims.y * clustered_dims.z) as usize;
        let clustered_offsets_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("coffsets"),
            size: ((clusters_total + 1) * 4) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let clustered_counts_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("ccounts"),
            size: (clusters_total * 4) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        // Reserve indices buffer capacity: lights * 64 as an upper bound placeholder
        let clustered_indices_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("cindices"),
            size: (64 * 64 * 4) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        // Fragment path doesn't use clustered data in this build; create a minimal bind group matching the layout (binding 4 as uniform).
        let clustered_bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("clustered bg"),
            layout: &clustered_bgl,
            entries: &[wgpu::BindGroupEntry {
                binding: 4,
                resource: clustered_params_buf.as_entire_binding(),
            }],
        });

        // Compute pipeline for clustered binning
        let clustered_comp_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("clustered comp"),
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(WGSL_CLUSTER_BIN)),
        });
        let clustered_comp_bgl =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("clustered comp bgl"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 3,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: false },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 4,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: false },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            });
        let clustered_comp_pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("clustered comp pl"),
            bind_group_layouts: &[&clustered_comp_bgl],
            push_constant_ranges: &[],
        });
        let clustered_comp_pipeline =
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("clustered comp pipeline"),
                layout: Some(&clustered_comp_pl),
                module: &clustered_comp_shader,
                entry_point: Some("cs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                cache: None,
            });
        let clustered_comp_bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("clustered comp bg"),
            layout: &clustered_comp_bgl,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: clustered_lights_buf.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: clustered_params_buf.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: clustered_offsets_buf.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: clustered_counts_buf.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: clustered_indices_buf.as_entire_binding(),
                },
            ],
        });

        // Create overlay resources now while `device` and `config` are still available.
        let overlay = crate::overlay::OverlayFx::new(&device, config.format);
        let overlay_params = crate::overlay::OverlayParams {
            fade: 0.0,
            letterbox: 0.0,
            _pad: [0.0; 2],
        };

        Ok(Self {
            surface,
            device,
            queue,
            config,
            depth,
            shader,
            pipeline,
            material_buf,
            material_bg,
            post_pipeline,
            post_bind_group,
            post_bgl,
            hdr_tex,
            hdr_view,
            hdr_sampler,
            #[cfg(feature = "postfx")]
            hdr_aux_tex: hdr_aux,
            #[cfg(feature = "postfx")]
            hdr_aux_view,
            #[cfg(feature = "postfx")]
            fx_gi_tex: fx_gi,
            #[cfg(feature = "postfx")]
            fx_gi_view,
            #[cfg(feature = "postfx")]
            fx_ao_tex: fx_ao,
            #[cfg(feature = "postfx")]
            fx_ao_view,
            #[cfg(feature = "postfx")]
            post_fx_bgl,
            #[cfg(feature = "postfx")]
            post_fx_bind_group,
            #[cfg(feature = "postfx")]
            post_fx_pipeline,
            #[cfg(feature = "postfx")]
            bloom_threshold_pipeline,
            #[cfg(feature = "postfx")]
            bloom_downsample_pipeline,
            #[cfg(feature = "postfx")]
            bloom_upsample_pipeline,
            #[cfg(feature = "postfx")]
            bloom_composite_pipeline,
            #[cfg(feature = "postfx")]
            bloom_mip_textures,
            #[cfg(feature = "postfx")]
            bloom_mip_views,
            #[cfg(feature = "postfx")]
            bloom_threshold_buf,
            #[cfg(feature = "postfx")]
            bloom_intensity_buf,
            #[cfg(feature = "postfx")]
            bloom_bgl,
            #[cfg(feature = "postfx")]
            bloom_bind_groups_down,
            #[cfg(feature = "postfx")]
            bloom_bind_groups_up,
            #[cfg(feature = "postfx")]
            bloom_threshold_bg,
            #[cfg(feature = "postfx")]
            bloom_composite_bg,
            shadow_tex,
            shadow_view,
            shadow_layer0_view,
            shadow_layer1_view,
            shadow_sampler,
            shadow_pipeline,
            light_buf,
            light_bg,
            shadow_bgl,
            light_bg_shadow,
            cascade0: glam::Mat4::IDENTITY,
            cascade1: glam::Mat4::IDENTITY,
            split0: 60.0,
            split1: 120.0,
            cascade0_extent: 40.0,
            cascade1_extent: 80.0,
            cascade_lambda: 0.5,
            shadow_pcf_radius_px: 1.0,
            shadow_depth_bias: 0.0006,
            shadow_slope_scale: 0.002,
            albedo_tex,
            albedo_view,
            albedo_sampler,
            tex_bgl,
            tex_bg,
            mr_tex,
            mr_view,
            mr_sampler,
            normal_tex,
            normal_view,
            normal_sampler,
            // combined tex_bgl/tex_bg used
            camera_ubo: CameraUBO {
                view_proj: Mat4::IDENTITY.to_cols_array_2d(),
                light_dir_pad: [0.5, 1.0, 0.8, 0.0],
            },
            camera_buf,
            camera_bind_group,
            mesh_cube,
            mesh_sphere,
            mesh_plane,
            mesh_external: None,
            instances: Vec::new(),
            instance_buf,
            overlay,
            overlay_params,
            weather,
            sky,
            skin_bgl,
            skin_bg,
            skin_palette_buf,
            skinned_pipeline,
            skinned_mesh: None,
            clustered_dims,
            clustered_params_buf,
            clustered_lights_buf,
            clustered_offsets_buf,
            clustered_counts_buf,
            clustered_indices_buf,
            clustered_bgl,
            clustered_bg,
            clustered_comp_bgl,
            clustered_comp_bg,
            clustered_comp_pipeline,
            point_lights: Vec::new(),
            #[cfg(feature = "gpu-tests")]
            timestamp_query_set,
            #[cfg(feature = "gpu-tests")]
            timestamp_buf,
            cin_tl: None,
            cin_seq: awc::Sequencer::new(),
            cin_playing: false,
            _residency_manager: crate::residency::ResidencyManager::new(
                std::sync::Arc::new(std::sync::Mutex::new(astraweave_asset::AssetDatabase::new())),
                512, // 512 MB default max memory
            ),
        })
    }

    // --- Cinematics wiring ---
    fn apply_camera_key(cam: &mut Camera, k: &awc::CameraKey) {
        let pos = glam::Vec3::new(k.pos.0, k.pos.1, k.pos.2);
        let look = glam::Vec3::new(k.look_at.0, k.look_at.1, k.look_at.2);
        let dir = (look - pos).normalize_or_zero();
        let yaw = dir.z.atan2(dir.x);
        let pitch = dir.y.clamp(-1.0, 1.0).asin();
        cam.position = pos;
        cam.yaw = yaw;
        cam.pitch = pitch;
        cam.fovy = k.fov_deg.to_radians();
    }

    pub fn load_timeline_json(&mut self, json: &str) -> Result<()> {
        let tl: awc::Timeline = serde_json::from_str(json)?;
        self.cin_tl = Some(tl);
        self.cin_seq.seek(awc::Time(0.0));
        Ok(())
    }

    pub fn save_timeline_json(&self) -> Option<String> {
        self.cin_tl
            .as_ref()
            .and_then(|tl| serde_json::to_string_pretty(tl).ok())
    }

    pub fn play_timeline(&mut self) {
        self.cin_playing = true;
    }
    pub fn stop_timeline(&mut self) {
        self.cin_playing = false;
    }
    pub fn seek_timeline(&mut self, t: f32) {
        self.cin_seq.seek(awc::Time(t));
    }

    /// Step the sequencer and apply camera keys; returns emitted events (for audio/FX handling by caller)
    pub fn tick_cinematics(&mut self, dt: f32, camera: &mut Camera) -> Vec<awc::SequencerEvent> {
        let mut out = Vec::new();
        if !self.cin_playing {
            return out;
        }
        if let Some(tl) = self.cin_tl.as_ref() {
            if let Ok(evs) = self.cin_seq.step(dt, tl) {
                for e in evs.iter() {
                    match e {
                        &awc::SequencerEvent::CameraKey(ref k) => Self::apply_camera_key(camera, k),
                        &awc::SequencerEvent::FxTrigger {
                            ref name,
                            ref params,
                        } => {
                            // Minimal FX: support fade-in by instantly clearing letterbox/fade
                            if name == "fade-in" {
                                let _ = params; // reserved
                                self.overlay_params.fade = 0.0;
                            }
                        }
                        _ => {}
                    }
                }
                out = evs;
            }
        }
        out
    }

    pub fn resize(&mut self, new_w: u32, new_h: u32) {
        if new_w == 0 || new_h == 0 {
            return;
        }
        self.config.width = new_w;
        self.config.height = new_h;
        self.surface.configure(&self.device, &self.config);
        self.depth = crate::depth::Depth::create(&self.device, &self.config);

        // Recreate HDR target and refresh the post-processing bind group.
        self.hdr_tex = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("hdr tex"),
            size: wgpu::Extent3d {
                width: self.config.width,
                height: self.config.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba16Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        self.hdr_view = self
            .hdr_tex
            .create_view(&wgpu::TextureViewDescriptor::default());
        self.post_bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("post bg"),
            layout: &self.post_bgl,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&self.hdr_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.hdr_sampler),
                },
            ],
        });
        // Update clustered params screen size
        #[repr(C)]
        #[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
        struct CParams {
            screen: [u32; 4],
            clusters: [u32; 4],
            params: [f32; 4],
        }
        let data: CParams = CParams {
            screen: [new_w.max(1), new_h.max(1), 0, 0],
            clusters: [
                self.clustered_dims.x,
                self.clustered_dims.y,
                self.clustered_dims.z,
                0,
            ],
            params: [0.1, 200.0, std::f32::consts::FRAC_PI_3, 0.0],
        };
        self.queue
            .write_buffer(&self.clustered_params_buf, 0, bytemuck::bytes_of(&data));
    }

    pub fn update_camera(&mut self, camera: &Camera) {
        self.camera_ubo.view_proj = camera.vp().to_cols_array_2d();
        // Update light dir from time-of-day system (simple linkage for Phase 0)
        let light_dir = self.sky.time_of_day().get_light_direction();
        self.camera_ubo.light_dir_pad = [light_dir.x, light_dir.y, light_dir.z, 0.0];
        self.queue
            .write_buffer(&self.camera_buf, 0, bytemuck::bytes_of(&self.camera_ubo));
        // Compute splits from camera frustum with lambda blend
        let n = camera.znear.max(0.01);
        let f = camera.zfar.max(n + 0.1);
        let c = 2.0; // cascades count (fixed to 2)
        let i = 1.0f32; // boundary between 0 and 1
        let u = n + (f - n) * (i / c);
        let l = n * (f / n).powf(i / c);
        let lambda = self.cascade_lambda.clamp(0.0, 1.0);
        let split = l * lambda + u * (1.0 - lambda);
        self.split0 = split;
        self.split1 = f;

        // Build frustum corners per range in world space
        let frustum0 = frustum_corners_ws(camera, n, self.split0);
        let frustum1 = frustum_corners_ws(camera, self.split0, f);
        // Build a light view looking towards the cascade centers
        let up = glam::Vec3::Y;
        let center0 = frustum_center(&frustum0);
        let center1 = frustum_center(&frustum1);
        let light_dist = 80.0f32;
        let view0 = glam::Mat4::look_to_rh(center0 - light_dir * light_dist, light_dir, up);
        let view1 = glam::Mat4::look_to_rh(center1 - light_dir * light_dist, light_dir, up);
        // Fit orthographic bounds to cascade frusta in light space
        let (min0, max0) = aabb_in_view_space(&view0, &frustum0);
        let (min1, max1) = aabb_in_view_space(&view1, &frustum1);
        let margin = 5.0f32;
        let proj0 = glam::Mat4::orthographic_rh(
            min0.x - margin,
            max0.x + margin,
            min0.y - margin,
            max0.y + margin,
            (-max0.z + 0.1).max(0.1),
            (-min0.z + margin + 0.1).max(1.0),
        );
        let proj1 = glam::Mat4::orthographic_rh(
            min1.x - margin,
            max1.x + margin,
            min1.y - margin,
            max1.y + margin,
            (-max1.z + 0.1).max(0.1),
            (-min1.z + margin + 0.1).max(1.0),
        );
        self.cascade0 = proj0 * view0;
        self.cascade1 = proj1 * view1;
        // Pack data for main pass buffer: [mat0, mat1, vec2(splits), vec2(extras)]
        let mut data: Vec<f32> = Vec::with_capacity(36);
        data.extend_from_slice(&self.cascade0.to_cols_array());
        data.extend_from_slice(&self.cascade1.to_cols_array());
        data.push(self.split0);
        data.push(self.split1);
        // extras: pack pcf radius in x, depth_bias in y
        data.push(self.shadow_pcf_radius_px);
        data.push(self.shadow_depth_bias);
        self.queue
            .write_buffer(&self.light_buf, 0, bytemuck::cast_slice(&data));
    }

    // --- CSM Tuning API ---
    pub fn set_cascade_splits(&mut self, split0: f32, split1: f32) {
        self.split0 = split0.max(0.01);
        self.split1 = split1.max(self.split0 + 0.01);
    }
    pub fn set_cascade_extents(&mut self, extent0: f32, extent1: f32) {
        self.cascade0_extent = extent0.max(1.0);
        self.cascade1_extent = extent1.max(self.cascade0_extent + 1.0);
    }

    /// Controls the split distribution between uniform (0) and logarithmic (1)
    pub fn set_cascade_lambda(&mut self, lambda: f32) {
        self.cascade_lambda = lambda.clamp(0.0, 1.0);
    }

    /// Sets shadow filtering and bias values. radius is in texels for 3x3 PCF when >=1.
    pub fn set_shadow_filter(&mut self, radius_px: f32, depth_bias: f32, slope_scale: f32) {
        self.shadow_pcf_radius_px = radius_px.max(0.0);
        self.shadow_depth_bias = depth_bias.max(0.0);
        self.shadow_slope_scale = slope_scale.max(0.0);
    }

    pub fn set_material_params(&mut self, base_color: [f32; 4], metallic: f32, roughness: f32) {
        // layout: vec4 + f32 + f32 + padding
        let mut data = [0f32; 8];
        data[0] = base_color[0];
        data[1] = base_color[1];
        data[2] = base_color[2];
        data[3] = base_color[3];
        data[4] = metallic;
        data[5] = roughness;
        self.queue
            .write_buffer(&self.material_buf, 0, bytemuck::cast_slice(&data));
    }

    pub fn create_mesh_from_arrays(
        &self,
        vertices: &[[f32; 3]],
        normals: &[[f32; 3]],
        indices: &[u32],
    ) -> Mesh {
        #[cfg(feature = "profiling")]
        span!("Render::MeshUpload");

        // Interleave into Vertex, derive simple defaults for tangent (+X) and uv (planar XZ)
        let verts: Vec<crate::types::Vertex> = vertices
            .iter()
            .zip(normals.iter())
            .map(|(p, n)| crate::types::Vertex {
                position: *p,
                normal: *n,
                tangent: [1.0, 0.0, 0.0, 1.0],
                uv: [p[0] * 0.5 + 0.5, p[2] * 0.5 + 0.5],
            })
            .collect();
        let vbuf = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("ext v"),
                contents: bytemuck::cast_slice(&verts),
                usage: wgpu::BufferUsages::VERTEX,
            });
        let ibuf = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("ext i"),
                contents: bytemuck::cast_slice(indices),
                usage: wgpu::BufferUsages::INDEX,
            });
        Mesh {
            vertex_buf: vbuf,
            index_buf: ibuf,
            index_count: indices.len() as u32,
        }
    }

    pub fn create_mesh_from_full_arrays(
        &self,
        positions: &[[f32; 3]],
        normals: &[[f32; 3]],
        tangents: &[[f32; 4]],
        uvs: &[[f32; 2]],
        indices: &[u32],
    ) -> Mesh {
        #[cfg(feature = "profiling")]
        span!("Render::MeshUpload::Full");

        assert!(
            positions.len() == normals.len()
                && positions.len() == tangents.len()
                && positions.len() == uvs.len()
        );
        let verts: Vec<crate::types::Vertex> = positions
            .iter()
            .zip(normals.iter())
            .zip(tangents.iter())
            .zip(uvs.iter())
            .map(|(((p, n), t), uv)| crate::types::Vertex {
                position: *p,
                normal: *n,
                tangent: *t,
                uv: *uv,
            })
            .collect();
        let vbuf = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("ext v (full)"),
                contents: bytemuck::cast_slice(&verts),
                usage: wgpu::BufferUsages::VERTEX,
            });
        let ibuf = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("ext i (full)"),
                contents: bytemuck::cast_slice(indices),
                usage: wgpu::BufferUsages::INDEX,
            });
        Mesh {
            vertex_buf: vbuf,
            index_buf: ibuf,
            index_count: indices.len() as u32,
        }
    }

    pub fn set_external_mesh(&mut self, mesh: Mesh) {
        self.mesh_external = Some(mesh);
    }

    pub fn update_instances(&mut self, instances: &[Instance]) {
        self.instances.clear();
        self.instances.extend_from_slice(instances);
        let raws: Vec<InstanceRaw> = self.instances.iter().map(|i| i.raw()).collect();
        let size = (raws.len() * std::mem::size_of::<InstanceRaw>()) as u64;

        if size > self.instance_buf.size() {
            self.instance_buf = self.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("instance buf (resized)"),
                size: size.next_power_of_two(),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
        }
        self.queue
            .write_buffer(&self.instance_buf, 0, bytemuck::cast_slice(&raws));
    }

    pub fn set_weather(&mut self, kind: crate::effects::WeatherKind) {
        self.weather.set_kind(kind);
    }

    pub fn tick_weather(&mut self, dt: f32) {
        self.weather.update(&self.queue, dt);
    }

    pub fn tick_environment(&mut self, dt: f32) {
        // Advance time-of-day; derive sky params
        self.sky.update(dt);
    }

    pub fn time_of_day_mut(&mut self) -> &mut crate::environment::TimeOfDay {
        self.sky.time_of_day_mut()
    }

    pub fn sky_config(&self) -> crate::environment::SkyConfig {
        self.sky.config().clone()
    }

    pub fn set_sky_config(&mut self, cfg: crate::environment::SkyConfig) {
        self.sky.set_config(cfg);
    }

    pub fn render(&mut self) -> Result<()> {
        #[cfg(feature = "profiling")]
        span!("Render::Frame");

        let frame = self.surface.get_current_texture()?;
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut enc = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("encoder"),
            });

        // Create plane buffer before render pass
        // Slightly lower the ground plane to avoid z-fighting with terrain at y=0
        let plane_xform = glam::Mat4::from_translation(vec3(0.0, -0.2, 0.0))
            * glam::Mat4::from_scale(vec3(50.0, 1.0, 50.0));
        let plane_inst = Instance {
            transform: plane_xform,
            color: [0.1, 0.12, 0.14, 1.0],
            material_id: 0,
        }
        .raw();
        let plane_buf = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("plane inst"),
                contents: bytemuck::bytes_of(&plane_inst),
                usage: wgpu::BufferUsages::VERTEX,
            });

        // Render sky first into HDR
        // TODO: Replace with the correct color target view for sky rendering (e.g., main color target or postprocess output)
        // self.sky.render(&mut enc, &self.main_color_view, &self.depth.view, Mat4::from_cols_array_2d(&self.camera_ubo.view_proj), &self.queue)?;

        {
            #[cfg(feature = "profiling")]
            span!("Render::ClusteredLighting");

            // Prepare clustered lighting for this frame: simple demo lights around origin
            if self.point_lights.is_empty() {
                self.point_lights.push(CpuLight {
                    pos: glam::Vec3::new(2.0, 2.0, 3.0),
                    radius: 6.0,
                });
                self.point_lights.push(CpuLight {
                    pos: glam::Vec3::new(-3.0, 1.0, 8.0),
                    radius: 5.0,
                });
            }
            // CPU pre-pass builds offsets array (exclusive scan) we share to GPU
            let (_counts_cpu, _indices_cpu, offsets_cpu) = bin_lights_cpu(
                &self.point_lights,
                self.clustered_dims,
                (self.config.width, self.config.height),
                0.1,
                200.0,
                std::f32::consts::FRAC_PI_3,
            );
            // Upload lights and offsets; zero counts and indices
            #[repr(C)]
            #[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
            struct GpuLight {
                pos_radius: [f32; 4],
            }
            let glights: Vec<GpuLight> = self
                .point_lights
                .iter()
                .map(|l| GpuLight {
                    pos_radius: [l.pos.x, l.pos.y, l.pos.z, l.radius],
                })
                .collect();
            if !glights.is_empty() {
                self.queue.write_buffer(
                    &self.clustered_lights_buf,
                    0,
                    bytemuck::cast_slice(&glights),
                );
            }
            self.queue.write_buffer(
                &self.clustered_offsets_buf,
                0,
                bytemuck::cast_slice(&offsets_cpu),
            );
            // Zero counts
            let clusters =
                (self.clustered_dims.x * self.clustered_dims.y * self.clustered_dims.z) as usize;
            let zero_counts = vec![0u32; clusters];
            self.queue.write_buffer(
                &self.clustered_counts_buf,
                0,
                bytemuck::cast_slice(&zero_counts),
            );
            // Run compute to fill counts/indices
            #[cfg(feature = "gpu-tests")]
            {
                enc.write_timestamp(&self.timestamp_query_set, 0);
            }
            let mut cpass = enc.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("cluster bin"),
                timestamp_writes: None,
            });
            cpass.set_pipeline(&self.clustered_comp_pipeline);
            cpass.set_bind_group(0, &self.clustered_comp_bg, &[]);
            cpass.dispatch_workgroups(glights.len() as u32, 1, 1);
            drop(cpass);
            #[cfg(feature = "gpu-tests")]
            {
                enc.write_timestamp(&self.timestamp_query_set, 1);
                enc.resolve_query_set(&self.timestamp_query_set, 0..2, &self.timestamp_buf, 0);
            }
        }
        // Prepare external mesh single-instance buffer if needed
        let ext_ibuf: Option<wgpu::Buffer> = if self.mesh_external.is_some() {
            let inst = Instance {
                transform: Mat4::IDENTITY,
                color: [1.0, 1.0, 1.0, 1.0],
                material_id: 0,
            }
            .raw();
            Some(
                self.device
                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("ext inst"),
                        contents: bytemuck::bytes_of(&inst),
                        usage: wgpu::BufferUsages::VERTEX,
                    }),
            )
        } else {
            None
        };
        // Frustum cull instances
        let (vis_raws, vis_count) = self.build_visible_instances();
        if vis_count > 0 {
            #[cfg(feature = "profiling")]
            {
                span!("Render::BufferWrite::Instances");
                plot!("Render::visible_instances", vis_count as u64);
            }
            self.queue
                .write_buffer(&self.instance_buf, 0, bytemuck::cast_slice(&vis_raws));
        }
        // Shadow passes (depth only) - one per cascade layer
        // Write cascade0 matrix, render to layer0; then cascade1, render to layer1
        #[cfg(feature = "profiling")]
        span!("Render::ShadowMaps");

        for (idx, layer_view) in [&self.shadow_layer0_view, &self.shadow_layer1_view]
            .iter()
            .enumerate()
        {
            let mat = if idx == 0 {
                self.cascade0
            } else {
                self.cascade1
            };
            let arr = mat.to_cols_array();
            self.queue
                .write_buffer(&self.light_buf, 0, bytemuck::cast_slice(&arr));
            let mut sp = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("shadow pass"),
                color_attachments: &[],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: layer_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            sp.set_pipeline(&self.shadow_pipeline);
            // Use the shadow-only bind group here so the shadow depth texture
            // isn't bound for sampling while we're writing to it.
            sp.set_bind_group(0, &self.light_bg_shadow, &[]);
            // Draw plane
            sp.set_vertex_buffer(0, self.mesh_plane.vertex_buf.slice(..));
            sp.set_index_buffer(
                self.mesh_plane.index_buf.slice(..),
                wgpu::IndexFormat::Uint32,
            );
            sp.set_vertex_buffer(1, plane_buf.slice(..));
            sp.draw_indexed(0..self.mesh_plane.index_count, 0, 0..1);
            // Draw tokens as spheres in shadow pass
            sp.set_vertex_buffer(0, self.mesh_sphere.vertex_buf.slice(..));
            sp.set_index_buffer(
                self.mesh_sphere.index_buf.slice(..),
                wgpu::IndexFormat::Uint32,
            );
            sp.set_vertex_buffer(1, self.instance_buf.slice(..));
            let inst_count = vis_count as u32;
            if inst_count > 0 {
                sp.draw_indexed(0..self.mesh_sphere.index_count, 0, 0..inst_count);
            }
            // External mesh
            if let (Some(mesh), Some(ibuf)) = (&self.mesh_external, ext_ibuf.as_ref()) {
                sp.set_vertex_buffer(0, mesh.vertex_buf.slice(..));
                sp.set_index_buffer(mesh.index_buf.slice(..), wgpu::IndexFormat::Uint32);
                sp.set_vertex_buffer(1, ibuf.slice(..));
                sp.draw_indexed(0..mesh.index_count, 0, 0..1);
            }
        }
        // After rendering shadow layers, restore full light buffer for main pass usage
        {
            let mut data: Vec<f32> = Vec::with_capacity(36);
            data.extend_from_slice(&self.cascade0.to_cols_array());
            data.extend_from_slice(&self.cascade1.to_cols_array());
            data.push(self.split0);
            data.push(self.split1);
            data.push(self.shadow_pcf_radius_px);
            data.push(self.shadow_depth_bias);
            self.queue
                .write_buffer(&self.light_buf, 0, bytemuck::cast_slice(&data));
        }

        // Render sky first into HDR target so we can layer geometry on top
        #[cfg(feature = "profiling")]
        span!("Render::Sky");

        self.sky.render(
            &mut enc,
            &self.hdr_view,
            &self.depth.view,
            Mat4::from_cols_array_2d(&self.camera_ubo.view_proj),
            &self.queue,
        )?;

        {
            #[cfg(feature = "profiling")]
            span!("Render::MainPass");

            let mut rp = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("main pass"),
                // Render the main scene into the HDR color target; a post-pass will tonemap to the surface.
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.hdr_view,
                    resolve_target: None,
                    // Preserve sky color drawn earlier
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            rp.set_pipeline(&self.pipeline);
            rp.set_bind_group(0, &self.camera_bind_group, &[]);
            rp.set_bind_group(1, &self.material_bg, &[]);
            rp.set_bind_group(2, &self.light_bg, &[]);
            rp.set_bind_group(3, &self.tex_bg, &[]);

            #[cfg(feature = "profiling")]
            let mut draw_calls = 0u64;

            // Ground plane (scaled)
            rp.set_vertex_buffer(0, self.mesh_plane.vertex_buf.slice(..));
            rp.set_index_buffer(
                self.mesh_plane.index_buf.slice(..),
                wgpu::IndexFormat::Uint32,
            );
            rp.set_vertex_buffer(1, plane_buf.slice(..));
            rp.draw_indexed(0..self.mesh_plane.index_count, 0, 0..1);
            #[cfg(feature = "profiling")]
            {
                draw_calls += 1;
            }

            // Tokens as lit spheres (instances)
            rp.set_vertex_buffer(0, self.mesh_sphere.vertex_buf.slice(..));
            rp.set_index_buffer(
                self.mesh_sphere.index_buf.slice(..),
                wgpu::IndexFormat::Uint32,
            );
            rp.set_vertex_buffer(1, self.instance_buf.slice(..));
            let inst_count = vis_count as u32;
            if inst_count > 0 {
                rp.draw_indexed(0..self.mesh_sphere.index_count, 0, 0..inst_count);
                #[cfg(feature = "profiling")]
                {
                    draw_calls += 1;
                }
            }

            // External mesh if present
            if let (Some(mesh), Some(ibuf)) = (&self.mesh_external, ext_ibuf.as_ref()) {
                rp.set_vertex_buffer(0, mesh.vertex_buf.slice(..));
                rp.set_index_buffer(mesh.index_buf.slice(..), wgpu::IndexFormat::Uint32);
                rp.set_vertex_buffer(1, ibuf.slice(..));
                rp.draw_indexed(0..mesh.index_count, 0, 0..1);
                #[cfg(feature = "profiling")]
                {
                    draw_calls += 1;
                }
            }

            #[cfg(feature = "profiling")]
            plot!("Render::draw_calls", draw_calls);
        }

        // Optional feature-gated post chain
        // TODO: Restore when postfx fields are added to Renderer struct
        /*
        #[cfg(feature = "postfx")]
        {
            let mut ssp = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("ssr pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.hdr_aux_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            ssp.set_pipeline(&self.ssr_pipeline);
            ssp.set_bind_group(0, &self.ssr_bind_group, &[]);
            ssp.draw(0..3, 0..1);
            drop(ssp);
            // SSAO
            let mut ao = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("ssao pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.fx_ao_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            ao.set_pipeline(&self.ssao_pipeline);
            ao.set_bind_group(0, &self.ssao_bind_group, &[]);
            ao.draw(0..3, 0..1);
            drop(ao);
            // SSGI
            let mut gi = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("ssgi pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.fx_gi_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            gi.set_pipeline(&self.ssgi_pipeline);
            gi.set_bind_group(0, &self.ssgi_bind_group, &[]);
            gi.draw(0..3, 0..1);
            drop(gi);
        }
        */

        // Bloom effect (threshold → downsample → upsample → composite)
        #[cfg(feature = "postfx")]
        {
            #[cfg(feature = "profiling")]
            span!("Render::Bloom");

            self.render_bloom(&mut enc);
        }

        // Postprocess HDR to surface
        {
            #[cfg(feature = "profiling")]
            span!("Render::Postprocess");

            let mut pp = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("post pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            #[cfg(feature = "postfx")]
            {
                pp.set_pipeline(&self.post_fx_pipeline);
                pp.set_bind_group(0, &self.post_fx_bind_group, &[]);
            }
            #[cfg(not(feature = "postfx"))]
            {
                pp.set_pipeline(&self.post_pipeline);
                pp.set_bind_group(0, &self.post_bind_group, &[]);
            }
            pp.draw(0..3, 0..1);
        }

        #[cfg(feature = "profiling")]
        {
            span!("Render::QueueSubmit");
            self.queue.submit(Some(enc.finish()));
        }
        #[cfg(not(feature = "profiling"))]
        self.queue.submit(Some(enc.finish()));

        #[cfg(feature = "profiling")]
        {
            span!("Render::Present");
            frame.present();
        }
        #[cfg(not(feature = "profiling"))]
        frame.present();

        Ok(())
    }

    /// Render bloom effect: threshold → downsample pyramid → upsample with blending → composite
    #[cfg(feature = "postfx")]
    fn render_bloom(&self, enc: &mut wgpu::CommandEncoder) {
        // Step 1: Threshold pass (HDR → mip0)
        {
            let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("bloom threshold pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.bloom_mip_views[0],
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            pass.set_pipeline(&self.bloom_threshold_pipeline);
            pass.set_bind_group(0, &self.bloom_threshold_bg, &[]);
            pass.draw(0..3, 0..1);
        }

        // Step 2: Downsample pyramid (mip0 → mip1 → mip2 → mip3 → mip4)
        for i in 1..self.bloom_mip_views.len() {
            let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some(&format!("bloom downsample pass {}", i)),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.bloom_mip_views[i],
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            pass.set_pipeline(&self.bloom_downsample_pipeline);
            pass.set_bind_group(0, &self.bloom_bind_groups_down[i], &[]);
            pass.draw(0..3, 0..1);
        }

        // Step 3: Upsample pyramid (mip4 → mip3 → mip2 → mip1 → mip0) with additive blending
        for i in (0..self.bloom_mip_views.len() - 1).rev() {
            let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some(&format!("bloom upsample pass {}", i)),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.bloom_mip_views[i],
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load, // Preserve existing content for additive blending
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            pass.set_pipeline(&self.bloom_upsample_pipeline);
            pass.set_bind_group(0, &self.bloom_bind_groups_up[i + 1], &[]);
            pass.draw(0..3, 0..1);
        }

        // Step 4: Composite bloom result with original HDR (result written to hdr_aux)
        {
            let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("bloom composite pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.hdr_aux_view, // Output to auxiliary HDR buffer
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            pass.set_pipeline(&self.bloom_composite_pipeline);
            pass.set_bind_group(0, &self.bloom_composite_bg, &[]);
            pass.draw(0..3, 0..1);
        }
    }

    pub fn draw_into(
        &mut self,
        view: &wgpu::TextureView,
        enc: &mut wgpu::CommandEncoder,
    ) -> Result<()> {
        // Render sky into HDR target first
        self.sky.render(
            enc,
            &self.hdr_view,
            &self.depth.view,
            Mat4::from_cols_array_2d(&self.camera_ubo.view_proj),
            &self.queue,
        )?;

        // Plane instance buffer
        let plane_xform = glam::Mat4::from_translation(vec3(0.0, -0.2, 0.0))
            * glam::Mat4::from_scale(vec3(50.0, 1.0, 50.0));
        let plane_inst = Instance {
            transform: plane_xform,
            color: [0.1, 0.12, 0.14, 1.0],
            material_id: 0,
        }
        .raw();
        let plane_buf = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("plane inst"),
                contents: bytemuck::bytes_of(&plane_inst),
                usage: wgpu::BufferUsages::VERTEX,
            });

        // Main pass (load color written by sky)
        // Prepare external mesh instance buffer if needed
        let ext_ibuf: Option<wgpu::Buffer> = if self.mesh_external.is_some() {
            let inst = Instance {
                transform: Mat4::IDENTITY,
                color: [1.0, 1.0, 1.0, 1.0],
                material_id: 0,
            }
            .raw();
            Some(
                self.device
                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("ext inst"),
                        contents: bytemuck::bytes_of(&inst),
                        usage: wgpu::BufferUsages::VERTEX,
                    }),
            )
        } else {
            None
        };

        // Prepare clustered lighting for this frame (same as render())
        if self.point_lights.is_empty() {
            self.point_lights.push(CpuLight {
                pos: glam::Vec3::new(2.0, 2.0, 3.0),
                radius: 6.0,
            });
            self.point_lights.push(CpuLight {
                pos: glam::Vec3::new(-3.0, 1.0, 8.0),
                radius: 5.0,
            });
        }
        let (_counts_cpu, _indices_cpu, offsets_cpu) = bin_lights_cpu(
            &self.point_lights,
            self.clustered_dims,
            (self.config.width, self.config.height),
            0.1,
            200.0,
            std::f32::consts::FRAC_PI_3,
        );
        #[repr(C)]
        #[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
        struct GpuLight {
            pos_radius: [f32; 4],
        }
        let glights: Vec<GpuLight> = self
            .point_lights
            .iter()
            .map(|l| GpuLight {
                pos_radius: [l.pos.x, l.pos.y, l.pos.z, l.radius],
            })
            .collect();
        if !glights.is_empty() {
            self.queue.write_buffer(
                &self.clustered_lights_buf,
                0,
                bytemuck::cast_slice(&glights),
            );
        }
        self.queue.write_buffer(
            &self.clustered_offsets_buf,
            0,
            bytemuck::cast_slice(&offsets_cpu),
        );
        let clusters =
            (self.clustered_dims.x * self.clustered_dims.y * self.clustered_dims.z) as usize;
        let zero_counts = vec![0u32; clusters];
        self.queue.write_buffer(
            &self.clustered_counts_buf,
            0,
            bytemuck::cast_slice(&zero_counts),
        );
        {
            let mut cpass = enc.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("cluster bin"),
                timestamp_writes: None,
            });
            cpass.set_pipeline(&self.clustered_comp_pipeline);
            cpass.set_bind_group(0, &self.clustered_comp_bg, &[]);
            cpass.dispatch_workgroups(glights.len() as u32, 1, 1);
        }

        // Frustum cull
        let (vis_raws, vis_count) = self.build_visible_instances();
        if vis_count > 0 {
            self.queue
                .write_buffer(&self.instance_buf, 0, bytemuck::cast_slice(&vis_raws));
        }

        // Shadow passes (depth only) - one per cascade layer
        for (idx, layer_view) in [&self.shadow_layer0_view, &self.shadow_layer1_view]
            .iter()
            .enumerate()
        {
            let mat = if idx == 0 {
                self.cascade0
            } else {
                self.cascade1
            };
            let arr = mat.to_cols_array();
            self.queue
                .write_buffer(&self.light_buf, 0, bytemuck::cast_slice(&arr));
            let mut sp = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("shadow pass"),
                color_attachments: &[],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: layer_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            sp.set_pipeline(&self.shadow_pipeline);
            // Use the shadow-only bind group here as well for external draw_into path
            sp.set_bind_group(0, &self.light_bg_shadow, &[]);
            sp.set_vertex_buffer(0, self.mesh_plane.vertex_buf.slice(..));
            sp.set_index_buffer(
                self.mesh_plane.index_buf.slice(..),
                wgpu::IndexFormat::Uint32,
            );
            sp.set_vertex_buffer(1, plane_buf.slice(..));
            sp.draw_indexed(0..self.mesh_plane.index_count, 0, 0..1);
            sp.set_vertex_buffer(0, self.mesh_sphere.vertex_buf.slice(..));
            sp.set_index_buffer(
                self.mesh_sphere.index_buf.slice(..),
                wgpu::IndexFormat::Uint32,
            );
            sp.set_vertex_buffer(1, self.instance_buf.slice(..));
            let inst_count = vis_count as u32;
            if inst_count > 0 {
                sp.draw_indexed(0..self.mesh_sphere.index_count, 0, 0..inst_count);
            }
            if let (Some(mesh), Some(ibuf)) = (&self.mesh_external, ext_ibuf.as_ref()) {
                sp.set_vertex_buffer(0, mesh.vertex_buf.slice(..));
                sp.set_index_buffer(mesh.index_buf.slice(..), wgpu::IndexFormat::Uint32);
                sp.set_vertex_buffer(1, ibuf.slice(..));
                sp.draw_indexed(0..mesh.index_count, 0, 0..1);
            }
        }
        // Restore full light buffer for main pass
        {
            let mut data: Vec<f32> = Vec::with_capacity(36);
            data.extend_from_slice(&self.cascade0.to_cols_array());
            data.extend_from_slice(&self.cascade1.to_cols_array());
            data.push(self.split0);
            data.push(self.split1);
            data.push(self.shadow_pcf_radius_px);
            data.push(self.shadow_depth_bias);
            self.queue
                .write_buffer(&self.light_buf, 0, bytemuck::cast_slice(&data));
        }

        // Prepare a temporary instance buffer for skinned mesh if needed
        let sk_inst_buf: Option<wgpu::Buffer> = if self.skinned_mesh.is_some() {
            let inst = Instance {
                transform: Mat4::IDENTITY,
                color: [1.0, 1.0, 1.0, 1.0],
                material_id: 0,
            }
            .raw();
            Some(
                self.device
                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("skinned inst"),
                        contents: bytemuck::bytes_of(&inst),
                        usage: wgpu::BufferUsages::VERTEX,
                    }),
            )
        } else {
            None
        };

        let mut rp = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("main pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &self.hdr_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &self.depth.view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        rp.set_pipeline(&self.pipeline);
        rp.set_bind_group(0, &self.camera_bind_group, &[]);
        rp.set_bind_group(1, &self.material_bg, &[]);
        rp.set_bind_group(2, &self.light_bg, &[]);
        rp.set_bind_group(3, &self.tex_bg, &[]);
        // Ground plane
        rp.set_vertex_buffer(0, self.mesh_plane.vertex_buf.slice(..));
        rp.set_index_buffer(
            self.mesh_plane.index_buf.slice(..),
            wgpu::IndexFormat::Uint32,
        );
        rp.set_vertex_buffer(1, plane_buf.slice(..));
        rp.draw_indexed(0..self.mesh_plane.index_count, 0, 0..1);
        // Tokens as spheres
        rp.set_vertex_buffer(0, self.mesh_sphere.vertex_buf.slice(..));
        rp.set_index_buffer(
            self.mesh_sphere.index_buf.slice(..),
            wgpu::IndexFormat::Uint32,
        );
        rp.set_vertex_buffer(1, self.instance_buf.slice(..));
        let inst_count = vis_count as u32;
        if inst_count > 0 {
            rp.draw_indexed(0..self.mesh_sphere.index_count, 0, 0..inst_count);
        }

        // External mesh
        if let (Some(mesh), Some(ibuf)) = (&self.mesh_external, ext_ibuf.as_ref()) {
            rp.set_vertex_buffer(0, mesh.vertex_buf.slice(..));
            rp.set_index_buffer(mesh.index_buf.slice(..), wgpu::IndexFormat::Uint32);
            rp.set_vertex_buffer(1, ibuf.slice(..));
            rp.draw_indexed(0..mesh.index_count, 0, 0..1);
        }
        // Skinned mesh (if present)
        if let (Some((vbuf, ibuf, index_count)), Some(sk_inst)) =
            (&self.skinned_mesh, sk_inst_buf.as_ref())
        {
            rp.set_pipeline(&self.skinned_pipeline);
            rp.set_bind_group(0, &self.camera_bind_group, &[]);
            rp.set_bind_group(1, &self.material_bg, &[]);
            rp.set_bind_group(2, &self.light_bg, &[]);
            // Combined textures + skin palette live in group 3
            rp.set_bind_group(3, &self.tex_bg, &[]);
            rp.set_vertex_buffer(0, vbuf.slice(..));
            rp.set_index_buffer(ibuf.slice(..), wgpu::IndexFormat::Uint32);
            rp.set_vertex_buffer(1, sk_inst.slice(..));
            rp.draw_indexed(0..*index_count, 0, 0..1);
        }
        drop(rp);

        // Optional postfx chain
        // TODO: Restore when postfx fields are added to Renderer struct
        /*
        #[cfg(feature = "postfx")]
        {
            let mut ssp = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("ssr pass (external)"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.hdr_aux_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            ssp.set_pipeline(&self.ssr_pipeline);
            ssp.set_bind_group(0, &self.ssr_bind_group, &[]);
            ssp.draw(0..3, 0..1);
            drop(ssp);
            let mut ao = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("ssao pass (external)"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.fx_ao_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            ao.set_pipeline(&self.ssao_pipeline);
            ao.set_bind_group(0, &self.ssao_bind_group, &[]);
            ao.draw(0..3, 0..1);
            drop(ao);
            let mut gi = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("ssgi pass (external)"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.fx_gi_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            gi.set_pipeline(&self.ssgi_pipeline);
            gi.set_bind_group(0, &self.ssgi_bind_group, &[]);
            gi.draw(0..3, 0..1);
            drop(gi);
        }
        */

        // Post to surface view provided
        let mut pp = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("post pass (external)"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });
        #[cfg(feature = "postfx")]
        {
            pp.set_pipeline(&self.post_fx_pipeline);
            pp.set_bind_group(0, &self.post_fx_bind_group, &[]);
        }
        #[cfg(not(feature = "postfx"))]
        {
            pp.set_pipeline(&self.post_pipeline);
            pp.set_bind_group(0, &self.post_bind_group, &[]);
        }
        pp.draw(0..3, 0..1);

        Ok(())
    }

    pub fn surface_size(&self) -> (u32, u32) {
        (self.config.width, self.config.height)
    }

    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }

    pub fn surface_format(&self) -> wgpu::TextureFormat {
        self.config.format
    }

    pub fn render_with<F>(&mut self, f: F) -> Result<()>
    where
        F: FnOnce(
            &wgpu::TextureView,
            &mut wgpu::CommandEncoder,
            &wgpu::Device,
            &wgpu::Queue,
            (u32, u32),
        ),
    {
        let frame = self.surface.get_current_texture()?;
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut enc = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("encoder"),
            });

        // First render the 3D scene into the frame (draw_into posts to view)
        self.draw_into(&view, &mut enc)?;

        // Then allow caller to composite additional passes (e.g., egui)
        f(
            &view,
            &mut enc,
            &self.device,
            &self.queue,
            self.surface_size(),
        );

        self.queue.submit(std::iter::once(enc.finish()));
        frame.present();
        Ok(())
    }

    /// Create a bind group layout deriving entries from a `MaterialPackage` bindings list.
    pub fn bgl_from_material_package(&self, pkg: &MaterialPackage) -> wgpu::BindGroupLayout {
        let mut entries: Vec<wgpu::BindGroupLayoutEntry> = Vec::new();
        let mut binding = 0u32;
        for _id in pkg.bindings.iter() {
            entries.push(wgpu::BindGroupLayoutEntry {
                binding,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            });
            binding += 1;
            entries.push(wgpu::BindGroupLayoutEntry {
                binding,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            });
            binding += 1;
        }
        self.device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("material bgl (derived)"),
                entries: &entries,
            })
    }

    /// Create a simple full-screen pipeline from a `MaterialPackage` (for previews or tests).
    pub fn pipeline_from_material_package(
        &self,
        pkg: &MaterialPackage,
        format: wgpu::TextureFormat,
    ) -> wgpu::RenderPipeline {
        let shader = self.shader_from_material_package(pkg);
        let bgl = self.bgl_from_material_package(pkg);
        let layout = self
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("material pipeline layout"),
                bind_group_layouts: &[&bgl],
                push_constant_ranges: &[],
            });
        self.device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("material preview pipeline"),
                layout: Some(&layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some("vs_main"),
                    buffers: &[],
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some("fs_main"),
                    targets: &[Some(wgpu::ColorTargetState {
                        format,
                        blend: None,
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                }),
                primitive: wgpu::PrimitiveState::default(),
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                multiview: None,
                cache: None,
            })
    }

    fn build_visible_instances(&self) -> (Vec<InstanceRaw>, usize) {
        let m = Mat4::from_cols_array_2d(&self.camera_ubo.view_proj);
        let mt = m.transpose();
        let r0 = mt.x_axis;
        let r1 = mt.y_axis;
        let r2 = mt.z_axis;
        let r3 = mt.w_axis;
        let planes = [
            r3 + r0, // left
            r3 - r0, // right
            r3 + r1, // bottom
            r3 - r1, // top
            r3 + r2, // near
            r3 - r2, // far
        ];
        let norm_planes: Vec<(glam::Vec3, f32)> = planes
            .iter()
            .map(|p| {
                let n = glam::Vec3::new(p.x, p.y, p.z);
                let len = n.length().max(1e-6);
                (n / len, p.w / len)
            })
            .collect();

        let mut out = Vec::with_capacity(self.instances.len());
        for inst in &self.instances {
            let center = inst.transform.w_axis.truncate();
            // approximate radius from basis vectors length (half-extents length)
            let sx = inst.transform.x_axis.truncate().length();
            let sy = inst.transform.y_axis.truncate().length();
            let sz = inst.transform.z_axis.truncate().length();
            let radius = 0.5 * glam::Vec3::new(sx, sy, sz).length();
            if inside_frustum_sphere(center, radius, &norm_planes) {
                out.push(inst.raw());
            }
        }
        let count = out.len();
        (out, count)
    }

    pub fn set_albedo_from_rgba8(&mut self, width: u32, height: u32, data: &[u8]) {
        assert_eq!(data.len() as u32, width * height * 4);
        // Recreate texture with provided dimensions
        self.albedo_tex = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("albedo"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        self.albedo_view = self
            .albedo_tex
            .create_view(&wgpu::TextureViewDescriptor::default());
        self.queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &self.albedo_tex,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            data,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(width * 4),
                rows_per_image: Some(height),
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );
        // Rebuild the combined tex+skin bind group with current views/samplers and skin buffer
        self.tex_bg = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("combined tex bg"),
            layout: &self.tex_bgl,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&self.albedo_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.albedo_sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&self.mr_view),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Sampler(&self.mr_sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::TextureView(&self.normal_view),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: wgpu::BindingResource::Sampler(&self.normal_sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 6,
                    resource: self.skin_palette_buf.as_entire_binding(),
                },
            ],
        });
    }

    pub fn set_metallic_roughness_from_rgba8(&mut self, width: u32, height: u32, data: &[u8]) {
        assert_eq!(data.len() as u32, width * height * 4);
        self.mr_tex = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("mr tex"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        self.mr_view = self
            .mr_tex
            .create_view(&wgpu::TextureViewDescriptor::default());
        self.queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &self.mr_tex,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            data,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(width * 4),
                rows_per_image: Some(height),
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );
        // Rebuild combined tex_bg so MR/normal updates are reflected
        self.tex_bg = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("combined tex bg"),
            layout: &self.tex_bgl,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&self.albedo_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.albedo_sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&self.mr_view),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Sampler(&self.mr_sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::TextureView(&self.normal_view),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: wgpu::BindingResource::Sampler(&self.normal_sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 6,
                    resource: self.skin_palette_buf.as_entire_binding(),
                },
            ],
        });
    }

    pub fn set_normal_from_rgba8(&mut self, width: u32, height: u32, data: &[u8]) {
        assert_eq!(data.len() as u32, width * height * 4);
        self.normal_tex = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("normal tex"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        self.normal_view = self
            .normal_tex
            .create_view(&wgpu::TextureViewDescriptor::default());
        self.queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &self.normal_tex,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            data,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(width * 4),
                rows_per_image: Some(height),
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );
        // Rebuild combined tex_bg so MR/normal updates are reflected
        self.tex_bg = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("combined tex bg"),
            layout: &self.tex_bgl,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&self.albedo_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.albedo_sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&self.mr_view),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Sampler(&self.mr_sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::TextureView(&self.normal_view),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: wgpu::BindingResource::Sampler(&self.normal_sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 6,
                    resource: self.skin_palette_buf.as_entire_binding(),
                },
            ],
        });
    }

    // --- Skinning API (v0) ---
    pub fn set_skinned_mesh(&mut self, vertices: &[SkinnedVertex], indices: &[u32]) {
        let vbuf = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("skinned vbuf"),
                contents: bytemuck::cast_slice(vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });
        let ibuf = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("skinned ibuf"),
                contents: bytemuck::cast_slice(indices),
                usage: wgpu::BufferUsages::INDEX,
            });
        self.skinned_mesh = Some((vbuf, ibuf, indices.len() as u32));
    }

    pub fn update_skin_palette(&mut self, mats: &[glam::Mat4]) {
        // Upload contiguous mat4 array

        let mut data: Vec<f32> = Vec::with_capacity(mats.len() * 16);
        for m in mats {
            data.extend_from_slice(&m.to_cols_array());
        }
        self.queue
            .write_buffer(&self.skin_palette_buf, 0, bytemuck::cast_slice(&data));
    }
}

#[cfg(test)]
mod mat_integration_tests {
    use astraweave_materials::{Graph, MaterialPackage, Node};

    #[test]
    fn material_package_composes_valid_shader() {
        let mut nodes = std::collections::BTreeMap::new();
        nodes.insert(
            "uv".into(),
            Node::Constant3 {
                value: [0.0, 0.0, 0.0],
            },
        );
        nodes.insert(
            "base_tex".into(),
            Node::Texture2D {
                id: "albedo".into(),
                uv: "uv".into(),
            },
        );
        let g = Graph {
            nodes,
            base_color: "base_tex".into(),
            mr: None,
            normal: None,
            clearcoat: None,
            anisotropy: None,
            transmission: None,
        };
        let pkg = MaterialPackage::from_graph(&g).expect("compile");
        // Compose shader text and validate via naga
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
        let full = format!(
            "{}\n{}\n@fragment fn fs_main(@location(0) uv: vec2<f32>) -> @location(0) vec4<f32> {{ let m = eval_material(uv); return vec4<f32>(m.base,1.0); }}\n",
            decls, pkg.wgsl
        );
        let res = naga::front::wgsl::parse_str(&full);
        assert!(
            res.is_ok(),
            "Material-composed WGSL failed to parse: {:?}",
            res.err()
        );
    }
}

// --- Simple CPU frustum culling for instances ---

fn inside_frustum_sphere(center: glam::Vec3, radius: f32, planes: &[(glam::Vec3, f32)]) -> bool {
    for (n, d) in planes.iter() {
        if n.dot(center) + d < -radius {
            return false;
        }
    }
    true
}

// --- CSM utilities ---
fn frustum_corners_ws(cam: &crate::camera::Camera, near: f32, far: f32) -> [glam::Vec3; 8] {
    let dir = crate::camera::Camera::dir(cam.yaw, cam.pitch);
    let right = dir.cross(glam::Vec3::Y).normalize();
    let up = glam::Vec3::Y;
    let h_near = (cam.fovy * 0.5).tan() * near;
    let w_near = h_near * cam.aspect.max(0.01);
    let h_far = (cam.fovy * 0.5).tan() * far;
    let w_far = h_far * cam.aspect.max(0.01);
    let c_near = cam.position + dir * near;
    let c_far = cam.position + dir * far;
    [
        c_near + up * h_near - right * w_near, // near TL
        c_near + up * h_near + right * w_near, // near TR
        c_near - up * h_near - right * w_near, // near BL
        c_near - up * h_near + right * w_near, // near BR
        c_far + up * h_far - right * w_far,    // far TL
        c_far + up * h_far + right * w_far,    // far TR
        c_far - up * h_far - right * w_far,    // far BL
        c_far - up * h_far + right * w_far,    // far BR
    ]
}

fn frustum_center(corners: &[glam::Vec3; 8]) -> glam::Vec3 {
    let mut acc = glam::Vec3::ZERO;
    for c in corners.iter() {
        acc += *c;
    }
    acc / 8.0
}

fn aabb_in_view_space(view: &glam::Mat4, corners_ws: &[glam::Vec3; 8]) -> (glam::Vec3, glam::Vec3) {
    let mut min = glam::Vec3::splat(f32::INFINITY);
    let mut max = glam::Vec3::splat(f32::NEG_INFINITY);
    for &c in corners_ws.iter() {
        let v = *view * glam::Vec4::new(c.x, c.y, c.z, 1.0);
        let p = v.xyz();
        min = min.min(p);
        max = max.max(p);
    }
    (min, max)
}

// End of file
