// Add this to examples/unified_showcase/src/main.rs before the run() function
// This patch will add detailed debugging information to help diagnose the texture rendering issues

fn debug_texture_rendering(render: &RenderStuff) -> Result<()> {
    println!("\n===== DEBUG TEXTURE RENDERING =====");
    
    // Check if ground texture and normal maps are loaded
    println!("Ground texture loaded: {}", render.ground_texture.is_some());
    println!("Ground normal map loaded: {}", render.ground_normal.is_some());
    println!("Ground bind group created: {}", render.ground_bind_group.is_some());
    
    // Verify bind group layout entries
    println!("\nChecking for potential texture binding issues:");
    
    // If the bind group is missing, that's likely the issue
    if render.ground_bind_group.is_none() {
        println!("ERROR: Missing ground bind group - textures won't render");
        println!("This could be caused by:");
        println!("1. Failed to load textures from disk");
        println!("2. Failed to create bind group due to layout mismatch");
        println!("3. Initialization function not called");
    }
    
    // Check for skybox and environment
    println!("\nEnvironment generation:");
    println!("- Check if skybox is created (missing skybox = gray void)");
    println!("- Verify environment objects are placed correctly");
    
    println!("\nGPU Information:");
    println!("Adapter info: {:?}", render.adapter_info);
    println!("===== END DEBUG INFO =====\n");
    
    Ok(())
}

// Add a call to this function just after the reload_texture_pack call in the run() function
// if let Err(e) = reload_texture_pack(&mut render, "grassland") {
//     println!("Warning: Failed to load initial grassland texture pack: {}", e);
//     ...
// } else {
//     println!("Successfully loaded initial grassland texture pack");
//     ...
//     debug_texture_rendering(&render).unwrap_or_else(|e| println!("Debug error: {}", e));
// }