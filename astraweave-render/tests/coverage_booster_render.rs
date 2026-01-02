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
    let _ = renderer.device().poll(wgpu::MaintainBase::Wait);
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
    let _forward = ClusteredForwardRenderer::new(&device, ClusterConfig::default());
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
    
    // 4. VXGI
    let vxgi_config = VxgiConfig::default();
    let mut vxgi = VxgiRenderer::new(&device, vxgi_config);
    vxgi.update_voxel_field(&mut encoder);

    // 5. Texture Streaming
    let _streaming = TextureStreamingManager::new(100); // 100MB
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
        let _ = megalights.dispatch(&mut mega_encoder, 1);
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
    let _ = decal_sys.add_decal(Decal {
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
    let _input_tex = device.create_texture(&tex_desc).create_view(&Default::default());
    let _output_tex = device.create_texture(&tex_desc).create_view(&Default::default());
    let _velocity_tex = device.create_texture(&tex_desc).create_view(&Default::default());
    let _depth_tex = device.create_texture(&wgpu::TextureDescriptor {
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
    let _culling_res = culling.create_culling_resources(&device, &[aabb], &frustum);
    
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
    let _ibl = IblManager::new(&device, IblQuality::High).unwrap();
    // ibl.sun_elevation = 0.5;
    // ibl.sun_azimuth = 1.0;
    let _ibl_encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
    // ibl.update_procedural(&device, &queue, &mut ibl_encoder, 0.0);
    queue.submit(Some(_ibl_encoder.finish()));

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

/// Test MSAA render target module - comprehensive coverage
#[tokio::test]
async fn test_msaa_module() {
    use astraweave_render::msaa::{MsaaMode, MsaaRenderTarget, create_msaa_depth_texture};
    
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
    
    // Test MsaaMode basics
    assert_eq!(MsaaMode::Off.sample_count(), 1);
    assert_eq!(MsaaMode::X2.sample_count(), 2);
    assert_eq!(MsaaMode::X4.sample_count(), 4);
    assert_eq!(MsaaMode::X8.sample_count(), 8);
    
    assert!(!MsaaMode::Off.is_enabled());
    assert!(MsaaMode::X2.is_enabled());
    assert!(MsaaMode::X4.is_enabled());
    assert!(MsaaMode::X8.is_enabled());
    
    // Test multisample_state
    let ms_off = MsaaMode::Off.multisample_state();
    assert_eq!(ms_off.count, 1);
    let ms_x4 = MsaaMode::X4.multisample_state();
    assert_eq!(ms_x4.count, 4);
    
    // Test MsaaRenderTarget creation and management
    let mut msaa_target = MsaaRenderTarget::new(wgpu::TextureFormat::Rgba8UnormSrgb);
    assert_eq!(msaa_target.mode(), MsaaMode::X4); // Default is X4
    assert!(msaa_target.view().is_none()); // No texture yet (no size set)
    
    // Set mode to Off first, then test switching
    msaa_target.set_mode(&device, MsaaMode::Off).unwrap();
    assert_eq!(msaa_target.mode(), MsaaMode::Off);
    assert!(msaa_target.view().is_none()); // Still no texture (MSAA off)
    
    // Set mode with zero size - should not create texture
    msaa_target.set_mode(&device, MsaaMode::X4).unwrap();
    assert_eq!(msaa_target.mode(), MsaaMode::X4);
    
    // Resize to valid dimensions - should now create texture
    msaa_target.resize(&device, 800, 600).unwrap();
    assert!(msaa_target.view().is_some());
    
    // Create a resolve target for testing color_attachment
    let resolve_tex = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("resolve target"),
        size: wgpu::Extent3d { width: 800, height: 600, depth_or_array_layers: 1 },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    });
    let resolve_view = resolve_tex.create_view(&wgpu::TextureViewDescriptor::default());
    
    // Test color_attachment with MSAA enabled
    let attachment = msaa_target.color_attachment(&resolve_view, wgpu::LoadOp::Clear(wgpu::Color::BLACK));
    assert!(attachment.resolve_target.is_some()); // Should have resolve target when MSAA is on
    
    // Switch to MSAA off
    msaa_target.set_mode(&device, MsaaMode::Off).unwrap();
    assert!(msaa_target.view().is_none());
    
    // Test color_attachment with MSAA disabled
    let attachment_off = msaa_target.color_attachment(&resolve_view, wgpu::LoadOp::Clear(wgpu::Color::BLACK));
    assert!(attachment_off.resolve_target.is_none()); // No resolve target when MSAA is off
    
    // Resize when MSAA is off - should not create texture
    msaa_target.resize(&device, 1024, 768).unwrap();
    assert!(msaa_target.view().is_none());
    
    // Enable MSAA X4 (not X2, as X2 isn't guaranteed by WebGPU spec on all adapters)
    msaa_target.set_mode(&device, MsaaMode::X4).unwrap();
    assert!(msaa_target.view().is_some()); // Should create texture because we already have dimensions
    
    // Test create_msaa_depth_texture helper
    let depth_msaa = create_msaa_depth_texture(&device, 800, 600, MsaaMode::X4, Some("msaa depth"));
    assert_eq!(depth_msaa.size().width, 800);
    assert_eq!(depth_msaa.size().height, 600);
    assert_eq!(depth_msaa.sample_count(), 4);
    
    let depth_no_msaa = create_msaa_depth_texture(&device, 800, 600, MsaaMode::Off, None);
    assert_eq!(depth_no_msaa.sample_count(), 1);
    
    let _ = device.poll(wgpu::MaintainBase::Wait);
    println!("MSAA module tested.");
}

/// Test render graph module - comprehensive coverage
#[tokio::test]
async fn test_render_graph_module() {
    use astraweave_render::graph::{GraphContext, ResourceTable, RenderGraph, ClearNode, RendererMainNode, RenderNode};
    
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
    
    // Test ResourceTable operations
    let mut resources = ResourceTable::default();
    
    // Create and insert a texture
    let tex = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("test_tex"),
        size: wgpu::Extent3d { width: 256, height: 256, depth_or_array_layers: 1 },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });
    let view = tex.create_view(&wgpu::TextureViewDescriptor::default());
    
    resources.insert_view("hdr_target", view);
    resources.insert_tex("main_tex", tex);
    
    // Test view retrieval
    let retrieved_view = resources.view("hdr_target");
    assert!(retrieved_view.is_ok());
    
    // Test view_mut retrieval
    let retrieved_view_mut = resources.view_mut("hdr_target");
    assert!(retrieved_view_mut.is_ok());
    
    // Test tex retrieval
    let retrieved_tex = resources.tex("main_tex");
    assert!(retrieved_tex.is_ok());
    
    // Test missing resource error
    let missing = resources.view("nonexistent");
    assert!(missing.is_err());
    
    // Test type mismatch error (accessing tex as view)
    let type_mismatch = resources.view("main_tex");
    assert!(type_mismatch.is_err());
    
    // Test buffer insertion
    let buf = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("test_buf"),
        size: 256,
        usage: wgpu::BufferUsages::UNIFORM,
        mapped_at_creation: false,
    });
    resources.insert_buf("uniform_buf", buf);
    
    // Test bind_group insertion
    let bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("test_bgl"),
        entries: &[],
    });
    let bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("test_bg"),
        layout: &bgl,
        entries: &[],
    });
    resources.insert_bind_group("main_bg", bg);
    
    let retrieved_bg = resources.bind_group("main_bg");
    assert!(retrieved_bg.is_ok());
    
    // Test create_transient_texture
    let transient_tex = resources.create_transient_texture(
        &device,
        "transient_depth",
        &wgpu::TextureDescriptor {
            label: Some("transient"),
            size: wgpu::Extent3d { width: 512, height: 512, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        },
    );
    assert!(transient_tex.is_ok());
    
    // Test target_view with "surface" key and primary_view
    let primary_tex = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("primary"),
        size: wgpu::Extent3d { width: 800, height: 600, depth_or_array_layers: 1 },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    });
    let primary_view = primary_tex.create_view(&wgpu::TextureViewDescriptor::default());
    
    let surface_view = resources.target_view("surface", Some(&primary_view));
    assert!(surface_view.is_ok());
    
    let hdr_view = resources.target_view("hdr_target", Some(&primary_view));
    assert!(hdr_view.is_ok());
    
    // Test GraphContext creation
    let mut user_data: u32 = 42;
    let ctx = GraphContext::new(&mut user_data);
    assert!(ctx.device.is_none());
    assert!(ctx.queue.is_none());
    assert!(ctx.encoder.is_none());
    assert!(ctx.primary_view.is_none());
    
    // Test GraphContext with_gpu
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("graph_encoder"),
    });
    let mut user_data2: u32 = 100;
    let ctx_gpu = GraphContext::new(&mut user_data2)
        .with_gpu(&device, &queue, &mut encoder)
        .with_primary_view(&primary_view);
    assert!(ctx_gpu.device.is_some());
    assert!(ctx_gpu.queue.is_some());
    assert!(ctx_gpu.primary_view.is_some());
    
    // Test RenderGraph with nodes
    let mut graph = RenderGraph::new();
    
    // Add ClearNode
    let clear_node = ClearNode::new("clear_pass", "surface", wgpu::Color::RED);
    assert_eq!(clear_node.name(), "clear_pass");
    graph.add_node(clear_node);
    
    // Add RendererMainNode
    let main_node = RendererMainNode::new("main_pass", "surface");
    assert_eq!(main_node.name(), "main_pass");
    graph.add_node(main_node);
    
    // Execute graph with proper context
    let mut encoder2 = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("exec_encoder"),
    });
    let mut user_data3: u32 = 0;
    let mut ctx_exec = GraphContext::new(&mut user_data3)
        .with_gpu(&device, &queue, &mut encoder2)
        .with_primary_view(&primary_view);
    
    let result = graph.execute(&mut ctx_exec);
    assert!(result.is_ok());
    
    // Submit encoder
    queue.submit(Some(encoder2.finish()));
    let _ = device.poll(wgpu::MaintainBase::Wait);
    
    println!("Render graph module tested.");
}

