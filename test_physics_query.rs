// Quick test to verify PhysicsWorld raycasting works
use glam::Vec3;
use astraweave_physics::PhysicsWorld;
use rapier3d::prelude::*;

fn main() {
    let mut phys = PhysicsWorld::new(Vec3::new(0.0, -9.81, 0.0));
    
    // Add two characters
    let id1 = phys.add_character(Vec3::ZERO, Vec3::new(0.5, 1.0, 0.5));
    let id2 = phys.add_character(Vec3::new(2.0, 0.0, 0.0), Vec3::new(0.5, 1.0, 0.5));
    
    println!("Created character 1: ID {}", id1);
    println!("Created character 2: ID {}", id2);
    
    // Step physics to update query pipeline
    phys.step();
    
    // Try raycasting
    let ray_from = Vec3::new(0.0, 1.0, 0.0);
    let ray_dir = Vec3::new(1.0, 0.0, 0.0);
    
    let ray = Ray::new(
        point![ray_from.x, ray_from.y, ray_from.z],
        vector![ray_dir.x, ray_dir.y, ray_dir.z],
    );
    
    let filter = QueryFilter::default();
    
    if let Some((collider_handle, hit)) = phys.query_pipeline.cast_ray_and_get_normal(
        &phys.bodies,
        &phys.colliders,
        &ray,
        10.0,
        true,
        filter,
    ) {
        println!("✅ Hit detected at distance: {}", hit.time_of_impact);
        
        if let Some(collider) = phys.colliders.get(collider_handle) {
            if let Some(body_handle) = collider.parent() {
                if let Some(body_id) = phys.id_of(body_handle) {
                    println!("✅ Hit body ID: {}", body_id);
                } else {
                    println!("❌ Could not get body ID from handle");
                }
            } else {
                println!("❌ Collider has no parent body");
            }
        } else {
            println!("❌ Could not get collider from handle");
        }
    } else {
        println!("❌ No hit detected - raycast failed");
    }
}
