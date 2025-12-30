use astraweave_render::{
    camera::{Camera, CameraController, CameraMode},
    clustered_forward::{ClusteredForwardRenderer, ClusterConfig},
    deferred::{GBuffer, GBufferFormats},
    gi::vxgi::{VxgiConfig, VxgiRenderer},
    ibl::{IblManager, IblQuality},
    renderer::Renderer,
    shadow_csm::CsmRenderer,
    texture_streaming::TextureStreamingManager,
    types::{Instance, SkinnedVertex},
    environment::{SkyRenderer, SkyConfig, WeatherSystem},
    effects::{WeatherFx, WeatherKind},
    material::{MaterialLoadStats, MaterialManager},
    animation::{Skeleton, AnimationClip, Transform, Joint, ChannelData, AnimationChannel, Interpolation},
    terrain::TerrainRenderer,
    culling::{InstanceAABB, FrustumPlanes, CullingPipeline},
    gpu_particles::GpuParticleSystem,
    clustered_megalights::MegaLightsRenderer,
    lod_generator::LODGenerator,
    vertex_compression::OctahedralEncoder,
    texture::Texture,
    instancing::{InstanceBatch, Instance as InstancingInstance, InstanceManager, InstancePatternBuilder, InstanceRaw},
};
use astraweave_materials::{Graph, Node, MaterialPackage};
use astraweave_terrain::WorldConfig;
use aw_asset_cli::{ColorSpace, CompressionFormat, TextureMetadata};
use image::{RgbaImage, Rgba};
use glam::{vec3, Mat4, Vec3, Vec2, Quat};
use std::sync::Arc;
use std::fs;
use tempfile::tempdir;
use wgpu::util::DeviceExt;