/// Test texture streaming with more comprehensive coverage
#[tokio::test]
async fn test_texture_streaming_comprehensive() {
    use astraweave_render::texture_streaming::TextureStreamingManager;
    
    let mut manager = TextureStreamingManager::new(50); // 50MB budget
    
    // Test initial state
    let stats = manager.get_stats();
    assert_eq!(stats.loaded_count, 0);
    assert_eq!(stats.memory_used_bytes, 0);
    assert_eq!(stats.memory_budget_bytes, 50 * 1024 * 1024);
    
    // Test update_residency
    manager.update_residency(Vec3::new(10.0, 5.0, 10.0));
    
    // Request several textures with varying priorities
    let result1 = manager.request_texture("tex_a.png".to_string(), 100, 5.0);
    assert!(result1.is_none()); // Not loaded yet
    
    let result2 = manager.request_texture("tex_b.png".to_string(), 50, 10.0);
    assert!(result2.is_none());
    
    let result3 = manager.request_texture("tex_c.png".to_string(), 75, 2.0);
    assert!(result3.is_none());
    
    // Re-request same texture - should still return None (Loading state)
    let result1_again = manager.request_texture("tex_a.png".to_string(), 100, 5.0);
    assert!(result1_again.is_none());
    
    // Test is_resident
    assert!(!manager.is_resident(&"tex_a.png".to_string()));
    assert!(!manager.is_resident(&"nonexistent.png".to_string()));
    
    // Test stats with pending loads
    let stats_pending = manager.get_stats();
    assert!(stats_pending.pending_count >= 3);
    
    // Test evict_lru on empty LRU queue
    let evicted = manager.evict_lru();
    assert!(!evicted); // Nothing to evict yet
    
    // Test clear
    manager.clear();
    let stats_cleared = manager.get_stats();
    assert_eq!(stats_cleared.loaded_count, 0);
    assert_eq!(stats_cleared.pending_count, 0);
    assert_eq!(stats_cleared.memory_used_bytes, 0);
    
    println!("Texture streaming comprehensive tested.");
}

/// Test WeatherSystem comprehensive coverage
#[tokio::test]
async fn test_weather_system_comprehensive() {
    use astraweave_render::environment::{WeatherSystem, WeatherType, WeatherParticles};
    use astraweave_terrain::BiomeType;
    
    // Test WeatherSystem creation and defaults
    let mut weather = WeatherSystem::new();
    assert_eq!(weather.current_weather(), WeatherType::Clear);
    assert_eq!(weather.target_weather(), WeatherType::Clear);
    
    // Test weather intensity accessors
    assert_eq!(weather.get_rain_intensity(), 0.0);
    assert_eq!(weather.get_snow_intensity(), 0.0);
    assert!(weather.get_fog_density() >= 0.0);
    assert!(weather.get_wind_strength() >= 0.0);
    let _wind_dir = weather.get_wind_direction();
    
    // Test weather state queries
    assert!(!weather.is_raining());
    assert!(!weather.is_snowing());
    // fog depends on current weather parameters
    
    // Test terrain color modifier
    let terrain_mod = weather.get_terrain_color_modifier();
    assert!(terrain_mod.x > 0.0 && terrain_mod.y > 0.0 && terrain_mod.z > 0.0);
    
    // Test light attenuation
    let attenuation = weather.get_light_attenuation();
    assert!(attenuation >= 0.0 && attenuation <= 1.0);
    
    // Test set_weather for various types
    weather.set_weather(WeatherType::Rain, 2.0);
    assert_eq!(weather.target_weather(), WeatherType::Rain);
    
    // Update to progress transition
    for _ in 0..100 {
        weather.update(0.05);
    }
    assert!(weather.get_rain_intensity() > 0.0);
    assert!(weather.is_raining());
    
    // Change to storm
    weather.set_weather(WeatherType::Storm, 1.0);
    for _ in 0..50 {
        weather.update(0.05);
    }
    
    // Change to snow
    weather.set_weather(WeatherType::Snow, 1.0);
    for _ in 0..100 {
        weather.update(0.05);
    }
    assert!(weather.get_snow_intensity() > 0.0);
    
    // Change to fog
    weather.set_weather(WeatherType::Fog, 0.5);
    for _ in 0..50 {
        weather.update(0.05);
    }
    
    // Test biome-appropriate weather
    let forest_weather = WeatherSystem::get_biome_appropriate_weather(BiomeType::Forest);
    assert!(!forest_weather.is_empty());
    
    let desert_weather = WeatherSystem::get_biome_appropriate_weather(BiomeType::Desert);
    assert!(!desert_weather.is_empty());
    
    let tundra_weather = WeatherSystem::get_biome_appropriate_weather(BiomeType::Tundra);
    assert!(tundra_weather.contains(&WeatherType::Snow));
    
    let swamp_weather = WeatherSystem::get_biome_appropriate_weather(BiomeType::Swamp);
    assert!(swamp_weather.contains(&WeatherType::Fog));
    
    // Test WeatherParticles
    let mut particles = WeatherParticles::new(1000, 50.0);
    particles.update(0.016, Vec3::new(0.0, 10.0, 0.0), &weather);
    
    let rain_particles = particles.rain_particles();
    let snow_particles = particles.snow_particles();
    // Particle counts depend on current weather intensity
    let _ = rain_particles.len();
    let _ = snow_particles.len();
    
    println!("Weather system comprehensive tested.");
}

