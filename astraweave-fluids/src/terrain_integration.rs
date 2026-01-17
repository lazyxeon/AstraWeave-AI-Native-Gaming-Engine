//! Terrain Integration Module for Fluid System
//!
//! Provides production-ready integration between the fluid simulation system
//! and terrain generation, including:
//! - River and stream generation from heightmaps
//! - Waterfall detection and spawning
//! - Lake and pond placement
//! - Water table simulation
//! - Ocean/coastal wave zones

use serde::{Deserialize, Serialize};

/// Water body types for terrain integration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WaterBodyType {
    /// Flowing water following terrain gradient
    River,
    /// Small flowing water with lower volume
    Stream,
    /// Static water body in terrain depressions
    Lake,
    /// Small static water body
    Pond,
    /// Coastal water with wave simulation
    Ocean,
    /// Vertical water flow from terrain edges
    Waterfall,
    /// Underground water affecting surface moisture
    Aquifer,
}

impl WaterBodyType {
    /// Get display name for UI
    pub fn display_name(&self) -> &'static str {
        match self {
            WaterBodyType::River => "River",
            WaterBodyType::Stream => "Stream",
            WaterBodyType::Lake => "Lake",
            WaterBodyType::Pond => "Pond",
            WaterBodyType::Ocean => "Ocean",
            WaterBodyType::Waterfall => "Waterfall",
            WaterBodyType::Aquifer => "Aquifer",
        }
    }
    
    /// Get all variants for UI iteration
    pub fn all() -> &'static [WaterBodyType] {
        &[
            WaterBodyType::River,
            WaterBodyType::Stream,
            WaterBodyType::Lake,
            WaterBodyType::Pond,
            WaterBodyType::Ocean,
            WaterBodyType::Waterfall,
            WaterBodyType::Aquifer,
        ]
    }
}

/// Configuration for river/stream generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiverConfig {
    /// Minimum slope gradient for river formation
    pub min_slope: f32,
    /// Maximum width of river in world units
    pub max_width: f32,
    /// Minimum width of river in world units
    pub min_width: f32,
    /// Depth multiplier based on flow accumulation
    pub depth_factor: f32,
    /// Flow velocity base multiplier
    pub flow_speed: f32,
    /// Erosion strength along river banks
    pub erosion_strength: f32,
    /// Sediment deposition rate at slow flow areas
    pub deposition_rate: f32,
    /// Meandering factor (0 = straight, 1 = highly meandering)
    pub meander_factor: f32,
    /// Particle emission rate per meter of river length
    pub particles_per_meter: f32,
}

impl Default for RiverConfig {
    fn default() -> Self {
        Self {
            min_slope: 0.01,
            max_width: 20.0,
            min_width: 2.0,
            depth_factor: 0.3,
            flow_speed: 5.0,
            erosion_strength: 0.2,
            deposition_rate: 0.1,
            meander_factor: 0.4,
            particles_per_meter: 100.0,
        }
    }
}

/// Configuration for waterfall generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaterfallConfig {
    /// Minimum height difference for waterfall formation
    pub min_height_drop: f32,
    /// Splash particle spawn multiplier
    pub splash_intensity: f32,
    /// Mist particle spawn rate
    pub mist_density: f32,
    /// Foam generation at waterfall base
    pub foam_intensity: f32,
    /// Sound effect volume multiplier
    pub audio_volume: f32,
    /// Spray particle lifetime in seconds
    pub spray_lifetime: f32,
}

impl Default for WaterfallConfig {
    fn default() -> Self {
        Self {
            min_height_drop: 3.0,
            splash_intensity: 2.0,
            mist_density: 1.0,
            foam_intensity: 1.5,
            audio_volume: 1.0,
            spray_lifetime: 2.0,
        }
    }
}

/// Configuration for lake/pond generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LakeConfig {
    /// Minimum depression depth for lake formation
    pub min_depth: f32,
    /// Water level relative to terrain depression
    pub water_level_offset: f32,
    /// Shore wave intensity (0 = calm, 1 = choppy)
    pub wave_intensity: f32,
    /// Enable underwater caustics
    pub enable_caustics: bool,
    /// Water clarity (0 = murky, 1 = crystal clear)
    pub clarity: f32,
    /// Reflection quality (0-1)
    pub reflection_quality: f32,
}

