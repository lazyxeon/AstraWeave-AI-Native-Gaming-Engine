use anyhow::Result;
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use tokio::sync::watch;
use serde::{Deserialize, Serialize};
use std::fs;
use walkdir;
use notify;
use notify::Watcher;
use toml;
use hex;
use std::io;

#[cfg(feature = "gltf")]
pub mod gltf_loader {
    use anyhow::{anyhow, bail, Context, Result};
    use base64::Engine as _;
    use gltf::Gltf;

    /// Minimal glTF loader prototype: validates header and detects JSON vs BIN format.
    /// Phase 0 scope: we only recognize GLB header and return an error if unsupported.
    pub fn load_gltf_bytes(bytes: &[u8]) -> Result<()> {
        if bytes.len() >= 12 && &bytes[0..4] == b"glTF" {
            // GLB header: magic, version, length
            let _version = u32::from_le_bytes(bytes[4..8].try_into().unwrap());
            let _length = u32::from_le_bytes(bytes[8..12].try_into().unwrap());
            // Further parsing omitted in Phase 0
            Ok(())
        } else {
            // If JSON (.gltf), just check it's UTF-8 and contains basic fields
            if let Ok(s) = std::str::from_utf8(bytes) {
                if s.contains("meshes") && s.contains("accessors") {
                    return Ok(());
                }
            }
            anyhow::bail!("Unsupported or invalid glTF data: expected .glb header or JSON with meshes/accessors")
        }
    }

    #[derive(Debug, Clone)]
    pub struct MeshData {
        pub positions: Vec<[f32; 3]>,
        pub normals: Vec<[f32; 3]>,
        pub tangents: Vec<[f32; 4]>,
        pub texcoords: Vec<[f32; 2]>,
        pub indices: Vec<u32>,
    }

    #[derive(Debug, Clone)]
    pub struct ImageData {
        pub width: u32,
        pub height: u32,
        pub rgba8: Vec<u8>,
    }

    #[derive(Debug, Clone, Default)]
    pub struct MaterialData {
        pub base_color_factor: [f32; 4],
        pub metallic_factor: f32,
        pub roughness_factor: f32,
        pub base_color_texture: Option<ImageData>,
        pub metallic_roughness_texture: Option<ImageData>,
        pub normal_texture: Option<ImageData>,
    }

    /// Load the first mesh primitive from a GLB (embedded bin) into MeshData.
    /// Limitations: GLB only, triangle lists, positions+normals required, u16/u32 indices supported.
    pub fn load_first_mesh_from_glb_bytes(bytes: &[u8]) -> Result<MeshData> {
        use gltf::buffer::Data as BufferData;

        // Parse GLB container
        let glb = gltf::binary::Glb::from_slice(bytes).context("Invalid GLB container")?;
        let json = std::str::from_utf8(&glb.json).context("GLB JSON is not UTF-8")?;
        let doc = Gltf::from_slice(json.as_bytes()).context("Failed to parse glTF JSON")?;
        let bin = glb.bin.context("GLB missing BIN chunk")?;

        // Build buffer lookup (only supports embedded BIN at index 0 or single buffer)
        let mut buffers: Vec<BufferData> = Vec::new();
        for b in doc.buffers() {
            match b.source() {
                gltf::buffer::Source::Bin => buffers.push(BufferData(bin.clone().into_owned())),
                gltf::buffer::Source::Uri(_) => {
                    bail!("External buffer URIs not supported in Phase 0")
                }
            }
        }

        let mesh = doc
            .meshes()
            .next()
            .ok_or_else(|| anyhow!("No meshes in GLB"))?;
        let prim = mesh
            .primitives()
            .next()
            .ok_or_else(|| anyhow!("No primitives in first mesh"))?;

        // Positions
        let reader = prim.reader(|buf| buffers.get(buf.index()).map(|d| d.0.as_slice()));
        let positions_iter = reader
            .read_positions()
            .ok_or_else(|| anyhow!("Positions missing"))?;
        let normals_iter = reader
            .read_normals()
            .ok_or_else(|| anyhow!("Normals missing"))?;
        let indices = reader
            .read_indices()
            .ok_or_else(|| anyhow!("Indices missing"))?;

        let positions: Vec<[f32; 3]> = positions_iter.collect();
        let normals: Vec<[f32; 3]> = normals_iter.collect();
        let tangents: Vec<[f32; 4]> = match reader.read_tangents() {
            Some(it) => it.collect(),
            None => vec![[1.0, 0.0, 0.0, 1.0]; positions.len()],
        };
        let texcoords: Vec<[f32; 2]> = match reader.read_tex_coords(0) {
            Some(c) => c.into_f32().collect(),
            None => vec![[0.0, 0.0]; positions.len()],
        };
        let indices: Vec<u32> = match indices {
            gltf::mesh::util::ReadIndices::U16(it) => it.map(|v| v as u32).collect(),
            gltf::mesh::util::ReadIndices::U32(it) => it.collect(),
            gltf::mesh::util::ReadIndices::U8(_) => bail!("u8 indices unsupported"),
        };

        Ok(MeshData {
            positions,
            normals,
            tangents,
            texcoords,
            indices,
        })
    }