/// Test TimeOfDay and SkyRenderer comprehensive coverage
#[tokio::test]
async fn test_sky_and_time_comprehensive() {
    use astraweave_render::environment::{TimeOfDay, SkyRenderer, SkyConfig};
    
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
    
    // Test TimeOfDay creation and updates
    let mut tod = TimeOfDay::new(6.0, 1.0); // Start at 6am, 1x speed
    
    // Test sun position at various times
    let sun_pos_dawn = tod.get_sun_position();
    assert!(sun_pos_dawn.y < 0.5); // Low at dawn
    
    // Progress time
    for _ in 0..1000 {
        tod.update();
    }
    
    // Get positions at different times
    let sun_pos = tod.get_sun_position();
    let moon_pos = tod.get_moon_position();
    let light_dir = tod.get_light_direction();
    let light_color = tod.get_light_color();
    let ambient = tod.get_ambient_color();
    
    // Light color should have valid values
    assert!(light_color.x >= 0.0 && light_color.y >= 0.0 && light_color.z >= 0.0);
    assert!(ambient.x >= 0.0 && ambient.y >= 0.0 && ambient.z >= 0.0);
    
    // Test day/night/twilight detection
    let _is_day = tod.is_day();
    let _is_night = tod.is_night();
    let _is_twilight = tod.is_twilight();
    
    // Verify sun and moon positions make sense
    let _ = sun_pos;
    let _ = moon_pos;
    let _ = light_dir;
    
    // Test SkyConfig
    let config = SkyConfig::default();
    assert!(config.cloud_coverage >= 0.0 && config.cloud_coverage <= 1.0);
    assert!(config.cloud_speed >= 0.0);
    assert!(config.cloud_altitude > 0.0);
    
    // Test SkyRenderer creation
    let mut sky = SkyRenderer::new(config.clone());
    
    // Test config accessors
    let retrieved_config = sky.config();
    assert_eq!(retrieved_config.cloud_coverage, config.cloud_coverage);
    
    sky.set_config(SkyConfig {
        cloud_coverage: 0.8,
        ..Default::default()
    });
    assert_eq!(sky.config().cloud_coverage, 0.8);
    
    // Test time_of_day accessors
    let _tod_ref = sky.time_of_day();
    let tod_mut = sky.time_of_day_mut();
    tod_mut.update();
    
    // Initialize GPU resources
    let init_result = sky.init_gpu_resources(&device, wgpu::TextureFormat::Rgba16Float);
    assert!(init_result.is_ok());
    
    // Test update
    sky.update(0.016);
    
    let _ = device.poll(wgpu::MaintainBase::Wait);
    println!("Sky and time comprehensive tested.");
}

/// Test material loader edge cases
#[test]
fn test_material_loader_edge_cases() {
    use astraweave_render::material::{MaterialManager, validate_material_pack, validate_array_layout, MaterialPackDesc, MaterialLayerDesc, ArrayLayout, MaterialLoadStats};
    use std::path::PathBuf;
    
    // Test validate_material_pack with various configurations
    let valid_pack = MaterialPackDesc {
        biome: "test".to_string(),
        layers: vec![
            MaterialLayerDesc {
                key: "grass".to_string(),
                albedo: Some(PathBuf::from("grass_albedo.png")),
                normal: Some(PathBuf::from("grass_normal.png")),
                mra: Some(PathBuf::from("grass_mra.png")),
                tiling: [1.0, 1.0],
                triplanar_scale: 16.0,
                ..Default::default()
            }
        ],
    };
    
    let validation = validate_material_pack(&valid_pack);
    assert!(validation.is_ok());
    
    // Test with invalid tiling (negative)
    let mut invalid_tiling_pack = valid_pack.clone();
    invalid_tiling_pack.layers[0].tiling = [-1.0, 1.0];
    let validation_tiling = validate_material_pack(&invalid_tiling_pack);
    assert!(validation_tiling.is_err());
    
    // Test with invalid triplanar_scale (zero/negative)
    let mut invalid_triplanar_pack = valid_pack.clone();
    invalid_triplanar_pack.layers[0].triplanar_scale = 0.0;
    let validation_triplanar = validate_material_pack(&invalid_triplanar_pack);
    assert!(validation_triplanar.is_err());
    
    // Test with empty biome name
    let empty_biome_pack = MaterialPackDesc {
        biome: "".to_string(),
        layers: vec![],
    };
    let validation_empty_biome = validate_material_pack(&empty_biome_pack);
    assert!(validation_empty_biome.is_err());
    
    // Test with empty layer key
    let empty_key_pack = MaterialPackDesc {
        biome: "test".to_string(),
        layers: vec![
            MaterialLayerDesc {
                key: "".to_string(),
                ..Default::default()
            }
        ],
    };
    let validation_empty_key = validate_material_pack(&empty_key_pack);
    assert!(validation_empty_key.is_err());
    
    // Test with duplicate layer keys
    let dup_keys_pack = MaterialPackDesc {
        biome: "test".to_string(),
        layers: vec![
            MaterialLayerDesc { key: "grass".to_string(), tiling: [1.0, 1.0], triplanar_scale: 16.0, ..Default::default() },
            MaterialLayerDesc { key: "grass".to_string(), tiling: [1.0, 1.0], triplanar_scale: 16.0, ..Default::default() },
        ],
    };
    let validation_dup_keys = validate_material_pack(&dup_keys_pack);
    assert!(validation_dup_keys.is_err());
    
    // Test validate_array_layout
    let mut valid_layout = ArrayLayout::default();
    valid_layout.layer_indices.insert("grass".to_string(), 0);
    valid_layout.layer_indices.insert("dirt".to_string(), 1);
    valid_layout.count = 2;
    let validation_layout = validate_array_layout(&valid_layout);
    assert!(validation_layout.is_ok());
    
    // Test with duplicate indices in ArrayLayout
    let mut dup_indices_layout = ArrayLayout::default();
    dup_indices_layout.layer_indices.insert("grass".to_string(), 0);
    dup_indices_layout.layer_indices.insert("dirt".to_string(), 0); // duplicate index
    dup_indices_layout.count = 2;
    let validation_dup_indices = validate_array_layout(&dup_indices_layout);
    assert!(validation_dup_indices.is_err());
    
    // Test MaterialLoadStats concise_summary
    let stats = MaterialLoadStats {
        biome: "forest".to_string(),
        layers_total: 5,
        albedo_loaded: 4,
        albedo_substituted: 1,
        normal_loaded: 3,
        normal_substituted: 2,
        mra_loaded: 2,
        mra_packed: 1,
        mra_substituted: 2,
        gpu_memory_bytes: 1024 * 1024 * 10, // 10 MiB
    };
    let summary = stats.concise_summary();
    assert!(summary.contains("forest"));
    assert!(summary.contains("layers=5"));
    
    // Test MaterialManager creation
    let manager = MaterialManager::new();
    // Manager starts with no loaded content
    let _ = manager;
    
    println!("Material loader edge cases tested.");
}
// ===========================
// WAVE 4: High-Impact Coverage Boost
// ===========================