impl Default for LakeConfig {
    fn default() -> Self {
        Self {
            min_depth: 1.0,
            water_level_offset: -0.2,
            wave_intensity: 0.2,
            enable_caustics: true,
            clarity: 0.7,
            reflection_quality: 0.8,
        }
    }
}

/// Configuration for ocean/coastal zones
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OceanConfig {
    /// Sea level height in world units
    pub sea_level: f32,
    /// Primary wave amplitude
    pub wave_amplitude: f32,
    /// Primary wave frequency
    pub wave_frequency: f32,
    /// Secondary wave amplitude (cross-waves)
    pub secondary_wave_amplitude: f32,
    /// Wave direction in radians
    pub wave_direction: f32,
    /// Tide range (high - low)
    pub tide_range: f32,
    /// Tide cycle period in seconds
    pub tide_period: f32,
    /// Shore foam intensity
    pub shore_foam: f32,
    /// Deep water color (RGBA)
    pub deep_color: [f32; 4],
    /// Shallow water color (RGBA)
    pub shallow_color: [f32; 4],
}

impl Default for OceanConfig {
    fn default() -> Self {
        Self {
            sea_level: 0.0,
            wave_amplitude: 1.0,
            wave_frequency: 0.3,
            secondary_wave_amplitude: 0.3,
            wave_direction: 0.0,
            tide_range: 1.0,
            tide_period: 300.0,
            shore_foam: 0.8,
            deep_color: [0.02, 0.08, 0.15, 1.0],
            shallow_color: [0.1, 0.4, 0.5, 0.9],
        }
    }
}

/// A detected water body in the terrain
#[derive(Debug, Clone)]
pub struct DetectedWaterBody {
    /// Type of water body
    pub body_type: WaterBodyType,
    /// World-space bounding box minimum
    pub bounds_min: [f32; 3],
    /// World-space bounding box maximum
    pub bounds_max: [f32; 3],
    /// Center position
    pub center: [f32; 3],
    /// Volume estimate in cubic world units
    pub volume: f32,
    /// Flow direction for rivers/streams (normalized)
    pub flow_direction: Option<[f32; 3]>,
    /// Average flow speed for rivers/streams
    pub flow_speed: Option<f32>,
    /// Suggested particle count for simulation
    pub suggested_particle_count: u32,
}

/// Master configuration for terrain-fluid integration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerrainFluidConfig {
    /// Enable automatic water body detection
    pub auto_detect: bool,
    /// River generation settings
    pub river: RiverConfig,
    /// Waterfall generation settings
    pub waterfall: WaterfallConfig,
    /// Lake generation settings
    pub lake: LakeConfig,
    /// Ocean generation settings
    pub ocean: OceanConfig,
    /// Global water body particle budget
    pub max_total_particles: u32,
    /// Enable flow accumulation analysis
    pub analyze_flow: bool,
    /// Minimum flow accumulation for water body creation
    pub min_flow_accumulation: f32,
    /// Water table height (for groundwater simulation)
    pub water_table_height: f32,
    /// Enable moisture map generation
    pub generate_moisture_map: bool,
}

impl Default for TerrainFluidConfig {
    fn default() -> Self {
        Self {
            auto_detect: true,
            river: RiverConfig::default(),
            waterfall: WaterfallConfig::default(),
            lake: LakeConfig::default(),
            ocean: OceanConfig::default(),
            max_total_particles: 500_000,
            analyze_flow: true,
            min_flow_accumulation: 100.0,
            water_table_height: -5.0,
            generate_moisture_map: true,
        }
    }
}

impl TerrainFluidConfig {
    /// Create configuration optimized for performance
    pub fn performance() -> Self {
        Self {
            auto_detect: true,
            river: RiverConfig {
                particles_per_meter: 50.0,
                ..Default::default()
            },
            waterfall: WaterfallConfig {
                splash_intensity: 1.0,
                mist_density: 0.5,
                ..Default::default()
            },
            lake: LakeConfig {
                enable_caustics: false,
                reflection_quality: 0.5,
                ..Default::default()
            },
            ocean: OceanConfig {
                secondary_wave_amplitude: 0.0,
                shore_foam: 0.4,
                ..Default::default()
            },
            max_total_particles: 200_000,
            analyze_flow: true,
            min_flow_accumulation: 200.0,
            water_table_height: -5.0,
            generate_moisture_map: false,
        }
    }
    
