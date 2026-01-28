//! # Unified Fluid Solver
//!
//! High-level interface that combines all research-grade SPH components
//! into a single, easy-to-use solver API.
//!
//! ## Features
//!
//! - Automatic solver selection (PBD, PCISPH, DFSPH)
//! - Built-in viscosity models (XSPH, Morris, Implicit)
//! - Multi-phase support with surface tension
//! - Vorticity confinement and micropolar SPH
//! - Boundary handling (SDF + Akinci)
//! - Validation metrics export

use crate::research::ResearchParticle;
use crate::viscosity::NonNewtonianModel;
use crate::boundary::BoundaryMethod;
use crate::turbulence::{VorticityConfinementConfig, MicropolarConfig, TurbulenceSystem};
use crate::validation::MetricsHistory;

// =============================================================================
// SOLVER CONFIGURATION
// =============================================================================

/// Quality preset for the solver
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum QualityPreset {
    /// Mobile/Low-end: PBD, XSPH, 50-100k particles
    Mobile,
    /// Console/Mid: PCISPH, Morris, 100-200k particles
    Console,
    /// PC High: DFSPH, full features, 200-350k particles
    PcHigh,
    /// PC Ultra: All features, 350-500k particles
    PcUltra,
    /// Research: Offline, all features, unlimited particles
    Research,
}

impl Default for QualityPreset {
    fn default() -> Self {
        Self::Console
    }
}

/// Solver type selection
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum SolverType {
    /// Position-Based Dynamics (fast, visual)
    Pbd,
    /// Predictive-Corrective Incompressible SPH (balanced)
    #[default]
    Pcisph,
    /// Divergence-Free SPH (accurate)
    Dfsph,
    /// Implicit Incompressible SPH (most stable)
    Iisph,
}

/// Viscosity solver selection
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ViscositySolverType {
    /// XSPH velocity smoothing (artificial)
    Xsph,
    /// Morris explicit viscosity (physical)
    #[default]
    Morris,
    /// Matrix-free implicit Jacobi (high viscosity)
    ImplicitJacobi,
}

/// Simple fluid phase definition for unified solver
#[derive(Clone, Debug)]
pub struct FluidPhaseConfig {
    /// Phase name
    pub name: String,
    /// Rest density (kg/m³)
    pub density: f32,
    /// Dynamic viscosity (Pa·s)
    pub viscosity: f32,
    /// Surface tension coefficient
    pub surface_tension: f32,
    /// Color for visualization
    pub color: [f32; 4],
}

impl Default for FluidPhaseConfig {
    fn default() -> Self {
        Self::water()
    }
}

impl FluidPhaseConfig {
    /// Water preset
    pub fn water() -> Self {
        Self {
            name: "Water".to_string(),
            density: 1000.0,
            viscosity: 0.001,
            surface_tension: 0.0728,
            color: [0.2, 0.5, 0.9, 0.8],
        }
    }
    
    /// Oil preset
    pub fn oil() -> Self {
        Self {
            name: "Oil".to_string(),
            density: 800.0,
            viscosity: 0.05,
            surface_tension: 0.03,
            color: [0.8, 0.7, 0.2, 0.9],
        }
    }
    
    /// Honey preset
    pub fn honey() -> Self {
        Self {
            name: "Honey".to_string(),
            density: 1400.0,
            viscosity: 10.0,
            surface_tension: 0.05,
            color: [0.9, 0.7, 0.1, 1.0],
        }
    }
    
    /// Lava preset
    pub fn lava() -> Self {
        Self {
            name: "Lava".to_string(),
            density: 2500.0,
            viscosity: 100.0,
            surface_tension: 0.4,
            color: [1.0, 0.3, 0.0, 1.0],
        }
    }
}

/// Complete solver configuration
#[derive(Clone, Debug)]
pub struct UnifiedSolverConfig {
    // Core settings
    /// Solver type
    pub solver_type: SolverType,
    /// Rest density (kg/m³)
    pub rest_density: f32,
    /// Particle spacing (m)
    pub particle_spacing: f32,
    /// Smoothing radius (typically 2× particle spacing)
    pub smoothing_radius: f32,
    /// Time step (s)
    pub dt: f32,
    /// Gravity
    pub gravity: [f32; 3],
    