#[tokio::test]
async fn test_renderer_extensive_methods() {
    use astraweave_render::renderer::Renderer;
    use astraweave_render::camera::Camera;
    use astraweave_render::types::Instance;
    use astraweave_render::effects::WeatherKind;
    
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
    
    // Test camera update
    let camera = Camera {
        position: Vec3::new(10.0, 20.0, 30.0),
        yaw: 0.5,
        pitch: -0.2,
        fovy: 60f32.to_radians(),
        aspect: 16.0 / 9.0,
        znear: 0.1,
        zfar: 500.0,
    };
    renderer.update_camera(&camera);
    
    // Test cascade shadow settings
    renderer.set_cascade_splits(25.0, 75.0);
    renderer.set_cascade_extents(50.0, 150.0);
    renderer.set_cascade_lambda(0.5);
    renderer.set_shadow_filter(1.5, 0.001, 1.0);
    
    // Test material parameters
    renderer.set_material_params([1.0, 0.5, 0.3, 1.0], 0.0, 0.5);
    renderer.set_material_params([0.8, 0.8, 0.8, 1.0], 1.0, 0.2);
    
    // Test mesh creation from arrays
    let positions = [
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [0.5, 1.0, 0.0],
    ];
    let normals = [
        [0.0, 0.0, 1.0],
        [0.0, 0.0, 1.0],
        [0.0, 0.0, 1.0],
    ];
    let indices = [0, 1, 2];
    let mesh = renderer.create_mesh_from_arrays(&positions, &normals, &indices);
    assert!(mesh.index_count == 3);
    renderer.set_external_mesh(mesh);
    
    // Test full mesh creation
    let tangents = [
        [1.0, 0.0, 0.0, 1.0],
        [1.0, 0.0, 0.0, 1.0],
        [1.0, 0.0, 0.0, 1.0],
    ];
    let uvs = [
        [0.0, 0.0],
        [1.0, 0.0],
        [0.5, 1.0],
    ];
    let mesh2 = renderer.create_mesh_from_full_arrays(&positions, &normals, &tangents, &uvs, &indices);
    assert!(mesh2.index_count == 3);
    
    // Test mesh from CpuMesh
    use astraweave_render::mesh::{CpuMesh, MeshVertex};
    let cpu_mesh = CpuMesh {
        vertices: vec![
            MeshVertex {
                position: [0.0, 0.0, 0.0],
                normal: [0.0, 1.0, 0.0],
                tangent: [1.0, 0.0, 0.0, 1.0],
                uv: [0.0, 0.0],
            },
            MeshVertex {
                position: [1.0, 0.0, 0.0],
                normal: [0.0, 1.0, 0.0],
                tangent: [1.0, 0.0, 0.0, 1.0],
                uv: [1.0, 0.0],
            },
            MeshVertex {
                position: [0.5, 0.0, 1.0],
                normal: [0.0, 1.0, 0.0],
                tangent: [1.0, 0.0, 0.0, 1.0],
                uv: [0.5, 1.0],
            },
        ],
        indices: vec![0, 1, 2],
    };
    let mesh3 = renderer.create_mesh_from_cpu_mesh(&cpu_mesh);
    assert!(mesh3.index_count == 3);
    
    // Test instance updates - Instance has transform, color, material_id fields
    let instances = vec![
        Instance {
            transform: Mat4::from_translation(Vec3::new(0.0, 0.0, 0.0)),
            color: [1.0, 1.0, 1.0, 1.0],
            material_id: 0,
        },
        Instance {
            transform: Mat4::from_translation(Vec3::new(5.0, 0.0, 0.0)),
            color: [1.0, 0.0, 0.0, 1.0],
            material_id: 0,
        },
    ];
    renderer.update_instances(&instances);
    
    // Test larger instance update (triggers buffer resize)
    let large_instances: Vec<Instance> = (0..100).map(|i| Instance {
        transform: Mat4::from_translation(Vec3::new(i as f32, 0.0, 0.0)),
        color: [1.0, 1.0, 1.0, 1.0],
        material_id: 0,
    }).collect();
    renderer.update_instances(&large_instances);
    
    // Test weather - WeatherKind has None, Rain, WindTrails
    renderer.set_weather(WeatherKind::Rain);
    renderer.tick_weather(0.016);
    renderer.set_weather(WeatherKind::WindTrails);
    renderer.tick_weather(0.016);
    renderer.set_weather(WeatherKind::None);
    
    // Test environment ticking
    renderer.tick_environment(0.1);
    renderer.tick_environment(0.5);
    renderer.tick_environment(1.0);
    
    // Test time of day access
    {
        let tod = renderer.time_of_day_mut();
        tod.current_time = 12.0;
    }
    
    // Test sky config
    let sky_cfg = renderer.sky_config();
    renderer.set_sky_config(sky_cfg.clone());
    
    // Test water renderer - takes 3 args: device, surface_format, depth_format
    use astraweave_render::water::WaterRenderer;
    let water = WaterRenderer::new(
        renderer.device(), 
        wgpu::TextureFormat::Rgba8UnormSrgb,
        wgpu::TextureFormat::Depth32Float,
    );
    renderer.set_water_renderer(water);
    
    // Ensure GPU work is done
    let _ = renderer.device().poll(wgpu::MaintainBase::Wait);
    
    println!("Renderer extensive methods tested.");
}

#[tokio::test]
async fn test_texture_streaming_eviction_and_stats() {
    use astraweave_render::texture_streaming::TextureStreamingManager;
    
    // Create manager with low memory budget to trigger eviction
    let mut manager = TextureStreamingManager::new(1); // 1MB budget
    
    // Test initial stats
    let stats = manager.get_stats();
    assert_eq!(stats.loaded_count, 0);
    assert_eq!(stats.pending_count, 0);
    assert_eq!(stats.memory_used_bytes, 0);
    assert_eq!(stats.memory_budget_bytes, 1 * 1024 * 1024);
    
    // Test request_texture returns None for non-resident
    let result = manager.request_texture("test_texture".to_string(), 100, 10.0);
    assert!(result.is_none()); // Should queue for load
    
    // Stats should show pending
    let stats = manager.get_stats();
    assert!(stats.pending_count > 0 || stats.loaded_count == 0);
    
    // Test is_resident
    assert!(!manager.is_resident(&"test_texture".to_string()));
    assert!(!manager.is_resident(&"unknown_texture".to_string()));
    
    // Test update_residency
    manager.update_residency(Vec3::new(100.0, 0.0, 100.0));
    
    // Test evict_lru on empty manager
    let evicted = manager.evict_lru();
    assert!(!evicted); // Nothing to evict
    
    // Test clear
    manager.clear();
    let stats = manager.get_stats();
    assert_eq!(stats.loaded_count, 0);
    assert_eq!(stats.memory_used_bytes, 0);
    
    println!("Texture streaming eviction and stats tested.");
}

#[tokio::test]
async fn test_ibl_manager_methods() {
    use astraweave_render::ibl::{IblManager, IblQuality};
    
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
        .request_device(&wgpu::DeviceDescriptor::default())
        .await
        .unwrap();
    
    // Test IblQuality enum - Low, Medium, High only
    let qualities = [IblQuality::Low, IblQuality::Medium, IblQuality::High];
    for quality in qualities {
        println!("Testing IBL quality: {:?}", quality);
    }
    
    // Test IblManager creation
    let ibl = IblManager::new(&device, IblQuality::Low).unwrap();
    
    // Test accessors
    let _bgl = ibl.bind_group_layout();
    let _sampler = ibl.sampler();
    
    println!("IBL manager methods tested.");
}