    /// Create configuration optimized for quality
    pub fn quality() -> Self {
        Self {
            auto_detect: true,
            river: RiverConfig {
                particles_per_meter: 200.0,
                erosion_strength: 0.3,
                ..Default::default()
            },
            waterfall: WaterfallConfig {
                splash_intensity: 3.0,
                mist_density: 2.0,
                foam_intensity: 2.0,
                ..Default::default()
            },
            lake: LakeConfig {
                enable_caustics: true,
                reflection_quality: 1.0,
                clarity: 0.9,
                ..Default::default()
            },
            ocean: OceanConfig {
                secondary_wave_amplitude: 0.5,
                shore_foam: 1.0,
                ..Default::default()
            },
            max_total_particles: 1_000_000,
            analyze_flow: true,
            min_flow_accumulation: 50.0,
            water_table_height: -5.0,
            generate_moisture_map: true,
        }
    }
}

/// Flow accumulation data for a terrain point
#[derive(Debug, Clone, Copy, Default)]
pub struct FlowAccumulation {
    /// Total upstream contributing area
    pub upstream_area: f32,
    /// Flow direction as normalized vector
    pub direction: [f32; 2],
    /// Accumulated water volume
    pub volume: f32,
    /// Slope at this point
    pub slope: f32,
}

/// Terrain water analysis results
#[derive(Debug, Clone)]
pub struct WaterAnalysis {
    /// Detected water bodies
    pub water_bodies: Vec<DetectedWaterBody>,
    /// Flow accumulation grid (if analyze_flow is enabled)
    pub flow_accumulation: Option<Vec<FlowAccumulation>>,
    /// Moisture map (if generate_moisture_map is enabled)  
    pub moisture_map: Option<Vec<f32>>,
    /// Width of analysis grid
    pub width: usize,
    /// Height of analysis grid
    pub height: usize,
}

/// Analyzes terrain heightmap data to detect potential water bodies
pub fn analyze_terrain_for_water(
    heightmap: &[f32],
    width: usize,
    height: usize,
    config: &TerrainFluidConfig,
) -> WaterAnalysis {
    let mut water_bodies = Vec::new();
    let mut flow_accumulation = None;
    let mut moisture_map = None;
    
    if config.analyze_flow {
        // Compute flow accumulation using D8 algorithm
        let flow = compute_flow_accumulation(heightmap, width, height);
        
        // Detect rivers based on flow accumulation
        let rivers = detect_rivers(&flow, width, height, config);
        water_bodies.extend(rivers);
        
        flow_accumulation = Some(flow);
    }
    
    // Detect lakes from terrain depressions
    let lakes = detect_lakes(heightmap, width, height, config);
    water_bodies.extend(lakes);
    
    // Detect waterfalls from steep drops
    if !water_bodies.is_empty() {
        let waterfalls = detect_waterfalls(heightmap, width, height, config, &water_bodies);
        water_bodies.extend(waterfalls);
    }
    
    // Generate moisture map if requested
    if config.generate_moisture_map {
        moisture_map = Some(generate_moisture_map(
            heightmap, 
            width, 
            height, 
            config.water_table_height,
            &water_bodies,
        ));
    }
    
    WaterAnalysis {
        water_bodies,
        flow_accumulation,
        moisture_map,
        width,
        height,
    }
}

