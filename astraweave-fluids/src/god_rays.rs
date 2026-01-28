//! God rays / light shafts system
//!
//! Implements volumetric light shafts that penetrate through water,
//! creating the iconic "god rays" effect seen in underwater scenes.

use glam::Vec3;

/// Configuration for god rays rendering
#[derive(Clone, Debug, PartialEq)]
pub struct GodRaysConfig {
    /// Number of ray samples for quality
    pub num_samples: u32,
    /// Ray density (controls spread of rays)
    pub density: f32,
    /// Overall intensity of rays (0-1)
    pub intensity: f32,
    /// How quickly rays fade with depth
    pub decay: f32,
    /// Weight of each sample contribution
    pub weight: f32,
    /// Exposure/brightness multiplier
    pub exposure: f32,
    /// Maximum ray length in world units
    pub max_length: f32,
    /// Ray animation speed
    pub animation_speed: f32,
    /// Noise scale for ray variation
    pub noise_scale: f32,
}

impl Default for GodRaysConfig {
    fn default() -> Self {
        Self {
            num_samples: 64,
            density: 1.0,
            intensity: 0.8,
            decay: 0.96,
            weight: 0.5,
            exposure: 0.3,
            max_length: 100.0,
            animation_speed: 0.5,
            noise_scale: 0.1,
        }
    }
}

impl GodRaysConfig {
    /// Preset for bright tropical waters
    pub fn tropical() -> Self {
        Self {
            num_samples: 80,
            density: 1.2,
            intensity: 1.0,
            decay: 0.97,
            weight: 0.6,
            exposure: 0.4,
            max_length: 80.0,
            animation_speed: 0.3,
            noise_scale: 0.08,
        }
    }

    /// Preset for murky/dark waters
    pub fn murky() -> Self {
        Self {
            num_samples: 48,
            density: 0.8,
            intensity: 0.3,
            decay: 0.92,
            weight: 0.4,
            exposure: 0.2,
            max_length: 30.0,
            animation_speed: 0.2,
            noise_scale: 0.15,
        }
    }

    /// Preset for dramatic cinematic rays
    pub fn cinematic() -> Self {
        Self {
            num_samples: 100,
            density: 1.5,
            intensity: 0.9,
            decay: 0.98,
            weight: 0.7,
            exposure: 0.5,
            max_length: 150.0,
            animation_speed: 0.1,
            noise_scale: 0.05,
        }
    }

    /// Low quality preset for performance
    pub fn low_quality() -> Self {
        Self {
            num_samples: 32,
            density: 1.0,
            intensity: 0.7,
            decay: 0.95,
            weight: 0.5,
            exposure: 0.3,
            max_length: 50.0,
            animation_speed: 0.5,
            noise_scale: 0.1,
        }
    }
}

/// GPU-compatible uniforms for god rays
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct GodRaysUniforms {
    /// Light direction (xyz) + intensity (w)
    pub light_dir_intensity: [f32; 4],
    /// Decay, weight, exposure, density
    pub params: [f32; 4],
    /// Num samples (u32 as f32), max_length, time, noise_scale
    pub params2: [f32; 4],
    /// Water surface height, camera underwater flag, padding
    pub surface_params: [f32; 4],
}

impl GodRaysUniforms {
    /// Create uniforms from config
    pub fn from_config(
        config: &GodRaysConfig,
        light_direction: Vec3,
        surface_height: f32,
        camera_underwater: bool,
        time: f32,
    ) -> Self {
        Self {
            light_dir_intensity: [
                light_direction.x,
                light_direction.y,
                light_direction.z,
                config.intensity,
            ],
            params: [config.decay, config.weight, config.exposure, config.density],
            params2: [
                config.num_samples as f32,
                config.max_length,
                time * config.animation_speed,
                config.noise_scale,
            ],
            surface_params: [
                surface_height,
                if camera_underwater { 1.0 } else { 0.0 },
                0.0,
                0.0,
            ],
        }
    }
}

/// Represents a light shaft ray for CPU-side calculations
#[derive(Clone, Copy, Debug)]
pub struct LightShaft {
    /// Start position (at water surface)
    pub start: Vec3,
    /// End position (into the water)
    pub end: Vec3,
    /// Ray direction
    pub direction: Vec3,
    /// Intensity at start
    pub intensity: f32,
    /// Width of the ray
    pub width: f32,
}

impl LightShaft {
    /// Create a new light shaft
    pub fn new(start: Vec3, direction: Vec3, length: f32, intensity: f32, width: f32) -> Self {
        Self {
            start,
            end: start + direction * length,
            direction: direction.normalize(),
            intensity,
            width,
        }
    }