#[tokio::test]
async fn test_render_advanced_features() {
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

    let config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        width: 1024,
        height: 768,
        present_mode: wgpu::PresentMode::Fifo,
        alpha_mode: wgpu::CompositeAlphaMode::Auto,
        view_formats: vec![],
        desired_maximum_frame_latency: 2,
    };

    println!("Initializing Renderer...");
    let mut renderer = Renderer::new_from_device(device, queue, None, config).await.unwrap();
    println!("Renderer initialized.");

    // 1. Material Package & Shader Generation
    println!("Testing Material Package...");
    let mut g = Graph {
        nodes: std::collections::BTreeMap::new(),
        base_color: "color".to_string(),
        mr: None,
        normal: None,
        clearcoat: None,
        anisotropy: None,
        transmission: None,
    };
    g.nodes.insert("color".to_string(), Node::Constant3 { value: [1.0, 0.0, 0.0] });
    let pkg = MaterialPackage::from_graph(&g).expect("compile");
    let _shader = renderer.shader_from_material_package(&pkg);
    let _bgl = renderer.bgl_from_material_package(&pkg);
    println!("Material Package tested.");

    // 2. Cinematics
    println!("Testing Cinematics...");
    let timeline_json = r#"{
        "name": "test",
        "duration": 10.0,
        "tracks": []
    }"#;
    renderer.load_timeline_json(timeline_json).unwrap();
    assert!(renderer.save_timeline_json().is_some());
    renderer.play_timeline();
    renderer.seek_timeline(5.0);
    let mut cam = Camera {
        position: Vec3::ZERO,
        yaw: 0.0,
        pitch: 0.0,
        fovy: 45.0f32.to_radians(),
        aspect: 1.0,
        znear: 0.1,
        zfar: 100.0,
    };
    renderer.tick_cinematics(0.1, &mut cam);
    renderer.stop_timeline();
    println!("Cinematics tested.");

    // 3. IBL
    println!("Testing IBL...");
    renderer.bake_environment(IblQuality::Low).unwrap();
    {
        let ibl = renderer.ibl_mut();
        ibl.sun_elevation = 0.5;
        ibl.sun_azimuth = 1.0;
        assert!(ibl.enabled);
    }
    println!("IBL tested.");

    // 4. Resize
    println!("Testing Resize...");
    renderer.resize(800, 600);
    renderer.resize(0, 0); // Should return early
    println!("Resize tested.");

    // 5. Material Validation & Manager
    println!("Testing Material Manager...");
    let mut mat_manager = astraweave_render::material::MaterialManager::new();
    let _bgl = mat_manager.get_or_create_bind_group_layout(renderer.device());
    
    let mat_gpu = astraweave_render::material::MaterialGpu::neutral(0);
    assert_eq!(mat_gpu.texture_indices[0], 0);
    println!("Material Manager tested.");

    // Final poll to ensure all GPU work is done before drop
    renderer.device().poll(wgpu::MaintainBase::Wait);
    println!("Test completed successfully.");
}

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
    csm.upload_to_gpu(&queue, &device);

    // Exercise render_shadow_maps with dummy buffers
    let v_data = [0.0f32; 12];
    let i_data = [0u32; 3];
    let v_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("v_buf"),
        contents: bytemuck::cast_slice(&v_data),
        usage: wgpu::BufferUsages::VERTEX,
    });
    let i_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("i_buf"),
        contents: bytemuck::cast_slice(&i_data),
        usage: wgpu::BufferUsages::INDEX,
    });
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
    csm.render_shadow_maps(&mut encoder, &v_buf, &i_buf, 3);

    // 3. Clustered Forward
    let forward = ClusteredForwardRenderer::new(&device, ClusterConfig::default());
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
    
    // 4. VXGI
    let vxgi_config = VxgiConfig::default();
    let mut vxgi = VxgiRenderer::new(&device, vxgi_config);
    vxgi.update_voxel_field(&mut encoder);

    // 5. Texture Streaming
    let streaming = TextureStreamingManager::new(100); // 100MB
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
    use astraweave_render::environment::WeatherType;
    weather_sys.set_weather(WeatherType::Rain, 1.0);
    weather_sys.update(0.5);
    let _ = weather_sys.current_weather();
    let _ = weather_sys.target_weather();
    let _ = weather_sys.get_rain_intensity();
    let _ = weather_sys.get_snow_intensity();
    let _ = weather_sys.get_fog_density();
    let _ = weather_sys.get_wind_strength();
    let _ = weather_sys.get_wind_direction();
    let _ = weather_sys.is_raining();
    let _ = weather_sys.is_snowing();
    let _ = weather_sys.is_foggy();
    let _ = weather_sys.get_terrain_color_modifier();
    let _ = weather_sys.get_light_attenuation();
    let _ = WeatherSystem::get_biome_appropriate_weather(astraweave_terrain::BiomeType::Forest);

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

    // 6. Exercise Clustered Lighting with Point Lights
    // renderer.point_lights is private, but the renderer adds default lights if empty
    // which we can trigger by drawing.

    // 7. Exercise GPU Particles
    {
        let device = renderer.device().clone();
        let queue = renderer.queue().clone();
        
        let mut particle_sys = GpuParticleSystem::new(&device, 1000).unwrap();
        let emitter_params = astraweave_render::gpu_particles::EmitterParams {
            position: [0.0, 0.0, 0.0, 1.0],
            velocity: [0.0, 1.0, 0.0, 0.0],
            emission_rate: 100.0,
            lifetime: 2.0,
            velocity_randomness: 0.2,
            delta_time: 0.016,
            gravity: [0.0, -9.81, 0.0, 0.0],
            particle_count: 0,
            max_particles: 1000,
            random_seed: 42,
            _padding: 0,
        };
        let mut particle_encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        particle_sys.update(&queue, &mut particle_encoder, &emitter_params);
        queue.submit(Some(particle_encoder.finish()));

        // 8. Exercise MegaLights
        let mut megalights = MegaLightsRenderer::new(&device, (16, 9, 24), 100).unwrap();
        let mut mega_encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        let light_data = vec![astraweave_render::clustered_megalights::GpuLight {
            position: [0.0, 5.0, 0.0, 10.0],
            color: [1.0, 1.0, 1.0, 1.0],
        }];
        let light_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&light_data),
            usage: wgpu::BufferUsages::STORAGE,
        });
        let cluster_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: (16 * 9 * 24 * 32) as u64,
            usage: wgpu::BufferUsages::STORAGE,
            mapped_at_creation: false,
        });
        let count_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: (16 * 9 * 24 * 4) as u64,
            usage: wgpu::BufferUsages::STORAGE,
            mapped_at_creation: false,
        });
        let offset_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: (16 * 9 * 24 * 4) as u64,
            usage: wgpu::BufferUsages::STORAGE,
            mapped_at_creation: false,
        });
        let index_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: (16 * 9 * 24 * 100 * 4) as u64,
            usage: wgpu::BufferUsages::STORAGE,
            mapped_at_creation: false,
        });
        let params_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: 64,
            usage: wgpu::BufferUsages::UNIFORM,
            mapped_at_creation: false,
        });
        let prefix_params_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: 64,
            usage: wgpu::BufferUsages::UNIFORM,
            mapped_at_creation: false,
        });
        
        megalights.update_bind_groups(
            &device, 
            &light_buffer, 
            &cluster_buffer, 
            &count_buffer, 
            &offset_buffer, 
            &index_buffer, 
            &params_buffer, 
            &prefix_params_buffer
        );
        megalights.dispatch(&mut mega_encoder, 1);
        queue.submit(Some(mega_encoder.finish()));

        // 9. Exercise Texture Streaming more thoroughly
        let mut streaming = TextureStreamingManager::new(10);
        streaming.request_texture("nonexistent.png".to_string(), 1, 10.0);
        streaming.process_next_load(&Arc::new(device.clone()), &Arc::new(queue.clone()));
    }

    // Update other systems
    renderer.set_material_params([1.0, 1.0, 1.0, 1.0], 0.5, 0.1);
    renderer.set_weather(astraweave_render::WeatherKind::Rain);
    renderer.tick_weather(0.016);
    renderer.tick_environment(0.016);

    // Create a dummy texture to render into
    let texture = renderer.device().create_texture(&wgpu::TextureDescriptor {
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
    let mut encoder = renderer.device().create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("booster_encoder"),
    });

    // Execute the main rendering logic
    let _ = renderer.draw_into(&view, &mut encoder);

    // Submit (optional, but good for coverage of queue logic)
    renderer.queue().submit(std::iter::once(encoder.finish()));
}