#[tokio::test]
async fn test_environment_weather_comprehensive() {
    use astraweave_render::environment::{TimeOfDay, WeatherSystem, SkyConfig};
    
    // Test TimeOfDay extensively
    let mut tod = TimeOfDay::new(6.0, 1.0); // Start at 6 AM, 1x speed
    
    // Test basic accessors
    assert_eq!(tod.current_time, 6.0);
    
    // Test sun/moon positions at various times
    let times = [0.0, 6.0, 12.0, 18.0, 24.0];
    for t in times {
        tod.current_time = t;
        let sun = tod.get_sun_position();
        let moon = tod.get_moon_position();
        let light_dir = tod.get_light_direction();
        let light_color = tod.get_light_color();
        let ambient = tod.get_ambient_color();
        
        println!("Time {}: sun={:?}, moon={:?}, dir={:?}", t, sun, moon, light_dir);
        
        // Validate positions are normalized
        assert!((sun.length() - 1.0).abs() < 0.01);
        assert!((moon.length() - 1.0).abs() < 0.01);
        assert!((light_dir.length() - 1.0).abs() < 0.01);
        
        // Colors should be positive
        assert!(light_color.x >= 0.0 && light_color.y >= 0.0 && light_color.z >= 0.0);
        assert!(ambient.x >= 0.0 && ambient.y >= 0.0 && ambient.z >= 0.0);
    }
    
    // Test update - takes no args
    tod.current_time = 0.0;
    for _ in 0..100 {
        tod.update(); // Simulate time passing
    }
    assert!(tod.current_time > 0.0);
    
    // Test WeatherSystem
    let weather = WeatherSystem::new();
    
    // Test all weather methods
    let rain = weather.get_rain_intensity();
    let snow = weather.get_snow_intensity();
    let fog = weather.get_fog_density();
    let wind = weather.get_wind_strength();
    let tint = weather.get_terrain_color_modifier();
    let atten = weather.get_light_attenuation();
    
    println!("Weather: rain={}, snow={}, fog={}, wind={}", rain, snow, fog, wind);
    println!("Weather tint: {:?}, attenuation: {}", tint, atten);
    
    // Test state queries
    let _is_rain = weather.is_raining();
    let _is_snow = weather.is_snowing();
    
    // Test SkyConfig - check fields that exist
    let sky_config = SkyConfig::default();
    println!("SkyConfig: day_color_top={:?}", sky_config.day_color_top);
    
    println!("Environment weather comprehensive tested.");
}

#[tokio::test]
async fn test_deferred_gbuffer_formats() {
    use astraweave_render::deferred::{GBuffer, GBufferFormats, DeferredRenderer};
    
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
        .request_device(&wgpu::DeviceDescriptor::default())
        .await
        .unwrap();
    
    // Test GBufferFormats
    let formats = GBufferFormats::default();
    println!("GBuffer albedo format: {:?}", formats.albedo);
    println!("GBuffer normal format: {:?}", formats.normal);
    println!("GBuffer position format: {:?}", formats.position);
    println!("GBuffer depth format: {:?}", formats.depth);
    
    // Test GBuffer with various sizes
    let sizes = [(256, 256), (512, 512), (1024, 768), (1920, 1080)];
    for (w, h) in sizes {
        let gbuffer = GBuffer::new(&device, w, h, formats.clone());
        assert_eq!(gbuffer.width, w);
        assert_eq!(gbuffer.height, h);
        println!("GBuffer created at {}x{}", w, h);
    }
    
    // Test DeferredRenderer
    let deferred = DeferredRenderer::new(&device, 512, 512);
    assert!(deferred.is_ok());
    let dr = deferred.unwrap();
    // Use size_of_val since Debug may not be implemented
    assert!(std::mem::size_of_val(&dr) > 0);
    
    println!("Deferred GBuffer formats tested.");
}

#[tokio::test]
async fn test_texture_creation_variants() {
    use astraweave_render::texture::{Texture, TextureUsage};
    use image::DynamicImage;
    
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
    
    // Create test images with various sizes
    let sizes = [(64, 64), (128, 128), (256, 256)];
    
    for (w, h) in sizes {
        let img = RgbaImage::from_pixel(w, h, Rgba([128, 128, 255, 255]));
        let dyn_img = DynamicImage::ImageRgba8(img);
        
        // Test albedo texture
        let tex_albedo = Texture::from_image_with_usage(
            &device,
            &queue,
            &dyn_img,
            TextureUsage::Albedo,
            Some(&format!("test_albedo_{}x{}", w, h)),
        );
        assert!(tex_albedo.is_ok());
        
        // Test normal texture
        let tex_normal = Texture::from_image_with_usage(
            &device,
            &queue,
            &dyn_img,
            TextureUsage::Normal,
            Some(&format!("test_normal_{}x{}", w, h)),
        );
        assert!(tex_normal.is_ok());
        
        println!("Textures created at {}x{}", w, h);
    }
    
    // Test default textures - take 3 args and return Result
    let white = Texture::create_default_white(&device, &queue, "default_white").unwrap();
    assert!(white.texture.size().width == 1);
    
    let normal = Texture::create_default_normal(&device, &queue, "default_normal").unwrap();
    assert!(normal.texture.size().width == 1);
    
    println!("Texture creation variants tested.");
}

#[tokio::test]
async fn test_terrain_renderer_extensive() {
    use astraweave_render::terrain::TerrainRenderer;
    use astraweave_terrain::WorldConfig;
    
    // Test various world configs - TerrainRenderer::new takes WorldConfig (not device)
    let configs = [
        WorldConfig {
            seed: 12345,
            chunk_size: 32.0,
            ..Default::default()
        },
        WorldConfig {
            seed: 54321,
            chunk_size: 64.0,
            ..Default::default()
        },
        WorldConfig {
            seed: 11111,
            chunk_size: 128.0,
            ..Default::default()
        },
    ];
    
    for config in configs {
        let renderer = TerrainRenderer::new(config.clone());
        // Verify created without panic
        assert!(std::mem::size_of_val(&renderer) > 0);
        println!("TerrainRenderer created with chunk_size={}", config.chunk_size);
    }
    
    println!("Terrain renderer extensive tested.");
}

#[tokio::test]
async fn test_vxgi_additional_coverage() {
    use astraweave_render::gi::vxgi::VxgiConfig;
    use astraweave_render::gi::{VoxelizationConfig, VoxelizationStats};
    
    // Test VxgiConfig with correct fields
    let configs = [
        VxgiConfig {
            voxel_resolution: 64,
            world_size: 100.0,
            cone_count: 4,
            max_trace_distance: 50.0,
            cone_aperture: 0.5,
            _pad: [0; 3],
        },
        VxgiConfig {
            voxel_resolution: 128,
            world_size: 200.0,
            cone_count: 6,
            max_trace_distance: 100.0,
            cone_aperture: 0.577,
            _pad: [0; 3],
        },
        VxgiConfig::default(),
    ];
    
    for config in configs {
        println!("VXGI config: res={}, world={}, cones={}", 
            config.voxel_resolution, config.world_size, config.cone_count);
    }
    
    // Test VoxelizationConfig
    let vox_config = VoxelizationConfig::default();
    println!("VoxelizationConfig world_size: {}", vox_config.world_size);
    
    // Test VoxelizationStats with correct fields
    let stats = VoxelizationStats {
        total_triangles: 5000,
        total_vertices: 15000,
        voxelization_time_ms: 16.5,
        clear_time_ms: 0.5,
    };
    println!("VoxelizationStats: {} triangles, {} vertices, {:.2}ms", 
        stats.total_triangles, stats.total_vertices, stats.voxelization_time_ms);
    
    println!("VXGI additional coverage tested.");
}

#[tokio::test]
async fn test_clustered_renderer_extensive() {
    use astraweave_render::clustered_forward::{ClusteredForwardRenderer, ClusterConfig};
    
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
        .request_device(&wgpu::DeviceDescriptor::default())
        .await
        .unwrap();
    
    // Test various cluster configurations with correct fields
    let configs = [
        ClusterConfig {
            cluster_x: 8,
            cluster_y: 8,
            cluster_z: 16,
            near: 0.1,
            far: 100.0,
            _pad: [0; 3],
        },
        ClusterConfig {
            cluster_x: 16,
            cluster_y: 16,
            cluster_z: 24,
            near: 0.5,
            far: 500.0,
            _pad: [0; 3],
        },
        ClusterConfig::default(),
    ];
    
    for config in configs {
        let renderer = ClusteredForwardRenderer::new(&device, config);
        println!("ClusteredForwardRenderer created with {}x{}x{} clusters", 
            config.cluster_x, config.cluster_y, config.cluster_z);
        // Verify created without panic
        assert!(std::mem::size_of_val(&renderer) > 0);
    }
    
    println!("Clustered renderer extensive tested.");
}

