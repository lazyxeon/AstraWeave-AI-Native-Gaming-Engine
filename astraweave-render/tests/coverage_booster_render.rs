use astraweave_render::{
    camera::Camera,
    clustered_forward::{ClusteredForwardRenderer, ClusterConfig},
    deferred::{GBuffer, GBufferFormats},
    gi::vxgi::{VxgiConfig, VxgiRenderer},
    ibl::{IblManager, IblQuality},
    renderer::Renderer,
    shadow_csm::CsmRenderer,
    texture_streaming::TextureStreamingManager,
    types::{Instance, Mesh, Vertex, SkinnedVertex},
    environment::{SkyRenderer, SkyConfig, TimeOfDay, WeatherSystem},
    effects::{WeatherFx, WeatherKind},
    msaa::MsaaMode,
    material::{MaterialLoadStats, MaterialManager},
    animation::{Skeleton, AnimationClip, Transform, Joint, ChannelData, AnimationChannel, Interpolation},
    terrain::TerrainRenderer,
    culling::{InstanceAABB, FrustumPlanes},
};
use astraweave_terrain::WorldConfig;
use glam::{vec3, Mat4, Vec3};
use std::sync::Arc;
use std::collections::HashMap;
use wgpu::util::DeviceExt;

#[tokio::test]
async fn test_render_systems_logic() {
    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: None,
            force_fallback_adapter: true,
        })
        .await
        .unwrap();

    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                memory_hints: Default::default(),
                trace: Default::default(),
            },
        )
        .await
        .unwrap();

    let width = 1024;
    let height = 768;
    let format = wgpu::TextureFormat::Rgba8UnormSrgb;

    // 1. GBuffer & Deferred
    let gbuffer = GBuffer::new(&device, width, height, GBufferFormats::default());
    assert_eq!(gbuffer.width, width);

    // 2. CSM Shadows
    let mut csm = CsmRenderer::new(&device).unwrap();
    let camera = Camera {
        position: vec3(0.0, 5.0, 10.0),
        yaw: 0.0,
        pitch: 0.0,
        fovy: 45.0f32.to_radians(),
        aspect: width as f32 / height as f32,
        znear: 0.1,
        zfar: 100.0,
    };
    csm.update_cascades(camera.position, Mat4::IDENTITY, Mat4::IDENTITY, -Vec3::Y, 0.1, 100.0);

    // 3. Clustered Forward
    let mut forward = ClusteredForwardRenderer::new(&device, ClusterConfig::default());
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
    
    // 4. VXGI
    let vxgi_config = VxgiConfig::default();
    let mut vxgi = VxgiRenderer::new(&device, vxgi_config);
    vxgi.update_voxel_field(&mut encoder);

    // 5. Texture Streaming
    let mut streaming = TextureStreamingManager::new(100); // 100MB
    let dummy_data = vec![0u8; 64 * 64 * 4];
    let _tex = device.create_texture_with_data(
        &queue,
        &wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d { width: 64, height: 64, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        },
        wgpu::util::TextureDataOrder::default(),
        &dummy_data,
    );

    // 6. Environment & Sky
    let mut sky = SkyRenderer::new(SkyConfig::default());
    let _ = sky.init_gpu_resources(&device, format);
    sky.update(0.016);

    // 7. Weather
    let mut weather_fx = WeatherFx::new(&device, 100);
    weather_fx.set_kind(WeatherKind::Rain);
    weather_fx.update(&queue, 0.016);

    let mut weather_sys = WeatherSystem::new();
    weather_sys.update(0.016);

    // 8. IBL
    let _ibl = IblManager::new(&device, IblQuality::High).unwrap();

    // 9. Material Loading (Internal Logic)
    let mut stats = MaterialLoadStats::default();
    let _ = MaterialManager::new();
    stats.biome = "forest".to_string();
    stats.layers_total = 1;
    let _ = stats.concise_summary();
    
    queue.submit(Some(encoder.finish()));
}

