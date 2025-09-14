//! Simple terrain generation demo
//! 
//! This example demonstrates the AstraWeave terrain generation system
//! by generating terrain chunks and outputting statistics about them.

use anyhow::Result;
use astraweave_terrain::*;
use astraweave_render::terrain::*;
use clap::Parser;

#[derive(Parser)]
#[command(name = "terrain_demo")]
#[command(about = "AstraWeave Terrain Generation Demo")]
struct Args {
    /// Random seed for terrain generation
    #[arg(short, long, default_value = "12345")]
    seed: u64,

    /// Size of terrain chunks
    #[arg(short, long, default_value = "256.0")]
    chunk_size: f32,

    /// Resolution of heightmaps (vertices per edge)
    #[arg(short, long, default_value = "64")]
    resolution: u32,

    /// Number of chunks to generate in each direction
    #[arg(short, long, default_value = "3")]
    grid_size: u32,

    /// Biome to focus on (grassland, desert, forest, mountain)
    #[arg(short, long)]
    biome: Option<String>,

    /// Export terrain data to files
    #[arg(short, long)]
    export: bool,

    /// Generate vegetation scatter
    #[arg(short, long)]
    vegetation: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    println!("AstraWeave Terrain Generation Demo");
    println!("===================================");
    println!();
    println!("Configuration:");
    println!("  Seed: {}", args.seed);
    println!("  Chunk Size: {}", args.chunk_size);
    println!("  Resolution: {}", args.resolution);
    println!("  Grid Size: {}x{}", args.grid_size, args.grid_size);
    if let Some(ref biome) = args.biome {
        println!("  Focused Biome: {}", biome);
    }
    println!();

    // Create world configuration
    let mut world_config = WorldConfig::default();
    world_config.seed = args.seed;
    world_config.chunk_size = args.chunk_size;
    world_config.heightmap_resolution = args.resolution;

    // Filter biomes if specified
    if let Some(biome_name) = args.biome {
        if let Some(biome_type) = BiomeType::from_str(&biome_name) {
            world_config.biomes.retain(|b| b.biome_type == biome_type);
            println!("Filtered to biome: {}", biome_name);
        } else {
            println!("Warning: Unknown biome '{}', using all biomes", biome_name);
        }
    }

    // Create terrain renderer
    let mut terrain_renderer = TerrainRenderer::new(world_config.clone());

    // Generate terrain chunks
    println!("Generating terrain...");
    let mut total_vertices = 0;
    let mut total_triangles = 0;
    let mut total_vegetation = 0;
    let mut total_resources = 0;
    let mut biome_counts = std::collections::HashMap::new();

    for x in 0..args.grid_size {
        for z in 0..args.grid_size {
            let chunk_id = ChunkId::new(x as i32, z as i32);
            
            if args.vegetation {
                let (mesh, scatter_result) = terrain_renderer.generate_chunk_complete(chunk_id)?;
                
                total_vertices += mesh.vertices.len();
                total_triangles += mesh.indices.len() / 3;
                total_vegetation += scatter_result.vegetation.len();
                total_resources += scatter_result.resources.len();

                // Count biomes in this chunk
                for vertex in &mesh.vertices {
                    let biome_id = vertex.biome_id;
                    *biome_counts.entry(biome_id).or_insert(0) += 1;
                }

                println!("  Chunk ({}, {}): {} vertices, {} vegetation, {} resources", 
                    x, z, mesh.vertices.len(), scatter_result.vegetation.len(), scatter_result.resources.len());

                if args.export {
                    export_chunk_data(&mesh, &scatter_result, &format!("chunk_{}_{}", x, z))?;
                }
            } else {
                let mesh = terrain_renderer.get_or_generate_chunk_mesh(chunk_id)?;
                total_vertices += mesh.vertices.len();
                total_triangles += mesh.indices.len() / 3;

                // Count biomes in this chunk
                for vertex in &mesh.vertices {
                    let biome_id = vertex.biome_id;
                    *biome_counts.entry(biome_id).or_insert(0) += 1;
                }

                println!("  Chunk ({}, {}): {} vertices", x, z, mesh.vertices.len());

                if args.export {
                    export_mesh_data(mesh, &format!("chunk_{}_{}", x, z))?;
                }
            }
        }
    }

    println!();
    println!("Generation Complete!");
    println!("===================");
    println!("Total Statistics:");
    println!("  Vertices: {}", total_vertices);
    println!("  Triangles: {}", total_triangles);
    if args.vegetation {
        println!("  Vegetation: {}", total_vegetation);
        println!("  Resources: {}", total_resources);
    }
    