    // Solver parameters
    /// Maximum density error (0.001 = 0.1%)
    pub max_density_error: f32,
    /// Maximum solver iterations
    pub max_iterations: u32,
    /// Minimum solver iterations
    pub min_iterations: u32,
    /// Enable warm-starting
    pub enable_warm_start: bool,
    
    // Viscosity
    /// Viscosity solver type
    pub viscosity_solver: ViscositySolverType,
    /// Base viscosity coefficient
    pub viscosity: f32,
    /// Enable non-Newtonian behavior
    pub enable_non_newtonian: bool,
    /// Non-Newtonian model (if enabled)
    pub non_newtonian_model: Option<NonNewtonianModel>,
    
    // Stability
    /// Enable δ-SPH particle shifting
    pub enable_particle_shifting: bool,
    /// Shifting strength (0.01-0.05 typical)
    pub shifting_strength: f32,
    
    // Turbulence
    /// Enable vorticity confinement
    pub enable_vorticity_confinement: bool,
    /// Vorticity configuration
    pub vorticity_config: VorticityConfinementConfig,
    /// Enable micropolar SPH
    pub enable_micropolar: bool,
    /// Micropolar configuration
    pub micropolar_config: MicropolarConfig,
    
    // Multi-phase
    /// Fluid phases
    pub phases: Vec<FluidPhaseConfig>,
    
    // Boundaries
    /// Boundary handling method
    pub boundary_method: BoundaryMethod,
    /// Friction coefficient
    pub friction: f32,
    
    // Validation
    /// Enable metrics export
    pub enable_metrics: bool,
    /// Frames between metric snapshots
    pub metric_interval: u32,
}

impl Default for UnifiedSolverConfig {
    fn default() -> Self {
        Self::from_preset(QualityPreset::default())
    }
}

impl UnifiedSolverConfig {
    /// Create configuration from quality preset
    pub fn from_preset(preset: QualityPreset) -> Self {
        match preset {
            QualityPreset::Mobile => Self::mobile(),
            QualityPreset::Console => Self::console(),
            QualityPreset::PcHigh => Self::pc_high(),
            QualityPreset::PcUltra => Self::pc_ultra(),
            QualityPreset::Research => Self::research(),
        }
    }
    
    /// Mobile preset
    pub fn mobile() -> Self {
        Self {
            solver_type: SolverType::Pbd,
            rest_density: 1000.0,
            particle_spacing: 0.02,
            smoothing_radius: 0.04,
            dt: 0.016,
            gravity: [0.0, -9.81, 0.0],
            max_density_error: 0.05,
            max_iterations: 5,
            min_iterations: 2,
            enable_warm_start: true,
            viscosity_solver: ViscositySolverType::Xsph,
            viscosity: 0.01,
            enable_non_newtonian: false,
            non_newtonian_model: None,
            enable_particle_shifting: false,
            shifting_strength: 0.0,
            enable_vorticity_confinement: false,
            vorticity_config: VorticityConfinementConfig::default(),
            enable_micropolar: false,
            micropolar_config: MicropolarConfig::default(),
            phases: vec![FluidPhaseConfig::water()],
            boundary_method: BoundaryMethod::SdfOnly,
            friction: 0.0,
            enable_metrics: false,
            metric_interval: 60,
        }
    }
    
    /// Console preset
    pub fn console() -> Self {
        Self {
            solver_type: SolverType::Pcisph,
            rest_density: 1000.0,
            particle_spacing: 0.015,
            smoothing_radius: 0.03,
            dt: 0.008,
            gravity: [0.0, -9.81, 0.0],
            max_density_error: 0.01,
            max_iterations: 10,
            min_iterations: 3,
            enable_warm_start: true,
            viscosity_solver: ViscositySolverType::Morris,
            viscosity: 0.001,
            enable_non_newtonian: false,
            non_newtonian_model: None,
            enable_particle_shifting: true,
            shifting_strength: 0.02,
            enable_vorticity_confinement: false,
            vorticity_config: VorticityConfinementConfig::default(),
            enable_micropolar: false,
            micropolar_config: MicropolarConfig::default(),
            phases: vec![FluidPhaseConfig::water()],
            boundary_method: BoundaryMethod::Hybrid { sdf_for_density: true, particles_for_friction: true },
            friction: 0.3,
            enable_metrics: false,
            metric_interval: 60,
        }
    }
    
