// Phase PBR-E: Rust-side Material Definitions
// Corresponds to MaterialGpuExtended in pbr_advanced.wgsl

use bytemuck::{Pod, Zeroable};
use glam::{Vec2, Vec3, Vec4};

/// Extended material structure supporting advanced PBR features
/// Size: 256 bytes (16-byte aligned for UBO/SSBO)
#[repr(C, align(16))]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct MaterialGpuExtended {
    // Base PBR (Phase PBR-D) - 64 bytes
    pub albedo_index: u32,
    pub normal_index: u32,
    pub orm_index: u32,
    pub flags: u32,
    
    pub base_color_factor: [f32; 4],
    pub metallic_factor: f32,
    pub roughness_factor: f32,
    pub occlusion_strength: f32,
    pub _pad0: f32,
    
    pub emissive_factor: [f32; 3],
    pub _pad1: f32,
    
    // Clearcoat (car paint, lacquer) - 16 bytes
    pub clearcoat_strength: f32,
    pub clearcoat_roughness: f32,
    pub clearcoat_normal_index: u32,
    pub _pad2: f32,
    
    // Anisotropy (brushed metal, hair) - 16 bytes
    pub anisotropy_strength: f32,
    pub anisotropy_rotation: f32,
    pub _pad3: [f32; 2],
    
    // Subsurface Scattering (skin, wax) - 32 bytes
    pub subsurface_color: [f32; 3],
    pub subsurface_scale: f32,
    pub subsurface_radius: f32,
    pub thickness_index: u32,
    pub _pad4: [f32; 2],
    
    // Sheen (velvet, fabric) - 16 bytes
    pub sheen_color: [f32; 3],
    pub sheen_roughness: f32,
    
    // Transmission (glass, water) - 48 bytes (increased for alignment)
    pub transmission_factor: f32,
    pub ior: f32,
    pub _pad5: [f32; 2],
    
    pub attenuation_color: [f32; 3],
    pub attenuation_distance: f32,
    
    // Additional padding to reach 256 bytes - 80 bytes
    pub _pad_final: [f32; 20],
}

// Feature flags (bitfield in MaterialGpuExtended.flags)
pub const MATERIAL_FLAG_CLEARCOAT: u32 = 0x01;
pub const MATERIAL_FLAG_ANISOTROPY: u32 = 0x02;
pub const MATERIAL_FLAG_SUBSURFACE: u32 = 0x04;
pub const MATERIAL_FLAG_SHEEN: u32 = 0x08;
pub const MATERIAL_FLAG_TRANSMISSION: u32 = 0x10;

impl Default for MaterialGpuExtended {
    fn default() -> Self {
        Self {
            // Base PBR defaults
            albedo_index: 0,
            normal_index: 0,
            orm_index: 0,
            flags: 0,
            base_color_factor: [1.0, 1.0, 1.0, 1.0],
            metallic_factor: 0.0,
            roughness_factor: 0.5,
            occlusion_strength: 1.0,
            _pad0: 0.0,
            emissive_factor: [0.0, 0.0, 0.0],
            _pad1: 0.0,
            
            // Clearcoat defaults (disabled)
            clearcoat_strength: 0.0,
            clearcoat_roughness: 0.03,
            clearcoat_normal_index: 0,
            _pad2: 0.0,
            
            // Anisotropy defaults (disabled)
            anisotropy_strength: 0.0,
            anisotropy_rotation: 0.0,
            _pad3: [0.0, 0.0],
            
            // Subsurface defaults (disabled)
            subsurface_color: [1.0, 1.0, 1.0],
            subsurface_scale: 0.0,
            subsurface_radius: 1.0,
            thickness_index: 0,
            _pad4: [0.0, 0.0],
            
            // Sheen defaults (disabled)
            sheen_color: [0.0, 0.0, 0.0],
            sheen_roughness: 0.5,
            
            // Transmission defaults (disabled)
            transmission_factor: 0.0,
            ior: 1.5,
            _pad5: [0.0, 0.0],
            attenuation_color: [1.0, 1.0, 1.0],
            attenuation_distance: 1.0,
            _pad_final: [0.0; 20],
        }
    }
}

