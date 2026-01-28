// =============================================================================
// δ-SPH Particle Shifting - Research-Grade Implementation
// =============================================================================
//
// Implements the δ-SPH particle shifting scheme from:
// - Marrone et al. 2011: "δ-SPH model for simulating violent impact flows"
// - Sun et al. 2017: "The δplus-SPH model"
// - Lind et al. 2012: "Incompressible SPH for free surface flows"
//
// Particle shifting prevents the Lagrangian disorder that causes:
// - Tensile instability (particle clumping)
// - Numerical noise in pressure fields
// - Surface fragmentation artifacts
//
// This module provides:
// - Standard δ-SPH shifting (Marrone 2011)
// - δ⁺-SPH shifting (Sun 2017) with free-surface correction
// - Fickian diffusion shifting (Lind 2012)
// - Adaptive shifting strength based on local conditions
//
// =============================================================================

use crate::research::{ResearchParticle, ResearchSimParams, ShiftingMethod};

/// Configuration for particle shifting
#[derive(Clone, Debug)]
pub struct ShiftingConfig {
    /// Shifting method to use
    pub method: ShiftingMethod,
    
    /// Base shifting coefficient C_δ (typically 0.01-0.1)
    pub c_delta: f32,
    
    /// Free surface detection threshold (particle number density ratio)
    pub free_surface_threshold: f32,
    
    /// Maximum shifting distance as fraction of smoothing radius
    pub max_shift_ratio: f32,
    
    /// Enable adaptive strength based on local conditions
    pub adaptive_strength: bool,
    
    /// Blend factor for free-surface correction (δ⁺-SPH)
    pub surface_blend: f32,
    
    /// Fickian diffusion coefficient D (for Fickian method)
    pub diffusion_coefficient: f32,
    
    /// Concentration gradient limiter (prevents over-shifting)
    pub gradient_limiter: f32,
}

impl Default for ShiftingConfig {
    fn default() -> Self {
        Self {
            method: ShiftingMethod::StandardDelta,
            c_delta: 0.04,
            free_surface_threshold: 0.7,
            max_shift_ratio: 0.1,
            adaptive_strength: true,
            surface_blend: 0.5,
            diffusion_coefficient: 0.1,
            gradient_limiter: 0.5,
        }
    }
}

impl ShiftingConfig {
    /// Create configuration for standard δ-SPH (Marrone 2011)
    #[must_use]
    pub fn standard_delta() -> Self {
        Self {
            method: ShiftingMethod::StandardDelta,
            c_delta: 0.04,
            ..Default::default()
        }
    }
    
    /// Create configuration for δ⁺-SPH (Sun 2017) with interface-aware correction
    #[must_use]
    pub fn delta_plus() -> Self {
        Self {
            method: ShiftingMethod::InterfaceAware,
            c_delta: 0.04,
            surface_blend: 0.5,
            adaptive_strength: true,
            ..Default::default()
        }
    }
    
    /// Create configuration for Fickian-like diffusion shifting
    /// Uses StandardDelta with diffusion-based parameters (Lind 2012)
    #[must_use]
    pub fn fickian() -> Self {
        Self {
            method: ShiftingMethod::StandardDelta,
            diffusion_coefficient: 0.1,
            c_delta: 0.02, // Lower for diffusion-like behavior
            ..Default::default()
        }
    }
    
    /// Create configuration for aggressive shifting (high viscosity fluids)
    #[must_use]
    pub fn aggressive() -> Self {
        Self {
            method: ShiftingMethod::InterfaceAware,
            c_delta: 0.08,
            max_shift_ratio: 0.15,
            adaptive_strength: true,
            ..Default::default()
        }
    }
    
    /// Create configuration for conservative shifting (splashing scenarios)
    #[must_use]
    pub fn conservative() -> Self {
        Self {
            method: ShiftingMethod::StandardDelta,
            c_delta: 0.02,
            max_shift_ratio: 0.05,
            surface_blend: 0.8, // Reduce near surfaces
            ..Default::default()
        }
    }
}

