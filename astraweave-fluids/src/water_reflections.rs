//! Screen-space water reflections
//!
//! Implements planar reflections and screen-space reflections (SSR)
//! for water surfaces.

use glam::{Mat4, Vec3, Vec4};

/// Configuration for water reflections
#[derive(Clone, Debug, PartialEq)]
pub struct WaterReflectionConfig {
    /// Resolution scale for reflection render target (0.25 = quarter res)
    pub resolution_scale: f32,
    /// Maximum ray march steps for SSR
    pub max_steps: u32,
    /// Ray step size in screen space
    pub step_size: f32,
    /// Thickness of geometry for hit detection
    pub thickness: f32,
    /// Fresnel effect strength (0 = none, 1 = full)
    pub fresnel_strength: f32,
    /// Base reflectivity at perpendicular angle
    pub base_reflectivity: f32,
    /// Reflection blur amount (0 = sharp, 1 = very blurry)
    pub blur_amount: f32,
    /// Whether to use planar reflection for distant objects
    pub use_planar_fallback: bool,
    /// Maximum distance for SSR before fallback
    pub max_distance: f32,
    /// Edge fade distance (prevents artifacts at screen edges)
    pub edge_fade: f32,
}

impl Default for WaterReflectionConfig {
    fn default() -> Self {
        Self {
            resolution_scale: 0.5,
            max_steps: 32,
            step_size: 0.05,
            thickness: 0.5,
            fresnel_strength: 1.0,
            base_reflectivity: 0.02,
            blur_amount: 0.0,
            use_planar_fallback: true,
            max_distance: 100.0,
            edge_fade: 0.1,
        }
    }
}

impl WaterReflectionConfig {
    /// High quality preset
    pub fn high_quality() -> Self {
        Self {
            resolution_scale: 1.0,
            max_steps: 64,
            step_size: 0.02,
            thickness: 0.3,
            fresnel_strength: 1.0,
            base_reflectivity: 0.02,
            blur_amount: 0.0,
            use_planar_fallback: true,
            max_distance: 200.0,
            edge_fade: 0.15,
        }
    }

    /// Low quality preset for performance
    pub fn low_quality() -> Self {
        Self {
            resolution_scale: 0.25,
            max_steps: 16,
            step_size: 0.1,
            thickness: 1.0,
            fresnel_strength: 1.0,
            base_reflectivity: 0.02,
            blur_amount: 0.2,
            use_planar_fallback: true,
            max_distance: 50.0,
            edge_fade: 0.2,
        }
    }

    /// Stylized preset (sharp, high fresnel)
    pub fn stylized() -> Self {
        Self {
            resolution_scale: 0.5,
            max_steps: 32,
            step_size: 0.05,
            thickness: 0.5,
            fresnel_strength: 1.5,
            base_reflectivity: 0.1,
            blur_amount: 0.0,
            use_planar_fallback: false,
            max_distance: 100.0,
            edge_fade: 0.1,
        }
    }
}

/// GPU-compatible reflection uniforms
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct ReflectionUniforms {
    /// View-projection matrix
    pub view_proj: [[f32; 4]; 4],
    /// Inverse view-projection matrix
    pub inv_view_proj: [[f32; 4]; 4],
    /// Camera position (xyz) + water height (w)
    pub camera_water: [f32; 4],
    /// Reflection params: max_steps, step_size, thickness, fresnel
    pub params: [f32; 4],
    /// More params: base_reflectivity, blur, max_distance, edge_fade
    pub params2: [f32; 4],
}

impl ReflectionUniforms {
    /// Create uniforms from config and matrices
    pub fn from_config(
        config: &WaterReflectionConfig,
        view_proj: Mat4,
        camera_pos: Vec3,
        water_height: f32,
    ) -> Self {
        Self {
            view_proj: view_proj.to_cols_array_2d(),
            inv_view_proj: view_proj.inverse().to_cols_array_2d(),
            camera_water: [camera_pos.x, camera_pos.y, camera_pos.z, water_height],
            params: [
                config.max_steps as f32,
                config.step_size,
                config.thickness,
                config.fresnel_strength,
            ],
            params2: [
                config.base_reflectivity,
                config.blur_amount,
                config.max_distance,
                config.edge_fade,
            ],
        }
    }
}

/// Planar reflection data for a water plane
#[derive(Clone, Debug)]
pub struct PlanarReflection {
    /// Water surface height
    pub surface_height: f32,
    /// Reflection plane normal (usually up)
    pub plane_normal: Vec3,
    /// Reflection matrix
    reflection_matrix: Mat4,
    /// Clipping plane for rendering
    pub clip_plane: Vec4,
}