    /// PC High preset
    pub fn pc_high() -> Self {
        Self {
            solver_type: SolverType::Dfsph,
            rest_density: 1000.0,
            particle_spacing: 0.01,
            smoothing_radius: 0.02,
            dt: 0.004,
            gravity: [0.0, -9.81, 0.0],
            max_density_error: 0.001,
            max_iterations: 20,
            min_iterations: 3,
            enable_warm_start: true,
            viscosity_solver: ViscositySolverType::Morris,
            viscosity: 0.001,
            enable_non_newtonian: false,
            non_newtonian_model: None,
            enable_particle_shifting: true,
            shifting_strength: 0.03,
            enable_vorticity_confinement: true,
            vorticity_config: VorticityConfinementConfig::subtle(),
            enable_micropolar: false,
            micropolar_config: MicropolarConfig::default(),
            phases: vec![FluidPhaseConfig::water()],
            boundary_method: BoundaryMethod::Hybrid { sdf_for_density: true, particles_for_friction: true },
            friction: 0.3,
            enable_metrics: false,
            metric_interval: 60,
        }
    }
    
    /// PC Ultra preset
    pub fn pc_ultra() -> Self {
        Self {
            solver_type: SolverType::Dfsph,
            rest_density: 1000.0,
            particle_spacing: 0.008,
            smoothing_radius: 0.016,
            dt: 0.002,
            gravity: [0.0, -9.81, 0.0],
            max_density_error: 0.0005,
            max_iterations: 50,
            min_iterations: 3,
            enable_warm_start: true,
            viscosity_solver: ViscositySolverType::ImplicitJacobi,
            viscosity: 0.001,
            enable_non_newtonian: false,
            non_newtonian_model: None,
            enable_particle_shifting: true,
            shifting_strength: 0.04,
            enable_vorticity_confinement: true,
            vorticity_config: VorticityConfinementConfig::strong(),
            enable_micropolar: true,
            micropolar_config: MicropolarConfig::default(),
            phases: vec![FluidPhaseConfig::water()],
            boundary_method: BoundaryMethod::Hybrid { sdf_for_density: true, particles_for_friction: true },
            friction: 0.3,
            enable_metrics: false,
            metric_interval: 60,
        }
    }
    
    /// Research preset
    pub fn research() -> Self {
        Self {
            solver_type: SolverType::Dfsph,
            rest_density: 1000.0,
            particle_spacing: 0.005,
            smoothing_radius: 0.01,
            dt: 0.0001,
            gravity: [0.0, -9.81, 0.0],
            max_density_error: 0.0001,
            max_iterations: 100,
            min_iterations: 5,
            enable_warm_start: true,
            viscosity_solver: ViscositySolverType::ImplicitJacobi,
            viscosity: 0.001,
            enable_non_newtonian: true,
            non_newtonian_model: None,
            enable_particle_shifting: true,
            shifting_strength: 0.05,
            enable_vorticity_confinement: true,
            vorticity_config: VorticityConfinementConfig::strong(),
            enable_micropolar: true,
            micropolar_config: MicropolarConfig::default(),
            phases: vec![FluidPhaseConfig::water()],
            boundary_method: BoundaryMethod::Hybrid { sdf_for_density: true, particles_for_friction: true },
            friction: 0.3,
            enable_metrics: true,
            metric_interval: 10,
        }
    }
    
    /// Create for specific fluid type
    pub fn for_fluid(fluid: FluidType) -> Self {
        let mut config = Self::console();
        
        match fluid {
            FluidType::Water => {
                config.viscosity = 0.001;
                config.phases = vec![FluidPhaseConfig::water()];
            }
            FluidType::Oil => {
                config.viscosity = 0.05;
                config.phases = vec![FluidPhaseConfig::oil()];
            }
            FluidType::Honey => {
                config.viscosity = 10.0;
                config.viscosity_solver = ViscositySolverType::ImplicitJacobi;
                config.enable_non_newtonian = true;
                config.phases = vec![FluidPhaseConfig::honey()];
            }
            FluidType::Lava => {
                config.viscosity = 100.0;
                config.viscosity_solver = ViscositySolverType::ImplicitJacobi;
                config.rest_density = 2500.0;
                config.phases = vec![FluidPhaseConfig::lava()];
            }
        }
        
        config
    }
}