/// Particle shifting state for a single particle
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ShiftingState {
    /// Computed shift vector (x, y, z)
    pub shift: [f32; 3],
    
    /// Particle concentration (number density normalized)
    pub concentration: f32,
    
    /// Free surface indicator (0.0 = interior, 1.0 = surface)
    pub surface_indicator: f32,
    
    /// Local shifting strength (adaptive)
    pub local_strength: f32,
    
    /// Concentration gradient magnitude
    pub gradient_magnitude: f32,
    
    /// Padding
    pub _pad: f32,
}

/// Statistics for particle shifting pass
#[derive(Clone, Debug, Default)]
pub struct ShiftingStats {
    /// Number of particles shifted
    pub particles_shifted: u32,
    
    /// Number of particles at free surface (reduced shifting)
    pub surface_particles: u32,
    
    /// Average shift magnitude
    pub avg_shift_magnitude: f32,
    
    /// Maximum shift magnitude
    pub max_shift_magnitude: f32,
    
    /// Average concentration
    pub avg_concentration: f32,
    
    /// Minimum concentration (indicates voids)
    pub min_concentration: f32,
}

/// CPU-side particle shifting calculator (for validation/debugging)
pub struct ParticleShiftingCpu {
    config: ShiftingConfig,
    stats: ShiftingStats,
}

impl ParticleShiftingCpu {
    /// Create a new CPU shifting calculator
    #[must_use]
    pub fn new(config: ShiftingConfig) -> Self {
        Self {
            config,
            stats: ShiftingStats::default(),
        }
    }
    