    /// Load first mesh and minimal PBR material (baseColor factor/texture, metallic/roughness) from either GLB or GLTF JSON bytes.
    /// Supports GLB embedded BIN and GLTF JSON with data: URIs for buffers/images.
    pub fn load_first_mesh_and_material(bytes: &[u8]) -> Result<(MeshData, MaterialData)> {
        if bytes.len() >= 12 && &bytes[0..4] == b"glTF" {
            load_from_glb(bytes)
        } else {
            load_from_gltf_json(bytes)
        }
    }

    fn load_from_glb(bytes: &[u8]) -> Result<(MeshData, MaterialData)> {
        use gltf::buffer::Data as BufferData;
        let glb = gltf::binary::Glb::from_slice(bytes).context("Invalid GLB container")?;
        let json = std::str::from_utf8(&glb.json).context("GLB JSON is not UTF-8")?;
        let doc = Gltf::from_slice(json.as_bytes()).context("Failed to parse glTF JSON")?;
        let bin = glb.bin.context("GLB missing BIN chunk")?;

        // Buffers
        let mut buffers: Vec<BufferData> = Vec::new();
        for b in doc.buffers() {
            match b.source() {
                gltf::buffer::Source::Bin => buffers.push(BufferData(bin.clone().into_owned())),
                gltf::buffer::Source::Uri(_) => {
                    bail!("External buffer URIs not supported in GLB path")
                }
            }
        }

        let mesh = doc.meshes().next().ok_or_else(|| anyhow!("No meshes"))?;
        let prim = mesh
            .primitives()
            .next()
            .ok_or_else(|| anyhow!("No primitives"))?;
        let reader = prim.reader(|buf| buffers.get(buf.index()).map(|d| d.0.as_slice()));
        let positions: Vec<[f32; 3]> = reader
            .read_positions()
            .ok_or_else(|| anyhow!("Positions missing"))?
            .collect();
        let normals: Vec<[f32; 3]> = reader
            .read_normals()
            .ok_or_else(|| anyhow!("Normals missing"))?
            .collect();
        let tangents: Vec<[f32; 4]> = match reader.read_tangents() {
            Some(it) => it.collect(),
            None => vec![[1.0, 0.0, 0.0, 1.0]; positions.len()],
        };
        let texcoords: Vec<[f32; 2]> = match reader.read_tex_coords(0) {
            Some(c) => c.into_f32().collect(),
            None => vec![[0.0, 0.0]; positions.len()],
        };
        let indices_read = reader
            .read_indices()
            .ok_or_else(|| anyhow!("Indices missing"))?;
        let indices: Vec<u32> = match indices_read {
            gltf::mesh::util::ReadIndices::U16(it) => it.map(|v| v as u32).collect(),
            gltf::mesh::util::ReadIndices::U32(it) => it.collect(),
            gltf::mesh::util::ReadIndices::U8(_) => bail!("u8 indices unsupported"),
        };

        let mut mat = MaterialData::default();
        {
            let mat_g = prim.material();
            let pbr = mat_g.pbr_metallic_roughness();
            mat.base_color_factor = pbr.base_color_factor();
            mat.metallic_factor = pbr.metallic_factor();
            mat.roughness_factor = pbr.roughness_factor();
            if let Some(tex) = pbr.base_color_texture() {
                let img = decode_image_from_gltf(tex.texture().source().source(), Some(&buffers))?;
                mat.base_color_texture = Some(img);
            }
            if let Some(tex) = pbr.metallic_roughness_texture() {
                let img = decode_image_from_gltf(tex.texture().source().source(), Some(&buffers))?;
                mat.metallic_roughness_texture = Some(img);
            }
            if let Some(n) = mat_g.normal_texture() {
                let img = decode_image_from_gltf(n.texture().source().source(), Some(&buffers))?;
                mat.normal_texture = Some(img);
            }
        }

        Ok((
            MeshData {
                positions,
                normals,
                tangents,
                texcoords,
                indices,
            },
            mat,
        ))
    }

