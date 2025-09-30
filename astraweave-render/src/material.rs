use std::{collections::HashMap, path::PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MaterialLayerDesc {
    pub key: String,
    pub albedo: Option<PathBuf>,
    pub normal: Option<PathBuf>,
    pub mra: Option<PathBuf>,
    pub metallic: Option<PathBuf>,
    pub roughness: Option<PathBuf>,
    pub ao: Option<PathBuf>,
    pub tiling: [f32; 2],
    pub triplanar_scale: f32,
    pub atlas: Option<String>,
}

impl Default for MaterialLayerDesc {
    fn default() -> Self {
        Self {
            key: String::new(),
            albedo: None,
            normal: None,
            mra: None,
            metallic: None,
            roughness: None,
            ao: None,
            tiling: [1.0, 1.0],
            triplanar_scale: 16.0,
            atlas: None,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct MaterialPackDesc {
    pub biome: String,
    pub layers: Vec<MaterialLayerDesc>,
}

#[derive(Clone, Debug, Default)]
pub struct ArrayLayout {
    pub layer_indices: HashMap<String, u32>,
    pub count: u32,
}

pub struct MaterialGpuArrays {
    pub albedo: wgpu::TextureView,
    pub normal: wgpu::TextureView,
    pub mra: wgpu::TextureView,
    pub sampler_albedo: wgpu::Sampler,
    pub sampler_linear: wgpu::Sampler,
    pub layout: ArrayLayout,
}

#[derive(Clone, Debug, Default)]
pub struct MaterialLoadStats {
    pub biome: String,
    pub layers_total: usize,
    pub albedo_loaded: usize,
    pub albedo_substituted: usize,
    pub normal_loaded: usize,
    pub normal_substituted: usize,
    pub mra_loaded: usize,
    pub mra_packed: usize,
    pub mra_substituted: usize,
    pub gpu_memory_bytes: u64,
}

impl MaterialLoadStats {
    /// Returns a concise single-line summary suitable for logs/telemetry.
    pub fn concise_summary(&self) -> String {
        format!(
            "[materials] biome={} layers={} | albedo L/S={}/{} | normal L/S={}/{} | mra L+P/S={}+{}/{} | gpu={:.2} MiB",
            self.biome,
            self.layers_total,
            self.albedo_loaded,
            self.albedo_substituted,
            self.normal_loaded,
            self.normal_substituted,
            self.mra_loaded,
            self.mra_packed,
            self.mra_substituted,
            (self.gpu_memory_bytes as f64) / (1024.0 * 1024.0)
        )
    }
}

pub struct MaterialManager {
    // Keep strong refs to textures so views remain valid
    _albedo_tex: Option<wgpu::Texture>,
    _normal_tex: Option<wgpu::Texture>,
    _mra_tex: Option<wgpu::Texture>,
}

impl MaterialManager {
    pub fn new() -> Self {
        Self {
            _albedo_tex: None,
            _normal_tex: None,
            _mra_tex: None,
        }
    }
}

#[cfg(feature = "textures")]
impl MaterialManager {
    /// Load a pack from authored TOML files under assets/materials/{biome}
    pub async fn load_pack_from_toml(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        base_dir: &std::path::Path,
        materials_toml: &std::path::Path,
        arrays_toml: &std::path::Path,
    ) -> Result<(MaterialGpuArrays, MaterialLoadStats)> {
        // Parse materials.toml
        #[derive(Deserialize)]
        struct MaterialsDoc {
            biome: BiomeHeader,
            #[serde(default)]
            layer: Vec<MaterialLayerToml>,
        }
        #[derive(Deserialize)]
        struct BiomeHeader {
            name: String,
        }
        #[derive(Deserialize, Default)]
        struct MaterialLayerToml {
            key: String,
            albedo: Option<String>,
            normal: Option<String>,
            mra: Option<String>,
            metallic: Option<String>,
            roughness: Option<String>,
            ao: Option<String>,
            #[serde(default = "default_tiling")]
            tiling: [f32; 2],
            #[serde(default = "default_triplanar")]
            triplanar_scale: f32,
            atlas: Option<String>,
        }
        fn default_tiling() -> [f32; 2] {
            [1.0, 1.0]
        }
        fn default_triplanar() -> f32 {
            16.0
        }

        let mats_src = std::fs::read_to_string(materials_toml)
            .with_context(|| format!("read {}", materials_toml.display()))?;
        let doc: MaterialsDoc = toml::from_str(&mats_src)
            .with_context(|| format!("parse {}", materials_toml.display()))?;

        // Parse arrays.toml mapping
        #[derive(Deserialize)]
        struct ArraysDoc {
            layers: HashMap<String, u32>,
        }
        let arrays_src = std::fs::read_to_string(arrays_toml)
            .with_context(|| format!("read {}", arrays_toml.display()))?;
        let arrays: ArraysDoc = toml::from_str(&arrays_src)
            .with_context(|| format!("parse {}", arrays_toml.display()))?;

        let mut layers: Vec<(String, MaterialLayerDesc)> = Vec::new();
        let mut skipped = 0usize;
        for l in doc.layer {
            if !arrays.layers.contains_key(&l.key) {
                skipped += 1;
                eprintln!(
                    "[materials] arrays.toml missing key '{}' → skip layer",
                    l.key
                );
                continue;
            }
            let to_path =
                |s: Option<String>| -> Option<PathBuf> { s.map(|p| base_dir.join(p).normalize()) };
            let desc = MaterialLayerDesc {
                key: l.key.clone(),
                albedo: to_path(l.albedo),
                normal: to_path(l.normal),
                mra: to_path(l.mra),
                metallic: to_path(l.metallic),
                roughness: to_path(l.roughness),
                ao: to_path(l.ao),
                tiling: l.tiling,
                triplanar_scale: l.triplanar_scale,
                atlas: l.atlas,
            };
            layers.push((l.key, desc));
        }

        // Stable order by arrays mapping index
        layers.sort_by_key(|(k, _)| arrays.layers.get(k).copied().unwrap_or(u32::MAX));

        // Upload into texture arrays (delegated to helper in this module)
        let (gpu, stats, albedo_tex, normal_tex, mra_tex) =
            crate::material_loader::material_loader_impl::build_arrays(
                device,
                queue,
                &layers,
                &arrays.layers,
                &doc.biome.name,
            )?;

        self._albedo_tex = Some(albedo_tex);
        self._normal_tex = Some(normal_tex);
        self._mra_tex = Some(mra_tex);

        if skipped > 0 {
            eprintln!(
                "[materials] skipped {} layers not present in arrays.toml",
                skipped
            );
        }

        Ok((gpu, stats))
    }

    pub fn unload_current(&mut self) {
        self._albedo_tex = None;
        self._normal_tex = None;
        self._mra_tex = None;
    }
}

#[cfg(not(feature = "textures"))]
impl MaterialManager {
    pub async fn load_pack_from_toml(
        &mut self,
        _device: &wgpu::Device,
        _queue: &wgpu::Queue,
        _base_dir: &std::path::Path,
        _materials_toml: &std::path::Path,
        _arrays_toml: &std::path::Path,
    ) -> anyhow::Result<(MaterialGpuArrays, MaterialLoadStats)> {
        Err(anyhow::anyhow!(
            "textures feature is disabled; material packs are unavailable"
        ))
    }

    pub fn unload_current(&mut self) { /* no-op */
    }
}

// Small helper to normalize PathBuf joins (remove .. etc.)
trait NormalizePath {
    fn normalize(self) -> PathBuf;
}
impl NormalizePath for PathBuf {
    fn normalize(self) -> PathBuf {
        std::path::Path::new(".")
            .join(self)
            .components()
            .as_path()
            .to_path_buf()
    }
}
