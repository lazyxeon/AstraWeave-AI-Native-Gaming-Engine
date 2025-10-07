//! PBR-E Advanced Materials Demo Module
//! 
//! This module provides helper functions to generate demonstration scenes
//! for Phase PBR-E advanced material features (clearcoat, anisotropy, SSS, sheen, transmission).

use astraweave_render::material_extended::*;
use glam::Vec3;

/// Material type for demo scene selection
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DemoMaterialType {
    Clearcoat,
    Anisotropy,
    Subsurface,
    Sheen,
    Transmission,
}

impl DemoMaterialType {
    pub fn all() -> &'static [DemoMaterialType] {
        &[
            DemoMaterialType::Clearcoat,
            DemoMaterialType::Anisotropy,
            DemoMaterialType::Subsurface,
            DemoMaterialType::Sheen,
            DemoMaterialType::Transmission,
        ]
    }
    
    pub fn name(&self) -> &'static str {
        match self {
            DemoMaterialType::Clearcoat => "Clearcoat (Car Paint)",
            DemoMaterialType::Anisotropy => "Anisotropy (Brushed Metal)",
            DemoMaterialType::Subsurface => "Subsurface (Skin)",
            DemoMaterialType::Sheen => "Sheen (Velvet)",
            DemoMaterialType::Transmission => "Transmission (Glass)",
        }
    }
}

/// Configuration for PBR-E demo scene
#[derive(Clone)]
pub struct PbrEDemoConfig {
    pub material_type: DemoMaterialType,
    pub grid_size: usize,
    pub sphere_spacing: f32,
    pub sphere_radius: f32,
}

impl Default for PbrEDemoConfig {
    fn default() -> Self {
        Self {
            material_type: DemoMaterialType::Clearcoat,
            grid_size: 5,
            sphere_spacing: 2.5,
            sphere_radius: 0.8,
        }
    }
}

/// Generate sphere grid scene for PBR-E material demonstration
/// Returns (materials, positions) where materials[i] corresponds to positions[i]
pub fn generate_demo_scene(config: &PbrEDemoConfig) -> (Vec<MaterialGpuExtended>, Vec<Vec3>) {
    let mut materials = Vec::with_capacity(config.grid_size * config.grid_size);
    let mut positions = Vec::with_capacity(config.grid_size * config.grid_size);
    
    let offset = (config.grid_size as f32 - 1.0) * config.sphere_spacing * 0.5;
    
    for y in 0..config.grid_size {
        for x in 0..config.grid_size {
            let param_x = x as f32 / (config.grid_size - 1) as f32;
            let param_y = y as f32 / (config.grid_size - 1) as f32;
            
            let mat = match config.material_type {
                DemoMaterialType::Clearcoat => generate_clearcoat(param_x, param_y),
                DemoMaterialType::Anisotropy => generate_anisotropy(param_x, param_y),
                DemoMaterialType::Subsurface => generate_subsurface(param_x, param_y),
                DemoMaterialType::Sheen => generate_sheen(param_x, param_y),
                DemoMaterialType::Transmission => generate_transmission(param_x, param_y),
            };
            
            let pos = Vec3::new(
                x as f32 * config.sphere_spacing - offset,
                config.sphere_radius, // Elevate spheres off ground
                y as f32 * config.sphere_spacing - offset,
            );
            
            materials.push(mat);
            positions.push(pos);
        }
    }
    
    (materials, positions)
}

/// Clearcoat grid: X = strength (0→1), Y = roughness (0→1)
fn generate_clearcoat(strength: f32, roughness: f32) -> MaterialGpuExtended {
    let mut mat = MaterialGpuExtended::car_paint(
        Vec3::new(0.8, 0.0, 0.0),  // Red base
        0.9,  // High metallic
        0.3,  // Base roughness
    );
    mat.clearcoat_strength = strength;
    mat.clearcoat_roughness = roughness;
    mat
}

/// Anisotropy grid: X = strength (-1→1), Y = rotation (0→2π)
fn generate_anisotropy(strength_norm: f32, rotation_norm: f32) -> MaterialGpuExtended {
    let strength = strength_norm * 2.0 - 1.0;  // Map [0,1] → [-1,1]
    let rotation = rotation_norm * std::f32::consts::TAU;  // Map [0,1] → [0,2π]
    
    MaterialGpuExtended::brushed_metal(
        Vec3::new(0.9, 0.9, 0.9),  // Silver
        0.4,  // Medium roughness
        strength,
        rotation,
    )
}

/// Subsurface grid: X = scale (0→1), Y = radius (0→5mm)
fn generate_subsurface(scale: f32, radius_norm: f32) -> MaterialGpuExtended {
    let radius = radius_norm * 5.0;  // Map [0,1] → [0,5mm]
    
    MaterialGpuExtended::skin(
        Vec3::new(0.95, 0.8, 0.7),  // Skin tone
        Vec3::new(0.9, 0.3, 0.3),  // Reddish subsurface
        radius,
        scale,
    )
}