    fn load_from_gltf_json(bytes: &[u8]) -> Result<(MeshData, MaterialData)> {
        let doc = Gltf::from_slice(bytes).context("Parse .gltf JSON")?;

        // Build buffer sources that may include data: URIs
        let mut buffers: Vec<Vec<u8>> = Vec::new();
        for b in doc.buffers() {
            match b.source() {
                gltf::buffer::Source::Bin => bail!(".gltf JSON should not have BIN source"),
                gltf::buffer::Source::Uri(uri) => buffers.push(load_uri_bytes(uri)?),
            }
        }

        let mesh = doc.meshes().next().ok_or_else(|| anyhow!("No meshes"))?;
        let prim = mesh
            .primitives()
            .next()
            .ok_or_else(|| anyhow!("No primitives"))?;
        let reader = prim.reader(|buf| buffers.get(buf.index()).map(|d| d.as_slice()));
        let positions: Vec<[f32; 3]> = reader
            .read_positions()
            .ok_or_else(|| anyhow!("Positions missing"))?
            .collect();
        let normals: Vec<[f32; 3]> = reader
            .read_normals()
            .ok_or_else(|| anyhow!("Normals missing"))?
            .collect();
        let tangents: Vec<[f32; 4]> = match reader.read_tangents() {
            Some(it) => it.collect(),
            None => vec![[1.0, 0.0, 0.0, 1.0]; positions.len()],
        };
        let texcoords: Vec<[f32; 2]> = match reader.read_tex_coords(0) {
            Some(c) => c.into_f32().collect(),
            None => vec![[0.0, 0.0]; positions.len()],
        };
        let indices_read = reader
            .read_indices()
            .ok_or_else(|| anyhow!("Indices missing"))?;
        let indices: Vec<u32> = match indices_read {
            gltf::mesh::util::ReadIndices::U16(it) => it.map(|v| v as u32).collect(),
            gltf::mesh::util::ReadIndices::U32(it) => it.collect(),
            gltf::mesh::util::ReadIndices::U8(_) => bail!("u8 indices unsupported"),
        };

        let mut mat = MaterialData::default();
        let prim_mat = prim.material();
        let pbr = prim_mat.pbr_metallic_roughness();
        mat.base_color_factor = pbr.base_color_factor();
        mat.metallic_factor = pbr.metallic_factor();
        mat.roughness_factor = pbr.roughness_factor();
        if let Some(tex) = pbr.base_color_texture() {
            let img = decode_image_from_gltf(tex.texture().source().source(), None)?;
            mat.base_color_texture = Some(img);
        }
        if let Some(tex) = pbr.metallic_roughness_texture() {
            let img = decode_image_from_gltf(tex.texture().source().source(), None)?;
            mat.metallic_roughness_texture = Some(img);
        }
        if let Some(n) = prim_mat.normal_texture() {
            let img = decode_image_from_gltf(n.texture().source().source(), None)?;
            mat.normal_texture = Some(img);
        }

        Ok((
            MeshData {
                positions,
                normals,
                tangents,
                texcoords,
                indices,
            },
            mat,
        ))
    }

    fn load_uri_bytes(uri: &str) -> Result<Vec<u8>> {
        if let Some(rest) = uri.strip_prefix("data:") {
            // data:[<mediatype>][;base64],<data>
            let parts: Vec<&str> = rest.split(',').collect();
            if parts.len() != 2 {
                bail!("Invalid data URI")
            }
            let data = base64::engine::general_purpose::STANDARD.decode(parts[1])?;
            Ok(data)
        } else {
            // Treat as file path
            let bytes = std::fs::read(uri).with_context(|| format!("Read uri {uri}"))?;
            Ok(bytes)
        }
    }

    fn decode_image_from_gltf(
        source: gltf::image::Source,
        buffers: Option<&Vec<gltf::buffer::Data>>,
    ) -> Result<ImageData> {
        match source {
            gltf::image::Source::View { view, mime_type: _ } => {
                let buf_idx = view.buffer().index();
                let offset = view.offset();
                let length = view.length();
                let data = &buffers
                    .ok_or_else(|| anyhow!("Missing buffers for buffer view image"))?[buf_idx]
                    .0
                    .as_slice()[offset..offset + length];
                decode_image_bytes(data)
            }
            gltf::image::Source::Uri { uri, .. } => {
                let bytes = load_uri_bytes(&uri)?;
                decode_image_bytes(&bytes)
            }
        }
    }