impl MaterialGpuExtended {
    /// Create a car paint material (metallic base + clearcoat)
    pub fn car_paint(base_color: Vec3, metallic: f32, roughness: f32) -> Self {
        let mut mat = Self::default();
        mat.base_color_factor = [base_color.x, base_color.y, base_color.z, 1.0];
        mat.metallic_factor = metallic;
        mat.roughness_factor = roughness;
        mat.clearcoat_strength = 1.0;
        mat.clearcoat_roughness = 0.05; // Glossy clear coat
        mat.flags |= MATERIAL_FLAG_CLEARCOAT;
        mat
    }
    
    /// Create a brushed metal material (anisotropic reflections)
    pub fn brushed_metal(base_color: Vec3, roughness: f32, anisotropy: f32, rotation: f32) -> Self {
        let mut mat = Self::default();
        mat.base_color_factor = [base_color.x, base_color.y, base_color.z, 1.0];
        mat.metallic_factor = 1.0;
        mat.roughness_factor = roughness;
        mat.anisotropy_strength = anisotropy;
        mat.anisotropy_rotation = rotation;
        mat.flags |= MATERIAL_FLAG_ANISOTROPY;
        mat
    }
    
    /// Create a skin material (subsurface scattering)
    pub fn skin(base_color: Vec3, subsurface_tint: Vec3, radius: f32, scale: f32) -> Self {
        let mut mat = Self::default();
        mat.base_color_factor = [base_color.x, base_color.y, base_color.z, 1.0];
        mat.metallic_factor = 0.0;
        mat.roughness_factor = 0.5;
        mat.subsurface_color = [subsurface_tint.x, subsurface_tint.y, subsurface_tint.z];
        mat.subsurface_radius = radius;
        mat.subsurface_scale = scale;
        mat.flags |= MATERIAL_FLAG_SUBSURFACE;
        mat
    }
    
    /// Create a velvet/fabric material (sheen)
    pub fn velvet(base_color: Vec3, sheen_color: Vec3, sheen_roughness: f32) -> Self {
        let mut mat = Self::default();
        mat.base_color_factor = [base_color.x, base_color.y, base_color.z, 1.0];
        mat.metallic_factor = 0.0;
        mat.roughness_factor = 0.8;
        mat.sheen_color = [sheen_color.x, sheen_color.y, sheen_color.z];
        mat.sheen_roughness = sheen_roughness;
        mat.flags |= MATERIAL_FLAG_SHEEN;
        mat
    }
    
    /// Create a glass material (transmission)
    pub fn glass(
        tint: Vec3,
        roughness: f32,
        transmission: f32,
        ior: f32,
        attenuation_color: Vec3,
        attenuation_dist: f32
    ) -> Self {
        let mut mat = Self::default();
        mat.base_color_factor = [tint.x, tint.y, tint.z, 1.0];
        mat.metallic_factor = 0.0;
        mat.roughness_factor = roughness;
        mat.transmission_factor = transmission;
        mat.ior = ior;
        mat.attenuation_color = [attenuation_color.x, attenuation_color.y, attenuation_color.z];
        mat.attenuation_distance = attenuation_dist;
        mat.flags |= MATERIAL_FLAG_TRANSMISSION;
        mat
    }
    
    /// Check if a feature is enabled
    pub fn has_feature(&self, flag: u32) -> bool {
        (self.flags & flag) != 0
    }
    
    /// Enable a feature flag
    pub fn enable_feature(&mut self, flag: u32) {
        self.flags |= flag;
    }
    
    /// Disable a feature flag
    pub fn disable_feature(&mut self, flag: u32) {
        self.flags &= !flag;
    }
}