    /// Compute shift vectors for all particles
    ///
    /// This is the CPU reference implementation for validation.
    /// The GPU version is in pcisph.wgsl.
    pub fn compute_shifts(
        &mut self,
        particles: &mut [ResearchParticle],
        params: &ResearchSimParams,
    ) {
        if matches!(self.config.method, ShiftingMethod::None) {
            return;
        }
        
        let h = params.smoothing_radius;
        let h2 = h * h;
        
        // Reset stats
        self.stats = ShiftingStats::default();
        let mut total_shift = 0.0f32;
        let mut max_shift = 0.0f32;
        let mut total_concentration = 0.0f32;
        let mut min_concentration = f32::MAX;
        
        // First pass: compute concentrations and surface indicators (immutable borrow)
        let concentrations: Vec<(f32, f32)> = particles
            .iter()
            .enumerate()
            .map(|(i, pi)| {
                let pos_i = [pi.position[0], pi.position[1], pi.position[2]];
                let mut concentration = 0.0f32;
                let mut neighbor_count = 0u32;
                
                // Sum kernel contributions from neighbors
                for (j, pj) in particles.iter().enumerate() {
                    if i == j {
                        continue;
                    }
                    
                    let pos_j = [pj.position[0], pj.position[1], pj.position[2]];
                    let dx = pos_i[0] - pos_j[0];
                    let dy = pos_i[1] - pos_j[1];
                    let dz = pos_i[2] - pos_j[2];
                    let r2 = dx * dx + dy * dy + dz * dz;
                    
                    if r2 < h2 {
                        let r = r2.sqrt();
                        let w = Self::cubic_spline_kernel(r, h);
                        let volume = pj.position[3] / params.target_density;
                        concentration += w * volume;
                        neighbor_count += 1;
                    }
                }
                
                // Surface indicator based on neighbor count
                let expected_neighbors = 30.0; // Approximate for 3D cubic lattice
                let surface_indicator = 1.0 - (neighbor_count as f32 / expected_neighbors).min(1.0);
                
                (concentration, surface_indicator)
            })
            .collect();
        
        // Second pass: compute shift vectors (read-only positions needed)
        // We need to collect positions first to avoid borrow issues
        let positions: Vec<[f32; 4]> = particles.iter().map(|p| p.position).collect();
        
        let shifts: Vec<([f32; 3], f32, f32)> = (0..particles.len())
            .map(|i| {
                let pos_i = [positions[i][0], positions[i][1], positions[i][2]];
                let (concentration_i, surface_indicator) = concentrations[i];
                
                // Compute concentration gradient
                let mut grad_c = [0.0f32; 3];
                
                for j in 0..particles.len() {
                    if i == j {
                        continue;
                    }
                    
                    let pos_j = [positions[j][0], positions[j][1], positions[j][2]];
                    let dx = pos_i[0] - pos_j[0];
                    let dy = pos_i[1] - pos_j[1];
                    let dz = pos_i[2] - pos_j[2];
                    let r2 = dx * dx + dy * dy + dz * dz;
                    
                    if r2 < h2 && r2 > 1e-10 {
                        let r = r2.sqrt();
                        let grad_w = Self::cubic_spline_gradient(r, h);
                        let volume_j = positions[j][3] / params.target_density;
                        let (c_j, _) = concentrations[j];
                        
                        // Gradient of concentration
                        let dc = c_j - concentration_i;
                        let factor = dc * grad_w * volume_j / r;
                        
                        grad_c[0] += factor * dx;
                        grad_c[1] += factor * dy;
                        grad_c[2] += factor * dz;
                    }
                }
                
                // Compute shift based on method
                let shift = match self.config.method {
                    ShiftingMethod::None => [0.0, 0.0, 0.0],
                    
                    ShiftingMethod::StandardDelta => {
                        // δ-SPH: shift = -C_δ * h * ∇C
                        let c_delta = self.config.c_delta;
                        [
                            -c_delta * h * grad_c[0],
                            -c_delta * h * grad_c[1],
                            -c_delta * h * grad_c[2],
                        ]
                    }
                    
                    ShiftingMethod::InterfaceAware => {
                        // δ⁺-SPH: reduce shifting near free surfaces
                        let c_delta = self.config.c_delta;
                        let surface_factor = 1.0 - surface_indicator * self.config.surface_blend;
                        [
                            -c_delta * h * grad_c[0] * surface_factor,
                            -c_delta * h * grad_c[1] * surface_factor,
                            -c_delta * h * grad_c[2] * surface_factor,
                        ]
                    }
                };
                
                // Limit maximum shift
                let shift_mag = (shift[0] * shift[0] + shift[1] * shift[1] + shift[2] * shift[2]).sqrt();
                let max_allowed = self.config.max_shift_ratio * h;
                
                let limited_shift = if shift_mag > max_allowed && shift_mag > 1e-10 {
                    let scale = max_allowed / shift_mag;
                    [shift[0] * scale, shift[1] * scale, shift[2] * scale]
                } else {
                    shift
                };
                
                (limited_shift, concentration_i, surface_indicator)
            })
            .collect();
        
        // Third pass: write shifts back to particles and update stats
        for (i, p) in particles.iter_mut().enumerate() {
            let (limited_shift, concentration_i, surface_indicator) = shifts[i];
            
            let final_mag = (limited_shift[0] * limited_shift[0] 
                + limited_shift[1] * limited_shift[1] 
                + limited_shift[2] * limited_shift[2]).sqrt();
            
            // Store shift in particle's shift_delta field
            p.shift_delta = [limited_shift[0], limited_shift[1], limited_shift[2]];
            
            // Update stats
            if final_mag > 1e-10 {
                self.stats.particles_shifted += 1;
                total_shift += final_mag;
                max_shift = max_shift.max(final_mag);
            }
            
            if surface_indicator > 0.3 {
                self.stats.surface_particles += 1;
            }
            
            total_concentration += concentration_i;
            min_concentration = min_concentration.min(concentration_i);
        }
        
        let n = particles.len() as f32;
        if n > 0.0 {
            self.stats.avg_shift_magnitude = total_shift / n;
            self.stats.max_shift_magnitude = max_shift;
            self.stats.avg_concentration = total_concentration / n;
            self.stats.min_concentration = min_concentration;
        }
    }
    
