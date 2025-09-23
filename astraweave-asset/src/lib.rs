use anyhow::Result;

#[cfg(feature = "gltf")]
pub mod gltf_loader {
    use anyhow::{anyhow, bail, Context, Result};
    use gltf::Gltf;
    use base64::Engine as _;

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
                gltf::buffer::Source::Uri(_) => bail!("External buffer URIs not supported in Phase 0"),
            }
        }

        let mesh = doc.meshes().next().ok_or_else(|| anyhow!("No meshes in GLB"))?;
        let prim = mesh.primitives().next().ok_or_else(|| anyhow!("No primitives in first mesh"))?;

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
    let tangents: Vec<[f32; 4]> = match reader.read_tangents() { Some(it) => it.collect(), None => vec![[1.0,0.0,0.0,1.0]; positions.len()] };
    let texcoords: Vec<[f32; 2]> = match reader.read_tex_coords(0) { Some(c) => c.into_f32().collect(), None => vec![[0.0,0.0]; positions.len()] };
        let indices: Vec<u32> = match indices {
            gltf::mesh::util::ReadIndices::U16(it) => it.map(|v| v as u32).collect(),
            gltf::mesh::util::ReadIndices::U32(it) => it.collect(),
            gltf::mesh::util::ReadIndices::U8(_) => bail!("u8 indices unsupported"),
        };

        Ok(MeshData { positions, normals, tangents, texcoords, indices })
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
                gltf::buffer::Source::Uri(_) => bail!("External buffer URIs not supported in GLB path"),
            }
        }

        let mesh = doc.meshes().next().ok_or_else(|| anyhow!("No meshes"))?;
        let prim = mesh.primitives().next().ok_or_else(|| anyhow!("No primitives"))?;
    let reader = prim.reader(|buf| buffers.get(buf.index()).map(|d| d.0.as_slice()));
    let positions: Vec<[f32;3]> = reader.read_positions().ok_or_else(|| anyhow!("Positions missing"))?.collect();
    let normals: Vec<[f32;3]> = reader.read_normals().ok_or_else(|| anyhow!("Normals missing"))?.collect();
    let tangents: Vec<[f32;4]> = match reader.read_tangents() { Some(it) => it.collect(), None => vec![[1.0,0.0,0.0,1.0]; positions.len()] };
    let texcoords: Vec<[f32;2]> = match reader.read_tex_coords(0) { Some(c) => c.into_f32().collect(), None => vec![[0.0,0.0]; positions.len()] };
        let indices_read = reader.read_indices().ok_or_else(|| anyhow!("Indices missing"))?;
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

        Ok((MeshData{ positions, normals, tangents, texcoords, indices }, mat))
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
        let prim = mesh.primitives().next().ok_or_else(|| anyhow!("No primitives"))?;
    let reader = prim.reader(|buf| buffers.get(buf.index()).map(|d| d.as_slice()));
    let positions: Vec<[f32;3]> = reader.read_positions().ok_or_else(|| anyhow!("Positions missing"))?.collect();
    let normals: Vec<[f32;3]> = reader.read_normals().ok_or_else(|| anyhow!("Normals missing"))?.collect();
    let tangents: Vec<[f32;4]> = match reader.read_tangents() { Some(it) => it.collect(), None => vec![[1.0,0.0,0.0,1.0]; positions.len()] };
    let texcoords: Vec<[f32;2]> = match reader.read_tex_coords(0) { Some(c) => c.into_f32().collect(), None => vec![[0.0,0.0]; positions.len()] };
        let indices_read = reader.read_indices().ok_or_else(|| anyhow!("Indices missing"))?;
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

        Ok((MeshData{ positions, normals, tangents, texcoords, indices }, mat))
    }

    fn load_uri_bytes(uri: &str) -> Result<Vec<u8>> {
        if let Some(rest) = uri.strip_prefix("data:") {
            // data:[<mediatype>][;base64],<data>
            let parts: Vec<&str> = rest.split(',').collect();
            if parts.len() != 2 { bail!("Invalid data URI") }
            let data = base64::engine::general_purpose::STANDARD.decode(parts[1])?;
            Ok(data)
        } else {
            // Treat as file path
            let bytes = std::fs::read(uri).with_context(|| format!("Read uri {uri}"))?;
            Ok(bytes)
        }
    }

    fn decode_image_from_gltf(source: gltf::image::Source, buffers: Option<&Vec<gltf::buffer::Data>>) -> Result<ImageData> {
        match source {
            gltf::image::Source::View { view, mime_type: _ } => {
                let buf_idx = view.buffer().index();
                let offset = view.offset();
                let length = view.length();
                let data = &buffers.ok_or_else(|| anyhow!("Missing buffers for buffer view image"))?[buf_idx].0.as_slice()[offset..offset+length];
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
        Ok(ImageData { width: w, height: h, rgba8: rgba.into_raw() })
    }

    #[inline]
    fn normalize_q(mut q: [f32;4]) -> [f32;4] {
        let len = (q[0]*q[0] + q[1]*q[1] + q[2]*q[2] + q[3]*q[3]).sqrt();
        if len > 0.0 { q[0]/=len; q[1]/=len; q[2]/=len; q[3]/=len; }
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
    pub fn load_first_skinned_mesh_and_idle(bytes: &[u8]) -> Result<(SkinnedMeshData, Option<AnimationClip>, Option<MaterialData>)> {
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
                    gltf::buffer::Source::Uri(_) => bail!("External buffer URIs not supported in GLB path"),
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
        let node = skinned_node.ok_or_else(|| anyhow!("No skinned node (node with mesh+skin) found"))?;
        let skin = node.skin().ok_or_else(|| anyhow!("Node missing skin"))?;
        let joint_count = skin.joints().len() as u32;
        let mesh = node.mesh().ok_or_else(|| anyhow!("Node missing mesh"))?;
    let prim = mesh.primitives().next().ok_or_else(|| anyhow!("No primitives in mesh"))?;

        let reader = prim.reader(|buf| buffers.get(buf.index()).map(|d| d.as_slice()));
        let positions: Vec<[f32; 3]> = reader.read_positions().ok_or_else(|| anyhow!("Positions missing"))?.collect();
        let normals: Vec<[f32; 3]> = reader.read_normals().ok_or_else(|| anyhow!("Normals missing"))?.collect();
        let tangents: Vec<[f32; 4]> = match reader.read_tangents() {
            Some(it) => it.collect(),
            None => vec![[1.0, 0.0, 0.0, 1.0]; positions.len()],
        };
        let indices_read = reader.read_indices().ok_or_else(|| anyhow!("Indices missing"))?;
        let indices: Vec<u32> = match indices_read {
            gltf::mesh::util::ReadIndices::U16(it) => it.map(|v| v as u32).collect(),
            gltf::mesh::util::ReadIndices::U32(it) => it.collect(),
            gltf::mesh::util::ReadIndices::U8(_) => bail!("u8 indices unsupported"),
        };
        let joints0: Vec<[u16; 4]> = match reader.read_joints(0).ok_or_else(|| anyhow!("JOINTS_0 missing"))? {
            gltf::mesh::util::ReadJoints::U8(it) => it.map(|j| [j[0] as u16, j[1] as u16, j[2] as u16, j[3] as u16]).collect(),
            gltf::mesh::util::ReadJoints::U16(it) => it.collect(),
        };
        let weights0: Vec<[f32; 4]> = {
            let rw = reader.read_weights(0).ok_or_else(|| anyhow!("WEIGHTS_0 missing"))?;
            match rw {
                gltf::mesh::util::ReadWeights::F32(it) => it.collect(),
                gltf::mesh::util::ReadWeights::U8(it) => it.map(|w| [w[0] as f32 / 255.0, w[1] as f32 / 255.0, w[2] as f32 / 255.0, w[3] as f32 / 255.0]).collect(),
                gltf::mesh::util::ReadWeights::U16(it) => it.map(|w| [w[0] as f32 / 65535.0, w[1] as f32 / 65535.0, w[2] as f32 / 65535.0, w[3] as f32 / 65535.0]).collect(),
            }
        };

        if positions.len() != normals.len() || positions.len() != joints0.len() || positions.len() != weights0.len() || positions.len() != tangents.len() {
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
                    if ch.target().node().index() == joint_idx && ch.target().property() == gltf::animation::Property::Rotation {
                        let reader = ch.reader(|buf| buffers.get(buf.index()).map(|d| d.as_slice()));
                        let inputs: Vec<f32> = reader.read_inputs().ok_or_else(|| anyhow!("Anim input missing"))?.collect();
                        let outputs: Vec<[f32; 4]> = match reader.read_outputs().ok_or_else(|| anyhow!("Anim output missing"))? {
                            gltf::animation::util::ReadOutputs::Rotations(rot_it) => match rot_it {
                                gltf::animation::util::Rotations::F32(it) => it.map(|r| [r[0], r[1], r[2], r[3]]).collect(),
                                gltf::animation::util::Rotations::I16(it) => it
                                    .map(|r| normalize_q([(r[0] as f32) / 32767.0, (r[1] as f32) / 32767.0, (r[2] as f32) / 32767.0, (r[3] as f32) / 32767.0]))
                                    .collect(),
                                gltf::animation::util::Rotations::I8(it) => it
                                    .map(|r| normalize_q([(r[0] as f32) / 127.0, (r[1] as f32) / 127.0, (r[2] as f32) / 127.0, (r[3] as f32) / 127.0]))
                                    .collect(),
                                gltf::animation::util::Rotations::U8(it) => it
                                    .map(|r| normalize_q([(r[0] as f32) / 255.0, (r[1] as f32) / 255.0, (r[2] as f32) / 255.0, (r[3] as f32) / 255.0]))
                                    .collect(),
                                gltf::animation::util::Rotations::U16(it) => it
                                    .map(|r| normalize_q([(r[0] as f32) / 65535.0, (r[1] as f32) / 65535.0, (r[2] as f32) / 65535.0, (r[3] as f32) / 65535.0]))
                                    .collect(),
                            },
                            _ => bail!("Anim outputs not rotations"),
                        };
                        if inputs.len() == outputs.len() && !inputs.is_empty() {
                            clip = Some(AnimationClip { times: inputs, rotations: outputs });
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
        let buffers_data: Vec<gltf::buffer::Data> = buffers.iter().cloned().map(gltf::buffer::Data).collect();
        if let Some(tex) = pbr.base_color_texture() {
            let img = decode_image_from_gltf(tex.texture().source().source(), Some(&buffers_data))
                .unwrap_or_else(|_| ImageData{ width:1, height:1, rgba8: vec![255,255,255,255] });
            mat.base_color_texture = Some(img);
        }
        if let Some(tex) = pbr.metallic_roughness_texture() {
            if let Ok(img) = decode_image_from_gltf(tex.texture().source().source(), Some(&buffers_data)) {
                mat.metallic_roughness_texture = Some(img);
            }
        }
        if let Some(n) = mat_g.normal_texture() {
            if let Ok(img) = decode_image_from_gltf(n.texture().source().source(), Some(&buffers_data)) {
                mat.normal_texture = Some(img);
            }
        }

        Ok((SkinnedMeshData { vertices, indices, joint_count }, clip, Some(mat)))
    }
}

pub struct AssetManifest;

impl AssetManifest {
    pub fn validate() -> Result<()> {
        // Placeholder: in Phase 0, no manifest; Phase 1 will add deterministic GUIDs
        Ok(())
    }
}
