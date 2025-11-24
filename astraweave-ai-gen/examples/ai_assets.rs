use astraweave_ai_gen::schema::TextureGenerationParams;
use astraweave_ai_gen::{AiClient, AiGenPlugin, AiTextureRequest, RequestStatus};
use astraweave_ecs::prelude::*;
use astraweave_llm::{LlmClient, MockLlm};
use std::sync::Arc;

fn main() {
    // 1. Initialize App
    let mut app = App::new();

    // 2. Add Plugins
    app.add_plugins(AiGenPlugin);

    // 3. Setup AI Client (using Mock for demo)
    let mock_llm = Arc::new(MockLlm);
    let ai_client = AiClient::new(mock_llm);
    app.insert_resource(ai_client); // We need to make sure AiClient is a Resource if we use it in systems

    // 4. Spawn a request
    app.add_systems(Startup, setup_request);

    // 5. Run a few updates to simulate frame loop
    println!("Starting AI Asset Generation Demo...");
    for i in 0..5 {
        println!("Frame {}", i);
        app.update();
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
}

fn setup_request(mut commands: Commands) {
    println!("Spawning AI Texture Request...");
    commands.spawn(AiTextureRequest {
        params: TextureGenerationParams {
            prompt: "A stone wall texture".to_string(),
            width: 256,
            height: 256,
            scale: 5.0,
            roughness: 0.8,
            seed: Some(123),
            usage: "albedo".to_string(),
        },
        status: RequestStatus::Pending,
    });
}