    /// Sample intensity at a point along the ray (0-1 parameter)
    pub fn sample_along(&self, t: f32, decay: f32) -> f32 {
        let decay_factor = decay.powf(t * 100.0);
        self.intensity * decay_factor
    }

    /// Check if a world position is within this light shaft
    pub fn contains_point(&self, point: Vec3) -> bool {
        // Project point onto ray
        let to_point = point - self.start;
        let t = to_point.dot(self.direction);
        
        // Check if within ray length
        if t < 0.0 || t > (self.end - self.start).length() {
            return false;
        }

        // Check distance from ray axis
        let closest = self.start + self.direction * t;
        let dist = (point - closest).length();
        
        dist <= self.width
    }

    /// Get intensity at a world position
    pub fn intensity_at(&self, point: Vec3, decay: f32) -> f32 {
        if !self.contains_point(point) {
            return 0.0;
        }

        let to_point = point - self.start;
        let t = to_point.dot(self.direction) / (self.end - self.start).length();
        
        // Distance falloff from ray axis
        let closest = self.start + self.direction * t * (self.end - self.start).length();
        let dist = (point - closest).length();
        let radial_falloff = 1.0 - (dist / self.width).clamp(0.0, 1.0);

        self.sample_along(t, decay) * radial_falloff
    }
}

/// God rays system manager
#[derive(Debug)]
pub struct GodRaysSystem {
    /// Configuration
    config: GodRaysConfig,
    /// Light direction (normalized)
    light_direction: Vec3,
    /// Water surface height
    surface_height: f32,
    /// Generated light shafts
    shafts: Vec<LightShaft>,
    /// Animation time
    time: f32,
    /// Random seed for variation
    seed: u32,
}

impl GodRaysSystem {
    /// Create a new god rays system
    pub fn new(config: GodRaysConfig) -> Self {
        Self {
            config,
            light_direction: Vec3::new(0.0, -1.0, 0.2).normalize(),
            surface_height: 0.0,
            shafts: Vec::new(),
            time: 0.0,
            seed: 12345,
        }
    }

    /// Set light direction
    pub fn set_light_direction(&mut self, direction: Vec3) {
        self.light_direction = direction.normalize();
    }

    /// Get light direction
    pub fn light_direction(&self) -> Vec3 {
        self.light_direction
    }

    /// Set water surface height
    pub fn set_surface_height(&mut self, height: f32) {
        self.surface_height = height;
    }

    /// Get surface height
    pub fn surface_height(&self) -> f32 {
        self.surface_height
    }

    /// Generate simple pseudo-random number
    fn next_random(&mut self) -> f32 {
        self.seed = self.seed.wrapping_mul(1103515245).wrapping_add(12345);
        self.seed as f32 / u32::MAX as f32
    }

    /// Generate light shafts for a camera position
    pub fn generate_shafts(&mut self, camera_pos: Vec3, view_distance: f32) {
        self.shafts.clear();

        // Only generate if camera is underwater
        if camera_pos.y >= self.surface_height {
            return;
        }

        // Generate a grid of potential shaft positions
        let density = self.config.density;
        let spacing = 10.0 / density;
        let half_dist = view_distance * 0.5;

        let min_x = (camera_pos.x - half_dist).floor() as i32;
        let max_x = (camera_pos.x + half_dist).ceil() as i32;
        let min_z = (camera_pos.z - half_dist).floor() as i32;
        let max_z = (camera_pos.z + half_dist).ceil() as i32;

        for x in (min_x..max_x).step_by(spacing.max(1.0) as usize) {
            for z in (min_z..max_z).step_by(spacing.max(1.0) as usize) {
                // Add some randomness to position
                let offset_x = self.next_random() * spacing - spacing * 0.5;
                let offset_z = self.next_random() * spacing - spacing * 0.5;

                let start = Vec3::new(
                    x as f32 + offset_x,
                    self.surface_height,
                    z as f32 + offset_z,
                );

                // Vary intensity and width
                let intensity = self.config.intensity * (0.5 + self.next_random() * 0.5);
                let width = 0.5 + self.next_random() * 1.5;
                let length = self.config.max_length * (0.5 + self.next_random() * 0.5);

                self.shafts.push(LightShaft::new(
                    start,
                    self.light_direction,
                    length,
                    intensity,
                    width,
                ));
            }
        }
    }

    /// Update animation
    pub fn update(&mut self, dt: f32) {
        self.time += dt * self.config.animation_speed;
    }

    /// Sample total god ray intensity at a world position
    pub fn sample(&self, position: Vec3) -> f32 {
        // Only underwater
        if position.y >= self.surface_height {
            return 0.0;
        }

        let mut total = 0.0;
        for shaft in &self.shafts {
            total += shaft.intensity_at(position, self.config.decay);
        }

        (total * self.config.exposure).min(1.0)
    }

