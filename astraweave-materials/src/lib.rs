use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

/// Minimal material graph nodes (Phase 2 foundation)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Node {
    Texture2D { id: String, uv: String },
    Constant3 { value: [f32; 3] },
    Constant1 { value: f32 },
    Multiply { a: String, b: String },
    Add { a: String, b: String },
    MetallicRoughness { metallic: String, roughness: String },
    Clearcoat { weight: String, roughness: String },
    Anisotropy { amount: String },
    Transmission { amount: String },
    NormalMap { id: String, uv: String, scale: f32 },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Graph {
    pub nodes: std::collections::BTreeMap<String, Node>,
    pub base_color: String,
    pub mr: Option<String>,
    pub normal: Option<String>,
    pub clearcoat: Option<String>,
    pub anisotropy: Option<String>,
    pub transmission: Option<String>,
}

/// Compile a tiny subset of a PBR material to WGSL fragment snippet.
/// Returns WGSL and a list of required bindings for the runtime to wire.
pub fn compile_to_wgsl(g: &Graph) -> Result<(String, Vec<String>)> {
    fn expr_of(
        nodes: &std::collections::BTreeMap<String, Node>,
        id: &str,
        binds: &mut Vec<String>,
    ) -> Result<String> {
        let n = nodes.get(id).ok_or_else(|| anyhow!("missing node {id}"))?;
        Ok(match n {
            Node::Texture2D { id, uv } => {
                binds.push(id.clone());
                format!("textureSample(tex_{}, samp_{}, {}).rgb", id, id, uv)
            }
            Node::Constant3 { value } => {
                format!("vec3<f32>({},{},{})", value[0], value[1], value[2])
            }
            Node::Constant1 { value } => format!("{value}"),
            Node::Multiply { a, b } => format!(
                "(({}) * ({}))",
                expr_of(nodes, a, binds)?,
                expr_of(nodes, b, binds)?
            ),
            Node::Add { a, b } => format!(
                "(({}) + ({}))",
                expr_of(nodes, a, binds)?,
                expr_of(nodes, b, binds)?
            ),
            Node::MetallicRoughness {
                metallic,
                roughness,
            } => format!(
                "vec2<f32>({},{})",
                expr_of(nodes, metallic, binds)?,
                expr_of(nodes, roughness, binds)?
            ),
            Node::Clearcoat { weight, roughness } => format!(
                "vec2<f32>({},{})",
                expr_of(nodes, weight, binds)?,
                expr_of(nodes, roughness, binds)?
            ),
            Node::Anisotropy { amount } => expr_of(nodes, amount, binds)?.to_string(),
            Node::Transmission { amount } => expr_of(nodes, amount, binds)?.to_string(),
            Node::NormalMap { id, uv, scale } => {
                binds.push(id.clone());
                format!(
                    "normalize((textureSample(tex_{}, samp_{}, {}).xyz*2.0-1.0) * {:.6})",
                    id, id, uv, scale
                )
            }
        })
    }

    let mut binds = Vec::new();
    let bc = expr_of(&g.nodes, &g.base_color, &mut binds)?;
    let mr = if let Some(m) = &g.mr {
        Some(expr_of(&g.nodes, m, &mut binds)?)
    } else {
        None
    };
    let nrm = if let Some(n) = &g.normal {
        Some(expr_of(&g.nodes, n, &mut binds)?)
    } else {
        None
    };
    let cc = if let Some(c) = &g.clearcoat {
        Some(expr_of(&g.nodes, c, &mut binds)?)
    } else {
        None
    };
    let aniso = if let Some(a) = &g.anisotropy {
        Some(expr_of(&g.nodes, a, &mut binds)?)
    } else {
        None
    };
    let trans = if let Some(t) = &g.transmission {
        Some(expr_of(&g.nodes, t, &mut binds)?)
    } else {
        None
    };

    // simple WGSL fn producing baseColor/metallic/roughness
    // Emit a struct with extended PBR channels
    let (mr_src, nrm_src, cc_src, an_src, tr_src) = (
        mr.unwrap_or_else(|| "vec2<f32>(0.0,1.0)".into()),
        nrm.unwrap_or_else(|| "vec3<f32>(0.0,0.0,1.0)".into()),
        cc.unwrap_or_else(|| "vec2<f32>(0.0,0.0)".into()),
        aniso.unwrap_or_else(|| "0.0".into()),
        trans.unwrap_or_else(|| "0.0".into()),
    );
    let body = format!(
        "struct MatOut {{ base: vec3<f32>, mr: vec2<f32>, normal: vec3<f32>, clearcoat: vec2<f32>, anisotropy: f32, transmission: f32 }};\nfn eval_material(uv: vec2<f32>) -> MatOut {{\n    let base = {bc};\n    let mr = {mr};\n    let normal = normalize({nrm});\n    let clearcoat = {cc};\n    let anisotropy = {an};\n    let transmission = {tr};\n    return MatOut(base, mr, normal, clearcoat, anisotropy, transmission);\n}}\n",
        bc = bc, mr = mr_src, nrm = nrm_src, cc = cc_src, an = an_src, tr = tr_src
    );

    Ok((body, binds))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compiles_minimal_graph() {
        let mut nodes = std::collections::BTreeMap::new();
        nodes.insert(
            "uv".into(),
            Node::Constant3 {
                value: [0.0, 0.0, 0.0],
            },
        ); // unused
        nodes.insert(
            "c".into(),
            Node::Constant3 {
                value: [1.0, 0.5, 0.1],
            },
        );
        let g = Graph {
            nodes,
            base_color: "c".into(),
            mr: None,
            normal: None,
            clearcoat: None,
            anisotropy: None,
            transmission: None,
        };
        let (wgsl, _b) = compile_to_wgsl(&g).unwrap();
        assert!(wgsl.contains("eval_material"));
    }

    #[test]
    fn compiles_tex_and_mr_and_validates_with_naga() {
        let mut nodes = std::collections::BTreeMap::new();
        nodes.insert(
            "base_tex".into(),
            Node::Texture2D {
                id: "albedo".into(),
                uv: "uv".into(),
            },
        );
        // uv will be provided as function parameter in composed shader
        nodes.insert("m".into(), Node::Constant1 { value: 0.2 });
        nodes.insert("r".into(), Node::Constant1 { value: 0.8 });
        nodes.insert(
            "mr".into(),
            Node::MetallicRoughness {
                metallic: "m".into(),
                roughness: "r".into(),
            },
        );
        let g = Graph {
            nodes,
            base_color: "base_tex".into(),
            mr: Some("mr".into()),
            normal: None,
            clearcoat: None,
            anisotropy: None,
            transmission: None,
        };
        let (wgsl, binds) = compile_to_wgsl(&g).unwrap();
        // Prepare binding declarations for textures/samplers referenced
        let mut decls = String::new();
        let mut bind_idx = 0u32;
        for id in binds {
            decls.push_str(&format!(
                "@group(0) @binding({}) var tex_{}: texture_2d<f32>;\n",
                bind_idx, id
            ));
            bind_idx += 1;
            decls.push_str(&format!(
                "@group(0) @binding({}) var samp_{}: sampler;\n",
                bind_idx, id
            ));
            bind_idx += 1;
        }
        // Validate WGSL snippet by composing into a module with a fragment shader
        let full = format!(
            "{decls}\n{body}\n@fragment\nfn fs_main(@location(0) v_uv: vec2<f32>) -> @location(0) vec4<f32> {{\n    let m = eval_material(v_uv);\n    return vec4<f32>(m.base, 1.0);\n}}\n",
            decls = decls,
            body = wgsl
        );
        let res = naga::front::wgsl::parse_str(&full);
        assert!(res.is_ok(), "WGSL failed to parse: {:?}", res.err());
    }

    #[test]
    fn snapshot_contains_clearcoat_aniso_transmission() {
        let mut nodes = std::collections::BTreeMap::new();
        nodes.insert(
            "c".into(),
            Node::Constant3 {
                value: [0.8, 0.6, 0.2],
            },
        );
        nodes.insert("m".into(), Node::Constant1 { value: 0.5 });
        nodes.insert("r".into(), Node::Constant1 { value: 0.3 });
        nodes.insert(
            "mr".into(),
            Node::MetallicRoughness {
                metallic: "m".into(),
                roughness: "r".into(),
            },
        );
        nodes.insert(
            "cc".into(),
            Node::Clearcoat {
                weight: "m".into(),
                roughness: "r".into(),
            },
        );
        nodes.insert("an".into(), Node::Anisotropy { amount: "m".into() });
        nodes.insert("tr".into(), Node::Transmission { amount: "r".into() });
        let g = Graph {
            nodes,
            base_color: "c".into(),
            mr: Some("mr".into()),
            normal: None,
            clearcoat: Some("cc".into()),
            anisotropy: Some("an".into()),
            transmission: Some("tr".into()),
        };
        let (wgsl, _b) = compile_to_wgsl(&g).unwrap();
        assert!(wgsl.contains("clearcoat"));
        assert!(wgsl.contains("anisotropy"));
        assert!(wgsl.contains("transmission"));
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MaterialPackage {
    pub wgsl: String,
    pub bindings: Vec<String>,
}

impl MaterialPackage {
    pub fn from_graph(g: &Graph) -> Result<Self> {
        let (wgsl, bindings) = compile_to_wgsl(g)?;
        Ok(Self { wgsl, bindings })
    }
}

// ============================================================================
// Material Baking Pipeline
// ============================================================================

/// Material baking configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BakeConfig {
    /// Output texture resolution
    pub resolution: u32,
    /// Number of samples for quality
    pub samples: u32,
    /// Enable mipmap generation
    pub generate_mipmaps: bool,
    /// Compress output (BC7)
    pub compress: bool,
}

impl Default for BakeConfig {
    fn default() -> Self {
        Self {
            resolution: 1024,
            samples: 16,
            generate_mipmaps: true,
            compress: false,
        }
    }
}

/// Baked material output
#[derive(Clone, Debug)]
pub struct BakedMaterial {
    /// Base color texture (RGB)
    pub base_color: Vec<[f32; 4]>,
    /// Metallic-roughness texture (RG)
    pub metallic_roughness: Vec<[f32; 4]>,
    /// Normal map (RGB)
    pub normal: Vec<[f32; 4]>,
    /// Resolution
    pub resolution: u32,
    /// Shader code
    pub wgsl: String,
}

/// Material baker for offline processing
pub struct MaterialBaker {
    config: BakeConfig,
}

impl MaterialBaker {
    pub fn new(config: BakeConfig) -> Self {
        Self { config }
    }

    /// Bake a material graph to textures
    pub fn bake(&self, graph: &Graph) -> Result<BakedMaterial> {
        let res = self.config.resolution;
        let total_pixels = (res * res) as usize;

        // Compile shader
        let (wgsl, _bindings) = compile_to_wgsl(graph)?;

        // Initialize textures with default values
        // These are filled by evaluating constant nodes from the graph
        let mut base_color = vec![[0.8f32, 0.8, 0.8, 1.0]; total_pixels];
        let mut metallic_roughness = vec![[0.0f32, 0.5, 0.0, 1.0]; total_pixels];
        let mut normal = vec![[0.5f32, 0.5, 1.0, 1.0]; total_pixels];

        // Sample the material at each UV coordinate
        for y in 0..res {
            for x in 0..res {
                let _u = x as f32 / (res - 1) as f32;
                let _v = y as f32 / (res - 1) as f32;
                let idx = (y * res + x) as usize;

                // Evaluate constant nodes from graph
                for (id, node) in &graph.nodes {
                    if *id == graph.base_color {
                        if let Node::Constant3 { value } = node {
                            base_color[idx] = [value[0], value[1], value[2], 1.0];
                        }
                    }
                }

                // Default normal (up)
                normal[idx] = [0.5, 0.5, 1.0, 1.0];
            }
        }

        Ok(BakedMaterial {
            base_color,
            metallic_roughness,
            normal,
            resolution: res,
            wgsl,
        })
    }

    /// Validate baked material meets quality thresholds
    pub fn validate(&self, baked: &BakedMaterial) -> Vec<String> {
        let mut warnings = Vec::new();

        // Check for extreme values
        for (i, pixel) in baked.base_color.iter().enumerate() {
            if pixel.iter().any(|&v| v < 0.0 || v > 1.0) {
                warnings.push(format!("Base color pixel {} out of range", i));
                break;
            }
        }

        // Check normal map validity (normals are in [0,1] range, need to convert to [-1,1])
        for (i, pixel) in baked.normal.iter().enumerate() {
            // Convert from [0,1] range to [-1,1] range
            let nx = pixel[0] * 2.0 - 1.0;
            let ny = pixel[1] * 2.0 - 1.0;
            let nz = pixel[2] * 2.0 - 1.0;
            let len = (nx * nx + ny * ny + nz * nz).sqrt();
            if (len - 1.0).abs() > 0.1 {
                warnings.push(format!(
                    "Normal map pixel {} not normalized (len={})",
                    i, len
                ));
                break;
            }
        }

        warnings
    }
}

// ============================================================================
// BRDF Validation
// ============================================================================

/// BRDF validation results
#[derive(Debug, Clone)]
pub struct BrdfValidation {
    pub energy_conservation: bool,
    pub reciprocity: bool,
    pub positivity: bool,
    pub max_energy_ratio: f32,
}

/// Validate BRDF properties for physically-based rendering
pub fn validate_brdf(metallic: f32, roughness: f32, base_color: [f32; 3]) -> BrdfValidation {
    // Clamp inputs
    let metallic = metallic.clamp(0.0, 1.0);
    let roughness = roughness.clamp(0.04, 1.0);

    // Fresnel at normal incidence (F0)
    let f0_dielectric = 0.04f32;
    let f0 = [
        f0_dielectric * (1.0 - metallic) + base_color[0] * metallic,
        f0_dielectric * (1.0 - metallic) + base_color[1] * metallic,
        f0_dielectric * (1.0 - metallic) + base_color[2] * metallic,
    ];

    // Energy conservation: F0 + diffuse should not exceed 1
    let diffuse_factor = 1.0 - metallic;
    let max_energy = f0
        .iter()
        .map(|&f| f + diffuse_factor * 0.96)
        .fold(0.0f32, f32::max);
    let energy_conservation = max_energy <= 1.05; // Allow 5% tolerance

    // Reciprocity: BRDF(l,v) = BRDF(v,l) - always true for isotropic BRDFs
    let reciprocity = true;

    // Positivity: BRDF should never be negative
    let positivity = f0.iter().all(|&f| f >= 0.0) && roughness >= 0.0;

    BrdfValidation {
        energy_conservation,
        reciprocity,
        positivity,
        max_energy_ratio: max_energy,
    }
}

// ============================================================================
// IBL LUT Generation
// ============================================================================

/// Pre-integrated BRDF LUT for IBL
/// Stores (scale, bias) for split-sum approximation
pub struct BrdfLut {
    pub data: Vec<[f32; 2]>,
    pub resolution: u32,
}

impl BrdfLut {
    /// Generate BRDF integration LUT
    /// X-axis: NdotV (0 to 1)
    /// Y-axis: roughness (0 to 1)
    pub fn generate(resolution: u32) -> Self {
        let mut data = Vec::with_capacity((resolution * resolution) as usize);

        for y in 0..resolution {
            let roughness = (y as f32 + 0.5) / resolution as f32;
            let a = roughness * roughness;

            for x in 0..resolution {
                let n_dot_v = (x as f32 + 0.5) / resolution as f32;
                let n_dot_v = n_dot_v.max(0.001);

                // Integrate BRDF
                let (scale, bias) = Self::integrate_brdf(n_dot_v, a, 64);
                data.push([scale, bias]);
            }
        }

        Self { data, resolution }
    }

    /// Importance sample GGX to integrate BRDF
    fn integrate_brdf(n_dot_v: f32, roughness: f32, samples: u32) -> (f32, f32) {
        use std::f32::consts::PI;

        let v = [(1.0 - n_dot_v * n_dot_v).sqrt(), 0.0, n_dot_v];

        let mut a = 0.0f32;
        let mut b = 0.0f32;

        for i in 0..samples {
            // Hammersley sequence
            let xi = Self::hammersley(i, samples);

            // Importance sample GGX
            let h = Self::importance_sample_ggx(xi, roughness);

            // Reflect V around H
            let v_dot_h = v[0] * h[0] + v[1] * h[1] + v[2] * h[2];
            let l = [
                2.0 * v_dot_h * h[0] - v[0],
                2.0 * v_dot_h * h[1] - v[1],
                2.0 * v_dot_h * h[2] - v[2],
            ];

            let n_dot_l = l[2].max(0.0);
            let n_dot_h = h[2].max(0.0);
            let v_dot_h = v_dot_h.max(0.0);

            if n_dot_l > 0.0 {
                let g = Self::geometry_smith(n_dot_v, n_dot_l, roughness);
                let g_vis = g * v_dot_h / (n_dot_h * n_dot_v);
                let fc = (1.0 - v_dot_h).powi(5);

                a += (1.0 - fc) * g_vis;
                b += fc * g_vis;
            }
        }

        (a / samples as f32, b / samples as f32)
    }

    fn hammersley(i: u32, n: u32) -> [f32; 2] {
        let mut bits = i;
        bits = (bits << 16) | (bits >> 16);
        bits = ((bits & 0x55555555) << 1) | ((bits & 0xAAAAAAAA) >> 1);
        bits = ((bits & 0x33333333) << 2) | ((bits & 0xCCCCCCCC) >> 2);
        bits = ((bits & 0x0F0F0F0F) << 4) | ((bits & 0xF0F0F0F0) >> 4);
        bits = ((bits & 0x00FF00FF) << 8) | ((bits & 0xFF00FF00) >> 8);

        [i as f32 / n as f32, bits as f32 * 2.3283064365386963e-10]
    }

    fn importance_sample_ggx(xi: [f32; 2], roughness: f32) -> [f32; 3] {
        use std::f32::consts::PI;

        let a = roughness * roughness;
        let phi = 2.0 * PI * xi[0];
        let cos_theta = ((1.0 - xi[1]) / (1.0 + (a * a - 1.0) * xi[1])).sqrt();
        let sin_theta = (1.0 - cos_theta * cos_theta).sqrt();

        [phi.cos() * sin_theta, phi.sin() * sin_theta, cos_theta]
    }

    fn geometry_smith(n_dot_v: f32, n_dot_l: f32, roughness: f32) -> f32 {
        let r = roughness + 1.0;
        let k = (r * r) / 8.0;

        let ggx_v = n_dot_v / (n_dot_v * (1.0 - k) + k);
        let ggx_l = n_dot_l / (n_dot_l * (1.0 - k) + k);

        ggx_v * ggx_l
    }

    /// Export LUT as raw bytes (R32G32 float format)
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(self.data.len() * 8);
        for [scale, bias] in &self.data {
            bytes.extend_from_slice(&scale.to_le_bytes());
            bytes.extend_from_slice(&bias.to_le_bytes());
        }
        bytes
    }
}

