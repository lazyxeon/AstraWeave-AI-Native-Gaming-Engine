pub struct Depth {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub format: wgpu::TextureFormat,
}

impl Depth {
    pub fn create(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration) -> Self {
        let format = wgpu::TextureFormat::Depth32Float;
        let size = wgpu::Extent3d {
            width: config.width,
            height: config.height,
            depth_or_array_layers: 1,
        };
        let desc = wgpu::TextureDescriptor {
            label: Some("depth"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        };
        let texture = device.create_texture(&desc);
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        Self {
            texture,
            view,
            format,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn create_test_device() -> (wgpu::Device, wgpu::Queue) {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                force_fallback_adapter: true,
                compatible_surface: None,
            })
            .await
            .expect("Failed to find adapter");

        adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("test_device"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::downlevel_defaults(),
                memory_hints: wgpu::MemoryHints::default(),
                trace: Default::default(),
            })
            .await
            .expect("Failed to create device")
    }

    #[test]
    fn test_depth_create() {
        pollster::block_on(async {
            let (device, _queue) = create_test_device().await;
            let config = wgpu::SurfaceConfiguration {
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                format: wgpu::TextureFormat::Bgra8UnormSrgb,
                width: 800,
                height: 600,
                present_mode: wgpu::PresentMode::Fifo,
                desired_maximum_frame_latency: 2,
                alpha_mode: wgpu::CompositeAlphaMode::Opaque,
                view_formats: vec![],
            };

            let depth = Depth::create(&device, &config);

            // Verify depth texture properties
            assert_eq!(depth.format, wgpu::TextureFormat::Depth32Float);
            assert_eq!(depth.texture.size().width, 800);
            assert_eq!(depth.texture.size().height, 600);
            assert_eq!(depth.texture.size().depth_or_array_layers, 1);
            assert_eq!(depth.texture.format(), wgpu::TextureFormat::Depth32Float);
        });
    }

    #[test]
    fn test_depth_create_different_sizes() {
        pollster::block_on(async {
            let (device, _queue) = create_test_device().await;

            let configs = vec![(1920, 1080), (1280, 720), (640, 480), (256, 256)];

            for (width, height) in configs {
                let config = wgpu::SurfaceConfiguration {
                    usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                    format: wgpu::TextureFormat::Bgra8UnormSrgb,
                    width,
                    height,
                    present_mode: wgpu::PresentMode::Fifo,
                    desired_maximum_frame_latency: 2,
                    alpha_mode: wgpu::CompositeAlphaMode::Opaque,
                    view_formats: vec![],
                };

                let depth = Depth::create(&device, &config);
                assert_eq!(depth.texture.size().width, width, "Width should match");
                assert_eq!(depth.texture.size().height, height, "Height should match");
            }
        });
    }

    #[test]
    fn test_depth_format_consistent() {
        pollster::block_on(async {
            let (device, _queue) = create_test_device().await;
            let config = wgpu::SurfaceConfiguration {
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                format: wgpu::TextureFormat::Bgra8UnormSrgb,
                width: 800,
                height: 600,
                present_mode: wgpu::PresentMode::Fifo,
                desired_maximum_frame_latency: 2,
                alpha_mode: wgpu::CompositeAlphaMode::Opaque,
                view_formats: vec![],
            };

            let depth = Depth::create(&device, &config);

            // Verify struct format matches texture format
            assert_eq!(depth.format, depth.texture.format());
        });
    }
}