    /// Get number of active shafts
    pub fn shaft_count(&self) -> usize {
        self.shafts.len()
    }

    /// Get shafts for rendering
    pub fn shafts(&self) -> &[LightShaft] {
        &self.shafts
    }

    /// Clear all shafts
    pub fn clear_shafts(&mut self) {
        self.shafts.clear();
    }

    /// Get uniforms for GPU rendering
    pub fn get_uniforms(&self, camera_underwater: bool) -> GodRaysUniforms {
        GodRaysUniforms::from_config(
            &self.config,
            self.light_direction,
            self.surface_height,
            camera_underwater,
            self.time,
        )
    }

    /// Get configuration
    pub fn config(&self) -> &GodRaysConfig {
        &self.config
    }

    /// Set configuration
    pub fn set_config(&mut self, config: GodRaysConfig) {
        self.config = config;
    }
}

/// WGSL shader code for god rays (radial blur approach)
pub const GOD_RAYS_WGSL: &str = r#"
// God rays uniforms
struct GodRaysUniforms {
    light_dir_intensity: vec4<f32>,
    params: vec4<f32>,  // decay, weight, exposure, density
    params2: vec4<f32>, // num_samples, max_length, time, noise_scale
    surface_params: vec4<f32>, // surface_height, underwater, padding
}

@group(2) @binding(1) var<uniform> god_rays: GodRaysUniforms;

// Screen-space god rays using radial blur
fn calculate_god_rays(
    uv: vec2<f32>,
    light_screen_pos: vec2<f32>,
    occlusion_texture: texture_2d<f32>,
    occlusion_sampler: sampler,
) -> f32 {
    let num_samples = i32(god_rays.params2.x);
    let decay = god_rays.params.x;
    let weight = god_rays.params.y;
    let exposure = god_rays.params.z;
    let density = god_rays.params.w;
    
    // Ray from current pixel to light
    let delta = (uv - light_screen_pos) * density / f32(num_samples);
    
    var sample_pos = uv;
    var illumination: f32 = 0.0;
    var decay_factor: f32 = 1.0;
    
    for (var i: i32 = 0; i < num_samples; i++) {
        sample_pos -= delta;
        
        // Sample occlusion texture (1 = lit, 0 = occluded)
        let occlusion = textureSample(occlusion_texture, occlusion_sampler, sample_pos).r;
        
        illumination += occlusion * decay_factor * weight;
        decay_factor *= decay;
    }
    
    return illumination * exposure;
}