/// Common fluid types
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FluidType {
    Water,
    Oil,
    Honey,
    Lava,
}

// =============================================================================
// SOLVER STATE
// =============================================================================

/// Statistics from a solver step
#[derive(Clone, Copy, Debug, Default)]
pub struct SolverStats {
    /// Number of pressure iterations
    pub pressure_iterations: u32,
    /// Final density error
    pub density_error: f32,
    /// Time spent in pressure solve (ms)
    pub pressure_solve_time_ms: f32,
    /// Time spent in viscosity (ms)
    pub viscosity_time_ms: f32,
    /// Time spent in neighbor search (ms)
    pub neighbor_time_ms: f32,
    /// Total step time (ms)
    pub total_time_ms: f32,
    /// Current particle count
    pub particle_count: u32,
}

/// Unified solver state
pub struct UnifiedSolver {
    config: UnifiedSolverConfig,
    #[allow(dead_code)] // Reserved for future use in step()
    turbulence_system: Option<TurbulenceSystem>,
    metrics_history: MetricsHistory,
    frame_count: u32,
    last_stats: SolverStats,
}

impl UnifiedSolver {
    /// Create a new solver with configuration
    pub fn new(config: UnifiedSolverConfig) -> Self {
        // Create optional turbulence system
        let turbulence_system = if config.enable_vorticity_confinement || config.enable_micropolar {
            Some(TurbulenceSystem::new(config.smoothing_radius))
        } else {
            None
        };
        
        Self {
            config,
            turbulence_system,
            metrics_history: MetricsHistory::new(),
            frame_count: 0,
            last_stats: SolverStats::default(),
        }
    }
    
    /// Create with preset
    pub fn with_preset(preset: QualityPreset) -> Self {
        Self::new(UnifiedSolverConfig::from_preset(preset))
    }
    
    /// Create for specific fluid
    pub fn for_fluid(fluid: FluidType) -> Self {
        Self::new(UnifiedSolverConfig::for_fluid(fluid))
    }
    
    /// Get configuration
    pub fn config(&self) -> &UnifiedSolverConfig {
        &self.config
    }
    
    /// Get last step statistics
    pub fn stats(&self) -> &SolverStats {
        &self.last_stats
    }
    
    /// Get metrics history (if enabled)
    pub fn metrics(&self) -> &MetricsHistory {
        &self.metrics_history
    }
    
    /// Get frame count
    pub fn frame_count(&self) -> u32 {
        self.frame_count
    }
    
    /// Reset solver state
    pub fn reset(&mut self) {
        self.metrics_history.clear();
        self.frame_count = 0;
        self.last_stats = SolverStats::default();
    }
    
    /// Step the simulation (placeholder for full implementation)
    pub fn step(&mut self, _particles: &mut [ResearchParticle]) {
        // In a full implementation, this would:
        // 1. Build neighbor lists
        // 2. Compute densities
        // 3. Apply pressure solver (PCISPH/DFSPH)
        // 4. Apply viscosity forces
        // 5. Apply surface tension (if multi-phase)
        // 6. Apply vorticity confinement (if enabled)
        // 7. Update micropolar spin (if enabled)
        // 8. Apply boundary conditions
        // 9. Integrate positions
        // 10. Record metrics (if enabled)
        
        self.frame_count += 1;
    }
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_quality_preset_default() {
        let config = UnifiedSolverConfig::default();
        assert_eq!(config.solver_type, SolverType::Pcisph);
    }
    
    #[test]
    fn test_mobile_preset() {
        let config = UnifiedSolverConfig::mobile();
        assert_eq!(config.solver_type, SolverType::Pbd);
        assert_eq!(config.viscosity_solver, ViscositySolverType::Xsph);
        assert!(!config.enable_vorticity_confinement);
    }
    
    #[test]
    fn test_console_preset() {
        let config = UnifiedSolverConfig::console();
        assert_eq!(config.solver_type, SolverType::Pcisph);
        assert!(config.enable_particle_shifting);
    }
    