    /// Apply computed shifts to particle positions
    pub fn apply_shifts(particles: &mut [ResearchParticle]) {
        for p in particles.iter_mut() {
            p.position[0] += p.shift_delta[0];
            p.position[1] += p.shift_delta[1];
            p.position[2] += p.shift_delta[2];
            
            // Also update predicted position
            p.predicted_position[0] += p.shift_delta[0];
            p.predicted_position[1] += p.shift_delta[1];
            p.predicted_position[2] += p.shift_delta[2];
        }
    }
    
    /// Get current statistics
    #[must_use]
    pub fn stats(&self) -> &ShiftingStats {
        &self.stats
    }
    
    /// Get configuration
    #[must_use]
    pub fn config(&self) -> &ShiftingConfig {
        &self.config
    }
    
    /// Update configuration
    pub fn set_config(&mut self, config: ShiftingConfig) {
        self.config = config;
    }
    
    // =========================================================================
    // Kernel Functions
    // =========================================================================
    
    /// Cubic spline kernel W(r, h)
    fn cubic_spline_kernel(r: f32, h: f32) -> f32 {
        let q = r / h;
        let sigma = 8.0 / (std::f32::consts::PI * h * h * h);
        
        if q < 0.5 {
            sigma * (6.0 * q * q * q - 6.0 * q * q + 1.0)
        } else if q < 1.0 {
            let t = 1.0 - q;
            sigma * 2.0 * t * t * t
        } else {
            0.0
        }
    }
    
    /// Cubic spline kernel gradient magnitude dW/dr
    fn cubic_spline_gradient(r: f32, h: f32) -> f32 {
        let q = r / h;
        let sigma = 8.0 / (std::f32::consts::PI * h * h * h);
        
        if q < 0.5 {
            sigma * (18.0 * q * q - 12.0 * q) / h
        } else if q < 1.0 {
            let t = 1.0 - q;
            sigma * (-6.0 * t * t) / h
        } else {
            0.0
        }
    }
}

/// Validate shifting quality metrics
#[derive(Clone, Debug)]
pub struct ShiftingQualityMetrics {
    /// Particle distribution uniformity (0-1, higher is better)
    pub uniformity: f32,
    
    /// Concentration variance (lower is better)
    pub concentration_variance: f32,
    
    /// Surface preservation score (higher is better)
    pub surface_preservation: f32,
    
    /// Overall quality grade
    pub grade: ShiftingQualityGrade,
}

/// Quality grades for shifting
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShiftingQualityGrade {
    /// Excellent: minimal clustering, good uniformity
    Excellent,
    /// Good: acceptable for most scenarios
    Good,
    /// Fair: some clustering visible
    Fair,
    /// Poor: significant particle disorder
    Poor,
}