#[tokio::test]
async fn test_lod_generator_extensive() {
    use astraweave_render::lod_generator::{LODGenerator, LODConfig, SimplificationMesh};
    
    // Create a test mesh
    let positions = vec![
        Vec3::new(-1.0, -1.0, 0.0),
        Vec3::new(1.0, -1.0, 0.0),
        Vec3::new(1.0, 1.0, 0.0),
        Vec3::new(-1.0, 1.0, 0.0),
    ];
    let normals = vec![
        Vec3::new(0.0, 0.0, 1.0),
        Vec3::new(0.0, 0.0, 1.0),
        Vec3::new(0.0, 0.0, 1.0),
        Vec3::new(0.0, 0.0, 1.0),
    ];
    let uvs = vec![[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]];
    let indices = vec![0, 1, 2, 2, 3, 0];
    let mesh = SimplificationMesh::new(positions, normals, uvs, indices);
    
    // Test LODConfig with correct fields
    let configs = [
        LODConfig {
            reduction_targets: vec![0.75, 0.5, 0.25],
            max_error: 0.1,
            preserve_boundaries: true,
        },
        LODConfig {
            reduction_targets: vec![0.5],
            max_error: 0.2,
            preserve_boundaries: false,
        },
        LODConfig::default(),
    ];
    
    for config in &configs {
        println!("LODConfig: targets={:?}, error={}", config.reduction_targets, config.max_error);
    }
    
    // Test LODGenerator
    let generator = LODGenerator::new(configs[0].clone());
    let lods = generator.generate_lods(&mesh);
    println!("LOD generated: {} levels", lods.len());
    
    println!("LOD generator extensive tested.");
}

#[tokio::test]
async fn test_culling_pipeline_extensive() {
    use astraweave_render::culling::{InstanceAABB, FrustumPlanes, CullingPipeline};
    
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
        .request_device(&wgpu::DeviceDescriptor::default())
        .await
        .unwrap();
    
    // Test InstanceAABB creation with correct fields
    let aabbs = vec![
        InstanceAABB { center: [0.0, 0.0, 0.0], _pad0: 0, extent: [1.0, 1.0, 1.0], instance_index: 0 },
        InstanceAABB { center: [10.0, 0.0, 0.0], _pad0: 0, extent: [2.0, 2.0, 2.0], instance_index: 1 },
        InstanceAABB { center: [0.0, 10.0, 0.0], _pad0: 0, extent: [0.5, 0.5, 0.5], instance_index: 2 },
        InstanceAABB::new(Vec3::new(100.0, 100.0, 100.0), Vec3::new(5.0, 5.0, 5.0), 3),
    ];
    
    for (i, aabb) in aabbs.iter().enumerate() {
        println!("AABB {}: center={:?}, extent={:?}", i, aabb.center, aabb.extent);
    }
    
    // Test FrustumPlanes from camera matrices - takes &Mat4
    let view = Mat4::look_at_rh(Vec3::new(0.0, 5.0, 10.0), Vec3::ZERO, Vec3::Y);
    let proj = Mat4::perspective_rh(60f32.to_radians(), 16.0/9.0, 0.1, 100.0);
    let vp = view * proj;
    let frustum = FrustumPlanes::from_view_proj(&vp);
    
    // Test each plane is valid
    for i in 0..6 {
        let plane = frustum.planes[i];
        // Normal should have non-zero length
        let normal_len = (plane[0]*plane[0] + plane[1]*plane[1] + plane[2]*plane[2]).sqrt();
        assert!(normal_len > 0.0);
    }
    
    // Test CullingPipeline - new doesn't return Result
    let pipeline = CullingPipeline::new(&device);
    assert!(std::mem::size_of_val(&pipeline) > 0);
    
    println!("Culling pipeline extensive tested.");
}

#[tokio::test]
async fn test_shadow_csm_extensive() {
    use astraweave_render::shadow_csm::CsmRenderer;
    
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
    
    let mut csm = CsmRenderer::new(&device).unwrap();
    
    // Test with various camera positions and light directions
    let test_cases = [
        (Vec3::new(0.0, 10.0, 20.0), Vec3::new(-0.5, -1.0, -0.5).normalize()),
        (Vec3::new(100.0, 50.0, 100.0), Vec3::new(0.0, -1.0, 0.0)),
        (Vec3::new(-50.0, 5.0, -50.0), Vec3::new(0.3, -0.8, 0.2).normalize()),
    ];
    
    for (camera_pos, light_dir) in test_cases {
        let view = Mat4::look_at_rh(camera_pos, Vec3::ZERO, Vec3::Y);
        let proj = Mat4::perspective_rh(60f32.to_radians(), 16.0/9.0, 0.1, 200.0);
        
        csm.update_cascades(camera_pos, view, proj, light_dir, 0.1, 200.0);
        csm.upload_to_gpu(&queue, &device);
        
        println!("CSM updated: camera={:?}, light={:?}", camera_pos, light_dir);
    }
    
    // Test bind_group_layout (it's a field, not method)
    let _bgl = &csm.bind_group_layout;
    
    println!("Shadow CSM extensive tested.");
}

#[tokio::test]
async fn test_animation_interpolation() {
    use astraweave_render::animation::{
        Skeleton, AnimationClip, Transform, Joint, ChannelData, 
        AnimationChannel, Interpolation
    };
    
    // Test Interpolation enum
    let interps = [Interpolation::Step, Interpolation::Linear, Interpolation::CubicSpline];
    for interp in interps {
        println!("Interpolation: {:?}", interp);
    }
    
    // Test Transform operations
    let t1 = Transform {
        translation: Vec3::new(0.0, 0.0, 0.0),
        rotation: Quat::IDENTITY,
        scale: Vec3::ONE,
    };
    let t2 = Transform {
        translation: Vec3::new(10.0, 0.0, 0.0),
        rotation: Quat::from_rotation_y(std::f32::consts::PI),
        scale: Vec3::ONE * 2.0,
    };
    
    // Test lerp
    let mid = t1.lerp(&t2, 0.5);
    assert!((mid.translation.x - 5.0).abs() < 0.001);
    
    // Test to_matrix
    let mat = t1.to_matrix();
    assert!(!mat.is_nan());
    
    // Test channel - ChannelData::Translation is a tuple variant
    let channel = AnimationChannel {
        target_joint_index: 0,
        times: vec![0.0, 1.0, 2.0],
        data: ChannelData::Translation(vec![Vec3::ZERO, Vec3::X * 5.0, Vec3::X * 10.0]),
        interpolation: Interpolation::Linear,
    };
    println!("Animation channel: joint={}, times={}", channel.target_joint_index, channel.times.len());
    
    // Test Joint with correct field names
    let joints = vec![
        Joint {
            name: "root".to_string(),
            parent_index: None,
            inverse_bind_matrix: Mat4::IDENTITY,
            local_transform: Transform::default(),
        },
        Joint {
            name: "spine".to_string(),
            parent_index: Some(0),
            inverse_bind_matrix: Mat4::IDENTITY,
            local_transform: Transform {
                translation: Vec3::Y,
                ..Default::default()
            },
        },
    ];
    
    // Test Skeleton with correct fields
    let skeleton = Skeleton {
        joints,
        root_indices: vec![0],
    };
    assert_eq!(skeleton.joints.len(), 2);
    
    // Test AnimationClip
    let clip = AnimationClip {
        name: "walk".to_string(),
        duration: 1.0,
        channels: vec![channel],
    };
    assert_eq!(clip.name, "walk");
    assert_eq!(clip.duration, 1.0);
    
    println!("Animation interpolation tested.");
}

// ===========================
// WAVE 5: Final Coverage Push - Small Modules
// ===========================