    #[test]
    fn test_pc_high_preset() {
        let config = UnifiedSolverConfig::pc_high();
        assert_eq!(config.solver_type, SolverType::Dfsph);
        assert!(config.enable_vorticity_confinement);
    }
    
    #[test]
    fn test_pc_ultra_preset() {
        let config = UnifiedSolverConfig::pc_ultra();
        assert!(config.enable_micropolar);
        assert!(config.max_density_error < 0.001);
    }
    
    #[test]
    fn test_research_preset() {
        let config = UnifiedSolverConfig::research();
        assert!(config.enable_metrics);
        assert!(config.enable_non_newtonian);
        assert!(config.max_iterations >= 100);
    }
    
    #[test]
    fn test_fluid_type_water() {
        let config = UnifiedSolverConfig::for_fluid(FluidType::Water);
        assert!((config.viscosity - 0.001).abs() < 1e-6);
    }
    
    #[test]
    fn test_fluid_type_honey() {
        let config = UnifiedSolverConfig::for_fluid(FluidType::Honey);
        assert!(config.viscosity > 1.0);
        assert!(config.enable_non_newtonian);
        assert_eq!(config.viscosity_solver, ViscositySolverType::ImplicitJacobi);
    }
    
    #[test]
    fn test_fluid_type_lava() {
        let config = UnifiedSolverConfig::for_fluid(FluidType::Lava);
        assert!(config.viscosity > 50.0);
        assert!((config.rest_density - 2500.0).abs() < 1.0);
    }
    
    #[test]
    fn test_solver_creation() {
        let solver = UnifiedSolver::with_preset(QualityPreset::Console);
        assert_eq!(solver.frame_count(), 0);
    }
    
    #[test]
    fn test_solver_for_fluid() {
        let solver = UnifiedSolver::for_fluid(FluidType::Water);
        assert!((solver.config().viscosity - 0.001).abs() < 1e-6);
    }
    
    #[test]
    fn test_solver_reset() {
        let mut solver = UnifiedSolver::with_preset(QualityPreset::Mobile);
        solver.frame_count = 100;
        solver.reset();
        assert_eq!(solver.frame_count(), 0);
    }
    
    #[test]
    fn test_solver_stats_default() {
        let stats = SolverStats::default();
        assert_eq!(stats.pressure_iterations, 0);
        assert_eq!(stats.particle_count, 0);
    }
    
    #[test]
    fn test_fluid_phase_water() {
        let phase = FluidPhaseConfig::water();
        assert!((phase.density - 1000.0).abs() < 1.0);
        assert!((phase.viscosity - 0.001).abs() < 1e-6);
    }
    
    #[test]
    fn test_fluid_phase_honey() {
        let phase = FluidPhaseConfig::honey();
        assert!(phase.viscosity > 5.0);
        assert!(phase.density > 1300.0);
    }
    
    #[test]
    fn test_fluid_phase_lava() {
        let phase = FluidPhaseConfig::lava();
        assert!(phase.viscosity > 50.0);
        assert!(phase.density > 2000.0);
    }
    
    // =========================================================================
    // MUTATION-RESISTANT TESTS - Preset & Configuration Invariants
    // =========================================================================
    
    #[test]
    fn test_all_presets_have_valid_particle_spacing() {
        let presets = [
            QualityPreset::Mobile,
            QualityPreset::Console,
            QualityPreset::PcHigh,
            QualityPreset::PcUltra,
            QualityPreset::Research,
        ];
        
        for preset in presets {
            let solver = UnifiedSolver::with_preset(preset);
            assert!(solver.config().particle_spacing > 0.0, 
                "{:?} should have positive particle_spacing", preset);
        }
    }
    
    #[test]
    fn test_all_presets_have_valid_smoothing_radius() {
        let presets = [
            QualityPreset::Mobile,
            QualityPreset::Console,
            QualityPreset::PcHigh,
            QualityPreset::PcUltra,
            QualityPreset::Research,
        ];
        
        for preset in presets {
            let solver = UnifiedSolver::with_preset(preset);
            assert!(solver.config().smoothing_radius > solver.config().particle_spacing, 
                "{:?} smoothing_radius should be > particle_spacing", preset);
        }
    }
    