#[tokio::test]
async fn test_renderer_full_pipeline() {
    use astraweave_render::Renderer;
    use astraweave_render::Camera;
    use glam::vec3;

    // Initialize headless renderer
    let mut renderer = Renderer::new_headless(800, 600).await
        .expect("Failed to create headless renderer");

    // 1. Add a model and instances to exercise model rendering loop
    let (v, i) = astraweave_render::primitives::cube();
    let mesh = renderer.create_mesh_from_arrays(
        &v.iter().map(|v| v.position).collect::<Vec<_>>(),
        &v.iter().map(|v| v.normal).collect::<Vec<_>>(),
        &i,
    );
    
    let model_instances = vec![
        Instance {
            transform: glam::Mat4::from_translation(glam::vec3(2.0, 0.0, 0.0)),
            color: [1.0, 0.0, 0.0, 1.0],
            material_id: 0,
        },
        Instance {
            transform: glam::Mat4::from_translation(glam::vec3(-2.0, 0.0, 0.0)),
            color: [0.0, 1.0, 0.0, 1.0],
            material_id: 0,
        },
    ];
    renderer.add_model("test_cube", mesh, &model_instances);

    // 2. Add a skinned mesh to exercise skinning shader paths
    let skinned_verts = vec![
        SkinnedVertex {
            position: [0.0, 0.0, 0.0],
            normal: [0.0, 1.0, 0.0],
            tangent: [1.0, 0.0, 0.0, 1.0],
            uv: [0.0, 0.0],
            joints: [0, 0, 0, 0],
            weights: [1.0, 0.0, 0.0, 0.0],
        },
    ];
    let skinned_indices = vec![0, 0, 0]; // Dummy triangle
    renderer.set_skinned_mesh(&skinned_verts, &skinned_indices);
    renderer.update_skin_palette(&[glam::Mat4::IDENTITY]);

    // 3. Exercise terrain rendering
    let terrain_config = astraweave_terrain::WorldConfig::default();
    let mut terrain_renderer = TerrainRenderer::new(terrain_config);
    let chunk_id = astraweave_terrain::ChunkId::new(0, 0);
    let terrain_mesh = terrain_renderer.get_or_generate_chunk_mesh(chunk_id).unwrap();
    
    let terrain_render_mesh = renderer.create_mesh_from_arrays(
        &terrain_mesh.vertices.iter().map(|v| v.position).collect::<Vec<_>>(),
        &terrain_mesh.vertices.iter().map(|v| v.normal).collect::<Vec<_>>(),
        &terrain_mesh.indices,
    );
    renderer.add_model("terrain_chunk", terrain_render_mesh, &[Instance {
        transform: glam::Mat4::IDENTITY,
        color: [1.0, 1.0, 1.0, 1.0],
        material_id: 0,
    }]);

    // 4. Exercise culling logic
    let _aabb = InstanceAABB::from_transform(
        &glam::Mat4::IDENTITY,
        glam::vec3(-1.0, -1.0, -1.0),
        glam::vec3(1.0, 1.0, 1.0),
        0,
    );
    let _planes = FrustumPlanes::from_view_proj(&glam::Mat4::IDENTITY);

    // 5. Exercise animation sampling
    let skeleton = Skeleton {
        joints: vec![Joint {
            name: "root".to_string(),
            parent_index: None,
            inverse_bind_matrix: glam::Mat4::IDENTITY,
            local_transform: Transform::default(),
        }],
        root_indices: vec![0],
    };
    let clip = AnimationClip {
        name: "test".to_string(),
        duration: 1.0,
        channels: vec![AnimationChannel {
            target_joint_index: 0,
            times: vec![0.0, 1.0],
            data: ChannelData::Translation(vec![glam::Vec3::ZERO, glam::Vec3::ONE]),
            interpolation: Interpolation::Linear,
        }],
    };
    let _sampled = clip.sample(0.5, &skeleton);

    // Setup camera
    let camera = Camera {
        position: vec3(0.0, 5.0, 10.0),
        yaw: 0.0,
        pitch: 0.0,
        fovy: 45.0f32.to_radians(),
        aspect: 800.0 / 600.0,
        znear: 0.1,
        zfar: 100.0,
    };
    renderer.update_camera(&camera);

    // Setup some instances to render
    let instances = vec![
        Instance {
            transform: glam::Mat4::from_translation(vec3(0.0, 0.0, 0.0)),
            color: [1.0, 1.0, 1.0, 1.0],
            material_id: 0,
        },
        Instance {
            transform: glam::Mat4::from_translation(vec3(2.0, 0.0, 0.0)),
            color: [1.0, 0.0, 0.0, 1.0],
            material_id: 0,
        },
    ];
    renderer.update_instances(&instances);

    // Update other systems
    renderer.set_material_params([1.0, 1.0, 1.0, 1.0], 0.5, 0.1);
    renderer.set_weather(astraweave_render::WeatherKind::Rain);
    renderer.tick_weather(0.016);
    renderer.tick_environment(0.016);

    // Create a dummy texture to render into
    let device = renderer.device();
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("dummy_render_target"),
        size: wgpu::Extent3d {
            width: 800,
            height: 600,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("booster_encoder"),
    });

    // Execute the main rendering logic
    let _ = renderer.draw_into(&view, &mut encoder);

    // Submit (optional, but good for coverage of queue logic)
    renderer.queue().submit(std::iter::once(encoder.finish()));
}