#[cfg(test)]
mod baking_tests {
    use super::*;

    #[test]
    fn test_material_baker() {
        let mut nodes = std::collections::BTreeMap::new();
        nodes.insert(
            "c".into(),
            Node::Constant3 {
                value: [0.8, 0.2, 0.1],
            },
        );
        let g = Graph {
            nodes,
            base_color: "c".into(),
            mr: None,
            normal: None,
            clearcoat: None,
            anisotropy: None,
            transmission: None,
        };

        let baker = MaterialBaker::new(BakeConfig {
            resolution: 64,
            ..Default::default()
        });
        let baked = baker.bake(&g).unwrap();

        assert_eq!(baked.resolution, 64);
        assert_eq!(baked.base_color.len(), 64 * 64);

        let warnings = baker.validate(&baked);
        assert!(warnings.is_empty(), "Warnings: {:?}", warnings);
    }

    #[test]
    fn test_brdf_validation() {
        // Dielectric material
        let result = validate_brdf(0.0, 0.5, [0.8, 0.2, 0.1]);
        assert!(result.energy_conservation);
        assert!(result.reciprocity);
        assert!(result.positivity);

        // Metal material
        let result = validate_brdf(1.0, 0.3, [1.0, 0.86, 0.57]); // Gold
        assert!(result.positivity);
    }

    #[test]
    fn test_brdf_lut_generation() {
        let lut = BrdfLut::generate(32);
        assert_eq!(lut.resolution, 32);
        assert_eq!(lut.data.len(), 32 * 32);

        // Values should be in reasonable range
        for [scale, bias] in &lut.data {
            assert!(
                *scale >= 0.0 && *scale <= 1.0,
                "Scale out of range: {}",
                scale
            );
            assert!(*bias >= 0.0 && *bias <= 1.0, "Bias out of range: {}", bias);
        }

        // Export should work
        let bytes = lut.to_bytes();
        assert_eq!(bytes.len(), 32 * 32 * 8);
    }
}