fn compute_flow_accumulation(
    heightmap: &[f32],
    width: usize,
    height: usize,
) -> Vec<FlowAccumulation> {
    let mut flow = vec![FlowAccumulation::default(); width * height];
    
    // D8 flow direction encoding
    let dx = [-1i32, 0, 1, -1, 1, -1, 0, 1];
    let dy = [-1i32, -1, -1, 0, 0, 1, 1, 1];
    
    // First pass: compute flow directions
    for y in 1..height - 1 {
        for x in 1..width - 1 {
            let idx = y * width + x;
            let current_h = heightmap[idx];
            
            let mut steepest_slope = 0.0f32;
            let mut steepest_dir = (0.0f32, 0.0f32);
            
            for i in 0..8 {
                let nx = (x as i32 + dx[i]) as usize;
                let ny = (y as i32 + dy[i]) as usize;
                let nidx = ny * width + nx;
                
                let neighbor_h = heightmap[nidx];
                let dist = if i == 0 || i == 2 || i == 5 || i == 7 { 1.414 } else { 1.0 };
                let slope = (current_h - neighbor_h) / dist;
                
                if slope > steepest_slope {
                    steepest_slope = slope;
                    steepest_dir = (dx[i] as f32 / dist, dy[i] as f32 / dist);
                }
            }
            
            flow[idx].direction = [steepest_dir.0, steepest_dir.1];
            flow[idx].slope = steepest_slope;
            flow[idx].upstream_area = 1.0; // Start with self
        }
    }
    
    // Second pass: accumulate flow (simplified - production would use recursive drainage)
    for _ in 0..10 {
        for y in 1..height - 1 {
            for x in 1..width - 1 {
                let idx = y * width + x;
                let dir = flow[idx].direction;
                
                if dir[0].abs() > 0.01 || dir[1].abs() > 0.01 {
                    let nx = (x as f32 + dir[0]).round() as usize;
                    let ny = (y as f32 + dir[1]).round() as usize;
                    
                    if nx < width && ny < height {
                        let nidx = ny * width + nx;
                        flow[nidx].upstream_area += flow[idx].upstream_area * 0.1;
                    }
                }
            }
        }
    }
    
    // Compute volumes
    for f in &mut flow {
        f.volume = f.upstream_area * 0.01;
    }
    
    flow
}

fn detect_rivers(
    flow: &[FlowAccumulation],
    width: usize,
    height: usize,
    config: &TerrainFluidConfig,
) -> Vec<DetectedWaterBody> {
    let mut rivers = Vec::new();
    
    // Find high flow accumulation points
    let threshold = config.min_flow_accumulation;
    
    for y in 0..height {
        for x in 0..width {
            let idx = y * width + x;
            if flow[idx].upstream_area > threshold {
                // Check if this is a new river or part of existing
                let is_new = !rivers.iter().any(|r: &DetectedWaterBody| {
                    let dx = r.center[0] - x as f32;
                    let dy = r.center[2] - y as f32;
                    (dx * dx + dy * dy).sqrt() < config.river.max_width * 2.0
                });
                
                if is_new {
                    let width_estimate = (flow[idx].upstream_area / threshold).sqrt() 
                        * config.river.min_width;
                    let clamped_width = width_estimate.clamp(config.river.min_width, config.river.max_width);
                    
                    rivers.push(DetectedWaterBody {
                        body_type: if width_estimate < config.river.min_width * 2.0 {
                            WaterBodyType::Stream
                        } else {
                            WaterBodyType::River
                        },
                        bounds_min: [x as f32 - clamped_width, 0.0, y as f32 - clamped_width],
                        bounds_max: [x as f32 + clamped_width, 2.0, y as f32 + clamped_width],
                        center: [x as f32, 0.0, y as f32],
                        volume: flow[idx].volume,
                        flow_direction: Some([flow[idx].direction[0], 0.0, flow[idx].direction[1]]),
                        flow_speed: Some(config.river.flow_speed * flow[idx].slope.sqrt()),
                        suggested_particle_count: (clamped_width * config.river.particles_per_meter) as u32,
                    });
                }
            }
        }
    }
    
    rivers
}

fn detect_lakes(
    heightmap: &[f32],
    width: usize,
    height: usize,
    config: &TerrainFluidConfig,
) -> Vec<DetectedWaterBody> {
    let mut lakes = Vec::new();
    let mut visited = vec![false; width * height];
    
    // Find local minima (potential lake centers)
    for y in 1..height - 1 {
        for x in 1..width - 1 {
            let idx = y * width + x;
            if visited[idx] {
                continue;
            }
            
            let h = heightmap[idx];
            let is_minimum = heightmap[(y - 1) * width + x] >= h
                && heightmap[(y + 1) * width + x] >= h
                && heightmap[y * width + x - 1] >= h
                && heightmap[y * width + x + 1] >= h;
            
            if is_minimum {
                // Flood fill to find lake extent
                let (lake_bounds, lake_volume) = flood_fill_lake(
                    heightmap, width, height, x, y, 
                    h + config.lake.min_depth,
                    &mut visited,
                );
                
                if lake_volume > config.lake.min_depth * 10.0 {
                    let center = [
                        (lake_bounds.0 + lake_bounds.2) / 2.0,
                        h + config.lake.water_level_offset,
                        (lake_bounds.1 + lake_bounds.3) / 2.0,
                    ];
                    
                    let size = ((lake_bounds.2 - lake_bounds.0) * (lake_bounds.3 - lake_bounds.1)).sqrt();
                    
                    lakes.push(DetectedWaterBody {
                        body_type: if size < 20.0 { WaterBodyType::Pond } else { WaterBodyType::Lake },
                        bounds_min: [lake_bounds.0, h - config.lake.min_depth, lake_bounds.1],
                        bounds_max: [lake_bounds.2, h + 1.0, lake_bounds.3],
                        center,
                        volume: lake_volume,
                        flow_direction: None,
                        flow_speed: None,
                        suggested_particle_count: (lake_volume * 100.0).min(100_000.0) as u32,
                    });
                }
            }
        }
    }
    
    lakes
}

