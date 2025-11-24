use crate::schema::{MeshGenerationParams, TextureGenerationParams};
use crate::{AiMeshRequest, AiTextureRequest, RequestStatus};
use astraweave_ecs::prelude::*;
use astraweave_pcg::{MeshGenerator, TextureGenerator};
use image::RgbaImage;

pub fn process_ai_requests(
    mut texture_requests: Query<&mut AiTextureRequest>,
    mut mesh_requests: Query<&mut AiMeshRequest>,
    // In a real async bevy setup, we'd use a task pool or a resource that handles async results.
    // For this synchronous proof-of-concept, we assume params are already populated
    // OR we trigger the async call here.
    // To keep it simple for Phase 2, we'll assume the request *contains* the prompt,
    // and we'd need an async system to fill params.
    // But wait, the schema has params.
    // Let's assume the user (or another system) uses AiClient to get params,
    // and then spawns the request with params.
    // So this system just executes the PCG based on params.
) {
    for mut req in texture_requests.iter_mut() {
        if req.status == RequestStatus::Pending {
            req.status = RequestStatus::Processing;

            // In a real implementation, this would be async or offloaded to a thread pool
            // For Phase 1, we do it synchronously to prove the pipeline
            let _generated = generate_texture(&req.params);

            // TODO: Upload to GPU via astraweave-render
            // For now, we just mark it as completed
            req.status = RequestStatus::Completed;
            tracing::info!("Generated texture for prompt: {}", req.params.prompt);
        }
    }

    for mut req in mesh_requests.iter_mut() {
        if req.status == RequestStatus::Pending {
            req.status = RequestStatus::Processing;

            let _generated = generate_mesh(&req.params);

            // TODO: Upload to GPU
            req.status = RequestStatus::Completed;
            tracing::info!("Generated mesh for prompt: {}", req.params.prompt);
        }
    }
}

fn generate_texture(params: &TextureGenerationParams) -> RgbaImage {
    let seed = params.seed.unwrap_or(42);
    let gen = TextureGenerator::new(params.width, params.height, seed);
    gen.generate_noise()
}

fn generate_mesh(params: &MeshGenerationParams) -> astraweave_pcg::PcgMesh {
    let seed = params.seed.unwrap_or(42);
    // Note: MeshGenerator currently doesn't use seed, but we would pass it if it did
    let gen = MeshGenerator::new(params.size, params.resolution);
    gen.generate_plane()
}
