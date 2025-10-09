//! BRDF Preview Module - Phase PBR-G Task 2.2
//!
//! Provides real-time BRDF visualization with software-rendered sphere:
//! - Cook-Torrance BRDF implementation
//! - Lighting controls (direction, intensity, color)
//! - Material parameter visualization
//! - Quality assessment for PBR materials

use egui::{Color32, ColorImage, TextureHandle, Ui, Vec2};
use glam::Vec3;

/// BRDF Preview state
pub struct BrdfPreview {
    /// Preview resolution (square)
    pub resolution: usize,

    /// Material parameters
    pub albedo: [f32; 3],
    pub metallic: f32,
    pub roughness: f32,

    /// Lighting parameters
    pub light_direction: Vec3,
    pub light_intensity: f32,
    pub light_color: [f32; 3],

    /// Cached preview texture
    texture_handle: Option<TextureHandle>,

    /// Dirty flag for re-rendering
    dirty: bool,
}

impl Default for BrdfPreview {
    fn default() -> Self {
        Self {
            resolution: 256,
            albedo: [0.8, 0.8, 0.8],
            metallic: 0.0,
            roughness: 0.5,
            light_direction: Vec3::new(0.5, 0.7, 0.3).normalize(),
            light_intensity: 1.0,
            light_color: [1.0, 1.0, 1.0],
            texture_handle: None,
            dirty: true,
        }
    }
}

impl BrdfPreview {
    /// Create new BRDF preview with default settings
    pub fn new() -> Self {
        Self::default()
    }

    /// Update material parameters
    pub fn set_material(&mut self, albedo: [f32; 3], metallic: f32, roughness: f32) {
        self.albedo = albedo;
        self.metallic = metallic;
        self.roughness = roughness;
        self.dirty = true;
    }

    /// Update lighting parameters
    pub fn set_lighting(&mut self, direction: Vec3, intensity: f32, color: [f32; 3]) {
        self.light_direction = direction.normalize();
        self.light_intensity = intensity;
        self.light_color = color;
        self.dirty = true;
    }

    /// Render the preview sphere
    fn render_sphere(&self) -> ColorImage {
        let res = self.resolution;
        let mut pixels = vec![Color32::from_rgb(32, 32, 32); res * res];

        let center = res as f32 / 2.0;
        let radius = center * 0.8;

        for y in 0..res {
            for x in 0..res {
                let px = x as f32 - center;
                let py = y as f32 - center;
                let dist_sq = px * px + py * py;

                if dist_sq <= radius * radius {
                    // Calculate sphere surface normal
                    let z = (radius * radius - dist_sq).sqrt();
                    let normal = Vec3::new(px / radius, -py / radius, z / radius).normalize();

                    // Calculate view direction (camera looking down -Z)
                    let view = Vec3::new(0.0, 0.0, 1.0);

                    // Evaluate BRDF
                    let color = self.evaluate_brdf(normal, view, self.light_direction);

                    // Tone mapping and gamma correction
                    let color = Self::tone_map_aces(color);
                    let color = Self::linear_to_srgb(color);

                    pixels[y * res + x] = Color32::from_rgb(
                        (color.x * 255.0).min(255.0) as u8,
                        (color.y * 255.0).min(255.0) as u8,
                        (color.z * 255.0).min(255.0) as u8,
                    );
                }
            }
        }

        ColorImage {
            size: [res, res],
            pixels,
            source_size: Vec2::new(res as f32, res as f32),
        }
    }

    /// Evaluate Cook-Torrance BRDF
    fn evaluate_brdf(&self, normal: Vec3, view: Vec3, light: Vec3) -> Vec3 {
        let n_dot_l = normal.dot(light).max(0.0);
        let n_dot_v = normal.dot(view).max(0.001);

        if n_dot_l <= 0.0 {
            return Vec3::ZERO;
        }

        // Half vector
        let half = (view + light).normalize();
        let n_dot_h = normal.dot(half).max(0.0);
        let v_dot_h = view.dot(half).max(0.0);

        // Fresnel (Schlick approximation)
        let f0 = Vec3::splat(0.04).lerp(
            Vec3::new(self.albedo[0], self.albedo[1], self.albedo[2]),
            self.metallic,
        );
        let f = Self::fresnel_schlick(v_dot_h, f0);

        // Distribution (GGX)
        let alpha = self.roughness * self.roughness;
        let d = Self::distribution_ggx(n_dot_h, alpha);

        // Geometry (Smith)
        let g = Self::geometry_smith(n_dot_v, n_dot_l, alpha);

        // Specular term
        let specular = (d * g * f) / (4.0 * n_dot_v * n_dot_l + 0.001);

        // Diffuse term (Lambertian with energy conservation)
        let k_d = (Vec3::ONE - f) * (1.0 - self.metallic);
        let diffuse =
            k_d * Vec3::new(self.albedo[0], self.albedo[1], self.albedo[2]) / std::f32::consts::PI;

        // Combine diffuse and specular
        let light_color = Vec3::new(
            self.light_color[0],
            self.light_color[1],
            self.light_color[2],
        );
        (diffuse + specular) * light_color * self.light_intensity * n_dot_l
    }

    /// Fresnel-Schlick approximation
    fn fresnel_schlick(cos_theta: f32, f0: Vec3) -> Vec3 {
        f0 + (Vec3::ONE - f0) * (1.0 - cos_theta).powi(5)
    }