#[tokio::test]
async fn test_render_extra_systems() {
    use astraweave_render::{
        decals::{DecalSystem, Decal},
        water::WaterRenderer,
        lod_generator::{SimplificationMesh, LODConfig},
        vertex_compression::OctahedralEncoder,
        deferred::DeferredRenderer,
        advanced_post::AdvancedPostFx,
    };

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
    let format = wgpu::TextureFormat::Bgra8UnormSrgb;

    // 1. Decals
    let mut decal_sys = DecalSystem::new(&device, 100, 1024, 4);
    decal_sys.add_decal(Decal {
        position: glam::Vec3::ZERO,
        rotation: glam::Quat::IDENTITY,
        scale: glam::Vec3::ONE,
        albedo_tint: [1.0, 1.0, 1.0, 1.0],
        normal_strength: 1.0,
        roughness: 0.5,
        metallic: 0.0,
        blend_mode: astraweave_render::decals::DecalBlendMode::AlphaBlend,
        atlas_uv: ([0.0, 0.0], [1.0, 1.0]),
        fade_duration: 1.0,
        fade_time: 0.0,
    });
    decal_sys.update(&queue, 0.016);

    // 2. Water
    let mut water = WaterRenderer::new(&device, format, wgpu::TextureFormat::Depth32Float);
    water.update(&queue, glam::Mat4::IDENTITY, glam::Vec3::ZERO, 0.016);

    // 3. LOD Generation
    let mesh = SimplificationMesh::new(
        vec![glam::Vec3::ZERO, glam::Vec3::X, glam::Vec3::Y],
        vec![glam::Vec3::Z, glam::Vec3::Z, glam::Vec3::Z],
        vec![[0.0, 0.0], [1.0, 0.0], [0.0, 1.0]],
        vec![0, 1, 2],
    );
    let lod_gen = LODGenerator::new(LODConfig::default());
    let _lods = lod_gen.generate_lods(&mesh);

    // 4. Vertex Compression
    let normal = glam::Vec3::new(0.0, 1.0, 0.0);
    let encoded = OctahedralEncoder::encode(normal);
    let decoded = OctahedralEncoder::decode(encoded);
    assert!((normal - decoded).length() < 0.01);

    // 5. Deferred Renderer
    let deferred = DeferredRenderer::new(&device, width, height).unwrap();
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
    let color_tex = device.create_texture(&wgpu::TextureDescriptor {
        label: None,
        size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });
    let color_view = color_tex.create_view(&wgpu::TextureViewDescriptor::default());
    
    deferred.light_pass(&mut encoder, &color_view);

    // 6. Advanced Post Fx
    let mut post = AdvancedPostFx::new(&device, width, height, format).unwrap();
    
    let tex_desc = wgpu::TextureDescriptor {
        label: None,
        size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba16Float,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    };
    let input_tex = device.create_texture(&tex_desc).create_view(&Default::default());
    let output_tex = device.create_texture(&tex_desc).create_view(&Default::default());
    let velocity_tex = device.create_texture(&tex_desc).create_view(&Default::default());
    let depth_tex = device.create_texture(&wgpu::TextureDescriptor {
        format: wgpu::TextureFormat::Depth32Float,
        ..tex_desc
    }).create_view(&Default::default());

    post.apply_taa(&mut encoder, &color_view, &color_view);
    post.next_frame();
    post.get_taa_jitter();
    
    queue.submit(Some(encoder.finish()));
}

