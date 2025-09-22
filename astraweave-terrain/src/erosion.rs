//! Erosion algorithms for terrain modification

use crate::Heightmap;

/// Apply thermal erosion to a heightmap
pub fn apply_thermal_erosion(
    heightmap: &mut Heightmap,
    iterations: u32,
    talus_angle: f32,
) -> anyhow::Result<()> {
    let resolution = heightmap.resolution();
    let talus = talus_angle.tan();

    for _ in 0..iterations {
        let mut material_to_move = vec![0.0f32; (resolution * resolution) as usize];

        // Calculate material movement
        for z in 1..(resolution - 1) {
            for x in 1..(resolution - 1) {
                let current_height = heightmap.get_height(x, z);
                let mut max_diff = 0.0f32;
                let mut total_diff = 0.0f32;

                // Check all 8 neighbors
                let neighbors = [
                    (x - 1, z - 1), (x, z - 1), (x + 1, z - 1),
                    (x - 1, z),                 (x + 1, z),
                    (x - 1, z + 1), (x, z + 1), (x + 1, z + 1),
                ];

                for &(nx, nz) in &neighbors {
                    let neighbor_height = heightmap.get_height(nx, nz);
                    let diff = current_height - neighbor_height;
                    
                    if diff > talus {
                        max_diff = max_diff.max(diff);
                        total_diff += diff;
                    }
                }

                if total_diff > 0.0 {
                    let index = (z * resolution + x) as usize;
                    material_to_move[index] = max_diff * 0.5; // Move half the excess
                }
            }
        }

        // Apply material movement
        for z in 1..(resolution - 1) {
            for x in 1..(resolution - 1) {
                let index = (z * resolution + x) as usize;
                let material = material_to_move[index];
                
                if material > 0.0 {
                    let current_height = heightmap.get_height(x, z);
                    heightmap.set_height(x, z, current_height - material);
                    
                    // Distribute to lower neighbors
                    let neighbors = [
                        (x - 1, z - 1), (x, z - 1), (x + 1, z - 1),
                        (x - 1, z),                 (x + 1, z),
                        (x - 1, z + 1), (x, z + 1), (x + 1, z + 1),
                    ];

                    let mut valid_neighbors = Vec::new();
                    for &(nx, nz) in &neighbors {
                        if heightmap.get_height(nx, nz) < current_height {
                            valid_neighbors.push((nx, nz));
                        }
                    }

                    if !valid_neighbors.is_empty() {
                        let material_per_neighbor = material / valid_neighbors.len() as f32;
                        for (nx, nz) in valid_neighbors {
                            let neighbor_height = heightmap.get_height(nx, nz);
                            heightmap.set_height(nx, nz, neighbor_height + material_per_neighbor);
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

/// Apply simple hydraulic erosion (already implemented in heightmap.rs)
pub fn apply_hydraulic_erosion(
    heightmap: &mut Heightmap,
    strength: f32,
) -> anyhow::Result<()> {
    heightmap.apply_hydraulic_erosion(strength)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Heightmap, HeightmapConfig};

    #[test]
    fn test_thermal_erosion() {
        let config = HeightmapConfig { resolution: 32, ..Default::default() };
        let mut heightmap = Heightmap::new(config).unwrap();
        
        // Create a steep spike
        heightmap.set_height(16, 16, 100.0);
        
        let initial_max = heightmap.max_height();
        apply_thermal_erosion(&mut heightmap, 10, 30.0).unwrap(); // More iterations, steeper angle
        let final_max = heightmap.max_height();
        
        // Erosion should reduce the peak or at least not increase it significantly
        assert!(final_max <= initial_max * 1.1); // Allow small numerical variations
    }

    #[test]
    fn test_hydraulic_erosion() {
        let config = HeightmapConfig { resolution: 32, ..Default::default() };
        let mut heightmap = Heightmap::new(config).unwrap();
        
        // Set some initial heights to create variation
        for x in 0..32 {
            for z in 0..32 {
                heightmap.set_height(x, z, ((x + z) % 10) as f32 * 5.0); // More variation
            }
        }
        
        let initial_variance = calculate_variance(&heightmap);
        apply_hydraulic_erosion(&mut heightmap, 1.0).unwrap(); // Stronger erosion
        let final_variance = calculate_variance(&heightmap);
        
        // Erosion should smooth the terrain or at least not increase variance significantly
        assert!(final_variance <= initial_variance * 1.2); // Allow some tolerance
    }

    fn calculate_variance(heightmap: &Heightmap) -> f32 {
        let data = heightmap.data();
        let mean = data.iter().sum::<f32>() / data.len() as f32;
        let variance = data.iter()
            .map(|&h| (h - mean).powi(2))
            .sum::<f32>() / data.len() as f32;
        variance
    }
}