// Volumetric god rays for underwater (3D raymarching)
fn calculate_volumetric_god_rays(
    world_pos: vec3<f32>,
    camera_pos: vec3<f32>,
    surface_height: f32,
) -> f32 {
    // Only underwater
    if (world_pos.y >= surface_height) {
        return 0.0;
    }
    
    let light_dir = god_rays.light_dir_intensity.xyz;
    let intensity = god_rays.light_dir_intensity.w;
    let max_length = god_rays.params2.y;
    let time = god_rays.params2.z;
    let noise_scale = god_rays.params2.w;
    
    // Calculate depth from surface
    let depth = surface_height - world_pos.y;
    let depth_factor = 1.0 - clamp(depth / max_length, 0.0, 1.0);
    
    // Animated noise for ray variation
    let noise_uv = world_pos.xz * noise_scale + time;
    let noise = (sin(noise_uv.x * 3.14159) * cos(noise_uv.y * 3.14159) + 1.0) * 0.5;
    
    // Ray pattern (using light direction)
    let ray_pattern = max(dot(normalize(vec3<f32>(0.0, 1.0, 0.0)), -light_dir), 0.0);
    
    return intensity * depth_factor * ray_pattern * noise;
}
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = GodRaysConfig::default();
        assert_eq!(config.num_samples, 64);
        assert_eq!(config.intensity, 0.8);
        assert!(config.decay > 0.0 && config.decay < 1.0);
    }

    #[test]
    fn test_config_presets() {
        let tropical = GodRaysConfig::tropical();
        let murky = GodRaysConfig::murky();
        let cinematic = GodRaysConfig::cinematic();
        let low = GodRaysConfig::low_quality();

        // Tropical should be brighter than murky
        assert!(tropical.intensity > murky.intensity);
        
        // Cinematic should have more samples
        assert!(cinematic.num_samples > low.num_samples);
        
        // Low quality should have fewer samples
        assert!(low.num_samples < tropical.num_samples);
    }

    #[test]
    fn test_uniforms_creation() {
        let config = GodRaysConfig::default();
        let uniforms = GodRaysUniforms::from_config(
            &config,
            Vec3::NEG_Y,
            10.0,
            true,
            1.0,
        );

        assert_eq!(uniforms.light_dir_intensity[3], config.intensity);
        assert_eq!(uniforms.surface_params[0], 10.0);
        assert_eq!(uniforms.surface_params[1], 1.0); // underwater = true
    }

    #[test]
    fn test_uniforms_size() {
        assert_eq!(std::mem::size_of::<GodRaysUniforms>(), 64);
    }

    #[test]
    fn test_light_shaft_creation() {
        let shaft = LightShaft::new(
            Vec3::new(0.0, 10.0, 0.0),
            Vec3::NEG_Y,
            50.0,
            1.0,
            2.0,
        );

        assert_eq!(shaft.start.y, 10.0);
        assert_eq!(shaft.end.y, -40.0);
        assert_eq!(shaft.intensity, 1.0);
    }

    #[test]
    fn test_light_shaft_contains_point() {
        let shaft = LightShaft::new(
            Vec3::new(0.0, 10.0, 0.0),
            Vec3::NEG_Y,
            50.0,
            1.0,
            2.0,
        );

        // Point on axis should be contained
        assert!(shaft.contains_point(Vec3::new(0.0, 5.0, 0.0)));
        
        // Point far from axis should not be contained
        assert!(!shaft.contains_point(Vec3::new(10.0, 5.0, 0.0)));
        
        // Point above start should not be contained
        assert!(!shaft.contains_point(Vec3::new(0.0, 20.0, 0.0)));
    }

    #[test]
    fn test_light_shaft_intensity() {
        let shaft = LightShaft::new(
            Vec3::new(0.0, 10.0, 0.0),
            Vec3::NEG_Y,
            50.0,
            1.0,
            2.0,
        );

        let intensity_top = shaft.intensity_at(Vec3::new(0.0, 9.0, 0.0), 0.96);
        let intensity_bottom = shaft.intensity_at(Vec3::new(0.0, -30.0, 0.0), 0.96);

        // Intensity should decrease with depth
        assert!(intensity_top > intensity_bottom);
    }

    #[test]
    fn test_system_creation() {
        let system = GodRaysSystem::new(GodRaysConfig::default());
        
        assert_eq!(system.shaft_count(), 0);
        assert_eq!(system.surface_height(), 0.0);
    }

    #[test]
    fn test_system_light_direction() {
        let mut system = GodRaysSystem::new(GodRaysConfig::default());
        
        system.set_light_direction(Vec3::new(1.0, -1.0, 0.0));
        
        let dir = system.light_direction();
        assert!((dir.length() - 1.0).abs() < 0.001); // Normalized
    }

    #[test]
    fn test_system_generate_shafts_above_water() {
        let mut system = GodRaysSystem::new(GodRaysConfig::default());
        system.set_surface_height(0.0);

        // Camera above water - no shafts
        system.generate_shafts(Vec3::new(0.0, 10.0, 0.0), 50.0);
        assert_eq!(system.shaft_count(), 0);
    }

    #[test]
    fn test_system_generate_shafts_underwater() {
        let mut system = GodRaysSystem::new(GodRaysConfig::default());
        system.set_surface_height(10.0);

        // Camera underwater - should generate shafts
        system.generate_shafts(Vec3::new(0.0, 5.0, 0.0), 20.0);
        assert!(system.shaft_count() > 0);
    }

    #[test]
    fn test_system_clear_shafts() {
        let mut system = GodRaysSystem::new(GodRaysConfig::default());
        system.set_surface_height(10.0);
        system.generate_shafts(Vec3::new(0.0, 5.0, 0.0), 20.0);
        
        system.clear_shafts();
        assert_eq!(system.shaft_count(), 0);
    }

    #[test]
    fn test_system_update() {
        let mut system = GodRaysSystem::new(GodRaysConfig::default());
        let initial_time = system.time;
        
        system.update(0.5);
        
        assert!(system.time > initial_time);
    }

    #[test]
    fn test_system_sample_above_water() {
        let system = GodRaysSystem::new(GodRaysConfig::default());
        
        // Sample above water should return 0
        let sample = system.sample(Vec3::new(0.0, 10.0, 0.0));
        assert_eq!(sample, 0.0);
    }

    #[test]
    fn test_system_get_uniforms() {
        let system = GodRaysSystem::new(GodRaysConfig::default());
        let uniforms = system.get_uniforms(true);

        assert_eq!(uniforms.surface_params[1], 1.0); // underwater
    }

    #[test]
    fn test_wgsl_shader_exists() {
        assert!(!GOD_RAYS_WGSL.is_empty());
        assert!(GOD_RAYS_WGSL.contains("calculate_god_rays"));
        assert!(GOD_RAYS_WGSL.contains("calculate_volumetric_god_rays"));
    }
}