    fn decode_image_bytes(bytes: &[u8]) -> Result<ImageData> {
        let img = image::load_from_memory(bytes)?;
        let rgba = img.to_rgba8();
        let (w, h) = rgba.dimensions();
        Ok(ImageData {
            width: w,
            height: h,
            rgba8: rgba.into_raw(),
        })
    }

    #[inline]
    fn normalize_q(mut q: [f32; 4]) -> [f32; 4] {
        let len = (q[0] * q[0] + q[1] * q[1] + q[2] * q[2] + q[3] * q[3]).sqrt();
        if len > 0.0 {
            q[0] /= len;
            q[1] /= len;
            q[2] /= len;
            q[3] /= len;
        }
        q
    }

    // --- Skinned mesh loading (v0 minimal) ---
    #[derive(Debug, Clone)]
    pub struct SkinnedVertexLite {
        pub position: [f32; 3],
        pub normal: [f32; 3],
        pub tangent: [f32; 4],
        pub joints: [u16; 4],
        pub weights: [f32; 4],
    }

    #[derive(Debug, Clone)]
    pub struct SkinnedMeshData {
        pub vertices: Vec<SkinnedVertexLite>,
        pub indices: Vec<u32>,
        pub joint_count: u32,
    }

    #[derive(Debug, Clone)]
    pub struct AnimationClip {
        pub times: Vec<f32>,
        pub rotations: Vec<[f32; 4]>, // quaternion vec4(x,y,z,w) targeting first joint
    }