#[tokio::test]
async fn test_render_core_systems() {
    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: None,
            force_fallback_adapter: false,
        })
        .await
        .unwrap();

    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                memory_hints: wgpu::MemoryHints::default(),
                trace: Default::default(),
            },
        )
        .await
        .unwrap();

    // 1. CSM Renderer
    let mut csm = CsmRenderer::new(&device).unwrap();
    csm.update_cascades(
        glam::Vec3::new(0.0, 10.0, 0.0),
        glam::Mat4::IDENTITY,
        glam::Mat4::IDENTITY,
        glam::Vec3::new(0.0, -1.0, 0.0),
        0.1,
        100.0,
    );
    csm.upload_to_gpu(&queue, &device);

    // 2. Material Manager
    let mut mat_manager = MaterialManager::new();
    let _ = mat_manager.get_or_create_bind_group_layout(&device);
    mat_manager.unload_current();
    let _ = mat_manager.current_stats();
    let _ = mat_manager.current_layout();
    let _ = mat_manager.albedo_texture();
    let _ = mat_manager.normal_texture();
    let _ = mat_manager.mra_texture();

    // 3. Texture
    let _white = Texture::create_default_white(&device, &queue, "test_white").unwrap();
    let _normal = Texture::create_default_normal(&device, &queue, "test_normal").unwrap();
    use astraweave_render::texture::TextureUsage;
    let _ = TextureUsage::Albedo.format();
    let _ = TextureUsage::Normal.needs_mipmaps();
    let _ = TextureUsage::MRA.description();

    // 4. Instancing
    let mut batch = InstanceBatch::new(1);
    batch.add_instance(InstancingInstance::identity());
    batch.update_buffer(&device);

    // 5. Culling
    let culling = CullingPipeline::new(&device);
    let aabb = InstanceAABB::new(glam::Vec3::ZERO, glam::Vec3::ONE, 0);
    let vp = glam::Mat4::perspective_rh(45.0, 1.0, 0.1, 100.0);
    let frustum = FrustumPlanes::from_view_proj(&vp);
    let culling_res = culling.create_culling_resources(&device, &[aabb], &frustum);
    
    let cull_encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
    // culling.dispatch(&mut cull_encoder, &culling_res, frustum, 1);
    queue.submit(Some(cull_encoder.finish()));

    // 6. Animation
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
        name: "idle".to_string(),
        duration: 1.0,
        channels: vec![AnimationChannel {
            target_joint_index: 0,
            times: vec![0.0, 1.0],
            data: ChannelData::Translation(vec![glam::Vec3::ZERO, glam::Vec3::X]),
            interpolation: Interpolation::Linear,
        }],
    };
    let _transforms = clip.sample(0.5, &skeleton);

    let mut anim_state = astraweave_render::animation::AnimationState::default();
    anim_state.play();
    anim_state.update(0.1, 1.0);
    anim_state.pause();
    anim_state.stop();
    anim_state.restart();

    let _ = astraweave_render::animation::compute_joint_matrices(&skeleton, &_transforms);
    let _ = astraweave_render::animation::skin_vertex_cpu(
        glam::Vec3::ZERO,
        glam::Vec3::Y,
        [0, 0, 0, 0],
        [1.0, 0.0, 0.0, 0.0],
        &[glam::Mat4::IDENTITY],
    );
    let _ = astraweave_render::animation::JointPalette::from_matrices(&[glam::Mat4::IDENTITY]);

    // 7. IBL Manager
    let ibl = IblManager::new(&device, IblQuality::High).unwrap();
    // ibl.sun_elevation = 0.5;
    // ibl.sun_azimuth = 1.0;
    let ibl_encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
    // ibl.update_procedural(&device, &queue, &mut ibl_encoder, 0.0);
    queue.submit(Some(ibl_encoder.finish()));

    // 8. Terrain Renderer
    let mut terrain = TerrainRenderer::new(WorldConfig::default());
    let _ = terrain.get_or_generate_chunk_mesh(astraweave_terrain::ChunkId::new(0, 0));

    // 9. Vertex Compression
    let _oct = OctahedralEncoder::encode(glam::Vec3::Y);

    // 10. Clustered Forward & MegaLights
    let cluster_config = astraweave_render::clustered_forward::ClusterConfig::default();
    let mut clustered_renderer = astraweave_render::clustered_forward::ClusteredForwardRenderer::new(&device, cluster_config);
    
    let lights = vec![
        astraweave_render::clustered_forward::GpuLight::new(
            glam::Vec3::new(0.0, 5.0, 0.0),
            10.0,
            glam::Vec3::new(1.0, 1.0, 1.0),
            1.0,
        ),
    ];
    clustered_renderer.update_lights(lights);
    clustered_renderer.build_clusters(&queue, glam::Mat4::IDENTITY, glam::Mat4::IDENTITY, (1024, 768));
    
    let _ = clustered_renderer.bind_group_layout();
    let _ = clustered_renderer.bind_group();
    let _ = clustered_renderer.config();
    let _ = clustered_renderer.light_count();

    #[cfg(feature = "megalights")]
    {
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        clustered_renderer.build_clusters_with_encoder(
            &device,
            &queue,
            &mut encoder,
            glam::Mat4::IDENTITY,
            glam::Mat4::IDENTITY,
            (1024, 768),
        );
    }

    println!("Core systems test completed successfully");
    let _ = device.poll(wgpu::MaintainBase::Wait);
}

#[tokio::test]
async fn test_render_loop_and_materials() {
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

    let config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        width: 1024,
        height: 768,
        present_mode: wgpu::PresentMode::Fifo,
        alpha_mode: wgpu::CompositeAlphaMode::Auto,
        view_formats: vec![],
        desired_maximum_frame_latency: 2,
    };

    let mut renderer = Renderer::new_from_device(device, queue, None, config).await.unwrap();

    // 1. Test Renderer::draw_into
    println!("Testing Renderer::draw_into...");
    let target_tex = renderer.device().create_texture(&wgpu::TextureDescriptor {
        label: Some("test-target"),
        size: wgpu::Extent3d {
            width: 1024,
            height: 768,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });
    let target_view = target_tex.create_view(&wgpu::TextureViewDescriptor::default());
    let mut encoder = renderer.device().create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("test-encoder"),
    });
    
    renderer.draw_into(&target_view, &mut encoder).unwrap();
    renderer.queue().submit(std::iter::once(encoder.finish()));
    let _ = renderer.device().poll(wgpu::MaintainBase::Wait);
    println!("Renderer::draw_into tested.");

    // 2. Test MaterialManager::load_biome
    println!("Testing MaterialManager::load_biome...");
    let temp_dir = tempdir().unwrap();
    let biome_dir = temp_dir.path();
    
    // Create materials.toml
    let materials_toml = r#"
[biome]
name = "test_biome"

[[layer]]
key = "grass"
albedo = "grass_albedo.png"
normal = "grass_normal.png"
mra = "grass_mra.png"
tiling = [2.0, 2.0]
triplanar_scale = 1.0
"#;
    fs::write(biome_dir.join("materials.toml"), materials_toml).unwrap();
    
    // Create arrays.toml
    let arrays_toml = r#"