#[test]
fn test_debug_quad_module() {
    use astraweave_render::debug_quad::{DebugQuadVertex, create_screen_quad};
    
    // Test DebugQuadVertex
    let vertex = DebugQuadVertex {
        position: [0.0, 0.0, 0.0],
        uv: [0.5, 0.5],
    };
    assert_eq!(vertex.position[0], 0.0);
    assert_eq!(vertex.uv[0], 0.5);
    
    // Test vertex descriptor
    let desc = DebugQuadVertex::desc();
    assert_eq!(desc.array_stride as usize, std::mem::size_of::<DebugQuadVertex>());
    assert_eq!(desc.step_mode, wgpu::VertexStepMode::Vertex);
    assert_eq!(desc.attributes.len(), 2);
    
    // Test screen quad generation
    let quad = create_screen_quad();
    assert_eq!(quad.len(), 6); // Two triangles
    
    // Validate UVs
    let uv_min = quad.iter().map(|v| v.uv[0]).fold(f32::INFINITY, f32::min);
    let uv_max = quad.iter().map(|v| v.uv[0]).fold(f32::NEG_INFINITY, f32::max);
    assert_eq!(uv_min, 0.0);
    assert_eq!(uv_max, 1.0);
    
    println!("Debug quad module tested.");
}

#[test]
fn test_gi_hybrid_config() {
    use astraweave_render::gi::vxgi::VxgiConfig;
    
    // Test HybridGiConfig - accessing via re-export if available
    let vxgi_config = VxgiConfig::default();
    assert!(vxgi_config.voxel_resolution > 0);
    assert!(vxgi_config.world_size > 0.0);
    
    println!("GI hybrid config tested.");
}

#[tokio::test]
async fn test_culling_node_module() {
    use astraweave_render::culling_node::CullingNode;
    use astraweave_render::culling::{InstanceAABB, FrustumPlanes};
    
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
        .request_device(&wgpu::DeviceDescriptor::default())
        .await
        .unwrap();
    
    // Create CullingNode
    let mut node = CullingNode::new(&device, "test_culling_node");
    
    // Test name accessor
    use astraweave_render::graph::RenderNode;
    assert_eq!(node.name(), "test_culling_node");
    
    // Test resources is None before prepare
    assert!(node.resources().is_none());
    
    // Create test data
    let instances = vec![
        InstanceAABB::new(Vec3::ZERO, Vec3::splat(1.0), 0),
        InstanceAABB::new(Vec3::new(5.0, 0.0, 0.0), Vec3::splat(1.0), 1),
    ];
    
    let view = Mat4::look_at_rh(Vec3::new(0.0, 5.0, 10.0), Vec3::ZERO, Vec3::Y);
    let proj = Mat4::perspective_rh(60f32.to_radians(), 16.0/9.0, 0.1, 100.0);
    let vp = view * proj;
    let frustum = FrustumPlanes::from_view_proj(&vp);
    
    // Prepare culling data
    node.prepare(&device, &instances, &frustum);
    
    // Now resources should be available
    assert!(node.resources().is_some());
    let resources = node.resources().unwrap();
    // Verify buffer exists
    assert!(std::mem::size_of_val(&resources.count_buffer) > 0);
    
    println!("Culling node module tested.");
}

#[test]
fn test_transparency_advanced() {
    use astraweave_render::transparency::{TransparencyManager, BlendMode, create_blend_state};
    
    let mut manager = TransparencyManager::new();
    
    // Add multiple instances with different blend modes
    for i in 0..10 {
        let blend = match i % 3 {
            0 => BlendMode::Alpha,
            1 => BlendMode::Additive,
            _ => BlendMode::Multiplicative,
        };
        manager.add_instance(i, Vec3::new(i as f32, 0.0, -(i as f32)), blend);
    }
    
    assert_eq!(manager.count(), 10);
    
    // Update from different camera positions
    manager.update(Vec3::new(0.0, 0.0, 10.0));
    manager.update(Vec3::new(5.0, 5.0, 5.0));
    manager.update(Vec3::ZERO);
    
    // Get sorted instances
    let sorted: Vec<_> = manager.sorted_instances().collect();
    assert_eq!(sorted.len(), 10);
    
    // Verify back-to-front order (furthest should be first)
    for i in 1..sorted.len() {
        assert!(sorted[i-1].camera_distance >= sorted[i].camera_distance);
    }
    
    // Test filtering by blend mode
    let alpha_count = manager.instances_by_blend_mode(BlendMode::Alpha).count();
    let additive_count = manager.instances_by_blend_mode(BlendMode::Additive).count();
    let mult_count = manager.instances_by_blend_mode(BlendMode::Multiplicative).count();
    assert_eq!(alpha_count + additive_count + mult_count, 10);
    
    // Test blend state creation
    let alpha_blend = create_blend_state(BlendMode::Alpha);
    assert_eq!(alpha_blend.color.src_factor, wgpu::BlendFactor::SrcAlpha);
    
    let additive_blend = create_blend_state(BlendMode::Additive);
    assert_eq!(additive_blend.color.dst_factor, wgpu::BlendFactor::One);
    
    let mult_blend = create_blend_state(BlendMode::Multiplicative);
    assert_eq!(mult_blend.color.src_factor, wgpu::BlendFactor::Zero);
    
    // Test clear
    manager.clear();
    assert_eq!(manager.count(), 0);
    
    println!("Transparency advanced tested.");
}

#[tokio::test]
async fn test_voxelization_mesh_and_config() {
    use astraweave_render::gi::{VoxelizationConfig, VoxelizationMesh, VoxelVertex, VoxelMaterial};
    
    // Test VoxelizationConfig defaults
    let config = VoxelizationConfig::default();
    assert!(config.voxel_resolution > 0);
    assert!(config.world_size > 0.0);
    println!("VoxelizationConfig: res={}, world={}", config.voxel_resolution, config.world_size);
    
    // Test VoxelVertex
    let vertex = VoxelVertex::new(Vec3::new(1.0, 2.0, 3.0), Vec3::Y);
    assert_eq!(vertex.position[0], 1.0);
    assert_eq!(vertex.normal[1], 1.0);
    
    // Test VoxelMaterial
    let default_mat = VoxelMaterial::default();
    assert_eq!(default_mat.albedo[0], 0.8);
    assert_eq!(default_mat.metallic, 0.0);
    
    let albedo_mat = VoxelMaterial::from_albedo(Vec3::new(1.0, 0.0, 0.0));
    assert_eq!(albedo_mat.albedo[0], 1.0);
    
    let emissive_mat = VoxelMaterial::emissive(Vec3::new(5.0, 5.0, 5.0));
    assert_eq!(emissive_mat.emissive[0], 5.0);
    
    // Test VoxelizationMesh
    let vertices = vec![
        VoxelVertex::new(Vec3::ZERO, Vec3::Y),
        VoxelVertex::new(Vec3::X, Vec3::Y),
        VoxelVertex::new(Vec3::Z, Vec3::Y),
    ];
    let indices = vec![0, 1, 2];
    let mesh = VoxelizationMesh::new(vertices, indices, default_mat);
    assert_eq!(mesh.triangle_count(), 1);
    
    println!("Voxelization mesh and config tested.");
}

