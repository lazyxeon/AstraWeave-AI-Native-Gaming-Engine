// Debug test for slope_exactly_at_max_threshold geometry

use astraweave_nav::Triangle;
use glam::Vec3;

#[test]
fn debug_slope_triangle() {
    let angle_rad = 60.0_f32.to_radians();
    
    println!("\n=== SLOPE TRIANGLE DEBUG ===");
    println!("Target slope: 60°");
    println!("angle_rad: {}", angle_rad);
    println!("tan(60°): {}", angle_rad.tan());
    
    // After swap
    let a = Vec3::new(0.0, 0.0, 0.0);
    let b = Vec3::new(0.5, angle_rad.tan(), 0.5);
    let c = Vec3::new(1.0, 0.0, 0.0);
    
    println!("\nVertices after swap:");
    println!("a: ({:.3}, {:.3}, {:.3})", a.x, a.y, a.z);
    println!("b: ({:.3}, {:.3}, {:.3})", b.x, b.y, b.z);
    println!("c: ({:.3}, {:.3}, {:.3})", c.x, c.y, c.z);
    
    let edge1 = b - a;
    let edge2 = c - a;
    
    println!("\nEdges:");
    println!("edge1 (b-a): ({:.3}, {:.3}, {:.3})", edge1.x, edge1.y, edge1.z);
    println!("edge2 (c-a): ({:.3}, {:.3}, {:.3})", edge2.x, edge2.y, edge2.z);
    
    let normal = edge1.cross(edge2);
    let normalized = normal.normalize_or_zero();
    
    println!("\nNormal:");
    println!("raw: ({:.3}, {:.3}, {:.3})", normal.x, normal.y, normal.z);
    println!("normalized: ({:.3}, {:.3}, {:.3})", normalized.x, normalized.y, normalized.z);
    
    let dot = normalized.dot(Vec3::Y);
    println!("\nDot product with Y: {:.3}", dot);
    
    if dot < 0.0 {
        println!("STATUS: ❌ DOWNWARD (dot < 0)");
    } else {
        let angle = dot.clamp(-1.0, 1.0).acos().to_degrees();
        println!("Angle from vertical: {:.3}°", angle);
        
        if angle <= 60.0 {
            println!("STATUS: ✅ Should be INCLUDED (angle {:.3}° <= 60°)", angle);
        } else {
            println!("STATUS: ❌ Should be FILTERED (angle {:.3}° > 60°)", angle);
        }
    }
    
    // Try original geometry (before swap) for comparison
    println!("\n=== ORIGINAL GEOMETRY (before swap) ===");
    let b_orig = Vec3::new(1.0, 0.0, 0.0);
    let c_orig = Vec3::new(0.5, angle_rad.tan(), 0.5);
    
    let edge1_orig = b_orig - a;
    let edge2_orig = c_orig - a;
    let normal_orig = edge1_orig.cross(edge2_orig);
    let normalized_orig = normal_orig.normalize_or_zero();
    
    println!("normal: ({:.3}, {:.3}, {:.3})", normalized_orig.x, normalized_orig.y, normalized_orig.z);
    let dot_orig = normalized_orig.dot(Vec3::Y);
    println!("dot: {:.3}", dot_orig);
    
    if dot_orig < 0.0 {
        println!("STATUS: ❌ DOWNWARD");
    } else {
        let angle_orig = dot_orig.clamp(-1.0, 1.0).acos().to_degrees();
        println!("angle: {:.3}°", angle_orig);
    }
    
    println!("\n=== END DEBUG ===\n");
}