[layers]
grass = 0
"#;
    fs::write(biome_dir.join("arrays.toml"), arrays_toml).unwrap();
    
    // Create dummy textures and metadata
    let create_dummy_tex = |name: &str, color: [u8; 4], color_space: ColorSpace| {
        let img_path = biome_dir.join(name);
        let img = RgbaImage::from_pixel(1024, 1024, Rgba(color));
        img.save(&img_path).unwrap();
        
        let meta = TextureMetadata {
            source_path: name.to_string(),
            output_path: name.to_string(),
            color_space,
            normal_y_convention: None,
            compression: CompressionFormat::None,
            mip_levels: 11, // 1024x1024 has 11 mips
            dimensions: (1024, 1024),
            sha256: "fake-hash".to_string(),
        };
        let meta_path = img_path.with_extension("png.meta.json");
        fs::write(meta_path, serde_json::to_string(&meta).unwrap()).unwrap();
    };
    
    create_dummy_tex("grass_albedo.png", [100, 200, 100, 255], ColorSpace::Srgb);
    create_dummy_tex("grass_normal.png", [128, 128, 255, 255], ColorSpace::Linear);
    create_dummy_tex("grass_mra.png", [0, 128, 255, 255], ColorSpace::Linear);
    
    let mut mat_manager = MaterialManager::new();
    mat_manager.load_biome(renderer.device(), renderer.queue(), biome_dir).await.unwrap();
    
    let bgl = mat_manager.get_or_create_bind_group_layout(renderer.device()).clone();
    let _bg = mat_manager.create_bind_group(renderer.device(), &bgl).unwrap();
    
    assert!(mat_manager.current_stats().is_some());
    assert!(mat_manager.current_layout().is_some());
    
    // Test reload
    mat_manager.reload_biome(renderer.device(), renderer.queue(), biome_dir).await.unwrap();
    println!("MaterialManager::load_biome tested.");
}

#[test]
fn test_camera_and_controller() {
    // Test Camera struct
    let mut camera = Camera {
        position: Vec3::new(0.0, 5.0, 10.0),
        yaw: 0.0,
        pitch: 0.0,
        fovy: 60f32.to_radians(),
        aspect: 16.0 / 9.0,
        znear: 0.1,
        zfar: 1000.0,
    };

    // Test view and projection matrices
    let view = camera.view_matrix();
    let proj = camera.proj_matrix();
    let vp = camera.vp();
    assert!(!view.is_nan());
    assert!(!proj.is_nan());
    assert!(!vp.is_nan());

    // Test direction calculation
    let dir = Camera::dir(0.0, 0.0);
    assert!((dir.length() - 1.0).abs() < 0.001);

    // Test CameraController
    let mut controller = CameraController::new(10.0, 0.002);
    
    // Test keyboard input - first test coverage by pressing each key
    controller.process_keyboard(winit::keyboard::KeyCode::KeyW, true);  // forward
    controller.process_keyboard(winit::keyboard::KeyCode::KeyW, false); // release
    controller.process_keyboard(winit::keyboard::KeyCode::KeyS, true);  // back
    controller.process_keyboard(winit::keyboard::KeyCode::KeyS, false);
    controller.process_keyboard(winit::keyboard::KeyCode::KeyA, true);  // left
    controller.process_keyboard(winit::keyboard::KeyCode::KeyA, false);
    controller.process_keyboard(winit::keyboard::KeyCode::KeyD, true);  // right
    controller.process_keyboard(winit::keyboard::KeyCode::KeyD, false);
    controller.process_keyboard(winit::keyboard::KeyCode::Space, true);  // up
    controller.process_keyboard(winit::keyboard::KeyCode::Space, false);
    controller.process_keyboard(winit::keyboard::KeyCode::KeyQ, true);  // down
    controller.process_keyboard(winit::keyboard::KeyCode::KeyQ, false);
    controller.process_keyboard(winit::keyboard::KeyCode::KeyE, true);  // roll
    controller.process_keyboard(winit::keyboard::KeyCode::KeyC, true);  // roll
    controller.process_keyboard(winit::keyboard::KeyCode::ShiftLeft, true);  // sprint
    controller.process_keyboard(winit::keyboard::KeyCode::ControlLeft, true);  // precision
    
    // Test mouse button
    controller.process_mouse_button(winit::event::MouseButton::Right, true);
    assert!(controller.is_dragging());
    
    // Test mouse move and delta
    controller.begin_frame();
    controller.process_mouse_move(&mut camera, Vec2::new(100.0, 100.0));
    controller.process_mouse_delta(&mut camera, Vec2::new(10.0, 5.0));
    
    // Test scroll (zoom)
    let initial_fov = camera.fovy;
    controller.process_scroll(&mut camera, 1.0);
    assert!(camera.fovy != initial_fov);
    
    // Now test actual movement - press W only (forward)
    controller.process_keyboard(winit::keyboard::KeyCode::KeyW, true);
    
    // Test update - camera should move forward
    let initial_pos = camera.position;
    controller.update_camera(&mut camera, 0.5);  // Use longer dt for visible movement
    // Position should change since W is pressed
    let moved = (camera.position - initial_pos).length();
    assert!(moved > 0.001, "Camera should have moved, but distance was {}", moved);
    
    // Test mode toggle
    controller.toggle_mode(&mut camera);
    assert!(matches!(controller.mode, CameraMode::Orbit));
    
    // Test orbit target
    controller.set_orbit_target(Vec3::new(0.0, 0.0, 0.0), &mut camera);
    
    // Test scroll in orbit mode
    controller.process_scroll(&mut camera, 1.0);
    
    // Update in orbit mode
    controller.update_camera(&mut camera, 0.016);
    
    // Toggle back
    controller.toggle_mode(&mut camera);
    assert!(matches!(controller.mode, CameraMode::FreeFly));
    
    // Release keys
    controller.process_keyboard(winit::keyboard::KeyCode::KeyW, false);
    controller.process_mouse_button(winit::event::MouseButton::Right, false);
    assert!(!controller.is_dragging());
    
    println!("Camera and CameraController tested.");
}

