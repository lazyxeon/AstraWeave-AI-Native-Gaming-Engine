//! Anisotropic Kernel Support for improved fluid rendering
//!
//! This module provides velocity-based ellipsoid stretching for particles
//! to improve visual quality during fast fluid motion.

use wgpu::util::DeviceExt;

/// Anisotropic data for a single particle
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct AnisotropicData {
    /// Major axis (velocity direction) + scale in w
    pub axis1: [f32; 4],
    /// Minor axis 1 + scale in w
    pub axis2: [f32; 4],
    /// Minor axis 2 + scale in w
    pub axis3: [f32; 4],
}

impl Default for AnisotropicData {
    fn default() -> Self {
        Self {
            axis1: [1.0, 0.0, 0.0, 1.0],
            axis2: [0.0, 1.0, 0.0, 1.0],
            axis3: [0.0, 0.0, 1.0, 1.0],
        }
    }
}

/// Configuration for anisotropic kernel behavior
#[derive(Clone, Debug)]
pub struct AnisotropicConfig {
    /// Minimum stretch factor (default: 1.0)
    pub min_stretch: f32,
    /// Maximum stretch factor (default: 4.0)
    pub max_stretch: f32,
    /// Velocity to stretch scaling factor (default: 0.15)
    pub velocity_scale: f32,
    /// Whether anisotropic rendering is enabled
    pub enabled: bool,
}

impl Default for AnisotropicConfig {
    fn default() -> Self {
        Self {
            min_stretch: 1.0,
            max_stretch: 4.0,
            velocity_scale: 0.15,
            enabled: true,
        }
    }
}

/// Manages anisotropic kernel computation and buffers
pub struct AnisotropicSystem {
    /// Configuration
    pub config: AnisotropicConfig,
    /// Buffer storing per-particle anisotropic data
    aniso_buffer: wgpu::Buffer,
    /// Compute pipeline for calculating anisotropic matrices
    compute_pipeline: wgpu::ComputePipeline,
    /// Bind group for compute
    bind_group: Option<wgpu::BindGroup>,
    /// Capacity in particles
    capacity: u32,
}

impl AnisotropicSystem {
    /// Create a new anisotropic system
    pub fn new(device: &wgpu::Device, max_particles: u32) -> Self {
        // Create anisotropic data buffer
        let buffer_size = (max_particles as usize * std::mem::size_of::<AnisotropicData>()) as u64;
        let aniso_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Anisotropic Data Buffer"),
            size: buffer_size,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Create compute shader module
        let shader_source = include_str!("../shaders/anisotropic.wgsl");
        let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Anisotropic Compute Shader"),
            source: wgpu::ShaderSource::Wgsl(shader_source.into()),
        });

        // Create bind group layout
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Anisotropic Bind Group Layout"),
            entries: &[
                // Params uniform
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // Particles (read)
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // Anisotropic data (read/write)
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
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

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Anisotropic Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Anisotropic Compute Pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader_module,
            entry_point: Some("compute_anisotropic"),
            compilation_options: Default::default(),
            cache: None,
        });

        Self {
            config: AnisotropicConfig::default(),
            aniso_buffer,
            compute_pipeline,
            bind_group: None,
            capacity: max_particles,
        }
    }

    /// Get the anisotropic data buffer for rendering
    pub fn get_buffer(&self) -> &wgpu::Buffer {
        &self.aniso_buffer
    }

    /// Check if anisotropic rendering is enabled
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }
}