impl PlanarReflection {
    /// Create a new planar reflection for a horizontal water surface
    pub fn new(surface_height: f32) -> Self {
        let plane_normal = Vec3::Y;
        
        // Reflection matrix: reflect across Y = surface_height
        let reflection_matrix = Mat4::from_cols(
            Vec4::new(1.0, 0.0, 0.0, 0.0),
            Vec4::new(0.0, -1.0, 0.0, 0.0),
            Vec4::new(0.0, 0.0, 1.0, 0.0),
            Vec4::new(0.0, 2.0 * surface_height, 0.0, 1.0),
        );

        // Clip plane: y > surface_height (objects below water are clipped)
        let clip_plane = Vec4::new(0.0, 1.0, 0.0, -surface_height);

        Self {
            surface_height,
            plane_normal,
            reflection_matrix,
            clip_plane,
        }
    }

    /// Get the reflection matrix
    pub fn reflection_matrix(&self) -> Mat4 {
        self.reflection_matrix
    }

    /// Create reflected view matrix
    pub fn create_reflected_view(&self, view_matrix: Mat4) -> Mat4 {
        view_matrix * self.reflection_matrix
    }

    /// Reflect a camera position
    pub fn reflect_position(&self, position: Vec3) -> Vec3 {
        Vec3::new(
            position.x,
            2.0 * self.surface_height - position.y,
            position.z,
        )
    }

    /// Update surface height
    pub fn set_surface_height(&mut self, height: f32) {
        self.surface_height = height;
        
        self.reflection_matrix = Mat4::from_cols(
            Vec4::new(1.0, 0.0, 0.0, 0.0),
            Vec4::new(0.0, -1.0, 0.0, 0.0),
            Vec4::new(0.0, 0.0, 1.0, 0.0),
            Vec4::new(0.0, 2.0 * height, 0.0, 1.0),
        );

        self.clip_plane = Vec4::new(0.0, 1.0, 0.0, -height);
    }
}

/// Manages water reflections
#[derive(Debug)]
pub struct WaterReflectionSystem {
    /// Configuration
    config: WaterReflectionConfig,
    /// Planar reflection data
    planar: Option<PlanarReflection>,
    /// Water surface height
    surface_height: f32,
}

impl WaterReflectionSystem {
    /// Create a new water reflection system
    pub fn new(config: WaterReflectionConfig) -> Self {
        Self {
            config,
            planar: None,
            surface_height: 0.0,
        }
    }

    /// Set up planar reflection for a water surface
    pub fn setup_planar(&mut self, surface_height: f32) {
        self.surface_height = surface_height;
        self.planar = Some(PlanarReflection::new(surface_height));
    }

    /// Update water surface height
    pub fn set_surface_height(&mut self, height: f32) {
        self.surface_height = height;
        if let Some(ref mut planar) = self.planar {
            planar.set_surface_height(height);
        }
    }

    /// Get surface height
    pub fn surface_height(&self) -> f32 {
        self.surface_height
    }

    /// Get planar reflection data
    pub fn planar(&self) -> Option<&PlanarReflection> {
        self.planar.as_ref()
    }

    /// Calculate Fresnel factor for a view angle
    pub fn calculate_fresnel(&self, view_dir: Vec3, normal: Vec3) -> f32 {
        let cos_theta = view_dir.dot(normal).abs();
        let base = self.config.base_reflectivity;
        let strength = self.config.fresnel_strength;

        // Schlick's approximation
        base + (1.0 - base) * (1.0 - cos_theta).powf(5.0 * strength)
    }

    /// Get reflection target dimensions
    pub fn get_target_dimensions(&self, screen_width: u32, screen_height: u32) -> (u32, u32) {
        let scale = self.config.resolution_scale;
        (
            ((screen_width as f32) * scale).max(1.0) as u32,
            ((screen_height as f32) * scale).max(1.0) as u32,
        )
    }

    /// Get uniforms for GPU rendering
    pub fn get_uniforms(&self, view_proj: Mat4, camera_pos: Vec3) -> ReflectionUniforms {
        ReflectionUniforms::from_config(&self.config, view_proj, camera_pos, self.surface_height)
    }

    /// Get configuration
    pub fn config(&self) -> &WaterReflectionConfig {
        &self.config
    }

    /// Set configuration
    pub fn set_config(&mut self, config: WaterReflectionConfig) {
        self.config = config;
    }

    /// Check if planar fallback is enabled
    pub fn has_planar_fallback(&self) -> bool {
        self.config.use_planar_fallback && self.planar.is_some()
    }
}

/// WGSL shader code for screen-space reflections
pub const SSR_WGSL: &str = r#"
// Reflection uniforms
struct ReflectionUniforms {
    view_proj: mat4x4<f32>,
    inv_view_proj: mat4x4<f32>,
    camera_water: vec4<f32>,
    params: vec4<f32>,   // max_steps, step_size, thickness, fresnel
    params2: vec4<f32>,  // base_reflectivity, blur, max_distance, edge_fade
}