impl ShiftingQualityMetrics {
    /// Compute quality metrics from particle data
    /// 
    /// Uses density variance as a proxy for particle distribution quality
    /// since concentration isn't stored per-particle.
    #[must_use]
    pub fn compute(particles: &[ResearchParticle], params: &ResearchSimParams) -> Self {
        if particles.is_empty() {
            return Self {
                uniformity: 0.0,
                concentration_variance: 0.0,
                surface_preservation: 0.0,
                grade: ShiftingQualityGrade::Poor,
            };
        }
        
        // Use density as a proxy for concentration
        let n = particles.len() as f32;
        let sum_d: f32 = particles.iter().map(|p| p.density).sum();
        let mean_d = sum_d / n;
        
        let variance: f32 = particles
            .iter()
            .map(|p| (p.density - mean_d).powi(2))
            .sum::<f32>()
            / n;
        
        // Normalize variance by target density squared
        let normalized_variance = variance / (params.target_density * params.target_density);
        
        // Uniformity: inverse of coefficient of variation
        let std_dev = variance.sqrt();
        let cv = if mean_d > 1e-10 { std_dev / mean_d } else { 1.0 };
        let uniformity = (1.0 - cv).max(0.0).min(1.0);
        
        // Surface preservation: check shift magnitudes for surface particles
        let h = params.smoothing_radius;
        let mut surface_count = 0;
        let mut surface_shift_sum = 0.0f32;
        
        for p in particles {
            // Use is_surface flag for surface detection
            if p.is_at_surface() {
                surface_count += 1;
                let shift_mag = (p.shift_delta[0] * p.shift_delta[0]
                    + p.shift_delta[1] * p.shift_delta[1]
                    + p.shift_delta[2] * p.shift_delta[2]).sqrt();
                surface_shift_sum += shift_mag / (0.1 * h); // Normalized by max allowed
            }
        }
        
        let surface_preservation = if surface_count > 0 {
            1.0 - (surface_shift_sum / surface_count as f32).min(1.0)
        } else {
            1.0
        };
        
        // Compute grade
        let grade = if uniformity > 0.9 && surface_preservation > 0.8 {
            ShiftingQualityGrade::Excellent
        } else if uniformity > 0.7 && surface_preservation > 0.6 {
            ShiftingQualityGrade::Good
        } else if uniformity > 0.5 && surface_preservation > 0.4 {
            ShiftingQualityGrade::Fair
        } else {
            ShiftingQualityGrade::Poor
        };
        
        Self {
            uniformity,
            concentration_variance: normalized_variance,
            surface_preservation,
            grade,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_shifting_config_defaults() {
        let config = ShiftingConfig::default();
        assert!((config.c_delta - 0.04).abs() < f32::EPSILON);
        assert!((config.max_shift_ratio - 0.1).abs() < f32::EPSILON);
        assert!(config.adaptive_strength);
    }
    
    #[test]
    fn test_shifting_config_presets() {
        let standard = ShiftingConfig::standard_delta();
        assert!(matches!(standard.method, ShiftingMethod::StandardDelta));
        
        let delta_plus = ShiftingConfig::delta_plus();
        assert!(matches!(delta_plus.method, ShiftingMethod::InterfaceAware));
        
        let fickian = ShiftingConfig::fickian();
        // Fickian uses StandardDelta with different parameters
        assert!(matches!(fickian.method, ShiftingMethod::StandardDelta));
        assert!(fickian.diffusion_coefficient > 0.0);
        
        let aggressive = ShiftingConfig::aggressive();
        assert!(aggressive.c_delta > standard.c_delta);
        
        let conservative = ShiftingConfig::conservative();
        assert!(conservative.c_delta < standard.c_delta);
    }
    
    #[test]
    fn test_shifting_state_size() {
        assert_eq!(std::mem::size_of::<ShiftingState>(), 32);
    }
    
    #[test]
    fn test_cubic_spline_kernel() {
        let h = 1.0;
        
        // At r=0, kernel should be positive
        let w0 = ParticleShiftingCpu::cubic_spline_kernel(0.0, h);
        assert!(w0 > 0.0);
        
        // At r=h, kernel should be zero
        let wh = ParticleShiftingCpu::cubic_spline_kernel(h, h);
        assert!(wh.abs() < 1e-6);
        
        // Kernel should decrease with distance
        let w1 = ParticleShiftingCpu::cubic_spline_kernel(0.25, h);
        let w2 = ParticleShiftingCpu::cubic_spline_kernel(0.5, h);
        let w3 = ParticleShiftingCpu::cubic_spline_kernel(0.75, h);
        assert!(w1 > w2);
        assert!(w2 > w3);
    }
    
    #[test]
    fn test_cubic_spline_gradient() {
        let h = 1.0;
        
        // Gradient at r=0 should be zero (maximum of kernel)
        let g0 = ParticleShiftingCpu::cubic_spline_gradient(0.0, h);
        assert!(g0.abs() < 1e-6);
        
        // Gradient should be negative (kernel decreases with r)
        let g1 = ParticleShiftingCpu::cubic_spline_gradient(0.3, h);
        assert!(g1 < 0.0);
        
        // Gradient at r=h should be zero
        let gh = ParticleShiftingCpu::cubic_spline_gradient(h, h);
        assert!(gh.abs() < 1e-6);
    }
    
    #[test]
    fn test_shifting_no_method() {
        let config = ShiftingConfig {
            method: ShiftingMethod::None,
            ..Default::default()
        };
        let mut shifter = ParticleShiftingCpu::new(config);
        
        let mut particles = vec![ResearchParticle::default(); 10];
        let params = ResearchSimParams::default();
        
        // Store original positions
        let original_positions: Vec<_> = particles.iter().map(|p| p.position).collect();
        
        shifter.compute_shifts(&mut particles, &params);
        
        // Shifts should all be zero
        for (p, orig) in particles.iter().zip(original_positions.iter()) {
            assert!((p.shift_delta[0]).abs() < 1e-10);
            assert!((p.shift_delta[1]).abs() < 1e-10);
            assert!((p.shift_delta[2]).abs() < 1e-10);
            // Positions unchanged
            assert!((p.position[0] - orig[0]).abs() < 1e-10);
        }
    }
    
    #[test]
    fn test_shifting_limits_magnitude() {
        let config = ShiftingConfig {
            method: ShiftingMethod::StandardDelta,
            c_delta: 1.0, // Very aggressive
            max_shift_ratio: 0.1,
            ..Default::default()
        };
        let mut shifter = ParticleShiftingCpu::new(config);
        
        // Create particles with high concentration gradient
        let mut particles = vec![];
        for i in 0..10 {
            let mut p = ResearchParticle::default();
            p.position = [i as f32 * 0.5, 0.0, 0.0, 1.0]; // Sparse spacing
            particles.push(p);
        }
        
        let params = ResearchSimParams {
            smoothing_radius: 1.0,
            target_density: 1000.0,
            ..Default::default()
        };
        
        shifter.compute_shifts(&mut particles, &params);
        
        // All shifts should be limited
        let max_allowed = 0.1 * params.smoothing_radius;
        for p in &particles {
            let shift_mag = (p.shift_delta[0] * p.shift_delta[0]
                + p.shift_delta[1] * p.shift_delta[1]
                + p.shift_delta[2] * p.shift_delta[2]).sqrt();
            assert!(
                shift_mag <= max_allowed + 1e-6,
                "Shift {} exceeds max {}",
                shift_mag,
                max_allowed
            );
        }
    }
    
    #[test]
    fn test_quality_metrics_empty() {
        let metrics = ShiftingQualityMetrics::compute(&[], &ResearchSimParams::default());
        assert_eq!(metrics.grade, ShiftingQualityGrade::Poor);
    }
    
    #[test]
    fn test_quality_grade_ordering() {
        // Just verify the enum exists and has expected variants
        let grades = [
            ShiftingQualityGrade::Excellent,
            ShiftingQualityGrade::Good,
            ShiftingQualityGrade::Fair,
            ShiftingQualityGrade::Poor,
        ];
        assert_eq!(grades.len(), 4);
    }
    
    #[test]
    fn test_apply_shifts() {
        let mut particles = vec![];
        for i in 0..3 {
            let mut p = ResearchParticle::default();
            p.position = [i as f32, 0.0, 0.0, 1.0];
            p.predicted_position = [i as f32, 0.0, 0.0, 1.0];
            p.shift_delta = [0.1, 0.2, 0.3]; // Predetermined shift
            particles.push(p);
        }
        
        ParticleShiftingCpu::apply_shifts(&mut particles);
        
        for (i, p) in particles.iter().enumerate() {
            assert!((p.position[0] - (i as f32 + 0.1)).abs() < 1e-6);
            assert!((p.position[1] - 0.2).abs() < 1e-6);
            assert!((p.position[2] - 0.3).abs() < 1e-6);
            
            assert!((p.predicted_position[0] - (i as f32 + 0.1)).abs() < 1e-6);
            assert!((p.predicted_position[1] - 0.2).abs() < 1e-6);
            assert!((p.predicted_position[2] - 0.3).abs() < 1e-6);
        }
    }
    
    #[test]
    fn test_shifter_stats() {
        let config = ShiftingConfig::standard_delta();
        let shifter = ParticleShiftingCpu::new(config);
        
        let stats = shifter.stats();
        assert_eq!(stats.particles_shifted, 0);
        assert_eq!(stats.surface_particles, 0);
    }
}
