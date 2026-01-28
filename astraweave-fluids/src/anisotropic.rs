//! Anisotropic Kernel Support for improved fluid rendering
//!
//! This module provides velocity-based ellipsoid stretching for particles
//! to improve visual quality during fast fluid motion.

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
#[allow(dead_code)]
pub struct AnisotropicSystem {
    /// Configuration
    pub config: AnisotropicConfig,
    /// Buffer storing per-particle anisotropic data
    aniso_buffer: wgpu::Buffer,
    /// Compute pipeline for calculating anisotropic matrices (prepared for future GPU compute)
    compute_pipeline: wgpu::ComputePipeline,
    /// Bind group for compute (prepared for future GPU compute)
    bind_group: Option<wgpu::BindGroup>,
    /// Capacity in particles (prepared for future GPU compute)
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

#[cfg(test)]
mod tests {
    use super::*;
    use bytemuck::Zeroable;

    // ==================== AnisotropicData Tests ====================

    #[test]
    fn test_anisotropic_data_default() {
        let data = AnisotropicData::default();
        
        // Default should be identity-like axes
        assert_eq!(data.axis1[0], 1.0);
        assert_eq!(data.axis1[1], 0.0);
        assert_eq!(data.axis1[2], 0.0);
        assert_eq!(data.axis1[3], 1.0); // scale
        
        assert_eq!(data.axis2[0], 0.0);
        assert_eq!(data.axis2[1], 1.0);
        assert_eq!(data.axis2[2], 0.0);
        assert_eq!(data.axis2[3], 1.0);
        
        assert_eq!(data.axis3[0], 0.0);
        assert_eq!(data.axis3[1], 0.0);
        assert_eq!(data.axis3[2], 1.0);
        assert_eq!(data.axis3[3], 1.0);
    }

    #[test]
    fn test_anisotropic_data_size() {
        // 3 * [f32; 4] = 48 bytes
        assert_eq!(std::mem::size_of::<AnisotropicData>(), 48);
    }

    #[test]
    fn test_anisotropic_data_copy_clone() {
        let data = AnisotropicData::default();
        let copied = data;
        let cloned = data.clone();
        
        assert_eq!(copied.axis1[0], data.axis1[0]);
        assert_eq!(cloned.axis1[0], data.axis1[0]);
    }

    #[test]
    fn test_anisotropic_data_bytemuck_cast() {
        let data = AnisotropicData {
            axis1: [1.0, 0.0, 0.0, 2.0],
            axis2: [0.0, 1.0, 0.0, 1.5],
            axis3: [0.0, 0.0, 1.0, 0.5],
        };
        
        let bytes: &[u8] = bytemuck::bytes_of(&data);
        assert_eq!(bytes.len(), 48);
        
        let recovered: &AnisotropicData = bytemuck::from_bytes(bytes);
        assert_eq!(recovered.axis1[3], 2.0);
        assert_eq!(recovered.axis2[3], 1.5);
        assert_eq!(recovered.axis3[3], 0.5);
    }

    #[test]
    fn test_anisotropic_data_zeroed() {
        let data = AnisotropicData::zeroed();
        
        assert_eq!(data.axis1[0], 0.0);
        assert_eq!(data.axis1[3], 0.0);
        assert_eq!(data.axis2[1], 0.0);
        assert_eq!(data.axis3[2], 0.0);
    }

    #[test]
    fn test_anisotropic_data_orthogonality() {
        let data = AnisotropicData::default();
        
        // Default axes should be orthogonal (dot products = 0)
        let dot12 = data.axis1[0] * data.axis2[0] 
                  + data.axis1[1] * data.axis2[1] 
                  + data.axis1[2] * data.axis2[2];
        let dot23 = data.axis2[0] * data.axis3[0] 
                  + data.axis2[1] * data.axis3[1] 
                  + data.axis2[2] * data.axis3[2];
        let dot13 = data.axis1[0] * data.axis3[0] 
                  + data.axis1[1] * data.axis3[1] 
                  + data.axis1[2] * data.axis3[2];
        
        assert!((dot12).abs() < 1e-6);
        assert!((dot23).abs() < 1e-6);
        assert!((dot13).abs() < 1e-6);
    }

    #[test]
    fn test_anisotropic_data_unit_length() {
        let data = AnisotropicData::default();
        
        // Default axes should be unit length (ignoring w component)
        let len1 = (data.axis1[0].powi(2) + data.axis1[1].powi(2) + data.axis1[2].powi(2)).sqrt();
        let len2 = (data.axis2[0].powi(2) + data.axis2[1].powi(2) + data.axis2[2].powi(2)).sqrt();
        let len3 = (data.axis3[0].powi(2) + data.axis3[1].powi(2) + data.axis3[2].powi(2)).sqrt();
        
        assert!((len1 - 1.0).abs() < 1e-6);
        assert!((len2 - 1.0).abs() < 1e-6);
        assert!((len3 - 1.0).abs() < 1e-6);
    }

    // ==================== AnisotropicConfig Tests ====================