    /// Load first skinned mesh primitive (positions, normals, JOINTS_0, WEIGHTS_0) and an optional idle rotation clip for the first joint.
    /// Notes: For Phase 0, we only support the first node that references a mesh and has a skin.
    pub fn load_first_skinned_mesh_and_idle(
        bytes: &[u8],
    ) -> Result<(SkinnedMeshData, Option<AnimationClip>, Option<MaterialData>)> {
        let doc = if bytes.len() >= 12 && &bytes[0..4] == b"glTF" {
            // GLB path
            let glb = gltf::binary::Glb::from_slice(bytes).context("Invalid GLB container")?;
            let json = std::str::from_utf8(&glb.json).context("GLB JSON is not UTF-8")?;
            Gltf::from_slice(json.as_bytes()).context("Failed to parse glTF JSON")?
        } else {
            // JSON path
            Gltf::from_slice(bytes).context("Parse .gltf JSON")?
        };

        // Gather buffer data (support GLB BIN or .gltf data: URIs)
        let mut buffers: Vec<Vec<u8>> = Vec::new();
        if bytes.len() >= 12 && &bytes[0..4] == b"glTF" {
            let glb = gltf::binary::Glb::from_slice(bytes).context("Invalid GLB container")?;
            let bin = glb.bin.context("GLB missing BIN chunk")?;
            for b in doc.buffers() {
                match b.source() {
                    gltf::buffer::Source::Bin => buffers.push(bin.clone().into_owned()),
                    gltf::buffer::Source::Uri(_) => {
                        bail!("External buffer URIs not supported in GLB path")
                    }
                }
            }
        } else {
            for b in doc.buffers() {
                match b.source() {
                    gltf::buffer::Source::Uri(uri) => buffers.push(load_uri_bytes(uri)?),
                    gltf::buffer::Source::Bin => bail!("Unexpected BIN in .gltf JSON"),
                }
            }
        }

        // Find first node with mesh and skin
        let mut skinned_node: Option<gltf::Node> = None;
        for n in doc.nodes() {
            if n.mesh().is_some() && n.skin().is_some() {
                skinned_node = Some(n);
                break;
            }
        }
        let node =
            skinned_node.ok_or_else(|| anyhow!("No skinned node (node with mesh+skin) found"))?;
        let skin = node.skin().ok_or_else(|| anyhow!("Node missing skin"))?;
        let joint_count = skin.joints().len() as u32;
        let mesh = node.mesh().ok_or_else(|| anyhow!("Node missing mesh"))?;
        let prim = mesh
            .primitives()
            .next()
            .ok_or_else(|| anyhow!("No primitives in mesh"))?;

        let reader = prim.reader(|buf| buffers.get(buf.index()).map(|d| d.as_slice()));
        let positions: Vec<[f32; 3]> = reader
            .read_positions()
            .ok_or_else(|| anyhow!("Positions missing"))?
            .collect();
        let normals: Vec<[f32; 3]> = reader
            .read_normals()
            .ok_or_else(|| anyhow!("Normals missing"))?
            .collect();
        let tangents: Vec<[f32; 4]> = match reader.read_tangents() {
            Some(it) => it.collect(),
            None => vec![[1.0, 0.0, 0.0, 1.0]; positions.len()],
        };
        let indices_read = reader
            .read_indices()
            .ok_or_else(|| anyhow!("Indices missing"))?;
        let indices: Vec<u32> = match indices_read {
            gltf::mesh::util::ReadIndices::U16(it) => it.map(|v| v as u32).collect(),
            gltf::mesh::util::ReadIndices::U32(it) => it.collect(),
            gltf::mesh::util::ReadIndices::U8(_) => bail!("u8 indices unsupported"),
        };
        let joints0: Vec<[u16; 4]> = match reader
            .read_joints(0)
            .ok_or_else(|| anyhow!("JOINTS_0 missing"))?
        {
            gltf::mesh::util::ReadJoints::U8(it) => it
                .map(|j| [j[0] as u16, j[1] as u16, j[2] as u16, j[3] as u16])
                .collect(),
            gltf::mesh::util::ReadJoints::U16(it) => it.collect(),
        };
        let weights0: Vec<[f32; 4]> = {
            let rw = reader
                .read_weights(0)
                .ok_or_else(|| anyhow!("WEIGHTS_0 missing"))?;
            match rw {
                gltf::mesh::util::ReadWeights::F32(it) => it.collect(),
                gltf::mesh::util::ReadWeights::U8(it) => it
                    .map(|w| {
                        [
                            w[0] as f32 / 255.0,
                            w[1] as f32 / 255.0,
                            w[2] as f32 / 255.0,
                            w[3] as f32 / 255.0,
                        ]
                    })
                    .collect(),
                gltf::mesh::util::ReadWeights::U16(it) => it
                    .map(|w| {
                        [
                            w[0] as f32 / 65535.0,
                            w[1] as f32 / 65535.0,
                            w[2] as f32 / 65535.0,
                            w[3] as f32 / 65535.0,
                        ]
                    })
                    .collect(),
            }
        };

        if positions.len() != normals.len()
            || positions.len() != joints0.len()
            || positions.len() != weights0.len()
            || positions.len() != tangents.len()
        {
            bail!("Attribute count mismatch for skinned vertex data");
        }

        let mut vertices = Vec::with_capacity(positions.len());
        for i in 0..positions.len() {
            vertices.push(SkinnedVertexLite {
                position: positions[i],
                normal: normals[i],
                tangent: tangents[i],
                joints: joints0[i],
                weights: weights0[i],
            });
        }

        // Optional idle animation clip: find first animation channel targeting the first joint
        let mut clip: Option<AnimationClip> = None;
        if let Some(anim) = doc.animations().next() {
            let first_joint_node_index = skin.joints().next().map(|jn| jn.index());
            if let Some(joint_idx) = first_joint_node_index {
                for ch in anim.channels() {
                    if ch.target().node().index() == joint_idx
                        && ch.target().property() == gltf::animation::Property::Rotation
                    {
                        let reader =
                            ch.reader(|buf| buffers.get(buf.index()).map(|d| d.as_slice()));
                        let inputs: Vec<f32> = reader
                            .read_inputs()
                            .ok_or_else(|| anyhow!("Anim input missing"))?
                            .collect();
                        let outputs: Vec<[f32; 4]> = match reader
                            .read_outputs()
                            .ok_or_else(|| anyhow!("Anim output missing"))?
                        {
                            gltf::animation::util::ReadOutputs::Rotations(rot_it) => match rot_it {
                                gltf::animation::util::Rotations::F32(it) => {
                                    it.map(|r| [r[0], r[1], r[2], r[3]]).collect()
                                }
                                gltf::animation::util::Rotations::I16(it) => it
                                    .map(|r| {
                                        normalize_q([
                                            (r[0] as f32) / 32767.0,
                                            (r[1] as f32) / 32767.0,
                                            (r[2] as f32) / 32767.0,
                                            (r[3] as f32) / 32767.0,
                                        ])
                                    })
                                    .collect(),
                                gltf::animation::util::Rotations::I8(it) => it
                                    .map(|r| {
                                        normalize_q([
                                            (r[0] as f32) / 127.0,
                                            (r[1] as f32) / 127.0,
                                            (r[2] as f32) / 127.0,
                                            (r[3] as f32) / 127.0,
                                        ])
                                    })
                                    .collect(),
                                gltf::animation::util::Rotations::U8(it) => it
                                    .map(|r| {
                                        normalize_q([
                                            (r[0] as f32) / 255.0,
                                            (r[1] as f32) / 255.0,
                                            (r[2] as f32) / 255.0,
                                            (r[3] as f32) / 255.0,
                                        ])
                                    })
                                    .collect(),
                                gltf::animation::util::Rotations::U16(it) => it
                                    .map(|r| {
                                        normalize_q([
                                            (r[0] as f32) / 65535.0,
                                            (r[1] as f32) / 65535.0,
                                            (r[2] as f32) / 65535.0,
                                            (r[3] as f32) / 65535.0,
                                        ])
                                    })
                                    .collect(),
                            },
                            _ => bail!("Anim outputs not rotations"),
                        };
                        if inputs.len() == outputs.len() && !inputs.is_empty() {
                            clip = Some(AnimationClip {
                                times: inputs,
                                rotations: outputs,
                            });
                            break;
                        }
                    }
                }
            }
        }

        // Material for the primitive (optional textures)
        let mut mat = MaterialData::default();
        let mat_g = prim.material();
        let pbr = mat_g.pbr_metallic_roughness();
        mat.base_color_factor = pbr.base_color_factor();
        mat.metallic_factor = pbr.metallic_factor();
        mat.roughness_factor = pbr.roughness_factor();
        // Prepare buffer views for image decode when images are embedded as buffer views
        let buffers_data: Vec<gltf::buffer::Data> =
            buffers.iter().cloned().map(gltf::buffer::Data).collect();
        if let Some(tex) = pbr.base_color_texture() {
            let img = decode_image_from_gltf(tex.texture().source().source(), Some(&buffers_data))
                .unwrap_or_else(|_| ImageData {
                    width: 1,
                    height: 1,
                    rgba8: vec![255, 255, 255, 255],
                });
            mat.base_color_texture = Some(img);
        }
        if let Some(tex) = pbr.metallic_roughness_texture() {
            if let Ok(img) =
                decode_image_from_gltf(tex.texture().source().source(), Some(&buffers_data))
            {
                mat.metallic_roughness_texture = Some(img);
            }
        }
        if let Some(n) = mat_g.normal_texture() {
            if let Ok(img) =
                decode_image_from_gltf(n.texture().source().source(), Some(&buffers_data))
            {
                mat.normal_texture = Some(img);
            }
        }

        Ok((
            SkinnedMeshData {
                vertices,
                indices,
                joint_count,
            },
            clip,
            Some(mat),
        ))
    }
}

