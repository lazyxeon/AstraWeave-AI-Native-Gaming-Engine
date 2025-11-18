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