@group(2) @binding(2) var<uniform> reflection: ReflectionUniforms;
@group(2) @binding(3) var color_texture: texture_2d<f32>;
@group(2) @binding(4) var depth_texture: texture_2d<f32>;
@group(2) @binding(5) var reflection_sampler: sampler;

// Reconstruct world position from depth
fn reconstruct_position(uv: vec2<f32>, depth: f32) -> vec3<f32> {
    let clip = vec4<f32>(uv * 2.0 - 1.0, depth, 1.0);
    let world = reflection.inv_view_proj * clip;
    return world.xyz / world.w;
}

// Project world position to screen UV
fn project_to_screen(world_pos: vec3<f32>) -> vec3<f32> {
    let clip = reflection.view_proj * vec4<f32>(world_pos, 1.0);
    let ndc = clip.xyz / clip.w;
    return vec3<f32>(ndc.xy * 0.5 + 0.5, ndc.z);
}

// Calculate Fresnel factor
fn calculate_fresnel(view_dir: vec3<f32>, normal: vec3<f32>) -> f32 {
    let base = reflection.params2.x;
    let strength = reflection.params.w;
    let cos_theta = abs(dot(view_dir, normal));
    return base + (1.0 - base) * pow(1.0 - cos_theta, 5.0 * strength);
}

// Screen-space ray marching
fn trace_ssr(
    ray_origin: vec3<f32>,
    ray_dir: vec3<f32>,
    uv: vec2<f32>,
) -> vec4<f32> {
    let max_steps = i32(reflection.params.x);
    let step_size = reflection.params.y;
    let thickness = reflection.params.z;
    let max_distance = reflection.params2.z;
    let edge_fade = reflection.params2.w;
    
    var ray_pos = ray_origin;
    var last_uv = uv;
    
    for (var i: i32 = 0; i < max_steps; i++) {
        ray_pos += ray_dir * step_size;
        
        // Check distance limit
        if (length(ray_pos - ray_origin) > max_distance) {
            break;
        }
        
        // Project to screen
        let screen = project_to_screen(ray_pos);
        let sample_uv = screen.xy;
        
        // Check if out of screen bounds
        if (sample_uv.x < 0.0 || sample_uv.x > 1.0 || 
            sample_uv.y < 0.0 || sample_uv.y > 1.0) {
            break;
        }
        
        // Sample depth
        let sampled_depth = textureSample(depth_texture, reflection_sampler, sample_uv).r;
        let sampled_pos = reconstruct_position(sample_uv, sampled_depth);
        
        // Check for hit
        let depth_diff = ray_pos.y - sampled_pos.y;
        if (depth_diff > 0.0 && depth_diff < thickness) {
            // Calculate edge fade
            let edge_x = min(sample_uv.x, 1.0 - sample_uv.x) / edge_fade;
            let edge_y = min(sample_uv.y, 1.0 - sample_uv.y) / edge_fade;
            let edge_factor = clamp(min(edge_x, edge_y), 0.0, 1.0);
            
            // Distance fade
            let dist_factor = 1.0 - length(ray_pos - ray_origin) / max_distance;
            
            let color = textureSample(color_texture, reflection_sampler, sample_uv);
            return vec4<f32>(color.rgb, edge_factor * dist_factor);
        }
        
        last_uv = sample_uv;
    }
    
    // No hit
    return vec4<f32>(0.0);
}