    /// GGX/Trowbridge-Reitz normal distribution
    fn distribution_ggx(n_dot_h: f32, alpha: f32) -> f32 {
        let a2 = alpha * alpha;
        let denom = n_dot_h * n_dot_h * (a2 - 1.0) + 1.0;
        a2 / (std::f32::consts::PI * denom * denom)
    }

    /// Smith geometry function (GGX)
    fn geometry_smith(n_dot_v: f32, n_dot_l: f32, alpha: f32) -> f32 {
        let g1_v = Self::geometry_schlick_ggx(n_dot_v, alpha);
        let g1_l = Self::geometry_schlick_ggx(n_dot_l, alpha);
        g1_v * g1_l
    }

    /// Schlick-GGX geometry function
    fn geometry_schlick_ggx(n_dot_x: f32, alpha: f32) -> f32 {
        let k = alpha / 2.0;
        n_dot_x / (n_dot_x * (1.0 - k) + k)
    }

    /// ACES tone mapping
    fn tone_map_aces(color: Vec3) -> Vec3 {
        let a = 2.51;
        let b = 0.03;
        let c = 2.43;
        let d = 0.59;
        let e = 0.14;

        Vec3::new(
            ((color.x * (a * color.x + b)) / (color.x * (c * color.x + d) + e)).clamp(0.0, 1.0),
            ((color.y * (a * color.y + b)) / (color.y * (c * color.y + d) + e)).clamp(0.0, 1.0),
            ((color.z * (a * color.z + b)) / (color.z * (c * color.z + d) + e)).clamp(0.0, 1.0),
        )
    }

    /// Linear to sRGB gamma correction
    fn linear_to_srgb(color: Vec3) -> Vec3 {
        Vec3::new(
            Self::linear_to_srgb_channel(color.x),
            Self::linear_to_srgb_channel(color.y),
            Self::linear_to_srgb_channel(color.z),
        )
    }

    fn linear_to_srgb_channel(c: f32) -> f32 {
        if c <= 0.0031308 {
            12.92 * c
        } else {
            1.055 * c.powf(1.0 / 2.4) - 0.055
        }
    }

    /// Render the preview UI
    pub fn show(&mut self, ui: &mut Ui, ctx: &egui::Context) {
        ui.heading("BRDF Preview");

        // Material controls
        ui.group(|ui| {
            ui.label("Material Parameters");

            let mut changed = false;

            ui.horizontal(|ui| {
                ui.label("Albedo:");
                changed |= ui.color_edit_button_rgb(&mut self.albedo).changed();
            });

            changed |= ui
                .add(egui::Slider::new(&mut self.metallic, 0.0..=1.0).text("Metallic"))
                .changed();

            changed |= ui
                .add(egui::Slider::new(&mut self.roughness, 0.0..=1.0).text("Roughness"))
                .changed();

            if changed {
                self.dirty = true;
            }
        });

        ui.add_space(8.0);

        // Lighting controls
        ui.group(|ui| {
            ui.label("Lighting");

            let mut changed = false;

            // Light direction (simplified - just X/Y controls)
            let mut light_x = self.light_direction.x;
            let mut light_y = self.light_direction.y;

            changed |= ui
                .add(egui::Slider::new(&mut light_x, -1.0..=1.0).text("Light X"))
                .changed();

            changed |= ui
                .add(egui::Slider::new(&mut light_y, -1.0..=1.0).text("Light Y"))
                .changed();

            if changed {
                // Reconstruct Z to maintain unit length
                let z_sq = 1.0 - light_x * light_x - light_y * light_y;
                let light_z = if z_sq > 0.0 { z_sq.sqrt() } else { 0.1 };
                self.light_direction = Vec3::new(light_x, light_y, light_z).normalize();
                self.dirty = true;
            }

            changed = false;
            changed |= ui
                .add(egui::Slider::new(&mut self.light_intensity, 0.0..=5.0).text("Intensity"))
                .changed();

            changed |= ui
                .horizontal(|ui| {
                    ui.label("Color:");
                    ui.color_edit_button_rgb(&mut self.light_color)
                })
                .inner
                .changed();

            if changed {
                self.dirty = true;
            }
        });

        ui.add_space(8.0);

        // Render preview if dirty
        if self.dirty {
            let image = self.render_sphere();
            let texture = ctx.load_texture("brdf_preview", image, Default::default());
            self.texture_handle = Some(texture);
            self.dirty = false;
        }

        // Display preview
        if let Some(texture) = &self.texture_handle {
            ui.image(texture);
        } else {
            ui.label("Rendering preview...");
        }

        ui.add_space(8.0);

        // Info panel
        ui.group(|ui| {
            ui.label("Material Info");
            ui.label(format!(
                "Albedo: RGB({:.2}, {:.2}, {:.2})",
                self.albedo[0], self.albedo[1], self.albedo[2]
            ));
            ui.label(format!("Metallic: {:.2}", self.metallic));
            ui.label(format!("Roughness: {:.2}", self.roughness));
            ui.label(format!(
                "Light: ({:.2}, {:.2}, {:.2})",
                self.light_direction.x, self.light_direction.y, self.light_direction.z
            ));
        });
    }
}