    #[test]
    fn test_anisotropic_config_default() {
        let config = AnisotropicConfig::default();
        
        assert_eq!(config.min_stretch, 1.0);
        assert_eq!(config.max_stretch, 4.0);
        assert_eq!(config.velocity_scale, 0.15);
        assert!(config.enabled);
    }

    #[test]
    fn test_anisotropic_config_clone() {
        let config = AnisotropicConfig {
            min_stretch: 0.5,
            max_stretch: 8.0,
            velocity_scale: 0.25,
            enabled: false,
        };
        
        let cloned = config.clone();
        assert_eq!(cloned.min_stretch, 0.5);
        assert_eq!(cloned.max_stretch, 8.0);
        assert_eq!(cloned.velocity_scale, 0.25);
        assert!(!cloned.enabled);
    }

    #[test]
    fn test_anisotropic_config_debug() {
        let config = AnisotropicConfig::default();
        let debug_str = format!("{:?}", config);
        
        assert!(debug_str.contains("AnisotropicConfig"));
        assert!(debug_str.contains("min_stretch"));
        assert!(debug_str.contains("max_stretch"));
    }

    #[test]
    fn test_anisotropic_config_stretch_range() {
        let config = AnisotropicConfig::default();
        
        // min_stretch <= max_stretch
        assert!(config.min_stretch <= config.max_stretch);
        // velocity_scale should be positive
        assert!(config.velocity_scale > 0.0);
    }

    #[test]
    fn test_anisotropic_config_custom_values() {
        let config = AnisotropicConfig {
            min_stretch: 0.1,
            max_stretch: 10.0,
            velocity_scale: 1.0,
            enabled: true,
        };
        
        assert_eq!(config.min_stretch, 0.1);
        assert_eq!(config.max_stretch, 10.0);
        assert_eq!(config.velocity_scale, 1.0);
    }

    // ==================== Mutation-Resistant Tests ====================

    #[test]
    fn test_anisotropic_data_field_independence() {
        // Changing one axis shouldn't affect others
        let mut data = AnisotropicData::default();
        data.axis1 = [2.0, 3.0, 4.0, 5.0];
        
        // axis2 and axis3 should still be default
        assert_eq!(data.axis2, [0.0, 1.0, 0.0, 1.0]);
        assert_eq!(data.axis3, [0.0, 0.0, 1.0, 1.0]);
    }

    #[test]
    fn test_anisotropic_data_scales_independent() {
        let data = AnisotropicData {
            axis1: [1.0, 0.0, 0.0, 2.0],  // scale = 2
            axis2: [0.0, 1.0, 0.0, 0.5],  // scale = 0.5
            axis3: [0.0, 0.0, 1.0, 1.0],  // scale = 1
        };
        
        // Each axis has independent scale in w component
        assert_eq!(data.axis1[3], 2.0);
        assert_eq!(data.axis2[3], 0.5);
        assert_eq!(data.axis3[3], 1.0);
        
        // Scales should not affect xyz components
        assert_eq!(data.axis1[0], 1.0);
        assert_eq!(data.axis2[1], 1.0);
        assert_eq!(data.axis3[2], 1.0);
    }

    #[test]
    fn test_anisotropic_config_disabled_state() {
        let config = AnisotropicConfig {
            enabled: false,
            ..Default::default()
        };
        
        assert!(!config.enabled);
        // Other values should still be valid defaults
        assert_eq!(config.min_stretch, 1.0);
        assert_eq!(config.max_stretch, 4.0);
    }

    #[test]
    fn test_anisotropic_data_array_slice() {
        let data_array = [
            AnisotropicData::default(),
            AnisotropicData {
                axis1: [0.0, 1.0, 0.0, 2.0],
                axis2: [1.0, 0.0, 0.0, 1.0],
                axis3: [0.0, 0.0, 1.0, 1.0],
            },
        ];
        
        let bytes: &[u8] = bytemuck::cast_slice(&data_array);
        assert_eq!(bytes.len(), 96); // 2 * 48 bytes
        
        let recovered: &[AnisotropicData] = bytemuck::cast_slice(bytes);
        assert_eq!(recovered.len(), 2);
        
        // First element should have axis1 pointing X
        assert_eq!(recovered[0].axis1[0], 1.0);
        // Second element should have axis1 pointing Y
        assert_eq!(recovered[1].axis1[1], 1.0);
    }

    #[test]
    fn test_anisotropic_config_stretch_ratio() {
        let config = AnisotropicConfig::default();
        
        // Stretch ratio should be positive
        let ratio = config.max_stretch / config.min_stretch;
        assert!(ratio >= 1.0);
        assert_eq!(ratio, 4.0);
    }

    #[test]
    fn test_anisotropic_data_debug_format() {
        let data = AnisotropicData::default();
        let debug_str = format!("{:?}", data);
        
        assert!(debug_str.contains("axis1"));
        assert!(debug_str.contains("axis2"));
        assert!(debug_str.contains("axis3"));
    }
}