/// TOML representation for material authoring
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct MaterialDefinitionExtended {
    pub name: String,
    
    // Base PBR
    pub albedo: Option<String>,
    pub normal: Option<String>,
    pub orm: Option<String>,
    #[serde(default = "default_one_vec4")]
    pub base_color_factor: [f32; 4],
    #[serde(default)]
    pub metallic_factor: f32,
    #[serde(default = "default_half")]
    pub roughness_factor: f32,
    #[serde(default = "default_one")]
    pub occlusion_strength: f32,
    #[serde(default)]
    pub emissive_factor: [f32; 3],
    
    // Clearcoat
    #[serde(default)]
    pub clearcoat_strength: f32,
    #[serde(default = "default_clearcoat_roughness")]
    pub clearcoat_roughness: f32,
    pub clearcoat_normal: Option<String>,
    
    // Anisotropy
    #[serde(default)]
    pub anisotropy_strength: f32,
    #[serde(default)]
    pub anisotropy_rotation: f32,
    
    // Subsurface
    #[serde(default = "default_one_vec3")]
    pub subsurface_color: [f32; 3],
    #[serde(default)]
    pub subsurface_scale: f32,
    #[serde(default = "default_one")]
    pub subsurface_radius: f32,
    pub thickness_map: Option<String>,
    
    // Sheen
    #[serde(default)]
    pub sheen_color: [f32; 3],
    #[serde(default = "default_half")]
    pub sheen_roughness: f32,
    
    // Transmission
    #[serde(default)]
    pub transmission_factor: f32,
    #[serde(default = "default_ior")]
    pub ior: f32,
    #[serde(default = "default_one_vec3")]
    pub attenuation_color: [f32; 3],
    #[serde(default = "default_one")]
    pub attenuation_distance: f32,
}

// TOML default helpers
fn default_one() -> f32 { 1.0 }
fn default_half() -> f32 { 0.5 }
fn default_one_vec3() -> [f32; 3] { [1.0, 1.0, 1.0] }
fn default_one_vec4() -> [f32; 4] { [1.0, 1.0, 1.0, 1.0] }
fn default_clearcoat_roughness() -> f32 { 0.03 }
fn default_ior() -> f32 { 1.5 }