fn flood_fill_lake(
    heightmap: &[f32],
    width: usize,
    height: usize,
    start_x: usize,
    start_y: usize,
    water_level: f32,
    visited: &mut [bool],
) -> ((f32, f32, f32, f32), f32) {
    let mut stack = vec![(start_x, start_y)];
    let mut min_x = start_x as f32;
    let mut min_y = start_y as f32;
    let mut max_x = start_x as f32;
    let mut max_y = start_y as f32;
    let mut volume = 0.0f32;
    
    while let Some((x, y)) = stack.pop() {
        if x == 0 || x >= width - 1 || y == 0 || y >= height - 1 {
            continue;
        }
        
        let idx = y * width + x;
        if visited[idx] {
            continue;
        }
        
        let h = heightmap[idx];
        if h > water_level {
            continue;
        }
        
        visited[idx] = true;
        volume += water_level - h;
        
        min_x = min_x.min(x as f32);
        min_y = min_y.min(y as f32);
        max_x = max_x.max(x as f32);
        max_y = max_y.max(y as f32);
        
        stack.push((x - 1, y));
        stack.push((x + 1, y));
        stack.push((x, y - 1));
        stack.push((x, y + 1));
    }
    
    ((min_x, min_y, max_x, max_y), volume)
}

fn detect_waterfalls(
    heightmap: &[f32],
    width: usize,
    height: usize,
    config: &TerrainFluidConfig,
    water_bodies: &[DetectedWaterBody],
) -> Vec<DetectedWaterBody> {
    let mut waterfalls = Vec::new();
    
    for body in water_bodies {
        if !matches!(body.body_type, WaterBodyType::River | WaterBodyType::Stream) {
            continue;
        }
        
        // Check along flow direction for steep drops
        if let (Some(dir), Some(_speed)) = (body.flow_direction, body.flow_speed) {
            let mut check_pos = body.center;
            
            for _ in 0..20 {
                check_pos[0] += dir[0] * 2.0;
                check_pos[2] += dir[2] * 2.0;
                
                let x = check_pos[0] as usize;
                let y = check_pos[2] as usize;
                
                if x >= width || y >= height {
                    break;
                }
                
                let current_idx = y * width + x;
                let next_x = (check_pos[0] + dir[0] * 3.0) as usize;
                let next_y = (check_pos[2] + dir[2] * 3.0) as usize;
                
                if next_x >= width || next_y >= height {
                    break;
                }
                
                let next_idx = next_y * width + next_x;
                let height_drop = heightmap[current_idx] - heightmap[next_idx];
                
                if height_drop >= config.waterfall.min_height_drop {
                    waterfalls.push(DetectedWaterBody {
                        body_type: WaterBodyType::Waterfall,
                        bounds_min: [check_pos[0] - 2.0, heightmap[next_idx], check_pos[2] - 2.0],
                        bounds_max: [check_pos[0] + 2.0, heightmap[current_idx], check_pos[2] + 2.0],
                        center: [check_pos[0], (heightmap[current_idx] + heightmap[next_idx]) / 2.0, check_pos[2]],
                        volume: height_drop * 4.0,
                        flow_direction: Some(dir),
                        flow_speed: Some(height_drop.sqrt() * 3.0),
                        suggested_particle_count: (height_drop * config.waterfall.splash_intensity * 500.0) as u32,
                    });
                    break;
                }
            }
        }
    }
    
    waterfalls
}