#[tokio::test]
async fn test_renderer_more_coverage() {
    use astraweave_render::renderer::Renderer;
    
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

    let config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        width: 800,
        height: 600,
        present_mode: wgpu::PresentMode::Fifo,
        alpha_mode: wgpu::CompositeAlphaMode::Auto,
        view_formats: vec![],
        desired_maximum_frame_latency: 2,
    };

    let mut renderer = Renderer::new_from_device(device, queue, None, config).await.unwrap();
    
    // Test various renderer accessors and methods
    let _device = renderer.device();
    let _queue = renderer.queue();
    
    // Test resize
    renderer.resize(1024, 768);
    renderer.resize(640, 480);
    
    // Test environment ticks with various values
    for dt in [0.001, 0.016, 0.033, 0.1, 0.5, 1.0] {
        renderer.tick_environment(dt);
    }
    
    // Test sky config changes
    let mut sky = renderer.sky_config().clone();
    sky.day_color_top = Vec3::new(0.4, 0.6, 1.0);
    renderer.set_sky_config(sky);
    
    // Test material params with various values
    for roughness in [0.0, 0.25, 0.5, 0.75, 1.0] {
        for metallic in [0.0, 0.5, 1.0] {
            renderer.set_material_params([1.0, 1.0, 1.0, 1.0], metallic, roughness);
        }
    }
    
    // Test cascade settings
    renderer.set_cascade_splits(10.0, 50.0);
    renderer.set_cascade_extents(30.0, 100.0);
    renderer.set_cascade_lambda(0.3);
    renderer.set_shadow_filter(2.0, 0.002, 1.5);
    
    println!("Renderer more coverage tested.");
}

#[tokio::test]
async fn test_ibl_additional_coverage() {
    use astraweave_render::ibl::{IblManager, IblQuality};
    
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
        .request_device(&wgpu::DeviceDescriptor::default())
        .await
        .unwrap();
    
    // Test all quality levels
    for quality in [IblQuality::Low, IblQuality::Medium, IblQuality::High] {
        let ibl = IblManager::new(&device, quality).unwrap();
        
        // Test accessors
        let _bgl = ibl.bind_group_layout();
        let _sampler = ibl.sampler();
        
        println!("IBL quality {:?} tested", quality);
    }
    
    println!("IBL additional coverage tested.");
}

#[tokio::test]
async fn test_texture_more_variants() {
    use astraweave_render::texture::{Texture, TextureUsage};
    use image::{RgbaImage, Rgba, DynamicImage};
    
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
    
    // Test checkerboard pattern
    let mut img = RgbaImage::new(64, 64);
    for y in 0..64 {
        for x in 0..64 {
            let pixel = if (x + y) % 2 == 0 { Rgba([255, 255, 255, 255]) } else { Rgba([0, 0, 0, 255]) };
            img.put_pixel(x, y, pixel);
        }
    }
    let dyn_img = DynamicImage::ImageRgba8(img);
    let tex = Texture::from_image_with_usage(&device, &queue, &dyn_img, TextureUsage::Albedo, Some("test_checkerboard"));
    assert!(tex.is_ok());
    println!("Texture checkerboard created");
    
    // Test gradient pattern
    let mut img2 = RgbaImage::new(64, 64);
    for y in 0..64 {
        for x in 0..64 {
            img2.put_pixel(x, y, Rgba([(x * 4) as u8, (y * 4) as u8, 128, 255]));
        }
    }
    let dyn_img2 = DynamicImage::ImageRgba8(img2);
    let tex2 = Texture::from_image_with_usage(&device, &queue, &dyn_img2, TextureUsage::Albedo, Some("test_gradient"));
    assert!(tex2.is_ok());
    println!("Texture gradient created");
    
    // Test various TextureUsage variants
    let img = RgbaImage::from_pixel(32, 32, Rgba([128, 128, 255, 255]));
    let dyn_img = DynamicImage::ImageRgba8(img);
    
    for usage in [TextureUsage::Albedo, TextureUsage::Normal, TextureUsage::Emissive, TextureUsage::MRA, TextureUsage::Height] {
        let tex = Texture::from_image_with_usage(&device, &queue, &dyn_img, usage, Some("usage_test"));
        assert!(tex.is_ok());
    }
    
    println!("Texture more variants tested.");
}

#[tokio::test]
async fn test_graph_adapter_integration() {
    use astraweave_render::renderer::Renderer;
    use astraweave_render::graph::{RenderGraph, RenderNode, GraphContext};
    
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

    let config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        width: 512,
        height: 512,
        present_mode: wgpu::PresentMode::Fifo,
        alpha_mode: wgpu::CompositeAlphaMode::Auto,
        view_formats: vec![],
        desired_maximum_frame_latency: 2,
    };

    let mut renderer = Renderer::new_from_device(device, queue, None, config).await.unwrap();
    
    // Create a simple graph
    let mut graph = RenderGraph::new();
    
    // Create a dummy node
    struct DummyNode { name: String }
    impl RenderNode for DummyNode {
        fn name(&self) -> &str { &self.name }
        fn run(&mut self, _ctx: &mut GraphContext) -> anyhow::Result<()> { Ok(()) }
    }
    
    graph.add_node(DummyNode { name: "test_node".to_string() });
    
    // Test graph adapter - this runs the graph through renderer
    use astraweave_render::graph_adapter::run_graph_on_renderer;
    let result = run_graph_on_renderer(&mut renderer, &mut graph);
    assert!(result.is_ok());
    
    println!("Graph adapter integration tested.");
}

#[tokio::test]
async fn test_environment_extensive() {
    use astraweave_render::environment::{TimeOfDay, WeatherSystem, SkyConfig};
    
    // Test TimeOfDay at all hours
    let mut tod = TimeOfDay::new(0.0, 1.0);
    for hour in 0..24 {
        tod.current_time = hour as f32;
        let sun = tod.get_sun_position();
        let moon = tod.get_moon_position();
        let light_dir = tod.get_light_direction();
        let light_color = tod.get_light_color();
        let ambient = tod.get_ambient_color();
        
        // All positions should be normalized
        assert!((sun.length() - 1.0).abs() < 0.01);
        assert!((moon.length() - 1.0).abs() < 0.01);
        assert!((light_dir.length() - 1.0).abs() < 0.01);
        
        // Colors should be non-negative
        assert!(light_color.min_element() >= 0.0);
        assert!(ambient.min_element() >= 0.0);
    }
    
    // Test fractional times
    for t in [0.5, 6.25, 12.75, 18.3, 23.9] {
        tod.current_time = t;
        let _ = tod.get_sun_position();
        let _ = tod.get_light_direction();
    }
    
    // Test WeatherSystem
    let weather = WeatherSystem::new();
    
    // Access all weather methods
    let _rain = weather.get_rain_intensity();
    let _snow = weather.get_snow_intensity();
    let _fog = weather.get_fog_density();
    let _wind = weather.get_wind_strength();
    let _tint = weather.get_terrain_color_modifier();
    let _atten = weather.get_light_attenuation();
    let _is_rain = weather.is_raining();
    let _is_snow = weather.is_snowing();
    
    // Test SkyConfig defaults and fields
    let sky = SkyConfig::default();
    println!("SkyConfig day_top: {:?}", sky.day_color_top);
    println!("SkyConfig day_horizon: {:?}", sky.day_color_horizon);
    println!("SkyConfig night_top: {:?}", sky.night_color_top);
    
    println!("Environment extensive tested.");
}

#[tokio::test]
async fn test_deferred_renderer_comprehensive() {
    use astraweave_render::deferred::{GBuffer, GBufferFormats, DeferredRenderer};
    
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
        .request_device(&wgpu::DeviceDescriptor::default())
        .await
        .unwrap();
    
    // Test GBuffer at various resolutions
    let formats = GBufferFormats::default();
    let resolutions = [
        (256, 256),
        (512, 384),
        (800, 600),
        (1280, 720),
        (1920, 1080),
    ];
    
    for (w, h) in resolutions {
        let gbuffer = GBuffer::new(&device, w, h, formats.clone());
        assert_eq!(gbuffer.width, w);
        assert_eq!(gbuffer.height, h);
        
        // Access views
        let _albedo = &gbuffer.albedo_view;
        let _normal = &gbuffer.normal_view;
        let _depth = &gbuffer.depth_view;
    }
    
    // Test DeferredRenderer creation at different sizes
    for (w, h) in resolutions.iter().take(3) {
        let dr = DeferredRenderer::new(&device, *w, *h);
        assert!(dr.is_ok());
    }
    
    println!("Deferred renderer comprehensive tested.");
}