pub struct AssetManifest;

impl AssetManifest {
    pub fn validate() -> Result<()> {
        // Placeholder: in Phase 0, no manifest; Phase 1 will add deterministic GUIDs
        Ok(())
    }
}

// ---- Phase 2 foundations: deterministic GUIDs and cache ----

/// Deterministic asset GUID using SHA-256 of canonicalized path.
pub fn guid_for_path(path: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(path.replace('\\', "/").to_lowercase());
    let out = hasher.finalize();
    hex::encode(&out[0..16]) // 128-bit hex for brevity
}

#[derive(Default)]
pub struct AssetCache<T> {
    map: HashMap<String, T>,
}

impl<T> AssetCache<T> {
    pub fn insert(&mut self, path: &str, val: T) -> String {
        let id = guid_for_path(path);
        self.map.insert(id.clone(), val);
        id
    }
    pub fn get(&self, id: &str) -> Option<&T> {
        self.map.get(id)
    }
    pub fn len(&self) -> usize {
        self.map.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn guid_is_deterministic_and_case_insensitive() {
        let a = guid_for_path("Assets/Characters/Hero.gltf");
        let b = guid_for_path("assets/characters/hero.gltf");
        let c = guid_for_path("assets\\characters\\hero.gltf");
        assert_eq!(a, b);
        assert_eq!(b, c);
        assert_eq!(a.len(), 32);
    }
    #[test]
    fn cache_inserts_and_retrieves() {
        let mut c = AssetCache::<i32>::default();
        let id = c.insert("assets/tex.png", 7);
        assert_eq!(c.get(&id), Some(&7));
        assert_eq!(c.len(), 1);
    }
}

// ---- Phase 3: Asset Database with Dependency Graph, GUIDs, Hot-Reload ----

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetMetadata {
    pub guid: String,
    pub path: String,
    pub kind: AssetKind,
    pub hash: String,
    pub dependencies: Vec<String>, // GUIDs of dependencies
    pub last_modified: u64,
    pub size_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AssetKind {
    Mesh,
    Texture,
    Audio,
    Dialogue,
    Material,
    Animation,
    Other,
}

#[derive(Debug)]
pub struct AssetDatabase {
    pub assets: HashMap<String, AssetMetadata>, // GUID -> metadata
    pub path_to_guid: HashMap<PathBuf, String>,
    pub dependency_graph: HashMap<String, HashSet<String>>, // GUID -> set of dependent GUIDs
    pub reverse_deps: HashMap<String, HashSet<String>>, // GUID -> set of GUIDs it depends on
    pub hot_reload_tx: watch::Sender<()>,
    pub hot_reload_rx: watch::Receiver<()>,
}

impl AssetDatabase {
    pub fn new() -> Self {
        let (tx, rx) = watch::channel(());
        Self {
            assets: HashMap::new(),
            path_to_guid: HashMap::new(),
            dependency_graph: HashMap::new(),
            reverse_deps: HashMap::new(),
            hot_reload_tx: tx,
            hot_reload_rx: rx,
        }
    }

    pub fn register_asset(&mut self, path: &Path, kind: AssetKind, dependencies: Vec<String>) -> Result<String> {
        let guid = if let Some(existing) = self.path_to_guid.get(path) {
            existing.clone()
        } else {
            guid_for_path(&path.to_string_lossy())
        };

        let metadata = fs::metadata(path)?;
        let hash = compute_file_hash(path)?;
        let size = metadata.len();

        let meta = AssetMetadata {
            guid: guid.clone(),
            path: path.to_string_lossy().to_string(),
            kind,
            hash,
            dependencies: dependencies.clone(),
            last_modified: metadata.modified()?.duration_since(std::time::UNIX_EPOCH)?.as_secs(),
            size_bytes: size,
        };

        self.assets.insert(guid.clone(), meta);
        self.path_to_guid.insert(path.to_path_buf(), guid.clone());

        // Update dependency graph
        for dep_guid in &dependencies {
            self.reverse_deps.entry(guid.clone()).or_insert(HashSet::new()).insert(dep_guid.clone());
            self.dependency_graph.entry(dep_guid.clone()).or_insert(HashSet::new()).insert(guid.clone());
        }

        Ok(guid)
    }

    pub fn get_asset(&self, guid: &str) -> Option<&AssetMetadata> {
        self.assets.get(guid)
    }

    pub fn get_guid_by_path(&self, path: &Path) -> Option<&String> {
        self.path_to_guid.get(path)
    }

    pub fn get_dependents(&self, guid: &str) -> Option<&HashSet<String>> {
        self.dependency_graph.get(guid)
    }

    pub fn get_dependencies(&self, guid: &str) -> Option<&HashSet<String>> {
        self.reverse_deps.get(guid)
    }

    pub fn invalidate_asset(&mut self, guid: &str) -> Result<()> {
        let dependents: Vec<String> = self.dependency_graph.get(guid).cloned().unwrap_or_default().into_iter().collect();
        for dep in dependents {
            // Mark dependents as needing reload
            if let Some(meta) = self.assets.get_mut(&dep) {
                meta.hash = "invalidated".to_string();
            }
        }
        self.hot_reload_tx.send(()).ok();
        Ok(())
    }

    pub fn scan_directory(&mut self, root: &Path) -> Result<()> {
        for entry in walkdir::WalkDir::new(root) {
            let entry = entry?;
            if entry.file_type().is_file() {
                let path = entry.path();
                let kind = infer_asset_kind(path);
                let dependencies = infer_dependencies(path, kind.clone())?;
                self.register_asset(path, kind, dependencies)?;
            }
        }
        Ok(())
    }

    pub fn save_manifest(&self, path: &Path) -> Result<()> {
        let manifest: Vec<&AssetMetadata> = self.assets.values().collect();
        let json = serde_json::to_string_pretty(&manifest)?;
        fs::write(path, json)?;
        Ok(())
    }

    pub fn load_manifest(&mut self, path: &Path) -> Result<()> {
        let json = fs::read_to_string(path)?;
        let manifest: Vec<AssetMetadata> = serde_json::from_str(&json)?;
        for meta in manifest {
            let guid = meta.guid.clone();
            let path_buf = PathBuf::from(&meta.path);
            self.assets.insert(guid.clone(), meta);
            self.path_to_guid.insert(path_buf, guid);
        }
        // Rebuild dependency graphs
        for (guid, meta) in &self.assets {
            for dep in &meta.dependencies {
                self.reverse_deps.entry(guid.clone()).or_insert(HashSet::new()).insert(dep.clone());
                self.dependency_graph.entry(dep.clone()).or_insert(HashSet::new()).insert(guid.clone());
            }
        }
        Ok(())
    }
}

fn infer_asset_kind(path: &Path) -> AssetKind {
    match path.extension().and_then(|e| e.to_str()) {
        Some("gltf") | Some("glb") | Some("obj") => AssetKind::Mesh,
        Some("png") | Some("jpg") | Some("jpeg") | Some("ktx2") | Some("dds") => AssetKind::Texture,
        Some("wav") | Some("ogg") | Some("mp3") => AssetKind::Audio,
        Some("dialogue") | Some("dialogue.toml") => AssetKind::Dialogue,
        Some("material") | Some("material.toml") => AssetKind::Material,
        Some("anim") | Some("animation") => AssetKind::Animation,
        _ => AssetKind::Other,
    }
}

fn infer_dependencies(path: &Path, kind: AssetKind) -> Result<Vec<String>> {
    match kind {
        AssetKind::Mesh => {
            // For glTF, parse and extract texture/material dependencies
            if path.extension().and_then(|e| e.to_str()) == Some("gltf") {
                let content = fs::read_to_string(path)?;
                let mut deps = Vec::new();
                // Simple regex-like search for URIs
                for line in content.lines() {
                    if line.contains("\"uri\":") {
                        if let Some(start) = line.find('"') {
                            if let Some(end) = line[start+1..].find('"') {
                                let uri = &line[start+1..start+1+end];
                                if !uri.starts_with("data:") {
                                    // Assume relative path, compute GUID
                                    let dep_path = path.parent().unwrap_or(Path::new(".")).join(uri);
                                    deps.push(guid_for_path(&dep_path.to_string_lossy()));
                                }
                            }
                        }
                    }
                }
                Ok(deps)
            } else {
                Ok(Vec::new())
            }
        }
        AssetKind::Material => {
            // Parse TOML for texture references
            let content = fs::read_to_string(path)?;
            let doc: toml::Value = toml::from_str(&content)?;
            let mut deps = Vec::new();
            if let Some(textures) = doc.get("textures") {
                if let Some(table) = textures.as_table() {
                    for (_name, value) in table {
                        if let Some(path_str) = value.as_str() {
                            let dep_path = path.parent().unwrap_or(Path::new(".")).join(path_str);
                            deps.push(guid_for_path(&dep_path.to_string_lossy()));
                        }
                    }
                }
            }
            Ok(deps)
        }
        _ => Ok(Vec::new()),
    }
}

fn compute_file_hash(path: &Path) -> Result<String> {
    let mut file = fs::File::open(path)?;
    let mut hasher = Sha256::new();
    std::io::copy(&mut file, &mut hasher)?;
    Ok(hex::encode(hasher.finalize()))
}

// Hot-reload watcher
pub struct AssetWatcher {
    db: Arc<Mutex<AssetDatabase>>,
    watcher: notify::RecommendedWatcher,
}

impl AssetWatcher {
    pub fn new(db: Arc<Mutex<AssetDatabase>>) -> Result<Self> {
        let db_clone = db.clone();
        let watcher = notify::recommended_watcher(move |res: Result<notify::Event, notify::Error>| {
            match res {
                Ok(event) => {
                    if matches!(event.kind, notify::EventKind::Modify(_) | notify::EventKind::Create(_) | notify::EventKind::Remove(_)) {
                        for path in &event.paths {
                            if let Ok(mut db) = db_clone.lock() {
                                if let Some(guid) = db.get_guid_by_path(path).cloned() {
                                    db.invalidate_asset(&guid).ok();
                                }
                            }
                        }
                    }
                }
                Err(e) => eprintln!("Watch error: {:?}", e),
            }
        })?;

        Ok(Self { db, watcher })
    }

    pub fn watch_directory(&mut self, path: &Path) -> Result<()> {
        self.watcher.watch(path, notify::RecursiveMode::Recursive)?;
        Ok(())
    }
}

// Import pipelines
pub mod import_pipelines {
    use super::*;
    use image::ImageFormat;

    pub fn import_texture(source: &Path, output: &Path) -> Result<()> {
        let img = image::open(source)?;
        let rgba = img.to_rgba8();
        rgba.save_with_format(output, ImageFormat::Png)?;
        Ok(())
    }

    pub fn import_audio(source: &Path, output: &Path) -> Result<()> {
        // For now, just copy; in full impl, use audio processing
        fs::copy(source, output)?;
        Ok(())
    }

    pub fn import_dialogue(source: &Path, output: &Path) -> Result<()> {
        // Validate TOML structure
        let content = fs::read_to_string(source)?;
        let _: toml::Value = toml::from_str(&content)?;
        fs::copy(source, output)?;
        Ok(())
    }
}