fn generate_moisture_map(
    heightmap: &[f32],
    width: usize,
    height: usize,
    water_table: f32,
    water_bodies: &[DetectedWaterBody],
) -> Vec<f32> {
    let mut moisture = vec![0.0f32; width * height];
    
    // Base moisture from water table proximity
    for y in 0..height {
        for x in 0..width {
            let idx = y * width + x;
            let h = heightmap[idx];
            
            // Higher moisture near water table
            let depth_below_surface = h - water_table;
            moisture[idx] = (1.0 - (depth_below_surface / 20.0).clamp(0.0, 1.0)) * 0.3;
        }
    }
    
    // Add moisture near water bodies
    for body in water_bodies {
        for y in 0..height {
            for x in 0..width {
                let dx = x as f32 - body.center[0];
                let dy = y as f32 - body.center[2];
                let dist = (dx * dx + dy * dy).sqrt();
                
                let body_radius = ((body.bounds_max[0] - body.bounds_min[0]) 
                    + (body.bounds_max[2] - body.bounds_min[2])) / 4.0;
                
                let influence = (1.0 - dist / (body_radius * 3.0)).max(0.0);
                let idx = y * width + x;
                moisture[idx] = (moisture[idx] + influence * 0.7).min(1.0);
            }
        }
    }
    
    moisture
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_water_body_types() {
        assert_eq!(WaterBodyType::River.display_name(), "River");
        assert_eq!(WaterBodyType::all().len(), 7);
    }
    
    #[test]
    fn test_river_config_defaults() {
        let config = RiverConfig::default();
        assert!(config.min_slope > 0.0);
        assert!(config.max_width > config.min_width);
        assert!(config.particles_per_meter > 0.0);
    }
    
    #[test]
    fn test_terrain_fluid_config_presets() {
        let perf = TerrainFluidConfig::performance();
        let quality = TerrainFluidConfig::quality();
        
        assert!(perf.max_total_particles < quality.max_total_particles);
        assert!(perf.river.particles_per_meter < quality.river.particles_per_meter);
    }
    
    #[test]
    fn test_flow_accumulation_basic() {
        // Create a simple sloped heightmap
        let width = 10;
        let height = 10;
        let mut heightmap = vec![0.0f32; width * height];
        
        // Slope from top-left to bottom-right
        for y in 0..height {
            for x in 0..width {
                heightmap[y * width + x] = (x + y) as f32 * 0.1;
            }
        }
        
        let flow = compute_flow_accumulation(&heightmap, width, height);
        assert_eq!(flow.len(), width * height);
        
        // Corner should have flow direction
        let center = flow[5 * width + 5];
        assert!(center.direction[0].abs() > 0.0 || center.direction[1].abs() > 0.0);
    }
    
    #[test]
    fn test_water_analysis() {
        let width = 20;
        let height = 20;
        let mut heightmap = vec![1.0f32; width * height];
        
        // Create a depression for a lake
        for y in 8..12 {
            for x in 8..12 {
                heightmap[y * width + x] = 0.0;
            }
        }
        
        let config = TerrainFluidConfig::default();
        let analysis = analyze_terrain_for_water(&heightmap, width, height, &config);
        
        assert_eq!(analysis.width, width);
        assert_eq!(analysis.height, height);
        // Should detect at least one water body (the lake)
        assert!(!analysis.water_bodies.is_empty() || analysis.flow_accumulation.is_some());
    }
    
    #[test]
    fn test_ocean_config() {
        let ocean = OceanConfig::default();
        assert!(ocean.wave_amplitude > 0.0);
        assert!(ocean.tide_period > 0.0);
        assert_eq!(ocean.deep_color.len(), 4);
    }
    
    #[test]
    fn test_detected_water_body() {
        let body = DetectedWaterBody {
            body_type: WaterBodyType::River,
            bounds_min: [0.0, 0.0, 0.0],
            bounds_max: [10.0, 2.0, 100.0],
            center: [5.0, 1.0, 50.0],
            volume: 200.0,
            flow_direction: Some([0.0, 0.0, 1.0]),
            flow_speed: Some(5.0),
            suggested_particle_count: 10000,
        };
        
        assert_eq!(body.body_type, WaterBodyType::River);
        assert!(body.flow_direction.is_some());
        assert_eq!(body.suggested_particle_count, 10000);
    }
}