#[tokio::test]
async fn test_instancing_system() {
    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: None,
            force_fallback_adapter: true,
        })
        .await
        .unwrap();

    let (device, _queue) = adapter
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

    // Test InstanceRaw
    let raw = InstanceRaw::from_transform(Vec3::ONE, Quat::IDENTITY, Vec3::splat(2.0));
    assert!(raw.model[0][0] != 0.0);
    
    let raw_mat = InstanceRaw::from_matrix(Mat4::IDENTITY);
    assert_eq!(raw_mat.model[0][0], 1.0);
    
    let _desc = InstanceRaw::desc();

    // Test InstancingInstance
    let inst = InstancingInstance::new(Vec3::new(1.0, 2.0, 3.0), Quat::IDENTITY, Vec3::ONE);
    assert_eq!(inst.position, Vec3::new(1.0, 2.0, 3.0));
    
    let _identity = InstancingInstance::identity();
    let _raw = inst.to_raw();

    // Test InstanceBatch
    let mut batch = InstanceBatch::new(42);
    assert_eq!(batch.instance_count(), 0);
    
    batch.add_instance(InstancingInstance::identity());
    batch.add_instance(InstancingInstance::new(Vec3::X, Quat::IDENTITY, Vec3::ONE));
    assert_eq!(batch.instance_count(), 2);
    
    batch.update_buffer(&device);
    assert!(batch.buffer.is_some());
    
    batch.clear();
    assert_eq!(batch.instance_count(), 0);
    
    batch.update_buffer(&device);
    assert!(batch.buffer.is_none());

    // Test InstanceManager
    let mut manager = InstanceManager::new();
    
    manager.add_instance(1, InstancingInstance::identity());
    manager.add_instance(1, InstancingInstance::identity());
    manager.add_instance(2, InstancingInstance::identity());
    
    assert_eq!(manager.total_instances(), 3);
    assert_eq!(manager.batch_count(), 2);
    
    let instances = vec![
        InstancingInstance::identity(),
        InstancingInstance::identity(),
    ];
    manager.add_instances(3, instances);
    assert_eq!(manager.total_instances(), 5);
    
    manager.update_buffers(&device);
    
    let _batch = manager.get_batch(1);
    let _batch_mut = manager.get_batch_mut(1);
    
    for _batch in manager.batches() {
        // iterate
    }
    
    let saved = manager.draw_calls_saved();
    let reduction = manager.draw_call_reduction_percent();
    assert!(saved > 0);
    assert!(reduction > 0.0);
    
    manager.clear();
    assert_eq!(manager.total_instances(), 0);

    // Test InstancePatternBuilder
    let grid_instances = InstancePatternBuilder::new()
        .grid(3, 3, 2.0)
        .build();
    assert_eq!(grid_instances.len(), 9);
    
    let circle_instances = InstancePatternBuilder::new()
        .circle(8, 5.0)
        .build();
    assert_eq!(circle_instances.len(), 8);
    
    let varied_instances = InstancePatternBuilder::new()
        .grid(2, 2, 1.0)
        .with_position_jitter(0.1)
        .with_scale_variation(0.8, 1.2)
        .with_random_rotation_y()
        .build();
    assert_eq!(varied_instances.len(), 4);
    
    let _default_builder = InstancePatternBuilder::default();
    let _default_manager = InstanceManager::default();
    
    println!("Instancing system tested.");
}