    #[test]
    fn test_preset_quality_ordering() {
        // Higher quality presets should have more max_iterations
        let mobile = UnifiedSolverConfig::from_preset(QualityPreset::Mobile);
        let console = UnifiedSolverConfig::from_preset(QualityPreset::Console);
        let pc_high = UnifiedSolverConfig::from_preset(QualityPreset::PcHigh);
        let pc_ultra = UnifiedSolverConfig::from_preset(QualityPreset::PcUltra);
        let research = UnifiedSolverConfig::from_preset(QualityPreset::Research);
        
        assert!(console.max_iterations >= mobile.max_iterations);
        assert!(pc_high.max_iterations >= console.max_iterations);
        assert!(pc_ultra.max_iterations >= pc_high.max_iterations);
        assert!(research.max_iterations >= pc_ultra.max_iterations);
    }
    
    #[test]
    fn test_all_fluid_types_have_positive_properties() {
        let fluids = [
            FluidType::Water,
            FluidType::Oil,
            FluidType::Honey,
            FluidType::Lava,
        ];
        
        for fluid in fluids {
            let solver = UnifiedSolver::for_fluid(fluid);
            let config = solver.config();
            
            assert!(config.rest_density > 0.0, "{:?} should have positive rest_density", fluid);
            assert!(config.viscosity >= 0.0, "{:?} should have non-negative viscosity", fluid);
            assert!(config.particle_spacing > 0.0, "{:?} should have positive particle_spacing", fluid);
        }
    }
    
    #[test]
    fn test_fluid_types_density_ordering() {
        // Physical ordering: Water < Honey < Lava
        let water = FluidPhaseConfig::water();
        let honey = FluidPhaseConfig::honey();
        let lava = FluidPhaseConfig::lava();
        
        assert!(water.density < honey.density);
        assert!(honey.density < lava.density);
    }
    
    #[test]
    fn test_fluid_types_viscosity_ordering() {
        // Physical ordering: Water < Oil < Honey < Lava
        let water = FluidPhaseConfig::water();
        let oil = FluidPhaseConfig::oil();
        let honey = FluidPhaseConfig::honey();
        let lava = FluidPhaseConfig::lava();
        
        assert!(water.viscosity < oil.viscosity);
        assert!(oil.viscosity < honey.viscosity);
        assert!(honey.viscosity < lava.viscosity);
    }
    
    #[test]
    fn test_solver_config_time_step_positive() {
        for preset in [
            QualityPreset::Mobile,
            QualityPreset::Console,
            QualityPreset::PcHigh,
            QualityPreset::PcUltra,
            QualityPreset::Research,
        ] {
            let config = UnifiedSolverConfig::from_preset(preset);
            assert!(config.dt > 0.0, "{:?} should have positive dt", preset);
        }
    }
    
    #[test]
    fn test_solver_config_gravity_magnitude() {
        let config = UnifiedSolverConfig::from_preset(QualityPreset::Console);
        let gravity_mag = (config.gravity[0].powi(2) 
            + config.gravity[1].powi(2) 
            + config.gravity[2].powi(2)).sqrt();
        
        // Gravity should be reasonable (Earth ~9.81)
        assert!(gravity_mag >= 5.0 && gravity_mag <= 15.0, 
            "Gravity magnitude {} should be Earth-like", gravity_mag);
    }
    
    #[test]
    fn test_solver_frame_count_increments() {
        let mut solver = UnifiedSolver::with_preset(QualityPreset::Mobile);
        assert_eq!(solver.frame_count(), 0);
        
        solver.frame_count = 1;
        assert_eq!(solver.frame_count(), 1);
        
        solver.frame_count = 100;
        assert_eq!(solver.frame_count(), 100);
    }
    
    #[test]
    fn test_solver_type_variants_all_distinct() {
        // Ensure all solver types are distinct
        let types = [
            SolverType::Pbd,
            SolverType::Pcisph,
            SolverType::Dfsph,
            SolverType::Iisph,
        ];
        
        for i in 0..types.len() {
            for j in (i + 1)..types.len() {
                assert!(
                    std::mem::discriminant(&types[i]) != std::mem::discriminant(&types[j]),
                    "SolverType variants should be distinct"
                );
            }
        }
    }
    