// Main SSR function
fn sample_ssr(
    world_pos: vec3<f32>,
    normal: vec3<f32>,
    view_dir: vec3<f32>,
    uv: vec2<f32>,
) -> vec4<f32> {
    // Calculate reflection direction
    let reflect_dir = reflect(-view_dir, normal);
    
    // Trace ray
    let result = trace_ssr(world_pos, reflect_dir, uv);
    
    // Apply Fresnel
    let fresnel = calculate_fresnel(view_dir, normal);
    
    return vec4<f32>(result.rgb, result.a * fresnel);
}
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = WaterReflectionConfig::default();
        assert_eq!(config.resolution_scale, 0.5);
        assert_eq!(config.max_steps, 32);
        assert!(config.fresnel_strength > 0.0);
    }

    #[test]
    fn test_config_presets() {
        let high = WaterReflectionConfig::high_quality();
        let low = WaterReflectionConfig::low_quality();
        let stylized = WaterReflectionConfig::stylized();

        // High quality should have more steps
        assert!(high.max_steps > low.max_steps);
        
        // High quality should have better resolution
        assert!(high.resolution_scale > low.resolution_scale);
        
        // Stylized should have higher fresnel
        assert!(stylized.fresnel_strength > high.fresnel_strength);
    }

    #[test]
    fn test_uniforms_creation() {
        let config = WaterReflectionConfig::default();
        let uniforms = ReflectionUniforms::from_config(
            &config,
            Mat4::IDENTITY,
            Vec3::new(0.0, 10.0, 0.0),
            5.0,
        );

        assert_eq!(uniforms.camera_water[1], 10.0);
        assert_eq!(uniforms.camera_water[3], 5.0);
        assert_eq!(uniforms.params[0], config.max_steps as f32);
    }

    #[test]
    fn test_uniforms_size() {
        assert_eq!(std::mem::size_of::<ReflectionUniforms>(), 176);
    }

    #[test]
    fn test_planar_reflection_creation() {
        let planar = PlanarReflection::new(10.0);

        assert_eq!(planar.surface_height, 10.0);
        assert_eq!(planar.plane_normal, Vec3::Y);
    }

    #[test]
    fn test_planar_reflect_position() {
        let planar = PlanarReflection::new(10.0);

        let pos = Vec3::new(5.0, 15.0, 3.0);
        let reflected = planar.reflect_position(pos);

        assert_eq!(reflected.x, 5.0);
        assert_eq!(reflected.y, 5.0); // 2*10 - 15 = 5
        assert_eq!(reflected.z, 3.0);
    }

    #[test]
    fn test_planar_update_height() {
        let mut planar = PlanarReflection::new(10.0);
        
        planar.set_surface_height(20.0);
        
        assert_eq!(planar.surface_height, 20.0);
        assert_eq!(planar.clip_plane.w, -20.0);
    }

    #[test]
    fn test_system_creation() {
        let system = WaterReflectionSystem::new(WaterReflectionConfig::default());

        assert!(system.planar().is_none());
        assert_eq!(system.surface_height(), 0.0);
    }

    #[test]
    fn test_system_setup_planar() {
        let mut system = WaterReflectionSystem::new(WaterReflectionConfig::default());
        
        system.setup_planar(15.0);
        
        assert!(system.planar().is_some());
        assert_eq!(system.surface_height(), 15.0);
    }

    #[test]
    fn test_system_update_height() {
        let mut system = WaterReflectionSystem::new(WaterReflectionConfig::default());
        system.setup_planar(10.0);
        
        system.set_surface_height(20.0);
        
        assert_eq!(system.surface_height(), 20.0);
        assert_eq!(system.planar().unwrap().surface_height, 20.0);
    }

    #[test]
    fn test_fresnel_calculation() {
        let system = WaterReflectionSystem::new(WaterReflectionConfig::default());

        // Looking straight down (perpendicular)
        let fresnel_perp = system.calculate_fresnel(Vec3::NEG_Y, Vec3::Y);
        
        // Looking at glancing angle
        let fresnel_glancing = system.calculate_fresnel(
            Vec3::new(0.1, -0.01, 0.0).normalize(),
            Vec3::Y,
        );

        // Glancing should have higher fresnel than perpendicular
        assert!(fresnel_glancing > fresnel_perp);
    }

    #[test]
    fn test_target_dimensions() {
        let config = WaterReflectionConfig {
            resolution_scale: 0.5,
            ..Default::default()
        };
        let system = WaterReflectionSystem::new(config);

        let (width, height) = system.get_target_dimensions(1920, 1080);

        assert_eq!(width, 960);
        assert_eq!(height, 540);
    }

    #[test]
    fn test_target_dimensions_minimum() {
        let config = WaterReflectionConfig {
            resolution_scale: 0.001, // Very small
            ..Default::default()
        };
        let system = WaterReflectionSystem::new(config);

        let (width, height) = system.get_target_dimensions(100, 100);

        // Should be at least 1x1
        assert!(width >= 1);
        assert!(height >= 1);
    }

    #[test]
    fn test_has_planar_fallback() {
        let mut system = WaterReflectionSystem::new(WaterReflectionConfig::default());

        // No planar set up
        assert!(!system.has_planar_fallback());

        // Set up planar
        system.setup_planar(0.0);
        assert!(system.has_planar_fallback());

        // Disable fallback in config
        let mut config = WaterReflectionConfig::default();
        config.use_planar_fallback = false;
        system.set_config(config);
        assert!(!system.has_planar_fallback());
    }

    #[test]
    fn test_wgsl_shader_exists() {
        assert!(!SSR_WGSL.is_empty());
        assert!(SSR_WGSL.contains("sample_ssr"));
        assert!(SSR_WGSL.contains("trace_ssr"));
        assert!(SSR_WGSL.contains("calculate_fresnel"));
    }
}