/// Test vertex compression module: OctahedralEncoder, HalfFloatEncoder, VertexCompressor
#[test]
fn test_vertex_compression() {
    use astraweave_render::vertex_compression::{
        CompressedVertex, OctahedralEncoder, HalfFloatEncoder, VertexCompressor
    };
    
    // Test OctahedralEncoder encode/decode
    let normals = [
        Vec3::new(0.0, 1.0, 0.0),   // Up
        Vec3::new(0.0, -1.0, 0.0),  // Down
        Vec3::new(1.0, 0.0, 0.0),   // Right
        Vec3::new(-1.0, 0.0, 0.0),  // Left
        Vec3::new(0.0, 0.0, 1.0),   // Forward
        Vec3::new(0.0, 0.0, -1.0),  // Back
        Vec3::new(0.577, 0.577, 0.577).normalize(), // Diagonal
        Vec3::new(-0.707, -0.707, 0.0).normalize(), // Diagonal negative hemisphere
    ];
    
    for normal in &normals {
        let encoded = OctahedralEncoder::encode(*normal);
        let decoded = OctahedralEncoder::decode(encoded);
        
        // Check that the decoded normal is close to the original
        let error = (*normal - decoded).length();
        assert!(error < 0.02, "Normal encoding error too high for {:?}: {}", normal, error);
        
        // Test encoding error calculation
        let err_rad = OctahedralEncoder::encoding_error(*normal);
        assert!(err_rad < 0.02);
    }
    
    // Test HalfFloatEncoder
    let test_values = [0.0f32, 0.5, 1.0, 0.25, 0.75, 0.001, 100.0, -0.5, -100.0];
    
    for val in test_values {
        let encoded = HalfFloatEncoder::encode(val);
        let decoded = HalfFloatEncoder::decode(encoded);
        
        // Half float has limited precision, check within reasonable tolerance
        let error = (val - decoded).abs();
        let rel_error = if val.abs() > 0.001 { error / val.abs() } else { error };
        assert!(rel_error < 0.02 || error < 0.002, "Half float encoding error for {}: {}", val, error);
    }
    
    // Test UV encoding with Vec2
    let uvs = [
        Vec2::new(0.0, 0.0),
        Vec2::new(1.0, 1.0),
        Vec2::new(0.5, 0.5),
        Vec2::new(0.25, 0.75),
    ];
    
    for uv in &uvs {
        let encoded = HalfFloatEncoder::encode_vec2(*uv);
        let decoded = HalfFloatEncoder::decode_vec2(encoded);
        
        let error = (*uv - decoded).length();
        assert!(error < 0.01, "UV encoding error too high for {:?}: {}", uv, error);
    }
    
    // Test CompressedVertex constants
    assert_eq!(CompressedVertex::STANDARD_SIZE, 32);
    assert_eq!(CompressedVertex::COMPRESSED_SIZE, 20);
    assert!((CompressedVertex::MEMORY_REDUCTION - 0.375).abs() < 0.001);
    
    // Test VertexCompressor::compress and decompress
    let positions = vec![
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(1.0, 0.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
    ];
    let normals_arr = vec![
        Vec3::new(0.0, 0.0, 1.0),
        Vec3::new(0.0, 0.0, 1.0),
        Vec3::new(0.0, 0.0, 1.0),
    ];
    let uvs_arr = vec![
        Vec2::new(0.0, 0.0),
        Vec2::new(1.0, 0.0),
        Vec2::new(0.0, 1.0),
    ];
    
    // Test single vertex compression
    let cv = VertexCompressor::compress(positions[0], normals_arr[0], uvs_arr[0]);
    assert!((cv.position[0] - positions[0].x).abs() < 0.001);
    
    // Test single vertex decompression
    let (dec_pos, dec_normal, dec_uv) = VertexCompressor::decompress(&cv);
    assert!((positions[0] - dec_pos).length() < 0.001);
    assert!((normals_arr[0] - dec_normal).length() < 0.02);
    assert!((uvs_arr[0] - dec_uv).length() < 0.01);
    
    // Test batch compression
    let compressed = VertexCompressor::compress_batch(&positions, &normals_arr, &uvs_arr);
    assert_eq!(compressed.len(), 3);
    
    // Verify compression data
    for (i, cv) in compressed.iter().enumerate() {
        assert!((cv.position[0] - positions[i].x).abs() < 0.001);
        assert!((cv.position[1] - positions[i].y).abs() < 0.001);
        assert!((cv.position[2] - positions[i].z).abs() < 0.001);
    }
    
    // Test memory savings calculation
    let (standard_bytes, compressed_bytes, savings_bytes, savings_percent) = 
        VertexCompressor::calculate_savings(1000);
    assert_eq!(standard_bytes, 32000);
    assert_eq!(compressed_bytes, 20000);
    assert_eq!(savings_bytes, 12000);
    assert!((savings_percent - 37.5).abs() < 0.1);
    
    println!("Vertex compression tested.");
}

/// Test texture module: TextureUsage, Texture default factories
#[tokio::test]
async fn test_texture_module() {
    use astraweave_render::texture::{Texture, TextureUsage};
    
    // Test TextureUsage enum
    let usages = [
        TextureUsage::Albedo,
        TextureUsage::Normal,
        TextureUsage::MRA,
        TextureUsage::Emissive,
        TextureUsage::Height,
    ];
    
    for usage in &usages {
        let _format = usage.format();
        let _needs_mips = usage.needs_mipmaps();
        let _desc = usage.description();
        
        // Check format is consistent with usage type
        match usage {
            TextureUsage::Albedo | TextureUsage::Emissive => {
                assert_eq!(usage.format(), wgpu::TextureFormat::Rgba8UnormSrgb);
                assert!(usage.needs_mipmaps());
            }
            TextureUsage::Normal | TextureUsage::Height => {
                assert_eq!(usage.format(), wgpu::TextureFormat::Rgba8Unorm);
                assert!(!usage.needs_mipmaps());
            }
            TextureUsage::MRA => {
                assert_eq!(usage.format(), wgpu::TextureFormat::Rgba8Unorm);
                assert!(usage.needs_mipmaps());
            }
        }
    }
    
    // Get device and queue for texture creation
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
        .request_device(&wgpu::DeviceDescriptor::default())
        .await
        .unwrap();
    
    // Test Texture::create_default_white
    let white_tex = Texture::create_default_white(&device, &queue, "white_test").unwrap();
    assert!(white_tex.texture.size().width == 1);
    assert!(white_tex.texture.size().height == 1);
    
    // Test Texture::create_default_normal
    let normal_tex = Texture::create_default_normal(&device, &queue, "normal_test").unwrap();
    assert!(normal_tex.texture.size().width == 1);
    assert!(normal_tex.texture.size().height == 1);
    
    // Create test PNG data in memory for from_bytes tests
    // Create a simple 4x4 RGBA image manually
    let mut png_data = Vec::new();
    {
        use std::io::Cursor;
        let mut img = image::RgbaImage::new(4, 4);
        for pixel in img.pixels_mut() {
            *pixel = image::Rgba([255, 128, 64, 255]);
        }
        let mut cursor = Cursor::new(&mut png_data);
        img.write_to(&mut cursor, image::ImageFormat::Png).unwrap();
    }
    
    // Test Texture::from_bytes
    let loaded_tex = Texture::from_bytes(&device, &queue, &png_data, "loaded_test").unwrap();
    assert_eq!(loaded_tex.texture.size().width, 4);
    assert_eq!(loaded_tex.texture.size().height, 4);
    
    // Test Texture::from_bytes_with_usage
    let normal_loaded = Texture::from_bytes_with_usage(
        &device, &queue, &png_data, "normal_loaded_test", TextureUsage::Normal
    ).unwrap();
    assert_eq!(normal_loaded.texture.size().width, 4);
    
    println!("Texture module tested.");
}

/// Test texture streaming manager's synchronous API
#[test]
fn test_texture_streaming_sync() {
    use astraweave_render::texture_streaming::TextureStreamingManager;
    use glam::Vec3;
    
    // Create manager with 100MB budget
    let mut manager = TextureStreamingManager::new(100);
    
    // Test request_texture on non-existent texture (returns None, queues for load)
    let result = manager.request_texture("test_texture.png".to_string(), 10, 5.0);
    assert!(result.is_none()); // Not loaded yet
    
    // Request again - still queued but no duplicate
    let result2 = manager.request_texture("test_texture.png".to_string(), 10, 5.0);
    assert!(result2.is_none());
    
    // Test is_resident
    assert!(!manager.is_resident(&"test_texture.png".to_string()));
    assert!(!manager.is_resident(&"nonexistent.png".to_string()));
    
    // Test update_residency
    manager.update_residency(Vec3::new(10.0, 20.0, 30.0));
    
    // Test get_stats
    let stats = manager.get_stats();
    assert_eq!(stats.loaded_count, 0);
    assert!(stats.pending_count >= 1); // At least the one we queued
    assert_eq!(stats.memory_used_bytes, 0);
    assert_eq!(stats.memory_budget_bytes, 100 * 1024 * 1024);
    assert!((stats.memory_used_percent - 0.0).abs() < 0.001);
    
    // Test evict_lru - nothing to evict
    let evicted = manager.evict_lru();
    assert!(!evicted); // No textures loaded to evict
    
    // Test clear
    manager.clear();
    let stats_after = manager.get_stats();
    assert_eq!(stats_after.loaded_count, 0);
    assert_eq!(stats_after.pending_count, 0);
    
    // Test LoadRequest ordering (priority-based)
    // Request multiple textures with different priorities
    manager.request_texture("low_priority.png".to_string(), 1, 10.0);
    manager.request_texture("high_priority.png".to_string(), 10, 10.0);
    manager.request_texture("medium_priority.png".to_string(), 5, 10.0);
    
    // Same priority, different distance
    manager.request_texture("close.png".to_string(), 5, 1.0);  // closer
    manager.request_texture("far.png".to_string(), 5, 100.0); // farther
    
    // Verify stats reflect pending loads
    let stats_queued = manager.get_stats();
    assert!(stats_queued.pending_count >= 5);
    
    println!("Texture streaming (sync API) tested.");
}

/// Test terrain renderer module
#[test]
fn test_terrain_renderer_module() {
    use astraweave_render::terrain::{TerrainRenderer, generate_terrain_preview};
    use astraweave_terrain::WorldConfig;
    
    // Create terrain renderer with default world config
    let config = WorldConfig::default();
    let mut terrain = TerrainRenderer::new(config.clone());
    
    // Test world_generator accessor
    let _gen = terrain.world_generator();
    let _gen_mut = terrain.world_generator_mut();
    
    // Test get_or_generate_chunk_mesh
    use astraweave_terrain::ChunkId;
    let chunk_id = ChunkId::new(0, 0);
    
    let mesh_result = terrain.get_or_generate_chunk_mesh(chunk_id);
    assert!(mesh_result.is_ok());
    let mesh = mesh_result.unwrap();
    
    // Verify the mesh has valid data
    assert!(!mesh.vertices.is_empty());
    assert!(!mesh.indices.is_empty());
    
    // Test get_loaded_mesh
    let loaded = terrain.get_loaded_mesh(chunk_id);
    assert!(loaded.is_some());
    
    // Non-existent chunk should return None
    let not_loaded = terrain.get_loaded_mesh(ChunkId::new(999, 999));
    assert!(not_loaded.is_none());
    
    // Test generate_chunk_complete
    let complete_result = terrain.generate_chunk_complete(ChunkId::new(1, 1));
    assert!(complete_result.is_ok());
    let (complete_mesh, scatter) = complete_result.unwrap();
    assert!(!complete_mesh.vertices.is_empty());
    // Scatter result may or may not have items depending on config
    let _ = scatter; // Just verify we can access it
    
    // Test get_chunks_in_radius - returns Result<Vec<ChunkId>>
    let nearby_result = terrain.get_chunks_in_radius(
        Vec3::new(16.0, 0.0, 16.0),
        1  // radius (chunk count)
    );
    assert!(nearby_result.is_ok());
    let nearby_chunks = nearby_result.unwrap();
    assert!(!nearby_chunks.is_empty());
    
    // Test generate_terrain_preview (standalone function) - returns Result<Vec<f32>>
    let preview_result = generate_terrain_preview(&config, Vec3::ZERO, 64);
    assert!(preview_result.is_ok());
    let preview = preview_result.unwrap();
    assert_eq!(preview.len(), 64 * 64); // Should have one height per pixel
    
    println!("Terrain renderer tested.");
}