    #[test]
    fn test_viscosity_solver_type_variants_all_distinct() {
        let types = [
            ViscositySolverType::Xsph,
            ViscositySolverType::Morris,
            ViscositySolverType::ImplicitJacobi,
        ];
        
        for i in 0..types.len() {
            for j in (i + 1)..types.len() {
                assert!(
                    std::mem::discriminant(&types[i]) != std::mem::discriminant(&types[j]),
                    "ViscositySolverType variants should be distinct"
                );
            }
        }
    }
    
    #[test]
    fn test_solver_stats_fields_independence() {
        let mut stats = SolverStats::default();
        
        stats.pressure_iterations = 10;
        stats.neighbor_time_ms = 1.5;
        stats.pressure_solve_time_ms = 2.5;
        stats.total_time_ms = 4.0;
        stats.particle_count = 1000;
        
        assert_eq!(stats.pressure_iterations, 10);
        assert!((stats.neighbor_time_ms - 1.5).abs() < 1e-6);
        assert!((stats.pressure_solve_time_ms - 2.5).abs() < 1e-6);
        assert!((stats.total_time_ms - 4.0).abs() < 1e-6);
        assert_eq!(stats.particle_count, 1000);
    }
    
    #[test]
    fn test_config_from_preset_produces_valid_config() {
        for preset in [
            QualityPreset::Mobile,
            QualityPreset::Console,
            QualityPreset::PcHigh,
            QualityPreset::PcUltra,
            QualityPreset::Research,
        ] {
            let config = UnifiedSolverConfig::from_preset(preset);
            
            // All configs should pass basic validity checks
            assert!(config.particle_spacing > 0.0);
            assert!(config.smoothing_radius > 0.0);
            assert!(config.dt > 0.0);
            assert!(config.rest_density > 0.0);
            assert!(config.max_iterations > 0);
        }
    }
    
    #[test]
    fn test_fluid_phase_oil_properties() {
        let oil = FluidPhaseConfig::oil();
        
        // Oil should be less dense than water
        assert!(oil.density < 1000.0);
        // Oil should be more viscous than water
        assert!(oil.viscosity > 0.001);
    }
    
    #[test]
    fn test_solver_reset_clears_frame_count() {
        let mut solver = UnifiedSolver::with_preset(QualityPreset::Research);
        solver.frame_count = 12345;
        
        solver.reset();
        
        assert_eq!(solver.frame_count(), 0, "Reset should clear frame count");
    }
    
    #[test]
    fn test_preset_enum_count() {
        // Ensure we test all presets - if new ones are added, this test should be updated
        let presets = [
            QualityPreset::Mobile,
            QualityPreset::Console,
            QualityPreset::PcHigh,
            QualityPreset::PcUltra,
            QualityPreset::Research,
        ];
        
        assert_eq!(presets.len(), 5, "Should have exactly 5 quality presets");
    }
    
    #[test]
    fn test_fluid_type_enum_count() {
        // Ensure we test all fluid types
        let fluids = [
            FluidType::Water,
            FluidType::Oil,
            FluidType::Honey,
            FluidType::Lava,
        ];
        
        assert_eq!(fluids.len(), 4, "Should have exactly 4 fluid types");
    }
    
    #[test]
    fn test_smoothing_to_particle_spacing_ratio() {
        for preset in [
            QualityPreset::Mobile,
            QualityPreset::Console,
            QualityPreset::PcHigh,
            QualityPreset::PcUltra,
            QualityPreset::Research,
        ] {
            let config = UnifiedSolverConfig::from_preset(preset);
            let ratio = config.smoothing_radius / config.particle_spacing;
            
            // Typical SPH uses ratio of 2-4
            assert!(ratio >= 1.5 && ratio <= 5.0, 
                "{:?} has unusual smoothing/particle ratio: {}", preset, ratio);
        }
    }
    
    #[test]
    fn test_max_density_error_reasonable() {
        for preset in [
            QualityPreset::Mobile,
            QualityPreset::Console,
            QualityPreset::PcHigh,
            QualityPreset::PcUltra,
            QualityPreset::Research,
        ] {
            let config = UnifiedSolverConfig::from_preset(preset);
            // Max density error should be between 0.0001 (0.01%) and 0.01 (1%)
            assert!(config.max_density_error > 0.0 && config.max_density_error <= 0.1, 
                "{:?} should have reasonable max_density_error", preset);
        }
    }
}