/// Sheen grid: X = sheen intensity (0→1), Y = roughness (0→1)
fn generate_sheen(intensity: f32, roughness: f32) -> MaterialGpuExtended {
    let sheen_color = Vec3::ONE * intensity;  // White sheen with variable intensity
    
    MaterialGpuExtended::velvet(
        Vec3::new(0.5, 0.0, 0.1),  // Deep red base
        sheen_color,
        roughness,
    )
}

/// Transmission grid: X = transmission factor (0→1), Y = IOR (1.0→2.5)
fn generate_transmission(transmission: f32, ior_norm: f32) -> MaterialGpuExtended {
    let ior = 1.0 + ior_norm * 1.5;  // Map [0,1] → [1.0,2.5]
    
    MaterialGpuExtended::glass(
        Vec3::ONE,  // Clear tint
        0.05,  // Very smooth
        transmission,
        ior,
        Vec3::new(0.9, 1.0, 0.9),  // Slight green tint
        10.0,  // 10cm attenuation distance
    )
}

/// Get parameter labels for UI display
pub fn get_param_labels(material_type: DemoMaterialType) -> (&'static str, &'static str) {
    match material_type {
        DemoMaterialType::Clearcoat => ("Strength (0→1)", "Roughness (0→1)"),
        DemoMaterialType::Anisotropy => ("Strength (-1→1)", "Rotation (0→2π)"),
        DemoMaterialType::Subsurface => ("Scale (0→1)", "Radius (0→5mm)"),
        DemoMaterialType::Sheen => ("Intensity (0→1)", "Roughness (0→1)"),
        DemoMaterialType::Transmission => ("Factor (0→1)", "IOR (1.0→2.5)"),
    }
}

/// Get description text for each material type
pub fn get_description(material_type: DemoMaterialType) -> &'static str {
    match material_type {
        DemoMaterialType::Clearcoat => {
            "Clearcoat adds a glossy coating layer (IOR 1.5, F0=0.04) on top of the base material. \
            Common uses: car paint, lacquer, varnish. Left→Right: coating strength increases. \
            Bottom→Top: coating roughness increases (more diffuse reflection)."
        }
        DemoMaterialType::Anisotropy => {
            "Anisotropic materials have directional highlights (elliptical GGX distribution). \
            Common uses: brushed metal, hair, fabric. Left→Right: anisotropy strength (-1=vertical, +1=horizontal). \
            Bottom→Top: groove rotation (0→2π radians)."
        }
        DemoMaterialType::Subsurface => {
            "Subsurface scattering (SSS) simulates light penetrating and scattering inside translucent materials. \
            Uses Burley diffusion profile. Common uses: skin, wax, marble. Left→Right: SSS scale (blend with Lambertian). \
            Bottom→Top: scattering radius increases (deeper light penetration)."
        }
        DemoMaterialType::Sheen => {
            "Sheen adds retroreflection at grazing angles (Charlie distribution). \
            Common uses: velvet, satin, cloth. Left→Right: sheen intensity increases. \
            Bottom→Top: sheen roughness increases (broader retroreflection peak)."
        }
        DemoMaterialType::Transmission => {
            "Transmission simulates transparent materials with refraction (Snell's law) and absorption (Beer-Lambert). \
            Common uses: glass, water, ice. Left→Right: transmission factor increases (more transparent). \
            Bottom→Top: IOR increases (1.0=air, 1.5=glass, 2.5=diamond, stronger refraction)."
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_demo_scene_generation() {
        let config = PbrEDemoConfig {
            material_type: DemoMaterialType::Clearcoat,
            grid_size: 4,
            sphere_spacing: 2.0,
            sphere_radius: 0.5,
        };
        
        let (materials, positions) = generate_demo_scene(&config);
        
        assert_eq!(materials.len(), 16);
        assert_eq!(positions.len(), 16);
        
        // Verify positions are centered
        let offset = (4.0 - 1.0) * 2.0 * 0.5;
        assert!((positions[0].x - (-offset)).abs() < 1e-5);
        assert!((positions[15].x - offset).abs() < 1e-5);
    }

    #[test]
    fn test_all_material_types() {
        for material_type in DemoMaterialType::all() {
            let config = PbrEDemoConfig {
                material_type: *material_type,
                grid_size: 3,
                ..Default::default()
            };
            
            let (materials, positions) = generate_demo_scene(&config);
            assert_eq!(materials.len(), 9);
            assert_eq!(positions.len(), 9);
            
            // Verify material has correct feature flag
            let expected_flag = match material_type {
                DemoMaterialType::Clearcoat => MATERIAL_FLAG_CLEARCOAT,
                DemoMaterialType::Anisotropy => MATERIAL_FLAG_ANISOTROPY,
                DemoMaterialType::Subsurface => MATERIAL_FLAG_SUBSURFACE,
                DemoMaterialType::Sheen => MATERIAL_FLAG_SHEEN,
                DemoMaterialType::Transmission => MATERIAL_FLAG_TRANSMISSION,
            };
            
            assert_eq!(materials[0].flags & expected_flag, expected_flag);
        }
    }
}