    println!();
    println!("Biome Distribution:");
    for (biome_id, count) in biome_counts {
        let biome_name = match biome_id {
            0 => "Grassland",
            1 => "Desert",
            2 => "Forest", 
            3 => "Mountain",
            4 => "Tundra",
            5 => "Swamp",
            6 => "Beach",
            7 => "River",
            _ => "Unknown",
        };
        let percentage = (count as f32 / total_vertices as f32) * 100.0;
        println!("  {}: {} vertices ({:.1}%)", biome_name, count, percentage);
    }

    if args.export {
        println!();
        println!("Data exported to current directory");
    }

    // Generate a terrain preview
    println!();
    println!("Generating terrain preview...");
    let center = glam::Vec3::new(args.chunk_size * 0.5, 0.0, args.chunk_size * 0.5);
    let preview = generate_terrain_preview(&world_config, center, 32)?;
    
    let min_height = preview.iter().copied().fold(f32::INFINITY, f32::min);
    let max_height = preview.iter().copied().fold(f32::NEG_INFINITY, f32::max);
    let avg_height = preview.iter().sum::<f32>() / preview.len() as f32;
    
    println!("Height Statistics:");
    println!("  Min: {:.2}", min_height);
    println!("  Max: {:.2}", max_height);
    println!("  Average: {:.2}", avg_height);
    println!("  Range: {:.2}", max_height - min_height);

    Ok(())
}

fn export_mesh_data(mesh: &TerrainMesh, filename: &str) -> Result<()> {
    use std::io::Write;
    
    let mut file = std::fs::File::create(format!("{}_mesh.txt", filename))?;
    
    writeln!(file, "# Terrain Mesh Data for chunk {:?}", mesh.chunk_id)?;
    writeln!(file, "# Vertices: {}", mesh.vertices.len())?;
    writeln!(file, "# Triangles: {}", mesh.indices.len() / 3)?;
    writeln!(file)?;
    
    writeln!(file, "# Vertices (x, y, z, nx, ny, nz, u, v, biome_id)")?;
    for vertex in &mesh.vertices {
        writeln!(file, "{:.3} {:.3} {:.3} {:.3} {:.3} {:.3} {:.3} {:.3} {}", 
            vertex.position[0], vertex.position[1], vertex.position[2],
            vertex.normal[0], vertex.normal[1], vertex.normal[2],
            vertex.uv[0], vertex.uv[1],
            vertex.biome_id)?;
    }
    
    writeln!(file)?;
    writeln!(file, "# Indices (triangles)")?;
    for triangle in mesh.indices.chunks(3) {
        writeln!(file, "{} {} {}", triangle[0], triangle[1], triangle[2])?;
    }
    
    Ok(())
}

fn export_chunk_data(mesh: &TerrainMesh, scatter: &ScatterResult, filename: &str) -> Result<()> {
    use std::io::Write;
    
    // Export mesh data
    export_mesh_data(mesh, filename)?;
    
    // Export vegetation data
    let mut veg_file = std::fs::File::create(format!("{}_vegetation.txt", filename))?;
    writeln!(veg_file, "# Vegetation Data for chunk {:?}", mesh.chunk_id)?;
    writeln!(veg_file, "# Count: {}", scatter.vegetation.len())?;
    writeln!(veg_file)?;
    writeln!(veg_file, "# Vegetation (x, y, z, rotation, scale, type_name, model_path)")?;
    
    for veg in &scatter.vegetation {
        writeln!(veg_file, "{:.3} {:.3} {:.3} {:.3} {:.3} {} {}", 
            veg.position.x, veg.position.y, veg.position.z,
            veg.rotation, veg.scale,
            veg.vegetation_type, veg.model_path)?;
    }
    
    // Export resource data
    let mut res_file = std::fs::File::create(format!("{}_resources.txt", filename))?;
    writeln!(res_file, "# Resource Data for chunk {:?}", mesh.chunk_id)?;
    writeln!(res_file, "# Count: {}", scatter.resources.len())?;
    writeln!(res_file)?;
    writeln!(res_file, "# Resources (x, y, z, kind, amount, respawn_time)")?;
    
    for res in &scatter.resources {
        writeln!(res_file, "{:.3} {:.3} {:.3} {:?} {} {:.2}", 
            res.pos.x, res.pos.y, res.pos.z,
            res.kind, res.amount, res.respawn_time)?;
    }
    
    Ok(())
}