impl MaterialDefinitionExtended {
    /// Convert TOML definition to GPU representation
    pub fn to_gpu(
        &self,
        albedo_index: u32,
        normal_index: u32,
        orm_index: u32,
        clearcoat_normal_index: u32,
        thickness_index: u32
    ) -> MaterialGpuExtended {
        let mut gpu = MaterialGpuExtended {
            albedo_index,
            normal_index,
            orm_index,
            flags: 0,
            base_color_factor: self.base_color_factor,
            metallic_factor: self.metallic_factor,
            roughness_factor: self.roughness_factor,
            occlusion_strength: self.occlusion_strength,
            _pad0: 0.0,
            emissive_factor: self.emissive_factor,
            _pad1: 0.0,
            
            clearcoat_strength: self.clearcoat_strength,
            clearcoat_roughness: self.clearcoat_roughness,
            clearcoat_normal_index,
            _pad2: 0.0,
            
            anisotropy_strength: self.anisotropy_strength,
            anisotropy_rotation: self.anisotropy_rotation,
            _pad3: [0.0, 0.0],
            
            subsurface_color: self.subsurface_color,
            subsurface_scale: self.subsurface_scale,
            subsurface_radius: self.subsurface_radius,
            thickness_index,
            _pad4: [0.0, 0.0],
            
            sheen_color: self.sheen_color,
            sheen_roughness: self.sheen_roughness,
            
            transmission_factor: self.transmission_factor,
            ior: self.ior,
            _pad5: [0.0, 0.0],
            attenuation_color: self.attenuation_color,
            attenuation_distance: self.attenuation_distance,
            _pad_final: [0.0; 20],
        };
        
        // Set feature flags based on non-zero parameters
        if self.clearcoat_strength > 0.0 {
            gpu.flags |= MATERIAL_FLAG_CLEARCOAT;
        }
        if self.anisotropy_strength.abs() > 0.001 {
            gpu.flags |= MATERIAL_FLAG_ANISOTROPY;
        }
        if self.subsurface_scale > 0.0 {
            gpu.flags |= MATERIAL_FLAG_SUBSURFACE;
        }
        let sheen_max = self.sheen_color.iter().fold(0.0f32, |a, &b| a.max(b));
        if sheen_max > 0.0 {
            gpu.flags |= MATERIAL_FLAG_SHEEN;
        }
        if self.transmission_factor > 0.0 {
            gpu.flags |= MATERIAL_FLAG_TRANSMISSION;
        }
        
        gpu
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_material_size() {
        // Verify 256-byte size for GPU alignment
        assert_eq!(std::mem::size_of::<MaterialGpuExtended>(), 256);
        assert_eq!(std::mem::align_of::<MaterialGpuExtended>(), 16);
    }
    
    #[test]
    fn test_car_paint_material() {
        let mat = MaterialGpuExtended::car_paint(Vec3::new(0.8, 0.0, 0.0), 0.9, 0.3);
        assert!(mat.has_feature(MATERIAL_FLAG_CLEARCOAT));
        assert_eq!(mat.clearcoat_strength, 1.0);
        assert_eq!(mat.metallic_factor, 0.9);
    }
    
    #[test]
    fn test_brushed_metal_material() {
        let mat = MaterialGpuExtended::brushed_metal(Vec3::new(0.9, 0.9, 0.9), 0.4, 0.8, 0.0);
        assert!(mat.has_feature(MATERIAL_FLAG_ANISOTROPY));
        assert_eq!(mat.metallic_factor, 1.0);
        assert_eq!(mat.anisotropy_strength, 0.8);
    }
    
    #[test]
    fn test_skin_material() {
        let mat = MaterialGpuExtended::skin(
            Vec3::new(0.95, 0.8, 0.7),
            Vec3::new(0.9, 0.3, 0.3),
            1.5,
            0.7
        );
        assert!(mat.has_feature(MATERIAL_FLAG_SUBSURFACE));
        assert_eq!(mat.subsurface_scale, 0.7);
        assert_eq!(mat.metallic_factor, 0.0);
    }
    
    #[test]
    fn test_velvet_material() {
        let mat = MaterialGpuExtended::velvet(
            Vec3::new(0.5, 0.0, 0.1),
            Vec3::ONE,
            0.3
        );
        assert!(mat.has_feature(MATERIAL_FLAG_SHEEN));
        assert_eq!(mat.sheen_roughness, 0.3);
    }
    
    #[test]
    fn test_glass_material() {
        let mat = MaterialGpuExtended::glass(
            Vec3::ONE,
            0.05,
            0.95,
            1.5,
            Vec3::new(0.9, 1.0, 0.9),
            10.0
        );
        assert!(mat.has_feature(MATERIAL_FLAG_TRANSMISSION));
        assert_eq!(mat.ior, 1.5);
        assert_eq!(mat.transmission_factor, 0.95);
    }
    
    #[test]
    fn test_feature_flags() {
        let mut mat = MaterialGpuExtended::default();
        assert!(!mat.has_feature(MATERIAL_FLAG_CLEARCOAT));
        
        mat.enable_feature(MATERIAL_FLAG_CLEARCOAT);
        assert!(mat.has_feature(MATERIAL_FLAG_CLEARCOAT));
        
        mat.disable_feature(MATERIAL_FLAG_CLEARCOAT);
        assert!(!mat.has_feature(MATERIAL_FLAG_CLEARCOAT));
    }
    
    #[test]
    fn test_toml_conversion() {
        let def = MaterialDefinitionExtended {
            name: "test_car_paint".to_string(),
            albedo: Some("red_albedo.ktx2".to_string()),
            normal: Some("normal.ktx2".to_string()),
            orm: Some("orm.ktx2".to_string()),
            base_color_factor: [0.8, 0.0, 0.0, 1.0],
            metallic_factor: 0.9,
            roughness_factor: 0.3,
            occlusion_strength: 1.0,
            emissive_factor: [0.0, 0.0, 0.0],
            clearcoat_strength: 1.0,
            clearcoat_roughness: 0.05,
            clearcoat_normal: None,
            anisotropy_strength: 0.0,
            anisotropy_rotation: 0.0,
            subsurface_color: [1.0, 1.0, 1.0],
            subsurface_scale: 0.0,
            subsurface_radius: 1.0,
            thickness_map: None,
            sheen_color: [0.0, 0.0, 0.0],
            sheen_roughness: 0.5,
            transmission_factor: 0.0,
            ior: 1.5,
            attenuation_color: [1.0, 1.0, 1.0],
            attenuation_distance: 1.0,
        };
        
        let gpu = def.to_gpu(0, 1, 2, 0, 0);
        assert!(gpu.has_feature(MATERIAL_FLAG_CLEARCOAT));
        assert!(!gpu.has_feature(MATERIAL_FLAG_ANISOTROPY));
        assert_eq!(gpu.metallic_factor, 0.9);
        assert_eq!(gpu.clearcoat_strength, 1.0);
    }
}
