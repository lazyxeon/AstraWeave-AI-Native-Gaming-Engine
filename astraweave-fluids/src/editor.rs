//! Visual Editor Integration for Fluid System
//!
//! Provides comprehensive inspector-friendly types, real-time parameter tweaking,
//! water body templates, and production-ready editor widgets for game developers.
//!
//! # Features
//! - Complete water body presets (ocean, river, pool, waterfall, etc.)
//! - Real-time parameter validation with safe clamping
//! - Visual effect configuration (caustics, foam, god rays, reflections)
//! - Emitter and drain placement tools
//! - Performance profiling integration
//! - JSON/TOML serialization for asset pipeline
//! - Undo/redo state tracking with configuration history
//! - Smooth parameter interpolation for live tweaking
//! - Scene integration helpers (AABB bounds, queries)
//! - Accessibility features (colorblind-safe palettes)
//! - Batch operations for multi-select editing
//!
//! # Example
//! ```rust,ignore
//! use astraweave_fluids::editor::*;
//!
//! // Create water from preset
//! let mut config = FluidEditorConfig::from_preset(WaterBodyPreset::TropicalOcean);
//!
//! // Enable undo history tracking
//! let mut history = ConfigHistory::new(config.clone());
//!
//! // Modify with real-time preview
//! config.waves.amplitude = 2.0;
//! history.push(config.clone());
//!
//! // Undo if needed
//! if let Some(previous) = history.undo() {
//!     config = previous;
//! }
//!
//! // Interpolate for smooth transitions
//! let target = FluidEditorConfig::from_preset(WaterBodyPreset::Ocean);
//! let blended = config.interpolate(&target, 0.5);
//! ```

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

// =============================================================================
// WATER BODY PRESETS
// =============================================================================

/// Pre-configured water body types for quick setup
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum WaterBodyPreset {
    /// Clear swimming pool water
    Pool,
    /// Calm lake with natural appearance
    Lake,
    /// Flowing river with currents
    River,
    /// Ocean with waves and foam
    Ocean,
    /// Tropical clear water
    #[default]
    TropicalOcean,
    /// Murky swamp water
    Swamp,
    /// Hot spring with steam
    HotSpring,
    /// Waterfall with spray and mist
    Waterfall,
    /// Underground cave water
    CaveWater,
    /// Arctic icy water
    ArcticWater,
    /// Custom configuration
    Custom,
}

impl WaterBodyPreset {
    /// Get a human-readable description for the editor UI
    pub fn description(&self) -> &'static str {
        match self {
            Self::Pool => "Swimming Pool - Crystal clear, calm water",
            Self::Lake => "Lake - Natural calm water with subtle movement",
            Self::River => "River - Flowing water with current effects",
            Self::Ocean => "Ocean - Waves, foam, and deep water effects",
            Self::TropicalOcean => "Tropical Ocean - Crystal clear with caustics",
            Self::Swamp => "Swamp - Murky water with particles",
            Self::HotSpring => "Hot Spring - Warm water with steam effects",
            Self::Waterfall => "Waterfall - Falling water with spray and mist",
            Self::CaveWater => "Cave Water - Dark, reflective underground water",
            Self::ArcticWater => "Arctic Water - Cold, icy water with fog",
            Self::Custom => "Custom - User-defined configuration",
        }
    }

    /// Get all available presets for UI dropdown
    pub fn all_presets() -> &'static [WaterBodyPreset] {
        &[
            Self::Pool,
            Self::Lake,
            Self::River,
            Self::Ocean,
            Self::TropicalOcean,
            Self::Swamp,
            Self::HotSpring,
            Self::Waterfall,
            Self::CaveWater,
            Self::ArcticWater,
            Self::Custom,
        ]
    }
}

// =============================================================================
// QUALITY PRESETS
// =============================================================================

/// Quality/performance balance presets
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum QualityPreset {
    /// Minimal effects for low-end hardware
    Low,
    /// Balanced quality and performance
    #[default]
    Medium,
    /// Enhanced visuals for mid-range hardware
    High,
    /// Maximum quality (demanding)
    Ultra,
}

impl QualityPreset {
    /// Get description for UI
    pub fn description(&self) -> &'static str {
        match self {
            Self::Low => "Low - Best performance, minimal effects",
            Self::Medium => "Medium - Balanced quality and performance",
            Self::High => "High - Enhanced visuals",
            Self::Ultra => "Ultra - Maximum quality (demanding)",
        }
    }

    /// Get recommended particle count
    pub fn recommended_particles(&self) -> u32 {
        match self {
            Self::Low => 5_000,
            Self::Medium => 20_000,
            Self::High => 50_000,
            Self::Ultra => 100_000,
        }
    }
    
    /// Get all quality presets for UI dropdown
    pub fn all_presets() -> &'static [QualityPreset] {
        &[Self::Low, Self::Medium, Self::High, Self::Ultra]
    }
    
    /// Get recommended god ray samples
    pub fn recommended_god_ray_samples(&self) -> u32 {
        match self {
            Self::Low => 8,
            Self::Medium => 16,
            Self::High => 32,
            Self::Ultra => 48,
        }
    }
    
    /// Get recommended reflection resolution
    pub fn recommended_reflection_resolution(&self) -> u32 {
        match self {
            Self::Low => 256,
            Self::Medium => 512,
            Self::High => 1024,
            Self::Ultra => 2048,
        }
    }
}

// =============================================================================
// CONFIGURATION HISTORY (UNDO/REDO)
// =============================================================================

/// Configuration history for undo/redo support
#[derive(Clone, Debug)]
pub struct ConfigHistory {
    /// Past states (for undo)
    past: VecDeque<FluidEditorConfig>,
    /// Future states (for redo)
    future: VecDeque<FluidEditorConfig>,
    /// Maximum history size
    max_size: usize,
    /// Current state
    current: FluidEditorConfig,
}

impl ConfigHistory {
    /// Create new history with initial state
    pub fn new(initial: FluidEditorConfig) -> Self {
        Self {
            past: VecDeque::new(),
            future: VecDeque::new(),
            max_size: 50,
            current: initial,
        }
    }
    
    /// Create history with custom max size
    pub fn with_max_size(initial: FluidEditorConfig, max_size: usize) -> Self {
        Self {
            past: VecDeque::new(),
            future: VecDeque::new(),
            max_size: max_size.max(5),
            current: initial,
        }
    }
    
    /// Push new state (clears redo history)
    pub fn push(&mut self, state: FluidEditorConfig) {
        self.past.push_back(self.current.clone());
        self.current = state;
        self.future.clear();
        
        // Trim old history
        while self.past.len() > self.max_size {
            self.past.pop_front();
        }
    }
    
    /// Undo to previous state
    pub fn undo(&mut self) -> Option<FluidEditorConfig> {
        if let Some(previous) = self.past.pop_back() {
            self.future.push_front(self.current.clone());
            self.current = previous.clone();
            Some(previous)
        } else {
            None
        }
    }
    
    /// Redo to next state
    pub fn redo(&mut self) -> Option<FluidEditorConfig> {
        if let Some(next) = self.future.pop_front() {
            self.past.push_back(self.current.clone());
            self.current = next.clone();
            Some(next)
        } else {
            None
        }
    }
    
    /// Check if undo is available
    pub fn can_undo(&self) -> bool {
        !self.past.is_empty()
    }
    
    /// Check if redo is available
    pub fn can_redo(&self) -> bool {
        !self.future.is_empty()
    }
    
    /// Get current state
    pub fn current(&self) -> &FluidEditorConfig {
        &self.current
    }
    
    /// Get number of undo steps available
    pub fn undo_count(&self) -> usize {
        self.past.len()
    }
    
    /// Get number of redo steps available
    pub fn redo_count(&self) -> usize {
        self.future.len()
    }
    
    /// Clear all history
    pub fn clear(&mut self) {
        self.past.clear();
        self.future.clear();
    }
}

// =============================================================================
// SCENE INTEGRATION
// =============================================================================

/// Axis-aligned bounding box for fluid volumes
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct FluidAABB {
    /// Minimum corner
    pub min: [f32; 3],
    /// Maximum corner
    pub max: [f32; 3],
}

impl FluidAABB {
    /// Create new AABB from min/max corners
    pub fn new(min: [f32; 3], max: [f32; 3]) -> Self {
        Self { min, max }
    }
    
    /// Create AABB from center and half-extents
    pub fn from_center_extents(center: [f32; 3], half_extents: [f32; 3]) -> Self {
        Self {
            min: [
                center[0] - half_extents[0],
                center[1] - half_extents[1],
                center[2] - half_extents[2],
            ],
            max: [
                center[0] + half_extents[0],
                center[1] + half_extents[1],
                center[2] + half_extents[2],
            ],
        }
    }
    
    /// Get center of AABB
    pub fn center(&self) -> [f32; 3] {
        [
            (self.min[0] + self.max[0]) * 0.5,
            (self.min[1] + self.max[1]) * 0.5,
            (self.min[2] + self.max[2]) * 0.5,
        ]
    }
    
    /// Get size (dimensions)
    pub fn size(&self) -> [f32; 3] {
        [
            self.max[0] - self.min[0],
            self.max[1] - self.min[1],
            self.max[2] - self.min[2],
        ]
    }
    
    /// Get volume
    pub fn volume(&self) -> f32 {
        let s = self.size();
        s[0] * s[1] * s[2]
    }
    
    /// Check if point is inside AABB
    pub fn contains_point(&self, point: [f32; 3]) -> bool {
        point[0] >= self.min[0] && point[0] <= self.max[0] &&
        point[1] >= self.min[1] && point[1] <= self.max[1] &&
        point[2] >= self.min[2] && point[2] <= self.max[2]
    }
    
    /// Check if two AABBs overlap
    pub fn overlaps(&self, other: &FluidAABB) -> bool {
        self.min[0] <= other.max[0] && self.max[0] >= other.min[0] &&
        self.min[1] <= other.max[1] && self.max[1] >= other.min[1] &&
        self.min[2] <= other.max[2] && self.max[2] >= other.min[2]
    }
    
    /// Expand AABB to include point
    pub fn expand(&mut self, point: [f32; 3]) {
        for i in 0..3 {
            self.min[i] = self.min[i].min(point[i]);
            self.max[i] = self.max[i].max(point[i]);
        }
    }
    
    /// Merge two AABBs
    pub fn merge(&self, other: &FluidAABB) -> FluidAABB {
        FluidAABB {
            min: [
                self.min[0].min(other.min[0]),
                self.min[1].min(other.min[1]),
                self.min[2].min(other.min[2]),
            ],
            max: [
                self.max[0].max(other.max[0]),
                self.max[1].max(other.max[1]),
                self.max[2].max(other.max[2]),
            ],
        }
    }
}

/// Scene placement data for fluid volumes
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FluidScenePlacement {
    /// Unique ID for scene graph
    pub id: String,
    /// Display name in hierarchy
    pub display_name: String,
    /// World-space bounds
    pub bounds: FluidAABB,
    /// Is this fluid volume active?
    pub active: bool,
    /// Layer mask for rendering
    pub layer_mask: u32,
    /// Tags for queries
    pub tags: Vec<String>,
}

impl Default for FluidScenePlacement {
    fn default() -> Self {
        Self {
            id: uuid_v4_simple(),
            display_name: "Water Volume".to_string(),
            bounds: FluidAABB::from_center_extents([0.0, 0.0, 0.0], [10.0, 5.0, 10.0]),
            active: true,
            layer_mask: 1,
            tags: vec!["water".to_string()],
        }
    }
}

/// Generate a simple UUID-like ID
fn uuid_v4_simple() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let t = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    format!("fluid-{:x}", t)
}

// =============================================================================
// PERFORMANCE PROFILING
// =============================================================================

/// Performance metrics for editor display
#[derive(Clone, Debug, Default)]
pub struct FluidPerformanceMetrics {
    /// Time spent on physics simulation (ms)
    pub physics_time_ms: f32,
    /// Time spent on rendering (ms)
    pub render_time_ms: f32,
    /// Time spent on particle spawning (ms)
    pub spawn_time_ms: f32,
    /// Active particle count
    pub active_particles: u32,
    /// GPU memory usage (bytes)
    pub gpu_memory_bytes: u64,
    /// Current FPS impact estimate
    pub fps_impact: f32,
    /// Frame number when captured
    pub frame_number: u64,
}

impl FluidPerformanceMetrics {
    /// Total frame time for fluids
    pub fn total_time_ms(&self) -> f32 {
        self.physics_time_ms + self.render_time_ms + self.spawn_time_ms
    }
    
    /// Format as human-readable string
    pub fn summary(&self) -> String {
        format!(
            "Particles: {} | Physics: {:.2}ms | Render: {:.2}ms | GPU: {}MB",
            self.active_particles,
            self.physics_time_ms,
            self.render_time_ms,
            self.gpu_memory_bytes / (1024 * 1024)
        )
    }
    
    /// Check if performance is within budget (16.67ms = 60 FPS)
    pub fn is_within_budget(&self, budget_ms: f32) -> bool {
        self.total_time_ms() <= budget_ms
    }
    
    /// Get performance grade (A-F)
    pub fn grade(&self) -> char {
        match self.total_time_ms() {
            t if t < 2.0 => 'A',
            t if t < 4.0 => 'B',
            t if t < 8.0 => 'C',
            t if t < 12.0 => 'D',
            _ => 'F',
        }
    }
}

// =============================================================================
// ACCESSIBILITY FEATURES
// =============================================================================

/// Colorblind-safe color palettes for fluid visualization
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ColorblindPalette {
    /// Standard colors (default)
    #[default]
    Standard,
    /// Deuteranopia-safe (red-green colorblind)
    Deuteranopia,
    /// Protanopia-safe (red colorblind)
    Protanopia,
    /// Tritanopia-safe (blue-yellow colorblind)
    Tritanopia,
    /// High contrast for low vision
    HighContrast,
}

impl ColorblindPalette {
    /// Get water color for this palette
    pub fn water_color(&self) -> [f32; 4] {
        match self {
            Self::Standard => [0.2, 0.5, 0.8, 0.9],
            Self::Deuteranopia => [0.1, 0.3, 0.9, 0.9],
            Self::Protanopia => [0.1, 0.4, 0.9, 0.9],
            Self::Tritanopia => [0.0, 0.6, 0.6, 0.9],
            Self::HighContrast => [0.0, 0.0, 1.0, 1.0],
        }
    }
    
    /// Get foam color for this palette
    pub fn foam_color(&self) -> [f32; 3] {
        match self {
            Self::Standard => [0.95, 0.98, 1.0],
            Self::Deuteranopia => [1.0, 1.0, 0.8],
            Self::Protanopia => [1.0, 1.0, 0.7],
            Self::Tritanopia => [0.9, 0.9, 0.9],
            Self::HighContrast => [1.0, 1.0, 1.0],
        }
    }
    
    /// Get description for UI
    pub fn description(&self) -> &'static str {
        match self {
            Self::Standard => "Standard - Default color palette",
            Self::Deuteranopia => "Deuteranopia - Red-green colorblind safe",
            Self::Protanopia => "Protanopia - Red colorblind safe",
            Self::Tritanopia => "Tritanopia - Blue-yellow colorblind safe",
            Self::HighContrast => "High Contrast - Enhanced visibility",
        }
    }
    
    /// Get all palettes for UI dropdown
    pub fn all_palettes() -> &'static [ColorblindPalette] {
        &[
            Self::Standard,
            Self::Deuteranopia,
            Self::Protanopia,
            Self::Tritanopia,
            Self::HighContrast,
        ]
    }
}

/// Accessibility settings for the editor
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AccessibilitySettings {
    /// Color palette for colorblind users
    pub palette: ColorblindPalette,
    /// Show numerical values on sliders
    pub show_slider_values: bool,
    /// Use larger fonts
    pub large_fonts: bool,
    /// Reduce motion/animations
    pub reduce_motion: bool,
    /// Screen reader hints
    pub screen_reader_enabled: bool,
}

impl Default for AccessibilitySettings {
    fn default() -> Self {
        Self {
            palette: ColorblindPalette::Standard,
            show_slider_values: true,
            large_fonts: false,
            reduce_motion: false,
            screen_reader_enabled: false,
        }
    }
}

// =============================================================================
// REAL-TIME PREVIEW HINTS
// =============================================================================

/// Preview hint for editor UI
#[derive(Clone, Debug)]
pub struct PreviewHint {
    /// Parameter being modified
    pub parameter: String,
    /// Current value description
    pub current_value: String,
    /// Effect description
    pub effect_description: String,
    /// Performance impact (Low/Medium/High)
    pub performance_impact: String,
    /// Suggested range
    pub suggested_range: Option<String>,
}

impl PreviewHint {
    /// Generate hint for wave amplitude change
    pub fn wave_amplitude(value: f32) -> Self {
        Self {
            parameter: "Wave Amplitude".to_string(),
            current_value: format!("{:.2}m", value),
            effect_description: if value < 0.2 {
                "Calm water with minimal surface movement".to_string()
            } else if value < 1.0 {
                "Moderate waves with gentle motion".to_string()
            } else {
                "Large waves with dramatic splashing".to_string()
            },
            performance_impact: "Low".to_string(),
            suggested_range: Some("0.1 - 2.0 for realistic water".to_string()),
        }
    }
    
    /// Generate hint for particle count change
    pub fn particle_count(value: u32) -> Self {
        let impact = if value < 10_000 {
            "Low"
        } else if value < 50_000 {
            "Medium"
        } else {
            "High"
        };
        
        Self {
            parameter: "Particle Count".to_string(),
            current_value: format!("{}", value),
            effect_description: format!(
                "{}. Higher counts provide more detail but cost more.",
                if value < 10_000 {
                    "Low detail, suitable for background water"
                } else if value < 30_000 {
                    "Medium detail, good for general gameplay"
                } else if value < 60_000 {
                    "High detail, ideal for close-up scenes"
                } else {
                    "Ultra detail, cinematic quality"
                }
            ),
            performance_impact: impact.to_string(),
            suggested_range: Some("10,000 - 50,000 for most games".to_string()),
        }
    }
    
    /// Generate hint for viscosity change
    pub fn viscosity(value: f32) -> Self {
        Self {
            parameter: "Viscosity".to_string(),
            current_value: format!("{:.1}", value),
            effect_description: if value < 5.0 {
                "Thin, runny fluid like water or alcohol".to_string()
            } else if value < 20.0 {
                "Medium thickness, like oil".to_string()
            } else if value < 50.0 {
                "Thick fluid like honey or syrup".to_string()
            } else {
                "Very thick, almost gel-like consistency".to_string()
            },
            performance_impact: "Low".to_string(),
            suggested_range: Some("1-10 water, 10-30 oil, 30-100 honey".to_string()),
        }
    }
}

// =============================================================================
// BATCH OPERATIONS
// =============================================================================

/// Batch operation for modifying multiple configs
#[derive(Clone, Debug)]
pub struct BatchOperation {
    /// Affected config indices
    pub indices: Vec<usize>,
    /// Operation name
    pub name: String,
}

impl BatchOperation {
    /// Create batch operation
    pub fn new(name: &str, indices: Vec<usize>) -> Self {
        Self {
            name: name.to_string(),
            indices,
        }
    }
    
    /// Apply scalar multiply to physics viscosity
    pub fn multiply_viscosity(configs: &mut [FluidEditorConfig], indices: &[usize], factor: f32) {
        for &i in indices {
            if let Some(config) = configs.get_mut(i) {
                config.physics.viscosity *= factor;
                config.physics.clamp();
            }
        }
    }
    
    /// Enable/disable caustics for multiple configs
    pub fn set_caustics(configs: &mut [FluidEditorConfig], indices: &[usize], enabled: bool) {
        for &i in indices {
            if let Some(config) = configs.get_mut(i) {
                config.caustics.enabled = enabled;
            }
        }
    }
    
    /// Set quality preset for multiple configs
    pub fn set_quality(configs: &mut [FluidEditorConfig], indices: &[usize], quality: QualityPreset) {
        for &i in indices {
            if let Some(config) = configs.get_mut(i) {
                config.quality = quality;
                config.max_particles = quality.recommended_particles();
            }
        }
    }
    
    /// Apply color palette for accessibility
    pub fn apply_palette(configs: &mut [FluidEditorConfig], indices: &[usize], palette: ColorblindPalette) {
        for &i in indices {
            if let Some(config) = configs.get_mut(i) {
                config.rendering.fluid_color = palette.water_color();
                config.foam.color = palette.foam_color();
            }
        }
    }
    
    /// Reset physics settings to preset defaults
    pub fn reset_physics(configs: &mut [FluidEditorConfig], indices: &[usize]) {
        for &i in indices {
            if let Some(config) = configs.get_mut(i) {
                let preset_config = FluidEditorConfig::from_preset(config.preset);
                config.physics = preset_config.physics;
            }
        }
    }
    
    /// Reset visual effects to preset defaults
    pub fn reset_visuals(configs: &mut [FluidEditorConfig], indices: &[usize]) {
        for &i in indices {
            if let Some(config) = configs.get_mut(i) {
                let preset_config = FluidEditorConfig::from_preset(config.preset);
                config.caustics = preset_config.caustics;
                config.god_rays = preset_config.god_rays;
                config.foam = preset_config.foam;
                config.reflections = preset_config.reflections;
                config.underwater = preset_config.underwater;
            }
        }
    }
}

// =============================================================================
// VALIDATION SYSTEM
// =============================================================================

/// Validation severity level
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ValidationSeverity {
    /// Informational message
    Info,
    /// Warning - suboptimal but functional
    Warning,
    /// Error - will cause issues
    Error,
}

/// Single validation issue
#[derive(Clone, Debug)]
pub struct ValidationIssue {
    /// Severity level
    pub severity: ValidationSeverity,
    /// Category (e.g., "Physics", "Performance")
    pub category: &'static str,
    /// Field name (if applicable)
    pub field: Option<&'static str>,
    /// Issue description
    pub message: String,
    /// Suggested fix
    pub suggestion: Option<String>,
}

/// Configuration validator
#[derive(Clone, Debug, Default)]
pub struct ConfigValidator {
    /// Target frame rate for performance checks
    pub target_fps: f32,
    /// Maximum allowed GPU memory (bytes)
    pub max_gpu_memory: u64,
    /// Enable strict mode (more warnings)
    pub strict_mode: bool,
}

impl ConfigValidator {
    /// Create validator with default settings (60 FPS, 512MB GPU limit)
    pub fn new() -> Self {
        Self {
            target_fps: 60.0,
            max_gpu_memory: 512 * 1024 * 1024,
            strict_mode: false,
        }
    }
    
    /// Create strict validator for production
    pub fn strict() -> Self {
        Self {
            target_fps: 60.0,
            max_gpu_memory: 256 * 1024 * 1024,
            strict_mode: true,
        }
    }
    
    /// Validate a configuration, returns list of issues
    pub fn validate(&self, config: &FluidEditorConfig) -> Vec<ValidationIssue> {
        let mut issues = Vec::new();
        
        // Physics validation
        self.validate_physics(config, &mut issues);
        
        // Performance validation
        self.validate_performance(config, &mut issues);
        
        // Visual effects validation
        self.validate_visuals(config, &mut issues);
        
        // Compatibility validation
        self.validate_compatibility(config, &mut issues);
        
        issues
    }
    
    fn validate_physics(&self, config: &FluidEditorConfig, issues: &mut Vec<ValidationIssue>) {
        // Check smoothing radius vs particle spacing
        let particle_spacing = 0.1; // Approximate
        if config.physics.smoothing_radius < particle_spacing * 2.0 {
            issues.push(ValidationIssue {
                severity: ValidationSeverity::Warning,
                category: "Physics",
                field: Some("smoothing_radius"),
                message: "Smoothing radius is very small relative to particle spacing".to_string(),
                suggestion: Some(format!(
                    "Consider increasing to at least {:.2}",
                    particle_spacing * 2.5
                )),
            });
        }
        
        // Check viscosity bounds
        if config.physics.viscosity > 50.0 && self.strict_mode {
            issues.push(ValidationIssue {
                severity: ValidationSeverity::Info,
                category: "Physics",
                field: Some("viscosity"),
                message: "High viscosity values may cause slow simulation".to_string(),
                suggestion: Some("For water, use values between 1-10".to_string()),
            });
        }
        
        // Check gravity magnitude
        let gravity_mag = (config.physics.gravity[0].powi(2) 
            + config.physics.gravity[1].powi(2) 
            + config.physics.gravity[2].powi(2)).sqrt();
        if gravity_mag > 30.0 {
            issues.push(ValidationIssue {
                severity: ValidationSeverity::Warning,
                category: "Physics",
                field: Some("gravity"),
                message: "Gravity magnitude is unusually high".to_string(),
                suggestion: Some("Earth gravity is approximately 9.81 m/s²".to_string()),
            });
        }
        
        // Check iterations
        if config.physics.iterations < 2 {
            issues.push(ValidationIssue {
                severity: ValidationSeverity::Warning,
                category: "Physics",
                field: Some("iterations"),
                message: "Very few solver iterations may cause instability".to_string(),
                suggestion: Some("Use at least 3-5 iterations for stable simulation".to_string()),
            });
        }
    }
    
    fn validate_performance(&self, config: &FluidEditorConfig, issues: &mut Vec<ValidationIssue>) {
        let frame_budget_ms = 1000.0 / self.target_fps;
        
        // Estimate GPU time based on settings
        let estimated_time = self.estimate_frame_time(config);
        
        if estimated_time > frame_budget_ms * 0.5 {
            issues.push(ValidationIssue {
                severity: if estimated_time > frame_budget_ms * 0.8 {
                    ValidationSeverity::Error
                } else {
                    ValidationSeverity::Warning
                },
                category: "Performance",
                field: None,
                message: format!(
                    "Estimated fluid cost is {:.1}ms ({:.0}% of {:.0} FPS budget)",
                    estimated_time,
                    (estimated_time / frame_budget_ms) * 100.0,
                    self.target_fps
                ),
                suggestion: Some("Consider reducing particle count or disabling effects".to_string()),
            });
        }
        
        // Check particle count
        if config.max_particles > 100_000 {
            issues.push(ValidationIssue {
                severity: ValidationSeverity::Warning,
                category: "Performance",
                field: Some("max_particles"),
                message: "Very high particle count may impact performance".to_string(),
                suggestion: Some("Use 20,000-50,000 for most games".to_string()),
            });
        }
        
        // Check for expensive effect combinations
        let expensive_effects = [
            config.caustics.enabled,
            config.god_rays.enabled,
            config.reflections.enabled && config.reflections.resolution > 512,
        ];
        let enabled_count = expensive_effects.iter().filter(|&&e| e).count();
        
        if enabled_count >= 3 && self.strict_mode {
            issues.push(ValidationIssue {
                severity: ValidationSeverity::Warning,
                category: "Performance",
                field: None,
                message: "Multiple expensive effects enabled simultaneously".to_string(),
                suggestion: Some("Consider disabling some effects for better performance".to_string()),
            });
        }
    }
    
    fn validate_visuals(&self, config: &FluidEditorConfig, issues: &mut Vec<ValidationIssue>) {
        // Check for zero-alpha water color
        if config.rendering.fluid_color[3] < 0.01 {
            issues.push(ValidationIssue {
                severity: ValidationSeverity::Warning,
                category: "Rendering",
                field: Some("fluid_color"),
                message: "Water opacity is nearly zero - water will be invisible".to_string(),
                suggestion: Some("Increase alpha to at least 0.3 for visible water".to_string()),
            });
        }
        
        // Check foam without waves
        if config.foam.enabled && config.waves.amplitude < 0.05 {
            issues.push(ValidationIssue {
                severity: ValidationSeverity::Info,
                category: "Visual Effects",
                field: Some("foam"),
                message: "Foam enabled but waves are very small".to_string(),
                suggestion: Some("Foam works best with visible wave motion".to_string()),
            });
        }
        
        // Check god rays without caustics (usually paired)
        if config.god_rays.enabled && !config.caustics.enabled && self.strict_mode {
            issues.push(ValidationIssue {
                severity: ValidationSeverity::Info,
                category: "Visual Effects",
                field: None,
                message: "God rays often look better with caustics enabled".to_string(),
                suggestion: None,
            });
        }
    }
    
    fn validate_compatibility(&self, config: &FluidEditorConfig, issues: &mut Vec<ValidationIssue>) {
        // Check for settings that might not work on lower-end hardware
        if config.quality == QualityPreset::Ultra {
            issues.push(ValidationIssue {
                severity: ValidationSeverity::Info,
                category: "Compatibility",
                field: Some("quality"),
                message: "Ultra quality may not run well on all hardware".to_string(),
                suggestion: Some("Consider using High for wider compatibility".to_string()),
            });
        }
        
        // Check reflection resolution
        if config.reflections.resolution > 1024 {
            issues.push(ValidationIssue {
                severity: ValidationSeverity::Info,
                category: "Compatibility",
                field: Some("reflections.resolution"),
                message: "Very high reflection resolution uses significant VRAM".to_string(),
                suggestion: None,
            });
        }
    }
    
    /// Estimate frame time in milliseconds
    fn estimate_frame_time(&self, config: &FluidEditorConfig) -> f32 {
        let mut time = 0.0;
        
        // Base physics cost (~0.0001ms per particle, optimized GPU)
        time += config.max_particles as f32 * 0.0001;
        
        // Effects cost
        if config.caustics.enabled {
            time += 0.5;
        }
        if config.god_rays.enabled {
            time += 0.8 + (config.god_rays.samples as f32 * 0.02);
        }
        if config.foam.enabled {
            time += 0.3;
        }
        if config.reflections.enabled {
            time += (config.reflections.resolution as f32 / 512.0) * 0.5;
        }
        
        // Iteration cost
        time += config.physics.iterations as f32 * 0.1;
        
        time
    }
    
    /// Check if config has any errors
    pub fn has_errors(&self, config: &FluidEditorConfig) -> bool {
        self.validate(config)
            .iter()
            .any(|i| i.severity == ValidationSeverity::Error)
    }
    
    /// Check if config has any warnings or errors
    pub fn has_warnings(&self, config: &FluidEditorConfig) -> bool {
        self.validate(config)
            .iter()
            .any(|i| i.severity == ValidationSeverity::Warning || i.severity == ValidationSeverity::Error)
    }
}

// =============================================================================
// ANIMATION EASING
// =============================================================================

/// Easing function type for animations
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum EasingFunction {
    /// Linear interpolation (no easing)
    #[default]
    Linear,
    /// Smooth start (ease in)
    EaseIn,
    /// Smooth end (ease out)
    EaseOut,
    /// Smooth start and end
    EaseInOut,
    /// Bouncy overshoot
    EaseOutBack,
    /// Elastic bounce
    EaseOutElastic,
    /// Quick start, slow end
    EaseOutQuad,
    /// Slow start, quick end
    EaseInQuad,
}

impl EasingFunction {
    /// Apply easing to t (0.0 to 1.0)
    pub fn apply(&self, t: f32) -> f32 {
        let t = t.clamp(0.0, 1.0);
        match self {
            Self::Linear => t,
            Self::EaseIn => t * t * t,
            Self::EaseOut => 1.0 - (1.0 - t).powi(3),
            Self::EaseInOut => {
                if t < 0.5 {
                    4.0 * t * t * t
                } else {
                    1.0 - (-2.0 * t + 2.0).powi(3) / 2.0
                }
            }
            Self::EaseOutBack => {
                let c1 = 1.70158;
                let c3 = c1 + 1.0;
                1.0 + c3 * (t - 1.0).powi(3) + c1 * (t - 1.0).powi(2)
            }
            Self::EaseOutElastic => {
                if t == 0.0 || t == 1.0 {
                    t
                } else {
                    let c4 = (2.0 * std::f32::consts::PI) / 3.0;
                    2.0_f32.powf(-10.0 * t) * ((t * 10.0 - 0.75) * c4).sin() + 1.0
                }
            }
            Self::EaseOutQuad => 1.0 - (1.0 - t) * (1.0 - t),
            Self::EaseInQuad => t * t,
        }
    }
    
    /// Get all easing functions for UI dropdown
    pub fn all_functions() -> &'static [EasingFunction] {
        &[
            Self::Linear,
            Self::EaseIn,
            Self::EaseOut,
            Self::EaseInOut,
            Self::EaseOutBack,
            Self::EaseOutElastic,
            Self::EaseOutQuad,
            Self::EaseInQuad,
        ]
    }
    
    /// Get description for UI
    pub fn description(&self) -> &'static str {
        match self {
            Self::Linear => "Linear - Constant speed",
            Self::EaseIn => "Ease In - Slow start, fast end",
            Self::EaseOut => "Ease Out - Fast start, slow end",
            Self::EaseInOut => "Ease In/Out - Smooth acceleration and deceleration",
            Self::EaseOutBack => "Ease Out Back - Overshoot then settle",
            Self::EaseOutElastic => "Elastic - Bouncy spring effect",
            Self::EaseOutQuad => "Ease Out Quad - Quadratic deceleration",
            Self::EaseInQuad => "Ease In Quad - Quadratic acceleration",
        }
    }
}

/// Animated transition between configurations
#[derive(Clone, Debug)]
pub struct ConfigTransition {
    /// Starting configuration
    pub from: FluidEditorConfig,
    /// Target configuration
    pub to: FluidEditorConfig,
    /// Transition duration in seconds
    pub duration: f32,
    /// Elapsed time
    pub elapsed: f32,
    /// Easing function
    pub easing: EasingFunction,
    /// Is transition complete?
    pub complete: bool,
}

impl ConfigTransition {
    /// Create new transition
    pub fn new(from: FluidEditorConfig, to: FluidEditorConfig, duration: f32) -> Self {
        Self {
            from,
            to,
            duration: duration.max(0.01),
            elapsed: 0.0,
            easing: EasingFunction::EaseInOut,
            complete: false,
        }
    }
    
    /// Create transition with custom easing
    pub fn with_easing(mut self, easing: EasingFunction) -> Self {
        self.easing = easing;
        self
    }
    
    /// Update transition, returns current interpolated config
    pub fn update(&mut self, delta_time: f32) -> FluidEditorConfig {
        self.elapsed += delta_time;
        
        if self.elapsed >= self.duration {
            self.elapsed = self.duration;
            self.complete = true;
            return self.to.clone();
        }
        
        let t = self.elapsed / self.duration;
        let eased_t = self.easing.apply(t);
        
        self.from.interpolate(&self.to, eased_t)
    }
    
    /// Get progress (0.0 to 1.0)
    pub fn progress(&self) -> f32 {
        (self.elapsed / self.duration).clamp(0.0, 1.0)
    }
    
    /// Skip to end
    pub fn skip(&mut self) {
        self.elapsed = self.duration;
        self.complete = true;
    }
}

// =============================================================================
// DEBUG VISUALIZATION
// =============================================================================

/// Debug visualization options for development
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DebugVisualization {
    /// Show particle positions as points
    pub show_particles: bool,
    /// Show particle velocities as arrows
    pub show_velocities: bool,
    /// Show fluid surface normals
    pub show_normals: bool,
    /// Show pressure field as color gradient
    pub show_pressure: bool,
    /// Show density field
    pub show_density: bool,
    /// Show spatial hash grid
    pub show_grid: bool,
    /// Show bounding boxes
    pub show_bounds: bool,
    /// Show emitter/drain gizmos
    pub show_emitters: bool,
    /// Show performance overlay
    pub show_performance: bool,
    /// Velocity arrow scale
    pub velocity_scale: f32,
    /// Particle point size
    pub particle_size: f32,
    /// Grid line opacity
    pub grid_opacity: f32,
}

impl Default for DebugVisualization {
    fn default() -> Self {
        Self {
            show_particles: false,
            show_velocities: false,
            show_normals: false,
            show_pressure: false,
            show_density: false,
            show_grid: false,
            show_bounds: true,
            show_emitters: true,
            show_performance: false,
            velocity_scale: 0.1,
            particle_size: 3.0,
            grid_opacity: 0.3,
        }
    }
}

impl DebugVisualization {
    /// All visualizations off (for release builds)
    pub fn none() -> Self {
        Self {
            show_particles: false,
            show_velocities: false,
            show_normals: false,
            show_pressure: false,
            show_density: false,
            show_grid: false,
            show_bounds: false,
            show_emitters: false,
            show_performance: false,
            velocity_scale: 0.1,
            particle_size: 3.0,
            grid_opacity: 0.3,
        }
    }
    
    /// Show all physics-related visualizations
    pub fn physics() -> Self {
        Self {
            show_particles: true,
            show_velocities: true,
            show_normals: false,
            show_pressure: true,
            show_density: false,
            show_grid: true,
            show_bounds: true,
            show_emitters: true,
            show_performance: true,
            velocity_scale: 0.2,
            particle_size: 5.0,
            grid_opacity: 0.4,
        }
    }
    
    /// Show rendering-related visualizations
    pub fn rendering() -> Self {
        Self {
            show_particles: false,
            show_velocities: false,
            show_normals: true,
            show_pressure: false,
            show_density: false,
            show_grid: false,
            show_bounds: true,
            show_emitters: false,
            show_performance: true,
            velocity_scale: 0.1,
            particle_size: 3.0,
            grid_opacity: 0.3,
        }
    }
    
    /// Check if any visualization is enabled
    pub fn any_enabled(&self) -> bool {
        self.show_particles || self.show_velocities || self.show_normals ||
        self.show_pressure || self.show_density || self.show_grid ||
        self.show_bounds || self.show_emitters || self.show_performance
    }
}

// =============================================================================
// KEYBOARD SHORTCUTS
// =============================================================================

/// Keyboard shortcut definition
#[derive(Clone, Debug)]
pub struct KeyboardShortcut {
    /// Primary key (e.g., "Z", "S", "Space")
    pub key: &'static str,
    /// Requires Ctrl/Cmd
    pub ctrl: bool,
    /// Requires Shift
    pub shift: bool,
    /// Requires Alt/Option
    pub alt: bool,
    /// Action description
    pub action: &'static str,
    /// Category for grouping
    pub category: &'static str,
}

impl KeyboardShortcut {
    /// Format as display string (e.g., "Ctrl+Z")
    pub fn display(&self) -> String {
        let mut parts = Vec::new();
        if self.ctrl { parts.push("Ctrl"); }
        if self.shift { parts.push("Shift"); }
        if self.alt { parts.push("Alt"); }
        parts.push(self.key);
        parts.join("+")
    }
}

/// Get all editor keyboard shortcuts
pub fn editor_shortcuts() -> Vec<KeyboardShortcut> {
    vec![
        // Edit operations
        KeyboardShortcut {
            key: "Z", ctrl: true, shift: false, alt: false,
            action: "Undo", category: "Edit",
        },
        KeyboardShortcut {
            key: "Y", ctrl: true, shift: false, alt: false,
            action: "Redo", category: "Edit",
        },
        KeyboardShortcut {
            key: "Z", ctrl: true, shift: true, alt: false,
            action: "Redo (alternate)", category: "Edit",
        },
        KeyboardShortcut {
            key: "C", ctrl: true, shift: false, alt: false,
            action: "Copy configuration", category: "Edit",
        },
        KeyboardShortcut {
            key: "V", ctrl: true, shift: false, alt: false,
            action: "Paste configuration", category: "Edit",
        },
        KeyboardShortcut {
            key: "D", ctrl: true, shift: false, alt: false,
            action: "Duplicate selected", category: "Edit",
        },
        KeyboardShortcut {
            key: "Delete", ctrl: false, shift: false, alt: false,
            action: "Delete selected", category: "Edit",
        },
        
        // View operations
        KeyboardShortcut {
            key: "F", ctrl: false, shift: false, alt: false,
            action: "Frame selection", category: "View",
        },
        KeyboardShortcut {
            key: "G", ctrl: false, shift: false, alt: false,
            action: "Toggle grid", category: "View",
        },
        KeyboardShortcut {
            key: "P", ctrl: false, shift: false, alt: false,
            action: "Toggle particles view", category: "View",
        },
        KeyboardShortcut {
            key: "B", ctrl: false, shift: false, alt: false,
            action: "Toggle bounds", category: "View",
        },
        
        // Simulation
        KeyboardShortcut {
            key: "Space", ctrl: false, shift: false, alt: false,
            action: "Play/Pause simulation", category: "Simulation",
        },
        KeyboardShortcut {
            key: ".", ctrl: false, shift: false, alt: false,
            action: "Step forward one frame", category: "Simulation",
        },
        KeyboardShortcut {
            key: "R", ctrl: false, shift: false, alt: false,
            action: "Reset simulation", category: "Simulation",
        },
        
        // Presets
        KeyboardShortcut {
            key: "1", ctrl: false, shift: false, alt: false,
            action: "Apply Low quality preset", category: "Presets",
        },
        KeyboardShortcut {
            key: "2", ctrl: false, shift: false, alt: false,
            action: "Apply Medium quality preset", category: "Presets",
        },
        KeyboardShortcut {
            key: "3", ctrl: false, shift: false, alt: false,
            action: "Apply High quality preset", category: "Presets",
        },
        KeyboardShortcut {
            key: "4", ctrl: false, shift: false, alt: false,
            action: "Apply Ultra quality preset", category: "Presets",
        },
        
        // File
        KeyboardShortcut {
            key: "S", ctrl: true, shift: false, alt: false,
            action: "Save configuration", category: "File",
        },
        KeyboardShortcut {
            key: "O", ctrl: true, shift: false, alt: false,
            action: "Open configuration", category: "File",
        },
        KeyboardShortcut {
            key: "E", ctrl: true, shift: false, alt: false,
            action: "Export preset", category: "File",
        },
    ]
}

// =============================================================================
// PRESET EXPORT/IMPORT
// =============================================================================

/// Exported preset format (for sharing between projects)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExportedPreset {
    /// Preset version (for migration)
    pub version: u32,
    /// Preset name
    pub name: String,
    /// Author/creator
    pub author: Option<String>,
    /// Description
    pub description: Option<String>,
    /// Creation timestamp (Unix epoch seconds)
    pub created_at: u64,
    /// Tags for categorization
    pub tags: Vec<String>,
    /// The configuration data
    pub config: FluidEditorConfig,
    /// Thumbnail image (base64 encoded PNG, optional)
    pub thumbnail: Option<String>,
}

impl ExportedPreset {
    /// Current export format version
    pub const CURRENT_VERSION: u32 = 1;
    
    /// Create export from config
    pub fn from_config(name: &str, config: FluidEditorConfig) -> Self {
        use std::time::{SystemTime, UNIX_EPOCH};
        
        let created_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        
        Self {
            version: Self::CURRENT_VERSION,
            name: name.to_string(),
            author: None,
            description: None,
            created_at,
            tags: Vec::new(),
            config,
            thumbnail: None,
        }
    }
    
    /// Set author
    pub fn with_author(mut self, author: &str) -> Self {
        self.author = Some(author.to_string());
        self
    }
    
    /// Set description
    pub fn with_description(mut self, desc: &str) -> Self {
        self.description = Some(desc.to_string());
        self
    }
    
    /// Add tags
    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }
    
    /// Export to JSON string
    pub fn to_json(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize preset: {}", e))
    }
    
    /// Import from JSON string
    pub fn from_json(json: &str) -> Result<Self, String> {
        serde_json::from_str(json)
            .map_err(|e| format!("Failed to parse preset: {}", e))
    }
    
    /// Export to TOML string
    pub fn to_toml(&self) -> Result<String, String> {
        toml::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize preset: {}", e))
    }
    
    /// Import from TOML string  
    pub fn from_toml(toml_str: &str) -> Result<Self, String> {
        toml::from_str(toml_str)
            .map_err(|e| format!("Failed to parse preset: {}", e))
    }
}

// =============================================================================
// CLIPBOARD SUPPORT
// =============================================================================

/// Configuration clipboard for copy/paste operations
#[derive(Clone, Debug, Default)]
pub struct ConfigClipboard {
    /// Copied configuration (if any)
    config: Option<FluidEditorConfig>,
    /// Copy timestamp
    copied_at: Option<u64>,
}

impl ConfigClipboard {
    /// Create new empty clipboard
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Copy configuration to clipboard
    pub fn copy(&mut self, config: &FluidEditorConfig) {
        use std::time::{SystemTime, UNIX_EPOCH};
        
        self.config = Some(config.clone());
        self.copied_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .ok();
    }
    
    /// Paste configuration from clipboard
    pub fn paste(&self) -> Option<FluidEditorConfig> {
        self.config.clone()
    }
    
    /// Check if clipboard has content
    pub fn has_content(&self) -> bool {
        self.config.is_some()
    }
    
    /// Clear clipboard
    pub fn clear(&mut self) {
        self.config = None;
        self.copied_at = None;
    }
    
    /// Copy to system clipboard as JSON (returns JSON string)
    pub fn to_system_clipboard(&self) -> Option<String> {
        self.config.as_ref().and_then(|c| serde_json::to_string(c).ok())
    }
    
    /// Paste from system clipboard JSON
    pub fn from_system_clipboard(json: &str) -> Result<FluidEditorConfig, String> {
        serde_json::from_str(json)
            .map_err(|e| format!("Invalid configuration: {}", e))
    }
}

// =============================================================================
// EDITOR WIDGET METADATA
// =============================================================================

/// Widget type hint for UI generation
#[derive(Clone, Debug, PartialEq)]
pub enum WidgetType {
    /// Slider with min, max, and step
    Slider { min: f32, max: f32, step: f32 },
    /// Integer slider
    IntSlider { min: i32, max: i32 },
    /// Checkbox / toggle
    Toggle,
    /// Color picker (RGB)
    ColorRgb,
    /// Color picker (RGBA)
    ColorRgba,
    /// Vector3 input
    Vector3,
    /// Dropdown with options
    Dropdown(Vec<String>),
    /// Text input
    TextInput,
    /// Button
    Button,
}

/// Metadata for a single editor field
#[derive(Clone, Debug)]
pub struct FieldMetadata {
    /// Field name (for display)
    pub name: &'static str,
    /// Tooltip description
    pub tooltip: &'static str,
    /// Category for grouping in inspector
    pub category: &'static str,
    /// Widget type hint
    pub widget: WidgetType,
    /// Advanced mode only (hidden in simple mode)
    pub advanced: bool,
}

/// Metadata for all editor fields
pub struct EditorMetadata;

impl EditorMetadata {
    // === Physics Fields ===
    
    /// Get metadata for smoothing radius
    pub fn smoothing_radius() -> FieldMetadata {
        FieldMetadata {
            name: "Smoothing Radius",
            tooltip: "Controls the radius of influence for each particle. \
                     Larger values create smoother but less detailed fluid.",
            category: "Physics",
            widget: WidgetType::Slider { min: 0.5, max: 5.0, step: 0.1 },
            advanced: false,
        }
    }
    
    /// Get metadata for target density
    pub fn target_density() -> FieldMetadata {
        FieldMetadata {
            name: "Target Density",
            tooltip: "The rest density the fluid tries to maintain. \
                     Higher values create heavier, denser fluid.",
            category: "Physics",
            widget: WidgetType::Slider { min: 1.0, max: 50.0, step: 0.5 },
            advanced: true,
        }
    }
    
    /// Get metadata for pressure multiplier
    pub fn pressure_multiplier() -> FieldMetadata {
        FieldMetadata {
            name: "Pressure",
            tooltip: "Controls how strongly particles repel each other. \
                     Higher values make the fluid more incompressible.",
            category: "Physics",
            widget: WidgetType::Slider { min: 10.0, max: 1000.0, step: 10.0 },
            advanced: true,
        }
    }
    
    /// Get metadata for viscosity
    pub fn viscosity() -> FieldMetadata {
        FieldMetadata {
            name: "Viscosity",
            tooltip: "How thick/syrupy the fluid is. Low = water, High = honey/mud.",
            category: "Physics",
            widget: WidgetType::Slider { min: 0.0, max: 100.0, step: 1.0 },
            advanced: false,
        }
    }
    
    /// Get metadata for surface tension
    pub fn surface_tension() -> FieldMetadata {
        FieldMetadata {
            name: "Surface Tension",
            tooltip: "How strongly the fluid surface holds together. \
                     Higher values create more cohesive droplets.",
            category: "Physics",
            widget: WidgetType::Slider { min: 0.0, max: 1.0, step: 0.01 },
            advanced: true,
        }
    }
    
    /// Get metadata for gravity
    pub fn gravity() -> FieldMetadata {
        FieldMetadata {
            name: "Gravity",
            tooltip: "Gravity vector applied to all particles. \
                     Y is typically negative (downward).",
            category: "Physics",
            widget: WidgetType::Vector3,
            advanced: false,
        }
    }
    
    /// Get metadata for iterations
    pub fn iterations() -> FieldMetadata {
        FieldMetadata {
            name: "Solver Iterations",
            tooltip: "Number of pressure solver iterations. \
                     More iterations = more accurate but slower.",
            category: "Physics",
            widget: WidgetType::IntSlider { min: 1, max: 20 },
            advanced: true,
        }
    }
    
    // === Visual Effects Fields ===
    
    /// Get metadata for caustics enabled
    pub fn caustics_enabled() -> FieldMetadata {
        FieldMetadata {
            name: "Enable Caustics",
            tooltip: "Light patterns created by refraction through water surface.",
            category: "Visual Effects",
            widget: WidgetType::Toggle,
            advanced: false,
        }
    }
    
    /// Get metadata for caustic intensity
    pub fn caustic_intensity() -> FieldMetadata {
        FieldMetadata {
            name: "Caustic Intensity",
            tooltip: "Brightness of caustic light patterns.",
            category: "Visual Effects",
            widget: WidgetType::Slider { min: 0.0, max: 5.0, step: 0.1 },
            advanced: false,
        }
    }
    
    /// Get metadata for god rays enabled
    pub fn god_rays_enabled() -> FieldMetadata {
        FieldMetadata {
            name: "Enable God Rays",
            tooltip: "Volumetric light shafts penetrating the water.",
            category: "Visual Effects",
            widget: WidgetType::Toggle,
            advanced: false,
        }
    }
    
    /// Get metadata for foam enabled
    pub fn foam_enabled() -> FieldMetadata {
        FieldMetadata {
            name: "Enable Foam",
            tooltip: "Foam particles at wave crests and along shores.",
            category: "Visual Effects",
            widget: WidgetType::Toggle,
            advanced: false,
        }
    }
    
    /// Get metadata for reflections enabled
    pub fn reflections_enabled() -> FieldMetadata {
        FieldMetadata {
            name: "Enable Reflections",
            tooltip: "Water surface reflections (screen-space or planar).",
            category: "Visual Effects",
            widget: WidgetType::Toggle,
            advanced: false,
        }
    }
    
    // === Rendering Fields ===
    
    /// Get metadata for fluid color
    pub fn fluid_color() -> FieldMetadata {
        FieldMetadata {
            name: "Water Color",
            tooltip: "Base color of the water with transparency.",
            category: "Rendering",
            widget: WidgetType::ColorRgba,
            advanced: false,
        }
    }
    
    /// Get metadata for absorption
    pub fn absorption() -> FieldMetadata {
        FieldMetadata {
            name: "Absorption",
            tooltip: "How quickly each color channel is absorbed (RGB). \
                     Higher values = more tinted at depth.",
            category: "Rendering",
            widget: WidgetType::ColorRgb,
            advanced: true,
        }
    }
    
    /// Get metadata for roughness
    pub fn roughness() -> FieldMetadata {
        FieldMetadata {
            name: "Roughness",
            tooltip: "Surface roughness affecting reflections. \
                     0 = mirror, 1 = diffuse.",
            category: "Rendering",
            widget: WidgetType::Slider { min: 0.0, max: 1.0, step: 0.01 },
            advanced: true,
        }
    }
    
    // === Wave Fields ===
    
    /// Get metadata for wave amplitude
    pub fn wave_amplitude() -> FieldMetadata {
        FieldMetadata {
            name: "Wave Height",
            tooltip: "Maximum wave height in world units.",
            category: "Waves",
            widget: WidgetType::Slider { min: 0.0, max: 10.0, step: 0.1 },
            advanced: false,
        }
    }
    
    /// Get metadata for wave frequency
    pub fn wave_frequency() -> FieldMetadata {
        FieldMetadata {
            name: "Wave Frequency",
            tooltip: "Number of waves per unit distance.",
            category: "Waves",
            widget: WidgetType::Slider { min: 0.1, max: 5.0, step: 0.1 },
            advanced: true,
        }
    }
    
    /// Get metadata for wind strength
    pub fn wind_strength() -> FieldMetadata {
        FieldMetadata {
            name: "Wind Strength",
            tooltip: "Wind speed affecting wave generation.",
            category: "Waves",
            widget: WidgetType::Slider { min: 0.0, max: 30.0, step: 0.5 },
            advanced: false,
        }
    }
    
    // === Performance Fields ===
    
    /// Get metadata for max particles
    pub fn max_particles() -> FieldMetadata {
        FieldMetadata {
            name: "Max Particles",
            tooltip: "Maximum number of fluid particles. \
                     More particles = better detail but slower.",
            category: "Performance",
            widget: WidgetType::IntSlider { min: 1000, max: 500000 },
            advanced: false,
        }
    }
    
    /// Get metadata for LOD enabled
    pub fn lod_enabled() -> FieldMetadata {
        FieldMetadata {
            name: "Enable LOD",
            tooltip: "Level of Detail system - reduces particles at distance.",
            category: "Performance",
            widget: WidgetType::Toggle,
            advanced: false,
        }
    }
    
    /// Get all field metadata grouped by category
    pub fn all_fields() -> Vec<FieldMetadata> {
        vec![
            // Physics
            Self::smoothing_radius(),
            Self::viscosity(),
            Self::gravity(),
            Self::target_density(),
            Self::pressure_multiplier(),
            Self::surface_tension(),
            Self::iterations(),
            // Visual Effects
            Self::caustics_enabled(),
            Self::caustic_intensity(),
            Self::god_rays_enabled(),
            Self::foam_enabled(),
            Self::reflections_enabled(),
            // Rendering
            Self::fluid_color(),
            Self::absorption(),
            Self::roughness(),
            // Waves
            Self::wave_amplitude(),
            Self::wave_frequency(),
            Self::wind_strength(),
            // Performance
            Self::max_particles(),
            Self::lod_enabled(),
        ]
    }
    
    /// Get categories in display order
    pub fn categories() -> Vec<&'static str> {
        vec![
            "Physics",
            "Visual Effects",
            "Rendering",
            "Waves",
            "Performance",
        ]
    }
    
    /// Get fields for a specific category
    pub fn fields_for_category(category: &str) -> Vec<FieldMetadata> {
        Self::all_fields()
            .into_iter()
            .filter(|f| f.category == category)
            .collect()
    }
    
    /// Get only non-advanced fields (for simple mode)
    pub fn simple_fields() -> Vec<FieldMetadata> {
        Self::all_fields()
            .into_iter()
            .filter(|f| !f.advanced)
            .collect()
    }
}

// =============================================================================
// PHYSICS CONFIGURATION
// =============================================================================

/// Physics simulation parameters for the editor
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PhysicsConfig {
    /// Particle smoothing radius (0.5 - 5.0)
    pub smoothing_radius: f32,
    /// Target fluid density (1.0 - 50.0)
    pub target_density: f32,
    /// Pressure multiplier (10.0 - 1000.0)
    pub pressure_multiplier: f32,
    /// Viscosity coefficient (0.0 - 100.0)
    pub viscosity: f32,
    /// Surface tension strength (0.0 - 1.0)
    pub surface_tension: f32,
    /// Gravity vector
    pub gravity: [f32; 3],
    /// Solver iterations (1 - 20)
    pub iterations: u32,
    /// Enable vorticity confinement for swirling effects
    pub enable_vorticity: bool,
    /// Vorticity strength (0.0 - 1.0)
    pub vorticity_strength: f32,
}

impl Default for PhysicsConfig {
    fn default() -> Self {
        Self {
            smoothing_radius: 1.0,
            target_density: 12.0,
            pressure_multiplier: 300.0,
            viscosity: 10.0,
            surface_tension: 0.02,
            gravity: [0.0, -9.8, 0.0],
            iterations: 4,
            enable_vorticity: true,
            vorticity_strength: 0.1,
        }
    }
}

impl PhysicsConfig {
    /// Clamp all values to safe ranges
    pub fn clamp(&mut self) {
        self.smoothing_radius = self.smoothing_radius.clamp(0.5, 5.0);
        self.target_density = self.target_density.clamp(1.0, 50.0);
        self.pressure_multiplier = self.pressure_multiplier.clamp(10.0, 1000.0);
        self.viscosity = self.viscosity.clamp(0.0, 100.0);
        self.surface_tension = self.surface_tension.clamp(0.0, 1.0);
        self.gravity[0] = self.gravity[0].clamp(-30.0, 30.0);
        self.gravity[1] = self.gravity[1].clamp(-30.0, 30.0);
        self.gravity[2] = self.gravity[2].clamp(-30.0, 30.0);
        self.iterations = self.iterations.clamp(1, 20);
        self.vorticity_strength = self.vorticity_strength.clamp(0.0, 1.0);
    }
}

// =============================================================================
// THERMAL CONFIGURATION
// =============================================================================

/// Thermal simulation parameters
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ThermalConfig {
    /// Enable temperature simulation
    pub enabled: bool,
    /// Ambient temperature (Celsius)
    pub ambient_temperature: f32,
    /// Thermal diffusivity (0.0 - 1.0)
    pub diffusivity: f32,
    /// Buoyancy strength (0.0 - 0.01)
    pub buoyancy_coefficient: f32,
    /// Enable steam/evaporation effects
    pub enable_evaporation: bool,
    /// Evaporation threshold temperature (Celsius)
    pub evaporation_temperature: f32,
}

impl Default for ThermalConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            ambient_temperature: 20.0,
            diffusivity: 0.1,
            buoyancy_coefficient: 0.0002,
            enable_evaporation: false,
            evaporation_temperature: 100.0,
        }
    }
}

impl ThermalConfig {
    /// Clamp values to safe ranges
    pub fn clamp(&mut self) {
        self.ambient_temperature = self.ambient_temperature.clamp(-50.0, 150.0);
        self.diffusivity = self.diffusivity.clamp(0.0, 1.0);
        self.buoyancy_coefficient = self.buoyancy_coefficient.clamp(0.0, 0.01);
        self.evaporation_temperature = self.evaporation_temperature.clamp(50.0, 200.0);
    }
}

// =============================================================================
// VISUAL EFFECTS CONFIGURATION
// =============================================================================

/// Caustics effect configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CausticsEditorConfig {
    /// Enable caustics
    pub enabled: bool,
    /// Caustic intensity (0.0 - 5.0)
    pub intensity: f32,
    /// Caustic scale (affects pattern size)
    pub scale: f32,
    /// Animation speed
    pub speed: f32,
    /// Maximum depth for caustics visibility
    pub max_depth: f32,
}

impl Default for CausticsEditorConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            intensity: 1.0,
            scale: 1.0,
            speed: 1.0,
            max_depth: 20.0,
        }
    }
}

/// God rays (volumetric light) configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GodRaysEditorConfig {
    /// Enable god rays
    pub enabled: bool,
    /// Ray intensity (0.0 - 2.0)
    pub intensity: f32,
    /// Number of ray samples (8 - 64)
    pub samples: u32,
    /// Ray decay rate
    pub decay: f32,
    /// Maximum visibility depth
    pub max_depth: f32,
}

impl Default for GodRaysEditorConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            intensity: 0.8,
            samples: 32,
            decay: 0.95,
            max_depth: 50.0,
        }
    }
}

/// Foam generation configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FoamEditorConfig {
    /// Enable foam
    pub enabled: bool,
    /// Maximum foam particles
    pub max_particles: u32,
    /// Foam lifetime (seconds)
    pub lifetime: f32,
    /// Whitecap threshold (0.0 - 1.0)
    pub whitecap_threshold: f32,
    /// Shore foam intensity
    pub shore_intensity: f32,
    /// Wake foam intensity
    pub wake_intensity: f32,
    /// Foam color tint
    pub color: [f32; 3],
}

impl Default for FoamEditorConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_particles: 10000,
            lifetime: 3.0,
            whitecap_threshold: 0.6,
            shore_intensity: 1.5,
            wake_intensity: 1.0,
            color: [0.95, 0.98, 1.0],
        }
    }
}

/// Water reflection configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReflectionEditorConfig {
    /// Enable reflections
    pub enabled: bool,
    /// Reflection intensity (0.0 - 1.0)
    pub intensity: f32,
    /// Fresnel power (controls angle-based reflection)
    pub fresnel_power: f32,
    /// Distortion strength
    pub distortion: f32,
    /// Use planar reflections (more accurate but expensive)
    pub use_planar: bool,
    /// Reflection texture resolution
    pub resolution: u32,
}

impl Default for ReflectionEditorConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            intensity: 0.8,
            fresnel_power: 5.0,
            distortion: 0.03,
            use_planar: false,
            resolution: 512,
        }
    }
}

/// Underwater visual effects configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UnderwaterEditorConfig {
    /// Enable underwater effects
    pub enabled: bool,
    /// Fog color
    pub fog_color: [f32; 3],
    /// Fog density
    pub fog_density: f32,
    /// Enable underwater particles (dust, bubbles)
    pub enable_particles: bool,
    /// Particle density
    pub particle_density: f32,
    /// Distortion strength
    pub distortion: f32,
}

impl Default for UnderwaterEditorConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            fog_color: [0.0, 0.1, 0.2],
            fog_density: 0.02,
            enable_particles: true,
            particle_density: 100.0,
            distortion: 0.02,
        }
    }
}

// =============================================================================
// WATERFALL CONFIGURATION
// =============================================================================

/// Waterfall effect configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WaterfallEditorConfig {
    /// Enable waterfall effects
    pub enabled: bool,
    /// Maximum spray particles
    pub max_particles: u32,
    /// Spawn rate (particles per second per meter)
    pub spawn_rate: f32,
    /// Mist density (0.0 - 1.0)
    pub mist_density: f32,
    /// Mist rise speed
    pub mist_rise_speed: f32,
    /// Spray spread angle (radians)
    pub spray_angle: f32,
    /// Splash intensity at impact
    pub splash_intensity: f32,
}

impl Default for WaterfallEditorConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_particles: 5000,
            spawn_rate: 200.0,
            mist_density: 0.5,
            mist_rise_speed: 0.8,
            spray_angle: 0.3,
            splash_intensity: 1.0,
        }
    }
}

// =============================================================================
// EMITTER CONFIGURATION
// =============================================================================

/// Emitter shape type for the editor
#[derive(Clone, Debug, Serialize, Deserialize, Default, PartialEq)]
pub enum EmitterShapeType {
    /// Single point emitter
    #[default]
    Point,
    /// Spherical volume
    Sphere,
    /// Box volume
    Box,
    /// Cylinder volume
    Cylinder,
    /// Mesh surface (requires mesh asset)
    Mesh,
}

/// Fluid emitter configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EmitterEditorConfig {
    /// Emitter name (for identification)
    pub name: String,
    /// Enable this emitter
    pub enabled: bool,
    /// Position in world space
    pub position: [f32; 3],
    /// Rotation (Euler angles in degrees)
    pub rotation: [f32; 3],
    /// Emitter shape type
    pub shape: EmitterShapeType,
    /// Shape size (radius for sphere, half-extents for box)
    pub size: [f32; 3],
    /// Emission rate (particles per second)
    pub rate: f32,
    /// Initial velocity
    pub velocity: [f32; 3],
    /// Velocity randomization
    pub velocity_jitter: f32,
    /// Particle color
    pub color: [f32; 4],
    /// Particle temperature (if thermal enabled)
    pub temperature: f32,
}

impl Default for EmitterEditorConfig {
    fn default() -> Self {
        Self {
            name: "New Emitter".to_string(),
            enabled: true,
            position: [0.0, 0.0, 0.0],
            rotation: [0.0, 0.0, 0.0],
            shape: EmitterShapeType::Point,
            size: [1.0, 1.0, 1.0],
            rate: 100.0,
            velocity: [0.0, -1.0, 0.0],
            velocity_jitter: 0.1,
            color: [0.2, 0.5, 0.8, 1.0],
            temperature: 20.0,
        }
    }
}

impl EmitterEditorConfig {
    /// Create a fountain emitter
    pub fn fountain(position: [f32; 3]) -> Self {
        Self {
            name: "Fountain".to_string(),
            position,
            velocity: [0.0, 5.0, 0.0],
            velocity_jitter: 0.3,
            rate: 500.0,
            ..Default::default()
        }
    }

    /// Create a waterfall source emitter
    pub fn waterfall_source(position: [f32; 3], width: f32) -> Self {
        Self {
            name: "Waterfall Source".to_string(),
            position,
            shape: EmitterShapeType::Box,
            size: [width, 0.2, 0.5],
            velocity: [0.0, -2.0, 0.0],
            velocity_jitter: 0.2,
            rate: 1000.0,
            ..Default::default()
        }
    }

    /// Create a rain emitter
    pub fn rain(area_size: [f32; 2], height: f32) -> Self {
        Self {
            name: "Rain".to_string(),
            position: [0.0, height, 0.0],
            shape: EmitterShapeType::Box,
            size: [area_size[0], 0.1, area_size[1]],
            velocity: [0.0, -8.0, 0.0],
            velocity_jitter: 0.5,
            rate: 2000.0,
            ..Default::default()
        }
    }
}

// =============================================================================
// DRAIN CONFIGURATION
// =============================================================================

/// Fluid drain configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DrainEditorConfig {
    /// Drain name
    pub name: String,
    /// Enable this drain
    pub enabled: bool,
    /// Position in world space
    pub position: [f32; 3],
    /// Drain radius
    pub radius: f32,
    /// Drain strength (particles removed per second)
    pub strength: f32,
    /// Visualize drain in editor
    pub show_gizmo: bool,
}

impl Default for DrainEditorConfig {
    fn default() -> Self {
        Self {
            name: "New Drain".to_string(),
            enabled: true,
            position: [0.0, 0.0, 0.0],
            radius: 1.0,
            strength: 100.0,
            show_gizmo: true,
        }
    }
}

// =============================================================================
// RENDERING CONFIGURATION
// =============================================================================

/// Main rendering configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RenderingConfig {
    /// Enable SSFR (Screen Space Fluid Rendering)
    pub enable_ssfr: bool,
    /// Fluid base color (RGBA)
    pub fluid_color: [f32; 4],
    /// Light absorption coefficients (RGB)
    pub absorption: [f32; 3],
    /// Scatter color (RGB)
    pub scatter_color: [f32; 3],
    /// Roughness (0.0 - 1.0)
    pub roughness: f32,
    /// Metallic (0.0 - 1.0)
    pub metallic: f32,
    /// Index of refraction
    pub ior: f32,
    /// Enable temporal reprojection (reduces flickering)
    pub enable_temporal: bool,
    /// Temporal blend factor (0.0 - 1.0)
    pub temporal_blend: f32,
}

impl Default for RenderingConfig {
    fn default() -> Self {
        Self {
            enable_ssfr: true,
            fluid_color: [0.2, 0.5, 0.8, 1.0],
            absorption: [1.5, 0.5, 0.05],
            scatter_color: [0.0, 0.1, 0.2],
            roughness: 0.1,
            metallic: 0.0,
            ior: 1.333, // Water IOR
            enable_temporal: true,
            temporal_blend: 0.9,
        }
    }
}

// =============================================================================
// LOD CONFIGURATION
// =============================================================================

/// Level of Detail configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LodConfig {
    /// Enable LOD system
    pub enabled: bool,
    /// LOD distance thresholds (4 levels)
    pub distances: [f32; 4],
    /// Particle reduction factors per LOD level
    pub particle_factors: [f32; 4],
    /// Enable adaptive LOD based on performance
    pub adaptive: bool,
    /// Target frame time for adaptive LOD (milliseconds)
    pub target_frame_time: f32,
}

impl Default for LodConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            distances: [20.0, 50.0, 100.0, 200.0],
            particle_factors: [1.0, 0.7, 0.4, 0.2],
            adaptive: true,
            target_frame_time: 16.67, // 60 FPS
        }
    }
}

// =============================================================================
// WAVE CONFIGURATION (for ocean/lake)
// =============================================================================

/// Wave generation configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WaveConfig {
    /// Enable wave simulation
    pub enabled: bool,
    /// Wave amplitude
    pub amplitude: f32,
    /// Wave frequency
    pub frequency: f32,
    /// Wave speed
    pub speed: f32,
    /// Wind direction (normalized 2D)
    pub wind_direction: [f32; 2],
    /// Wind strength
    pub wind_strength: f32,
    /// Number of wave octaves (1 - 8)
    pub octaves: u32,
    /// Enable foam generation on wave peaks
    pub foam_on_peaks: bool,
}

impl Default for WaveConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            amplitude: 0.5,
            frequency: 1.0,
            speed: 1.0,
            wind_direction: [1.0, 0.0],
            wind_strength: 5.0,
            octaves: 4,
            foam_on_peaks: true,
        }
    }
}

impl WaveConfig {
    /// Calm water waves
    pub fn calm() -> Self {
        Self {
            amplitude: 0.1,
            frequency: 0.5,
            wind_strength: 2.0,
            octaves: 2,
            foam_on_peaks: false,
            ..Default::default()
        }
    }

    /// Stormy waves
    pub fn stormy() -> Self {
        Self {
            amplitude: 2.0,
            frequency: 1.5,
            speed: 2.0,
            wind_strength: 15.0,
            octaves: 6,
            foam_on_peaks: true,
            ..Default::default()
        }
    }
}

// =============================================================================
// CURRENT/FLOW CONFIGURATION
// =============================================================================

/// Flow/current configuration for rivers
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FlowConfig {
    /// Enable flow simulation
    pub enabled: bool,
    /// Base flow direction
    pub direction: [f32; 3],
    /// Flow speed
    pub speed: f32,
    /// Enable turbulence
    pub turbulence: bool,
    /// Turbulence strength
    pub turbulence_strength: f32,
    /// Enable eddies/whirlpools
    pub enable_eddies: bool,
}

impl Default for FlowConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            direction: [1.0, 0.0, 0.0],
            speed: 1.0,
            turbulence: true,
            turbulence_strength: 0.3,
            enable_eddies: true,
        }
    }
}

// =============================================================================
// MAIN EDITOR CONFIGURATION
// =============================================================================

/// Complete fluid system configuration for the visual editor
/// 
/// This is the main configuration struct that encompasses all fluid features
/// and can be serialized to/from JSON/TOML for the asset pipeline.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FluidEditorConfig {
    /// Configuration name
    pub name: String,
    /// Water body preset (for quick setup)
    pub preset: WaterBodyPreset,
    /// Quality preset
    pub quality: QualityPreset,
    /// Maximum particle count
    pub max_particles: u32,
    /// Physics configuration
    pub physics: PhysicsConfig,
    /// Thermal configuration
    pub thermal: ThermalConfig,
    /// Rendering configuration
    pub rendering: RenderingConfig,
    /// Caustics configuration
    pub caustics: CausticsEditorConfig,
    /// God rays configuration
    pub god_rays: GodRaysEditorConfig,
    /// Foam configuration
    pub foam: FoamEditorConfig,
    /// Reflection configuration
    pub reflections: ReflectionEditorConfig,
    /// Underwater configuration
    pub underwater: UnderwaterEditorConfig,
    /// Waterfall configuration
    pub waterfall: WaterfallEditorConfig,
    /// Wave configuration
    pub waves: WaveConfig,
    /// Flow/current configuration
    pub flow: FlowConfig,
    /// LOD configuration
    pub lod: LodConfig,
    /// Emitters
    pub emitters: Vec<EmitterEditorConfig>,
    /// Drains
    pub drains: Vec<DrainEditorConfig>,
}

impl Default for FluidEditorConfig {
    fn default() -> Self {
        Self::from_preset(WaterBodyPreset::TropicalOcean)
    }
}

impl FluidEditorConfig {
    /// Create configuration from a water body preset
    pub fn from_preset(preset: WaterBodyPreset) -> Self {
        match preset {
            WaterBodyPreset::Pool => Self::pool(),
            WaterBodyPreset::Lake => Self::lake(),
            WaterBodyPreset::River => Self::river(),
            WaterBodyPreset::Ocean => Self::ocean(),
            WaterBodyPreset::TropicalOcean => Self::tropical_ocean(),
            WaterBodyPreset::Swamp => Self::swamp(),
            WaterBodyPreset::HotSpring => Self::hot_spring(),
            WaterBodyPreset::Waterfall => Self::waterfall(),
            WaterBodyPreset::CaveWater => Self::cave_water(),
            WaterBodyPreset::ArcticWater => Self::arctic_water(),
            WaterBodyPreset::Custom => Self::custom(),
        }
    }

    /// Swimming pool preset
    pub fn pool() -> Self {
        Self {
            name: "Swimming Pool".to_string(),
            preset: WaterBodyPreset::Pool,
            quality: QualityPreset::High,
            max_particles: 20_000,
            physics: PhysicsConfig {
                viscosity: 5.0,
                surface_tension: 0.03,
                ..Default::default()
            },
            thermal: ThermalConfig {
                enabled: true,
                ambient_temperature: 28.0,
                ..Default::default()
            },
            rendering: RenderingConfig {
                fluid_color: [0.3, 0.7, 0.9, 0.9],
                absorption: [0.5, 0.2, 0.02],
                ..Default::default()
            },
            caustics: CausticsEditorConfig {
                enabled: true,
                intensity: 2.0,
                ..Default::default()
            },
            god_rays: GodRaysEditorConfig {
                enabled: true,
                intensity: 1.0,
                ..Default::default()
            },
            foam: FoamEditorConfig {
                enabled: false,
                ..Default::default()
            },
            reflections: ReflectionEditorConfig {
                enabled: true,
                intensity: 0.9,
                use_planar: true,
                resolution: 1024,
                ..Default::default()
            },
            underwater: UnderwaterEditorConfig {
                enabled: true,
                fog_color: [0.1, 0.3, 0.4],
                fog_density: 0.01,
                ..Default::default()
            },
            waterfall: WaterfallEditorConfig { enabled: false, ..Default::default() },
            waves: WaveConfig { enabled: false, ..Default::default() },
            flow: FlowConfig { enabled: false, ..Default::default() },
            lod: LodConfig::default(),
            emitters: Vec::new(),
            drains: Vec::new(),
        }
    }

    /// Lake preset
    pub fn lake() -> Self {
        Self {
            name: "Lake".to_string(),
            preset: WaterBodyPreset::Lake,
            quality: QualityPreset::Medium,
            max_particles: 30_000,
            physics: PhysicsConfig {
                viscosity: 8.0,
                surface_tension: 0.02,
                ..Default::default()
            },
            thermal: ThermalConfig::default(),
            rendering: RenderingConfig {
                fluid_color: [0.15, 0.4, 0.5, 0.95],
                absorption: [1.0, 0.4, 0.03],
                scatter_color: [0.0, 0.08, 0.12],
                ..Default::default()
            },
            caustics: CausticsEditorConfig {
                enabled: true,
                intensity: 0.8,
                max_depth: 10.0,
                ..Default::default()
            },
            god_rays: GodRaysEditorConfig {
                enabled: true,
                intensity: 0.5,
                ..Default::default()
            },
            foam: FoamEditorConfig {
                enabled: true,
                whitecap_threshold: 0.8,
                shore_intensity: 1.0,
                ..Default::default()
            },
            reflections: ReflectionEditorConfig::default(),
            underwater: UnderwaterEditorConfig::default(),
            waterfall: WaterfallEditorConfig { enabled: false, ..Default::default() },
            waves: WaveConfig::calm(),
            flow: FlowConfig { enabled: false, ..Default::default() },
            lod: LodConfig::default(),
            emitters: Vec::new(),
            drains: Vec::new(),
        }
    }

    /// River preset
    pub fn river() -> Self {
        Self {
            name: "River".to_string(),
            preset: WaterBodyPreset::River,
            quality: QualityPreset::Medium,
            max_particles: 40_000,
            physics: PhysicsConfig {
                viscosity: 6.0,
                surface_tension: 0.015,
                enable_vorticity: true,
                vorticity_strength: 0.3,
                ..Default::default()
            },
            thermal: ThermalConfig::default(),
            rendering: RenderingConfig {
                fluid_color: [0.2, 0.45, 0.55, 0.9],
                absorption: [1.2, 0.5, 0.04],
                ..Default::default()
            },
            caustics: CausticsEditorConfig {
                enabled: true,
                intensity: 0.6,
                speed: 1.5,
                ..Default::default()
            },
            god_rays: GodRaysEditorConfig {
                enabled: false,
                ..Default::default()
            },
            foam: FoamEditorConfig {
                enabled: true,
                whitecap_threshold: 0.5,
                shore_intensity: 2.0,
                ..Default::default()
            },
            reflections: ReflectionEditorConfig {
                enabled: true,
                distortion: 0.05,
                ..Default::default()
            },
            underwater: UnderwaterEditorConfig::default(),
            waterfall: WaterfallEditorConfig { enabled: false, ..Default::default() },
            waves: WaveConfig {
                enabled: true,
                amplitude: 0.1,
                frequency: 2.0,
                ..Default::default()
            },
            flow: FlowConfig {
                enabled: true,
                speed: 2.0,
                turbulence: true,
                turbulence_strength: 0.4,
                enable_eddies: true,
                ..Default::default()
            },
            lod: LodConfig::default(),
            emitters: Vec::new(),
            drains: Vec::new(),
        }
    }

    /// Ocean preset
    pub fn ocean() -> Self {
        Self {
            name: "Ocean".to_string(),
            preset: WaterBodyPreset::Ocean,
            quality: QualityPreset::High,
            max_particles: 50_000,
            physics: PhysicsConfig {
                viscosity: 10.0,
                surface_tension: 0.02,
                ..Default::default()
            },
            thermal: ThermalConfig::default(),
            rendering: RenderingConfig {
                fluid_color: [0.05, 0.2, 0.4, 1.0],
                absorption: [2.0, 0.8, 0.1],
                scatter_color: [0.0, 0.15, 0.25],
                ..Default::default()
            },
            caustics: CausticsEditorConfig {
                enabled: true,
                intensity: 0.5,
                max_depth: 30.0,
                ..Default::default()
            },
            god_rays: GodRaysEditorConfig {
                enabled: true,
                intensity: 0.7,
                max_depth: 60.0,
                ..Default::default()
            },
            foam: FoamEditorConfig {
                enabled: true,
                max_particles: 15000,
                whitecap_threshold: 0.4,
                shore_intensity: 2.0,
                ..Default::default()
            },
            reflections: ReflectionEditorConfig {
                enabled: true,
                fresnel_power: 4.0,
                distortion: 0.04,
                ..Default::default()
            },
            underwater: UnderwaterEditorConfig {
                enabled: true,
                fog_color: [0.0, 0.05, 0.15],
                fog_density: 0.03,
                ..Default::default()
            },
            waterfall: WaterfallEditorConfig { enabled: false, ..Default::default() },
            waves: WaveConfig {
                enabled: true,
                amplitude: 1.0,
                frequency: 0.8,
                wind_strength: 8.0,
                octaves: 5,
                foam_on_peaks: true,
                ..Default::default()
            },
            flow: FlowConfig { enabled: false, ..Default::default() },
            lod: LodConfig {
                distances: [30.0, 80.0, 200.0, 500.0],
                ..Default::default()
            },
            emitters: Vec::new(),
            drains: Vec::new(),
        }
    }

    /// Tropical ocean preset (crystal clear)
    pub fn tropical_ocean() -> Self {
        Self {
            name: "Tropical Ocean".to_string(),
            preset: WaterBodyPreset::TropicalOcean,
            quality: QualityPreset::Ultra,
            max_particles: 60_000,
            physics: PhysicsConfig {
                viscosity: 8.0,
                surface_tension: 0.025,
                ..Default::default()
            },
            thermal: ThermalConfig {
                enabled: true,
                ambient_temperature: 26.0,
                ..Default::default()
            },
            rendering: RenderingConfig {
                fluid_color: [0.1, 0.6, 0.8, 0.95],
                absorption: [0.3, 0.15, 0.02],
                scatter_color: [0.0, 0.2, 0.3],
                ..Default::default()
            },
            caustics: CausticsEditorConfig {
                enabled: true,
                intensity: 2.5,
                max_depth: 40.0,
                ..Default::default()
            },
            god_rays: GodRaysEditorConfig {
                enabled: true,
                intensity: 1.2,
                samples: 48,
                max_depth: 80.0,
                ..Default::default()
            },
            foam: FoamEditorConfig {
                enabled: true,
                whitecap_threshold: 0.6,
                shore_intensity: 1.5,
                ..Default::default()
            },
            reflections: ReflectionEditorConfig {
                enabled: true,
                intensity: 0.9,
                fresnel_power: 5.0,
                use_planar: true,
                resolution: 1024,
                ..Default::default()
            },
            underwater: UnderwaterEditorConfig {
                enabled: true,
                fog_color: [0.0, 0.15, 0.25],
                fog_density: 0.008,
                enable_particles: true,
                particle_density: 50.0,
                ..Default::default()
            },
            waterfall: WaterfallEditorConfig { enabled: false, ..Default::default() },
            waves: WaveConfig {
                enabled: true,
                amplitude: 0.6,
                frequency: 1.0,
                wind_strength: 5.0,
                octaves: 4,
                foam_on_peaks: true,
                ..Default::default()
            },
            flow: FlowConfig { enabled: false, ..Default::default() },
            lod: LodConfig {
                distances: [40.0, 100.0, 250.0, 600.0],
                ..Default::default()
            },
            emitters: Vec::new(),
            drains: Vec::new(),
        }
    }

    /// Swamp preset
    pub fn swamp() -> Self {
        Self {
            name: "Swamp".to_string(),
            preset: WaterBodyPreset::Swamp,
            quality: QualityPreset::Medium,
            max_particles: 25_000,
            physics: PhysicsConfig {
                viscosity: 20.0,
                surface_tension: 0.01,
                ..Default::default()
            },
            thermal: ThermalConfig::default(),
            rendering: RenderingConfig {
                fluid_color: [0.15, 0.25, 0.1, 1.0],
                absorption: [3.0, 2.0, 1.5],
                scatter_color: [0.1, 0.15, 0.05],
                roughness: 0.3,
                ..Default::default()
            },
            caustics: CausticsEditorConfig { enabled: false, ..Default::default() },
            god_rays: GodRaysEditorConfig { enabled: false, ..Default::default() },
            foam: FoamEditorConfig {
                enabled: true,
                color: [0.8, 0.85, 0.7],
                lifetime: 5.0,
                ..Default::default()
            },
            reflections: ReflectionEditorConfig {
                enabled: true,
                intensity: 0.5,
                distortion: 0.02,
                ..Default::default()
            },
            underwater: UnderwaterEditorConfig {
                enabled: true,
                fog_color: [0.1, 0.15, 0.05],
                fog_density: 0.1,
                enable_particles: true,
                particle_density: 200.0,
                ..Default::default()
            },
            waterfall: WaterfallEditorConfig { enabled: false, ..Default::default() },
            waves: WaveConfig { enabled: false, ..Default::default() },
            flow: FlowConfig {
                enabled: true,
                speed: 0.3,
                turbulence: true,
                turbulence_strength: 0.1,
                ..Default::default()
            },
            lod: LodConfig::default(),
            emitters: Vec::new(),
            drains: Vec::new(),
        }
    }

    /// Hot spring preset
    pub fn hot_spring() -> Self {
        Self {
            name: "Hot Spring".to_string(),
            preset: WaterBodyPreset::HotSpring,
            quality: QualityPreset::High,
            max_particles: 20_000,
            physics: PhysicsConfig {
                viscosity: 4.0,
                surface_tension: 0.02,
                ..Default::default()
            },
            thermal: ThermalConfig {
                enabled: true,
                ambient_temperature: 45.0,
                diffusivity: 0.2,
                buoyancy_coefficient: 0.0005,
                enable_evaporation: true,
                evaporation_temperature: 80.0,
            },
            rendering: RenderingConfig {
                fluid_color: [0.4, 0.6, 0.7, 0.85],
                absorption: [0.8, 0.4, 0.1],
                scatter_color: [0.1, 0.15, 0.2],
                ..Default::default()
            },
            caustics: CausticsEditorConfig {
                enabled: true,
                intensity: 0.8,
                ..Default::default()
            },
            god_rays: GodRaysEditorConfig { enabled: false, ..Default::default() },
            foam: FoamEditorConfig {
                enabled: true,
                lifetime: 2.0,
                ..Default::default()
            },
            reflections: ReflectionEditorConfig {
                enabled: true,
                distortion: 0.06,
                ..Default::default()
            },
            underwater: UnderwaterEditorConfig::default(),
            waterfall: WaterfallEditorConfig { enabled: false, ..Default::default() },
            waves: WaveConfig {
                enabled: true,
                amplitude: 0.05,
                frequency: 3.0,
                ..Default::default()
            },
            flow: FlowConfig { enabled: false, ..Default::default() },
            lod: LodConfig::default(),
            emitters: Vec::new(),
            drains: Vec::new(),
        }
    }

    /// Waterfall preset
    pub fn waterfall() -> Self {
        Self {
            name: "Waterfall".to_string(),
            preset: WaterBodyPreset::Waterfall,
            quality: QualityPreset::High,
            max_particles: 50_000,
            physics: PhysicsConfig {
                viscosity: 5.0,
                surface_tension: 0.015,
                enable_vorticity: true,
                vorticity_strength: 0.4,
                ..Default::default()
            },
            thermal: ThermalConfig::default(),
            rendering: RenderingConfig {
                fluid_color: [0.3, 0.6, 0.8, 0.8],
                absorption: [0.6, 0.3, 0.02],
                ..Default::default()
            },
            caustics: CausticsEditorConfig {
                enabled: true,
                intensity: 1.5,
                speed: 2.0,
                ..Default::default()
            },
            god_rays: GodRaysEditorConfig {
                enabled: true,
                intensity: 0.6,
                ..Default::default()
            },
            foam: FoamEditorConfig {
                enabled: true,
                max_particles: 20000,
                whitecap_threshold: 0.3,
                shore_intensity: 3.0,
                wake_intensity: 2.0,
                ..Default::default()
            },
            reflections: ReflectionEditorConfig {
                enabled: true,
                distortion: 0.08,
                ..Default::default()
            },
            underwater: UnderwaterEditorConfig {
                enabled: true,
                enable_particles: true,
                particle_density: 150.0,
                ..Default::default()
            },
            waterfall: WaterfallEditorConfig {
                enabled: true,
                max_particles: 10000,
                spawn_rate: 500.0,
                mist_density: 0.7,
                mist_rise_speed: 1.2,
                spray_angle: 0.5,
                splash_intensity: 2.0,
            },
            waves: WaveConfig { enabled: false, ..Default::default() },
            flow: FlowConfig {
                enabled: true,
                direction: [0.0, -1.0, 0.0],
                speed: 5.0,
                turbulence: true,
                turbulence_strength: 0.6,
                enable_eddies: true,
            },
            lod: LodConfig::default(),
            emitters: vec![EmitterEditorConfig::waterfall_source([0.0, 10.0, 0.0], 5.0)],
            drains: Vec::new(),
        }
    }

    /// Cave water preset
    pub fn cave_water() -> Self {
        Self {
            name: "Cave Water".to_string(),
            preset: WaterBodyPreset::CaveWater,
            quality: QualityPreset::High,
            max_particles: 15_000,
            physics: PhysicsConfig {
                viscosity: 12.0,
                surface_tension: 0.025,
                ..Default::default()
            },
            thermal: ThermalConfig {
                enabled: true,
                ambient_temperature: 12.0,
                ..Default::default()
            },
            rendering: RenderingConfig {
                fluid_color: [0.05, 0.1, 0.15, 1.0],
                absorption: [2.5, 1.5, 0.8],
                scatter_color: [0.0, 0.02, 0.04],
                ..Default::default()
            },
            caustics: CausticsEditorConfig {
                enabled: true,
                intensity: 0.3,
                max_depth: 5.0,
                ..Default::default()
            },
            god_rays: GodRaysEditorConfig { enabled: false, ..Default::default() },
            foam: FoamEditorConfig { enabled: false, ..Default::default() },
            reflections: ReflectionEditorConfig {
                enabled: true,
                intensity: 0.95,
                fresnel_power: 6.0,
                use_planar: true,
                resolution: 512,
                ..Default::default()
            },
            underwater: UnderwaterEditorConfig {
                enabled: true,
                fog_color: [0.0, 0.02, 0.05],
                fog_density: 0.05,
                ..Default::default()
            },
            waterfall: WaterfallEditorConfig { enabled: false, ..Default::default() },
            waves: WaveConfig { enabled: false, ..Default::default() },
            flow: FlowConfig {
                enabled: true,
                speed: 0.2,
                turbulence: false,
                ..Default::default()
            },
            lod: LodConfig::default(),
            emitters: Vec::new(),
            drains: Vec::new(),
        }
    }

    /// Arctic water preset
    pub fn arctic_water() -> Self {
        Self {
            name: "Arctic Water".to_string(),
            preset: WaterBodyPreset::ArcticWater,
            quality: QualityPreset::High,
            max_particles: 30_000,
            physics: PhysicsConfig {
                viscosity: 15.0,
                surface_tension: 0.03,
                ..Default::default()
            },
            thermal: ThermalConfig {
                enabled: true,
                ambient_temperature: 2.0,
                diffusivity: 0.05,
                ..Default::default()
            },
            rendering: RenderingConfig {
                fluid_color: [0.4, 0.6, 0.7, 0.98],
                absorption: [0.8, 0.5, 0.3],
                scatter_color: [0.1, 0.2, 0.25],
                roughness: 0.05,
                ..Default::default()
            },
            caustics: CausticsEditorConfig {
                enabled: true,
                intensity: 0.6,
                ..Default::default()
            },
            god_rays: GodRaysEditorConfig {
                enabled: true,
                intensity: 0.4,
                ..Default::default()
            },
            foam: FoamEditorConfig {
                enabled: true,
                color: [0.98, 0.99, 1.0],
                ..Default::default()
            },
            reflections: ReflectionEditorConfig {
                enabled: true,
                intensity: 0.85,
                ..Default::default()
            },
            underwater: UnderwaterEditorConfig {
                enabled: true,
                fog_color: [0.1, 0.2, 0.25],
                fog_density: 0.015,
                ..Default::default()
            },
            waterfall: WaterfallEditorConfig { enabled: false, ..Default::default() },
            waves: WaveConfig {
                enabled: true,
                amplitude: 0.3,
                frequency: 0.6,
                wind_strength: 6.0,
                ..Default::default()
            },
            flow: FlowConfig { enabled: false, ..Default::default() },
            lod: LodConfig::default(),
            emitters: Vec::new(),
            drains: Vec::new(),
        }
    }

    /// Custom preset (empty configuration)
    pub fn custom() -> Self {
        Self {
            name: "Custom Water".to_string(),
            preset: WaterBodyPreset::Custom,
            quality: QualityPreset::Medium,
            max_particles: 20_000,
            physics: PhysicsConfig::default(),
            thermal: ThermalConfig::default(),
            rendering: RenderingConfig::default(),
            caustics: CausticsEditorConfig::default(),
            god_rays: GodRaysEditorConfig::default(),
            foam: FoamEditorConfig::default(),
            reflections: ReflectionEditorConfig::default(),
            underwater: UnderwaterEditorConfig::default(),
            waterfall: WaterfallEditorConfig::default(),
            waves: WaveConfig::default(),
            flow: FlowConfig::default(),
            lod: LodConfig::default(),
            emitters: Vec::new(),
            drains: Vec::new(),
        }
    }

    // === Legacy compatibility methods ===
    
    /// Create config optimized for performance (legacy API)
    pub fn performance() -> Self {
        let mut config = Self::from_preset(WaterBodyPreset::Lake);
        config.quality = QualityPreset::Low;
        config.max_particles = 5_000;
        config.caustics.enabled = false;
        config.god_rays.enabled = false;
        config.foam.enabled = false;
        config.reflections.use_planar = false;
        config.physics.iterations = 2;
        config
    }

    /// Create config optimized for quality (legacy API)
    pub fn quality() -> Self {
        Self::from_preset(WaterBodyPreset::TropicalOcean)
    }

    // === Utility Methods ===

    /// Clamp all values to safe ranges
    pub fn clamp(&mut self) {
        self.max_particles = self.max_particles.clamp(1_000, 500_000);
        self.physics.clamp();
        self.thermal.clamp();
        self.caustics.intensity = self.caustics.intensity.clamp(0.0, 5.0);
        self.god_rays.intensity = self.god_rays.intensity.clamp(0.0, 2.0);
        self.god_rays.samples = self.god_rays.samples.clamp(8, 64);
        self.foam.max_particles = self.foam.max_particles.clamp(0, 100_000);
        self.foam.lifetime = self.foam.lifetime.clamp(0.5, 10.0);
        self.reflections.intensity = self.reflections.intensity.clamp(0.0, 1.0);
        self.reflections.resolution = self.reflections.resolution.clamp(128, 2048);
        self.rendering.temporal_blend = self.rendering.temporal_blend.clamp(0.0, 1.0);
        self.waves.amplitude = self.waves.amplitude.clamp(0.0, 10.0);
        self.waves.octaves = self.waves.octaves.clamp(1, 8);
    }

    /// Validate configuration and return any warnings
    pub fn validate(&self) -> Vec<String> {
        let mut warnings = Vec::new();

        if self.max_particles > 100_000 {
            warnings.push("High particle count may impact performance".to_string());
        }

        if self.caustics.enabled && self.god_rays.enabled && self.reflections.use_planar {
            warnings.push("Multiple expensive effects enabled - consider disabling some for better performance".to_string());
        }

        if self.physics.iterations > 10 {
            warnings.push("High iteration count may impact performance".to_string());
        }

        if self.waves.octaves > 6 {
            warnings.push("High wave octave count may impact performance".to_string());
        }

        if self.foam.max_particles > 50_000 {
            warnings.push("High foam particle count may impact performance".to_string());
        }

        warnings
    }

    /// Get estimated performance cost (0-100, higher = more expensive)
    pub fn estimated_performance_cost(&self) -> u32 {
        let mut cost = 0u32;

        // Base particle cost
        cost += (self.max_particles / 10_000) as u32 * 5;

        // Physics cost
        cost += self.physics.iterations * 2;
        if self.physics.enable_vorticity { cost += 3; }
        if self.thermal.enabled { cost += 5; }

        // Visual effects cost
        if self.caustics.enabled { cost += 8; }
        if self.god_rays.enabled { cost += (self.god_rays.samples / 8) as u32; }
        if self.foam.enabled { cost += (self.foam.max_particles / 5000) as u32 * 2; }
        if self.reflections.enabled {
            cost += if self.reflections.use_planar { 15 } else { 5 };
        }
        if self.waterfall.enabled { cost += 10; }

        // Wave cost
        if self.waves.enabled { cost += self.waves.octaves; }

        cost.min(100)
    }

    /// Add an emitter
    pub fn add_emitter(&mut self, emitter: EmitterEditorConfig) {
        self.emitters.push(emitter);
    }

    /// Add a drain
    pub fn add_drain(&mut self, drain: DrainEditorConfig) {
        self.drains.push(drain);
    }

    /// Remove emitter by index
    pub fn remove_emitter(&mut self, index: usize) -> Option<EmitterEditorConfig> {
        if index < self.emitters.len() {
            Some(self.emitters.remove(index))
        } else {
            None
        }
    }

    /// Remove drain by index
    pub fn remove_drain(&mut self, index: usize) -> Option<DrainEditorConfig> {
        if index < self.drains.len() {
            Some(self.drains.remove(index))
        } else {
            None
        }
    }

    /// Save config to JSON string
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Load config from JSON string
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        let mut config: Self = serde_json::from_str(json)?;
        config.clamp();
        Ok(config)
    }

    /// Save config to TOML string
    pub fn to_toml(&self) -> Result<String, toml::ser::Error> {
        toml::to_string_pretty(self)
    }

    /// Load config from TOML string
    pub fn from_toml(toml_str: &str) -> Result<Self, toml::de::Error> {
        let mut config: Self = toml::from_str(toml_str)?;
        config.clamp();
        Ok(config)
    }
    
    // === Interpolation Methods ===
    
    /// Linearly interpolate between two configs
    /// 
    /// Useful for smooth transitions when changing presets or tweaking parameters.
    /// `t` should be in range 0.0 - 1.0 (0.0 = self, 1.0 = other)
    pub fn interpolate(&self, other: &FluidEditorConfig, t: f32) -> FluidEditorConfig {
        let t = t.clamp(0.0, 1.0);
        let lerp_f32 = |a: f32, b: f32| a + (b - a) * t;
        let lerp_u32 = |a: u32, b: u32| ((a as f32) + ((b as f32) - (a as f32)) * t) as u32;
        let lerp_arr3 = |a: [f32; 3], b: [f32; 3]| [
            lerp_f32(a[0], b[0]),
            lerp_f32(a[1], b[1]),
            lerp_f32(a[2], b[2]),
        ];
        let lerp_arr4 = |a: [f32; 4], b: [f32; 4]| [
            lerp_f32(a[0], b[0]),
            lerp_f32(a[1], b[1]),
            lerp_f32(a[2], b[2]),
            lerp_f32(a[3], b[3]),
        ];
        
        FluidEditorConfig {
            name: if t < 0.5 { self.name.clone() } else { other.name.clone() },
            preset: if t < 0.5 { self.preset.clone() } else { other.preset.clone() },
            quality: if t < 0.5 { self.quality } else { other.quality },
            max_particles: lerp_u32(self.max_particles, other.max_particles),
            physics: PhysicsConfig {
                smoothing_radius: lerp_f32(self.physics.smoothing_radius, other.physics.smoothing_radius),
                target_density: lerp_f32(self.physics.target_density, other.physics.target_density),
                pressure_multiplier: lerp_f32(self.physics.pressure_multiplier, other.physics.pressure_multiplier),
                viscosity: lerp_f32(self.physics.viscosity, other.physics.viscosity),
                surface_tension: lerp_f32(self.physics.surface_tension, other.physics.surface_tension),
                gravity: lerp_arr3(self.physics.gravity, other.physics.gravity),
                iterations: lerp_u32(self.physics.iterations, other.physics.iterations),
                enable_vorticity: if t < 0.5 { self.physics.enable_vorticity } else { other.physics.enable_vorticity },
                vorticity_strength: lerp_f32(self.physics.vorticity_strength, other.physics.vorticity_strength),
            },
            thermal: ThermalConfig {
                enabled: if t < 0.5 { self.thermal.enabled } else { other.thermal.enabled },
                ambient_temperature: lerp_f32(self.thermal.ambient_temperature, other.thermal.ambient_temperature),
                diffusivity: lerp_f32(self.thermal.diffusivity, other.thermal.diffusivity),
                buoyancy_coefficient: lerp_f32(self.thermal.buoyancy_coefficient, other.thermal.buoyancy_coefficient),
                enable_evaporation: if t < 0.5 { self.thermal.enable_evaporation } else { other.thermal.enable_evaporation },
                evaporation_temperature: lerp_f32(self.thermal.evaporation_temperature, other.thermal.evaporation_temperature),
            },
            rendering: RenderingConfig {
                enable_ssfr: if t < 0.5 { self.rendering.enable_ssfr } else { other.rendering.enable_ssfr },
                fluid_color: lerp_arr4(self.rendering.fluid_color, other.rendering.fluid_color),
                absorption: lerp_arr3(self.rendering.absorption, other.rendering.absorption),
                scatter_color: lerp_arr3(self.rendering.scatter_color, other.rendering.scatter_color),
                roughness: lerp_f32(self.rendering.roughness, other.rendering.roughness),
                metallic: lerp_f32(self.rendering.metallic, other.rendering.metallic),
                ior: lerp_f32(self.rendering.ior, other.rendering.ior),
                enable_temporal: if t < 0.5 { self.rendering.enable_temporal } else { other.rendering.enable_temporal },
                temporal_blend: lerp_f32(self.rendering.temporal_blend, other.rendering.temporal_blend),
            },
            caustics: CausticsEditorConfig {
                enabled: if t < 0.5 { self.caustics.enabled } else { other.caustics.enabled },
                intensity: lerp_f32(self.caustics.intensity, other.caustics.intensity),
                scale: lerp_f32(self.caustics.scale, other.caustics.scale),
                speed: lerp_f32(self.caustics.speed, other.caustics.speed),
                max_depth: lerp_f32(self.caustics.max_depth, other.caustics.max_depth),
            },
            god_rays: GodRaysEditorConfig {
                enabled: if t < 0.5 { self.god_rays.enabled } else { other.god_rays.enabled },
                intensity: lerp_f32(self.god_rays.intensity, other.god_rays.intensity),
                samples: lerp_u32(self.god_rays.samples, other.god_rays.samples),
                decay: lerp_f32(self.god_rays.decay, other.god_rays.decay),
                max_depth: lerp_f32(self.god_rays.max_depth, other.god_rays.max_depth),
            },
            foam: FoamEditorConfig {
                enabled: if t < 0.5 { self.foam.enabled } else { other.foam.enabled },
                max_particles: lerp_u32(self.foam.max_particles, other.foam.max_particles),
                lifetime: lerp_f32(self.foam.lifetime, other.foam.lifetime),
                whitecap_threshold: lerp_f32(self.foam.whitecap_threshold, other.foam.whitecap_threshold),
                shore_intensity: lerp_f32(self.foam.shore_intensity, other.foam.shore_intensity),
                wake_intensity: lerp_f32(self.foam.wake_intensity, other.foam.wake_intensity),
                color: lerp_arr3(self.foam.color, other.foam.color),
            },
            reflections: ReflectionEditorConfig {
                enabled: if t < 0.5 { self.reflections.enabled } else { other.reflections.enabled },
                intensity: lerp_f32(self.reflections.intensity, other.reflections.intensity),
                fresnel_power: lerp_f32(self.reflections.fresnel_power, other.reflections.fresnel_power),
                distortion: lerp_f32(self.reflections.distortion, other.reflections.distortion),
                use_planar: if t < 0.5 { self.reflections.use_planar } else { other.reflections.use_planar },
                resolution: lerp_u32(self.reflections.resolution, other.reflections.resolution),
            },
            underwater: UnderwaterEditorConfig {
                enabled: if t < 0.5 { self.underwater.enabled } else { other.underwater.enabled },
                fog_color: lerp_arr3(self.underwater.fog_color, other.underwater.fog_color),
                fog_density: lerp_f32(self.underwater.fog_density, other.underwater.fog_density),
                enable_particles: if t < 0.5 { self.underwater.enable_particles } else { other.underwater.enable_particles },
                particle_density: lerp_f32(self.underwater.particle_density, other.underwater.particle_density),
                distortion: lerp_f32(self.underwater.distortion, other.underwater.distortion),
            },
            waterfall: WaterfallEditorConfig {
                enabled: if t < 0.5 { self.waterfall.enabled } else { other.waterfall.enabled },
                max_particles: lerp_u32(self.waterfall.max_particles, other.waterfall.max_particles),
                spawn_rate: lerp_f32(self.waterfall.spawn_rate, other.waterfall.spawn_rate),
                mist_density: lerp_f32(self.waterfall.mist_density, other.waterfall.mist_density),
                mist_rise_speed: lerp_f32(self.waterfall.mist_rise_speed, other.waterfall.mist_rise_speed),
                spray_angle: lerp_f32(self.waterfall.spray_angle, other.waterfall.spray_angle),
                splash_intensity: lerp_f32(self.waterfall.splash_intensity, other.waterfall.splash_intensity),
            },
            waves: WaveConfig {
                enabled: if t < 0.5 { self.waves.enabled } else { other.waves.enabled },
                amplitude: lerp_f32(self.waves.amplitude, other.waves.amplitude),
                frequency: lerp_f32(self.waves.frequency, other.waves.frequency),
                speed: lerp_f32(self.waves.speed, other.waves.speed),
                wind_direction: [
                    lerp_f32(self.waves.wind_direction[0], other.waves.wind_direction[0]),
                    lerp_f32(self.waves.wind_direction[1], other.waves.wind_direction[1]),
                ],
                wind_strength: lerp_f32(self.waves.wind_strength, other.waves.wind_strength),
                octaves: lerp_u32(self.waves.octaves, other.waves.octaves),
                foam_on_peaks: if t < 0.5 { self.waves.foam_on_peaks } else { other.waves.foam_on_peaks },
            },
            flow: FlowConfig {
                enabled: if t < 0.5 { self.flow.enabled } else { other.flow.enabled },
                direction: lerp_arr3(self.flow.direction, other.flow.direction),
                speed: lerp_f32(self.flow.speed, other.flow.speed),
                turbulence: if t < 0.5 { self.flow.turbulence } else { other.flow.turbulence },
                turbulence_strength: lerp_f32(self.flow.turbulence_strength, other.flow.turbulence_strength),
                enable_eddies: if t < 0.5 { self.flow.enable_eddies } else { other.flow.enable_eddies },
            },
            lod: LodConfig {
                enabled: if t < 0.5 { self.lod.enabled } else { other.lod.enabled },
                distances: [
                    lerp_f32(self.lod.distances[0], other.lod.distances[0]),
                    lerp_f32(self.lod.distances[1], other.lod.distances[1]),
                    lerp_f32(self.lod.distances[2], other.lod.distances[2]),
                    lerp_f32(self.lod.distances[3], other.lod.distances[3]),
                ],
                particle_factors: [
                    lerp_f32(self.lod.particle_factors[0], other.lod.particle_factors[0]),
                    lerp_f32(self.lod.particle_factors[1], other.lod.particle_factors[1]),
                    lerp_f32(self.lod.particle_factors[2], other.lod.particle_factors[2]),
                    lerp_f32(self.lod.particle_factors[3], other.lod.particle_factors[3]),
                ],
                adaptive: if t < 0.5 { self.lod.adaptive } else { other.lod.adaptive },
                target_frame_time: lerp_f32(self.lod.target_frame_time, other.lod.target_frame_time),
            },
            // Don't interpolate emitters/drains - use target's
            emitters: if t < 0.5 { self.emitters.clone() } else { other.emitters.clone() },
            drains: if t < 0.5 { self.drains.clone() } else { other.drains.clone() },
        }
    }
    
    /// Smoothly transition to a target config over time
    /// 
    /// Returns the blended config. Call this each frame with increasing t.
    pub fn smooth_transition(&self, target: &FluidEditorConfig, t: f32, smoothing: f32) -> FluidEditorConfig {
        // Apply smoothstep for more natural easing
        let t = t.clamp(0.0, 1.0);
        let smooth_t = t * t * (3.0 - 2.0 * t) * smoothing + t * (1.0 - smoothing);
        self.interpolate(target, smooth_t)
    }
    
    /// Create a diff summary between two configs
    pub fn diff(&self, other: &FluidEditorConfig) -> Vec<String> {
        let mut diffs = Vec::new();
        
        if self.preset != other.preset {
            diffs.push(format!("Preset: {:?} → {:?}", self.preset, other.preset));
        }
        if self.max_particles != other.max_particles {
            diffs.push(format!("Max Particles: {} → {}", self.max_particles, other.max_particles));
        }
        if (self.physics.viscosity - other.physics.viscosity).abs() > 0.01 {
            diffs.push(format!("Viscosity: {:.1} → {:.1}", self.physics.viscosity, other.physics.viscosity));
        }
        if self.caustics.enabled != other.caustics.enabled {
            diffs.push(format!("Caustics: {} → {}", self.caustics.enabled, other.caustics.enabled));
        }
        if self.god_rays.enabled != other.god_rays.enabled {
            diffs.push(format!("God Rays: {} → {}", self.god_rays.enabled, other.god_rays.enabled));
        }
        if self.foam.enabled != other.foam.enabled {
            diffs.push(format!("Foam: {} → {}", self.foam.enabled, other.foam.enabled));
        }
        if self.waves.enabled != other.waves.enabled {
            diffs.push(format!("Waves: {} → {}", self.waves.enabled, other.waves.enabled));
        }
        if self.flow.enabled != other.flow.enabled {
            diffs.push(format!("Flow: {} → {}", self.flow.enabled, other.flow.enabled));
        }
        
        diffs
    }
    
    /// Apply quality preset settings
    pub fn apply_quality_preset(&mut self, quality: QualityPreset) {
        self.quality = quality;
        self.max_particles = quality.recommended_particles();
        self.god_rays.samples = quality.recommended_god_ray_samples();
        self.reflections.resolution = quality.recommended_reflection_resolution();
        
        // Adjust LOD based on quality
        match quality {
            QualityPreset::Low => {
                self.caustics.enabled = false;
                self.god_rays.enabled = false;
                self.reflections.use_planar = false;
                self.waves.octaves = 2;
            }
            QualityPreset::Medium => {
                self.reflections.use_planar = false;
                self.waves.octaves = 3;
            }
            QualityPreset::High => {
                self.waves.octaves = 4;
            }
            QualityPreset::Ultra => {
                self.reflections.use_planar = true;
                self.waves.octaves = 6;
            }
        }
    }
    
    /// Get a summary string for the editor
    pub fn summary(&self) -> String {
        let effects: Vec<&str> = [
            if self.caustics.enabled { Some("Caustics") } else { None },
            if self.god_rays.enabled { Some("God Rays") } else { None },
            if self.foam.enabled { Some("Foam") } else { None },
            if self.reflections.enabled { Some("Reflections") } else { None },
            if self.waves.enabled { Some("Waves") } else { None },
            if self.flow.enabled { Some("Flow") } else { None },
        ]
        .into_iter()
        .flatten()
        .collect();
        
        format!(
            "{} ({:?}) - {}K particles, Effects: {}",
            self.name,
            self.quality,
            self.max_particles / 1000,
            if effects.is_empty() { "None".to_string() } else { effects.join(", ") }
        )
    }

    // === Legacy API compatibility ===

    /// Get smoothing radius (legacy API)
    pub fn smoothing_radius(&self) -> f32 {
        self.physics.smoothing_radius
    }

    /// Get target density (legacy API)
    pub fn target_density(&self) -> f32 {
        self.physics.target_density
    }

    /// Get pressure multiplier (legacy API)
    pub fn pressure_multiplier(&self) -> f32 {
        self.physics.pressure_multiplier
    }

    /// Get viscosity (legacy API)
    pub fn viscosity(&self) -> f32 {
        self.physics.viscosity
    }

    /// Get surface tension (legacy API)
    pub fn surface_tension(&self) -> f32 {
        self.physics.surface_tension
    }

    /// Get gravity as scalar (legacy API - returns Y component)
    pub fn gravity(&self) -> f32 {
        self.physics.gravity[1]
    }

    /// Get iterations (legacy API)
    pub fn iterations(&self) -> u32 {
        self.physics.iterations
    }

    /// Check if temperature is enabled (legacy API)
    pub fn enable_temperature(&self) -> bool {
        self.thermal.enabled
    }

    /// Get thermal diffusivity (legacy API)
    pub fn thermal_diffusivity(&self) -> f32 {
        self.thermal.diffusivity
    }

    /// Get buoyancy coefficient (legacy API)
    pub fn buoyancy_coefficient(&self) -> f32 {
        self.thermal.buoyancy_coefficient
    }

    /// Check if SSFR is enabled (legacy API)
    pub fn enable_ssfr(&self) -> bool {
        self.rendering.enable_ssfr
    }

    /// Get fluid color (legacy API)
    pub fn fluid_color(&self) -> [f32; 4] {
        self.rendering.fluid_color
    }

    /// Get absorption (legacy API)
    pub fn absorption(&self) -> [f32; 3] {
        self.rendering.absorption
    }

    /// Get scatter color (legacy API)
    pub fn scatter_color(&self) -> [f32; 3] {
        self.rendering.scatter_color
    }

    /// Check if caustics are enabled (legacy API)
    pub fn enable_caustics(&self) -> bool {
        self.caustics.enabled
    }

    /// Get caustic intensity (legacy API)
    pub fn caustic_intensity(&self) -> f32 {
        self.caustics.intensity
    }

    /// Check if temporal reprojection is enabled (legacy API)
    pub fn enable_temporal(&self) -> bool {
        self.rendering.enable_temporal
    }

    /// Get temporal blend (legacy API)
    pub fn temporal_blend(&self) -> f32 {
        self.rendering.temporal_blend
    }

    /// Get LOD distances (legacy API)
    pub fn lod_distances(&self) -> [f32; 4] {
        self.lod.distances
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== Preset Enum Tests ====================

    #[test]
    fn test_water_body_preset_descriptions() {
        let presets = [
            WaterBodyPreset::Pool,
            WaterBodyPreset::Lake,
            WaterBodyPreset::River,
            WaterBodyPreset::Ocean,
            WaterBodyPreset::TropicalOcean,
            WaterBodyPreset::Swamp,
            WaterBodyPreset::HotSpring,
            WaterBodyPreset::Waterfall,
            WaterBodyPreset::CaveWater,
            WaterBodyPreset::ArcticWater,
            WaterBodyPreset::Custom,
        ];
        
        for preset in presets {
            let desc = preset.description();
            assert!(!desc.is_empty(), "Preset {:?} should have description", preset);
        }
    }

    #[test]
    fn test_quality_preset_particles() {
        assert_eq!(QualityPreset::Low.recommended_particles(), 5_000);
        assert_eq!(QualityPreset::Medium.recommended_particles(), 20_000);
        assert_eq!(QualityPreset::High.recommended_particles(), 50_000);
        assert_eq!(QualityPreset::Ultra.recommended_particles(), 100_000);
    }

    #[test]
    fn test_quality_preset_descriptions() {
        assert!(!QualityPreset::Low.description().is_empty());
        assert!(!QualityPreset::Medium.description().is_empty());
        assert!(!QualityPreset::High.description().is_empty());
        assert!(!QualityPreset::Ultra.description().is_empty());
    }

    // ==================== Default Config Tests ====================

    #[test]
    fn test_config_default() {
        let config = FluidEditorConfig::default();
        
        // Uses TropicalOcean as default
        assert_eq!(config.preset, WaterBodyPreset::TropicalOcean);
        assert_eq!(config.quality, QualityPreset::Ultra);
    }

    #[test]
    fn test_config_default_physics() {
        let config = FluidEditorConfig::default();
        
        // Legacy API compatibility
        assert_eq!(config.smoothing_radius(), config.physics.smoothing_radius);
        assert_eq!(config.target_density(), config.physics.target_density);
        assert_eq!(config.pressure_multiplier(), config.physics.pressure_multiplier);
        assert_eq!(config.viscosity(), config.physics.viscosity);
        assert_eq!(config.iterations(), config.physics.iterations);
    }

    #[test]
    fn test_config_default_thermal() {
        let config = FluidEditorConfig::default();
        
        assert!(config.enable_temperature());
        assert_eq!(config.thermal_diffusivity(), config.thermal.diffusivity);
        assert_eq!(config.buoyancy_coefficient(), config.thermal.buoyancy_coefficient);
    }

    #[test]
    fn test_config_default_rendering() {
        let config = FluidEditorConfig::default();
        
        assert!(config.enable_ssfr());
        assert!(config.enable_caustics());
        assert!(config.enable_temporal());
        assert!(config.caustic_intensity() > 0.0);
    }

    #[test]
    fn test_config_default_lod() {
        let config = FluidEditorConfig::default();
        
        assert_eq!(config.lod_distances().len(), 4);
        assert!(config.lod_distances()[0] > 0.0);
    }

    // ==================== Water Body Preset Tests ====================

    #[test]
    fn test_all_water_body_presets() {
        let presets = [
            WaterBodyPreset::Pool,
            WaterBodyPreset::Lake,
            WaterBodyPreset::River,
            WaterBodyPreset::Ocean,
            WaterBodyPreset::TropicalOcean,
            WaterBodyPreset::Swamp,
            WaterBodyPreset::HotSpring,
            WaterBodyPreset::Waterfall,
            WaterBodyPreset::CaveWater,
            WaterBodyPreset::ArcticWater,
            WaterBodyPreset::Custom,
        ];
        
        for preset in presets {
            let config = FluidEditorConfig::from_preset(preset.clone());
            assert_eq!(config.preset, preset);
            assert!(config.max_particles > 0);
            assert!(config.physics.smoothing_radius > 0.0);
        }
    }

    #[test]
    fn test_pool_preset() {
        let config = FluidEditorConfig::pool();
        
        assert_eq!(config.preset, WaterBodyPreset::Pool);
        assert!(config.caustics.enabled);
        assert!(config.reflections.use_planar);
        assert!(!config.waves.enabled);
        assert!(!config.flow.enabled);
    }

    #[test]
    fn test_river_preset() {
        let config = FluidEditorConfig::river();
        
        assert_eq!(config.preset, WaterBodyPreset::River);
        assert!(config.flow.enabled);
        assert!(config.physics.enable_vorticity);
        assert!(config.foam.enabled);
    }

    #[test]
    fn test_ocean_preset() {
        let config = FluidEditorConfig::ocean();
        
        assert_eq!(config.preset, WaterBodyPreset::Ocean);
        assert!(config.waves.enabled);
        assert!(config.waves.foam_on_peaks);
        assert!(config.god_rays.enabled);
    }

    #[test]
    fn test_waterfall_preset() {
        let config = FluidEditorConfig::waterfall();
        
        assert_eq!(config.preset, WaterBodyPreset::Waterfall);
        assert!(config.waterfall.enabled);
        assert!(config.flow.enabled);
        assert!(!config.emitters.is_empty());
    }

    #[test]
    fn test_swamp_preset() {
        let config = FluidEditorConfig::swamp();
        
        assert_eq!(config.preset, WaterBodyPreset::Swamp);
        assert!(config.physics.viscosity > 15.0); // Thick water
        assert!(!config.caustics.enabled);
        assert!(!config.god_rays.enabled);
    }

    #[test]
    fn test_hot_spring_preset() {
        let config = FluidEditorConfig::hot_spring();
        
        assert_eq!(config.preset, WaterBodyPreset::HotSpring);
        assert!(config.thermal.enabled);
        assert!(config.thermal.ambient_temperature > 30.0);
        assert!(config.thermal.enable_evaporation);
    }

    // ==================== Legacy API Tests ====================

    #[test]
    fn test_config_performance_preset() {
        let config = FluidEditorConfig::performance();
        
        // Performance should have fewer iterations and disabled features
        assert_eq!(config.quality, QualityPreset::Low);
        assert!(!config.enable_caustics());
        assert!(!config.god_rays.enabled);
        assert!(!config.foam.enabled);
    }

    #[test]
    fn test_config_quality_preset() {
        let config = FluidEditorConfig::quality();
        
        // Quality should be TropicalOcean with all features
        assert!(config.enable_caustics());
        assert!(config.enable_temperature());
        assert!(config.caustic_intensity() > 1.0);
    }

    // ==================== Clamp Tests ====================

    #[test]
    fn test_clamp_smoothing_radius() {
        let mut config = FluidEditorConfig::default();
        
        config.physics.smoothing_radius = 0.1;
        config.clamp();
        assert_eq!(config.physics.smoothing_radius, 0.5);
        
        config.physics.smoothing_radius = 10.0;
        config.clamp();
        assert_eq!(config.physics.smoothing_radius, 5.0);
    }

    #[test]
    fn test_clamp_target_density() {
        let mut config = FluidEditorConfig::default();
        
        config.physics.target_density = 0.1;
        config.clamp();
        assert_eq!(config.physics.target_density, 1.0);
        
        config.physics.target_density = 100.0;
        config.clamp();
        assert_eq!(config.physics.target_density, 50.0);
    }

    #[test]
    fn test_clamp_pressure_multiplier() {
        let mut config = FluidEditorConfig::default();
        
        config.physics.pressure_multiplier = 1.0;
        config.clamp();
        assert_eq!(config.physics.pressure_multiplier, 10.0);
        
        config.physics.pressure_multiplier = 2000.0;
        config.clamp();
        assert_eq!(config.physics.pressure_multiplier, 1000.0);
    }

    #[test]
    fn test_clamp_iterations() {
        let mut config = FluidEditorConfig::default();
        
        config.physics.iterations = 0;
        config.clamp();
        assert_eq!(config.physics.iterations, 1);
        
        config.physics.iterations = 100;
        config.clamp();
        assert_eq!(config.physics.iterations, 20);
    }

    #[test]
    fn test_clamp_gravity() {
        let mut config = FluidEditorConfig::default();
        
        config.physics.gravity[1] = -50.0;
        config.clamp();
        assert_eq!(config.physics.gravity[1], -30.0);
        
        config.physics.gravity[1] = 50.0;
        config.clamp();
        assert_eq!(config.physics.gravity[1], 30.0);
    }

    #[test]
    fn test_clamp_temporal_blend() {
        let mut config = FluidEditorConfig::default();
        
        config.rendering.temporal_blend = -0.5;
        config.clamp();
        assert_eq!(config.rendering.temporal_blend, 0.0);
        
        config.rendering.temporal_blend = 1.5;
        config.clamp();
        assert_eq!(config.rendering.temporal_blend, 1.0);
    }

    #[test]
    fn test_clamp_preserves_valid_values() {
        let config_before = FluidEditorConfig::default();
        let mut config = config_before.clone();
        config.clamp();
        
        // Default values should be unchanged after clamp
        assert_eq!(config.physics.smoothing_radius, config_before.physics.smoothing_radius);
        assert_eq!(config.physics.iterations, config_before.physics.iterations);
    }

    // ==================== Serialization Tests ====================

    #[test]
    fn test_config_roundtrip_json() {
        let config = FluidEditorConfig::quality();
        let json = config.to_json().unwrap();
        let loaded = FluidEditorConfig::from_json(&json).unwrap();
        
        assert_eq!(config.physics.iterations, loaded.physics.iterations);
        assert_eq!(config.preset, loaded.preset);
        assert_eq!(config.max_particles, loaded.max_particles);
    }

    #[test]
    fn test_config_to_json() {
        let config = FluidEditorConfig::default();
        let json = config.to_json().unwrap();
        
        assert!(json.contains("physics"));
        assert!(json.contains("thermal"));
        assert!(json.contains("rendering"));
        assert!(json.contains("caustics"));
        assert!(json.contains("waves"));
    }

    #[test]
    fn test_config_roundtrip_toml() {
        let config = FluidEditorConfig::ocean();
        let toml = config.to_toml().unwrap();
        let loaded = FluidEditorConfig::from_toml(&toml).unwrap();
        
        assert_eq!(config.physics.iterations, loaded.physics.iterations);
        assert_eq!(config.preset, loaded.preset);
    }

    #[test]
    fn test_config_from_json_invalid() {
        let json = "not valid json";
        let result = FluidEditorConfig::from_json(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_config_json_preserves_colors() {
        let config = FluidEditorConfig::default();
        let json = config.to_json().unwrap();
        let loaded = FluidEditorConfig::from_json(&json).unwrap();
        
        assert_eq!(config.rendering.fluid_color, loaded.rendering.fluid_color);
        assert_eq!(config.rendering.absorption, loaded.rendering.absorption);
        assert_eq!(config.rendering.scatter_color, loaded.rendering.scatter_color);
    }

    // ==================== Clone/Debug Tests ====================

    #[test]
    fn test_config_clone() {
        let config = FluidEditorConfig::quality();
        let cloned = config.clone();
        
        assert_eq!(config.physics.iterations, cloned.physics.iterations);
        assert_eq!(config.physics.smoothing_radius, cloned.physics.smoothing_radius);
        assert_eq!(config.rendering.fluid_color, cloned.rendering.fluid_color);
    }

    #[test]
    fn test_config_debug() {
        let config = FluidEditorConfig::default();
        let debug_str = format!("{:?}", config);
        
        assert!(debug_str.contains("FluidEditorConfig"));
        assert!(debug_str.contains("physics"));
    }

    // ==================== Emitter Tests ====================

    #[test]
    fn test_emitter_default() {
        let emitter = EmitterEditorConfig::default();
        
        assert!(emitter.enabled);
        assert_eq!(emitter.shape, EmitterShapeType::Point);
        assert!(emitter.rate > 0.0);
    }

    #[test]
    fn test_emitter_fountain() {
        let emitter = EmitterEditorConfig::fountain([0.0, 0.0, 0.0]);
        
        assert_eq!(emitter.name, "Fountain");
        assert!(emitter.velocity[1] > 0.0); // Upward
    }

    #[test]
    fn test_emitter_waterfall_source() {
        let emitter = EmitterEditorConfig::waterfall_source([0.0, 10.0, 0.0], 5.0);
        
        assert_eq!(emitter.name, "Waterfall Source");
        assert_eq!(emitter.shape, EmitterShapeType::Box);
        assert!(emitter.velocity[1] < 0.0); // Downward
    }

    #[test]
    fn test_emitter_rain() {
        let emitter = EmitterEditorConfig::rain([100.0, 100.0], 50.0);
        
        assert_eq!(emitter.name, "Rain");
        assert_eq!(emitter.position[1], 50.0);
        assert!(emitter.velocity[1] < 0.0); // Downward
    }

    // ==================== Drain Tests ====================

    #[test]
    fn test_drain_default() {
        let drain = DrainEditorConfig::default();
        
        assert!(drain.enabled);
        assert!(drain.radius > 0.0);
        assert!(drain.strength > 0.0);
        assert!(drain.show_gizmo);
    }

    // ==================== Config Management Tests ====================

    #[test]
    fn test_add_emitter() {
        let mut config = FluidEditorConfig::default();
        let initial_count = config.emitters.len();
        
        config.add_emitter(EmitterEditorConfig::fountain([0.0, 0.0, 0.0]));
        assert_eq!(config.emitters.len(), initial_count + 1);
    }

    #[test]
    fn test_remove_emitter() {
        let mut config = FluidEditorConfig::default();
        config.add_emitter(EmitterEditorConfig::fountain([0.0, 0.0, 0.0]));
        config.add_emitter(EmitterEditorConfig::rain([10.0, 10.0], 20.0));
        
        let removed = config.remove_emitter(0);
        assert!(removed.is_some());
        assert_eq!(removed.unwrap().name, "Fountain");
    }

    #[test]
    fn test_remove_emitter_out_of_bounds() {
        let mut config = FluidEditorConfig::default();
        let removed = config.remove_emitter(999);
        assert!(removed.is_none());
    }

    #[test]
    fn test_add_drain() {
        let mut config = FluidEditorConfig::default();
        let initial_count = config.drains.len();
        
        config.add_drain(DrainEditorConfig::default());
        assert_eq!(config.drains.len(), initial_count + 1);
    }

    // ==================== Validation Tests ====================

    #[test]
    fn test_validate_returns_warnings() {
        let mut config = FluidEditorConfig::default();
        config.max_particles = 200_000;
        config.physics.iterations = 15;
        
        let warnings = config.validate();
        assert!(!warnings.is_empty());
    }

    #[test]
    fn test_validate_performance_no_warnings() {
        let config = FluidEditorConfig::performance();
        let warnings = config.validate();
        
        // Performance preset should have no warnings
        assert!(warnings.is_empty());
    }

    // ==================== Performance Cost Tests ====================

    #[test]
    fn test_estimated_performance_cost_low() {
        let config = FluidEditorConfig::performance();
        let cost = config.estimated_performance_cost();
        
        assert!(cost < 30, "Performance preset should have low cost");
    }

    #[test]
    fn test_estimated_performance_cost_high() {
        let config = FluidEditorConfig::quality();
        let cost = config.estimated_performance_cost();
        
        assert!(cost > 30, "Quality preset should have higher cost");
    }

    // ==================== Wave Config Tests ====================

    #[test]
    fn test_wave_calm() {
        let wave = WaveConfig::calm();
        
        assert!(wave.amplitude < 0.5);
        assert!(!wave.foam_on_peaks);
    }

    #[test]
    fn test_wave_stormy() {
        let wave = WaveConfig::stormy();
        
        assert!(wave.amplitude > 1.0);
        assert!(wave.foam_on_peaks);
        assert!(wave.octaves > 4);
    }

    // ==================== LOD Tests ====================

    #[test]
    fn test_config_lod_distances_monotonic() {
        let config = FluidEditorConfig::default();
        
        // LOD distances should be increasing
        assert!(config.lod.distances[0] < config.lod.distances[1]);
        assert!(config.lod.distances[1] < config.lod.distances[2]);
        assert!(config.lod.distances[2] < config.lod.distances[3]);
    }

    // ==================== Color Validity Tests ====================

    #[test]
    fn test_config_color_valid_range() {
        let config = FluidEditorConfig::default();
        
        for c in &config.rendering.fluid_color {
            assert!(*c >= 0.0 && *c <= 1.0);
        }
    }

    // ==================== All Presets Valid After Clamp ====================

    #[test]
    fn test_all_presets_valid_after_clamp() {
        let presets = [
            FluidEditorConfig::pool(),
            FluidEditorConfig::lake(),
            FluidEditorConfig::river(),
            FluidEditorConfig::ocean(),
            FluidEditorConfig::tropical_ocean(),
            FluidEditorConfig::swamp(),
            FluidEditorConfig::hot_spring(),
            FluidEditorConfig::waterfall(),
            FluidEditorConfig::cave_water(),
            FluidEditorConfig::arctic_water(),
            FluidEditorConfig::custom(),
            FluidEditorConfig::performance(),
            FluidEditorConfig::quality(),
        ];
        
        for mut preset in presets {
            let original_iterations = preset.physics.iterations;
            preset.clamp();
            // Presets should already be within valid ranges
            assert_eq!(preset.physics.iterations, original_iterations);
        }
    }

    #[test]
    fn test_config_viscosity_range() {
        let mut config = FluidEditorConfig::default();
        
        // Test lower bound
        config.physics.viscosity = -5.0;
        config.clamp();
        assert_eq!(config.physics.viscosity, 0.0);
        
        // Test upper bound
        config.physics.viscosity = 200.0;
        config.clamp();
        assert_eq!(config.physics.viscosity, 100.0);
    }

    #[test]
    fn test_config_surface_tension_range() {
        let mut config = FluidEditorConfig::default();
        
        config.physics.surface_tension = -0.5;
        config.clamp();
        assert_eq!(config.physics.surface_tension, 0.0);
        
        config.physics.surface_tension = 5.0;
        config.clamp();
        assert_eq!(config.physics.surface_tension, 1.0);
    }

    #[test]
    fn test_config_caustic_intensity_range() {
        let mut config = FluidEditorConfig::default();
        
        config.caustics.intensity = -1.0;
        config.clamp();
        assert_eq!(config.caustics.intensity, 0.0);
        
        config.caustics.intensity = 10.0;
        config.clamp();
        assert_eq!(config.caustics.intensity, 5.0);
    }

    // ==================== Physics Config Tests ====================

    #[test]
    fn test_physics_config_default() {
        let physics = PhysicsConfig::default();
        
        assert_eq!(physics.smoothing_radius, 1.0);
        assert_eq!(physics.iterations, 4);
        assert!(physics.enable_vorticity);
    }

    #[test]
    fn test_physics_config_clamp() {
        let mut physics = PhysicsConfig::default();
        physics.smoothing_radius = 100.0;
        physics.clamp();
        assert_eq!(physics.smoothing_radius, 5.0);
    }

    // ==================== Thermal Config Tests ====================

    #[test]
    fn test_thermal_config_default() {
        let thermal = ThermalConfig::default();
        
        assert!(thermal.enabled);
        assert_eq!(thermal.ambient_temperature, 20.0);
    }

    #[test]
    fn test_thermal_config_clamp() {
        let mut thermal = ThermalConfig::default();
        thermal.ambient_temperature = 500.0;
        thermal.clamp();
        assert_eq!(thermal.ambient_temperature, 150.0);
    }

    // ==================== Legacy API Compatibility ====================

    #[test]
    fn test_legacy_api_accessors() {
        let config = FluidEditorConfig::default();
        
        // All legacy accessors should work
        let _ = config.smoothing_radius();
        let _ = config.target_density();
        let _ = config.pressure_multiplier();
        let _ = config.viscosity();
        let _ = config.surface_tension();
        let _ = config.gravity();
        let _ = config.iterations();
        let _ = config.enable_temperature();
        let _ = config.thermal_diffusivity();
        let _ = config.buoyancy_coefficient();
        let _ = config.enable_ssfr();
        let _ = config.fluid_color();
        let _ = config.absorption();
        let _ = config.scatter_color();
        let _ = config.enable_caustics();
        let _ = config.caustic_intensity();
        let _ = config.enable_temporal();
        let _ = config.temporal_blend();
        let _ = config.lod_distances();
    }

    // ==================== Widget Metadata Tests ====================

    #[test]
    fn test_editor_metadata_all_fields() {
        let fields = EditorMetadata::all_fields();
        assert!(!fields.is_empty());
        assert!(fields.len() >= 15); // At least 15 documented fields
    }

    #[test]
    fn test_editor_metadata_categories() {
        let categories = EditorMetadata::categories();
        assert!(categories.contains(&"Physics"));
        assert!(categories.contains(&"Visual Effects"));
        assert!(categories.contains(&"Rendering"));
        assert!(categories.contains(&"Waves"));
        assert!(categories.contains(&"Performance"));
    }

    #[test]
    fn test_editor_metadata_fields_for_category() {
        let physics_fields = EditorMetadata::fields_for_category("Physics");
        assert!(!physics_fields.is_empty());
        
        for field in physics_fields {
            assert_eq!(field.category, "Physics");
        }
    }

    #[test]
    fn test_editor_metadata_simple_fields() {
        let simple = EditorMetadata::simple_fields();
        let all = EditorMetadata::all_fields();
        
        // Simple should have fewer fields than all
        assert!(simple.len() <= all.len());
        
        // None of the simple fields should be marked advanced
        for field in simple {
            assert!(!field.advanced);
        }
    }

    #[test]
    fn test_field_metadata_has_tooltips() {
        let fields = EditorMetadata::all_fields();
        
        for field in fields {
            assert!(!field.tooltip.is_empty(), "Field {} should have tooltip", field.name);
        }
    }

    #[test]
    fn test_widget_type_slider() {
        let field = EditorMetadata::smoothing_radius();
        
        match field.widget {
            WidgetType::Slider { min, max, step } => {
                assert!(min < max);
                assert!(step > 0.0);
            }
            _ => panic!("Expected Slider widget type"),
        }
    }

    #[test]
    fn test_widget_type_toggle() {
        let field = EditorMetadata::caustics_enabled();
        
        assert!(matches!(field.widget, WidgetType::Toggle));
    }

    #[test]
    fn test_widget_type_color() {
        let field = EditorMetadata::fluid_color();
        
        assert!(matches!(field.widget, WidgetType::ColorRgba));
    }

    // ==================== Configuration History Tests ====================

    #[test]
    fn test_config_history_new() {
        let config = FluidEditorConfig::default();
        let history = ConfigHistory::new(config.clone());
        
        assert!(!history.can_undo());
        assert!(!history.can_redo());
        assert_eq!(history.undo_count(), 0);
        assert_eq!(history.redo_count(), 0);
    }

    #[test]
    fn test_config_history_push() {
        let config = FluidEditorConfig::default();
        let mut history = ConfigHistory::new(config.clone());
        
        let mut new_config = config.clone();
        new_config.max_particles = 50_000;
        history.push(new_config);
        
        assert!(history.can_undo());
        assert!(!history.can_redo());
        assert_eq!(history.undo_count(), 1);
    }

    #[test]
    fn test_config_history_undo_redo() {
        let config = FluidEditorConfig::default();
        let mut history = ConfigHistory::new(config.clone());
        
        let mut config2 = config.clone();
        config2.max_particles = 50_000;
        history.push(config2);
        
        // Undo
        let undone = history.undo();
        assert!(undone.is_some());
        assert!(history.can_redo());
        assert!(!history.can_undo());
        
        // Redo
        let redone = history.redo();
        assert!(redone.is_some());
        assert_eq!(redone.unwrap().max_particles, 50_000);
    }

    #[test]
    fn test_config_history_max_size() {
        let config = FluidEditorConfig::default();
        let mut history = ConfigHistory::with_max_size(config.clone(), 5);
        
        // Push more than max size
        for i in 0..10 {
            let mut c = config.clone();
            c.max_particles = i * 1000;
            history.push(c);
        }
        
        // Should only keep max_size entries
        assert!(history.undo_count() <= 5);
    }

    #[test]
    fn test_config_history_clear() {
        let config = FluidEditorConfig::default();
        let mut history = ConfigHistory::new(config.clone());
        
        history.push(FluidEditorConfig::pool());
        history.push(FluidEditorConfig::ocean());
        
        history.clear();
        
        assert!(!history.can_undo());
        assert!(!history.can_redo());
    }

    // ==================== AABB Tests ====================

    #[test]
    fn test_aabb_new() {
        let aabb = FluidAABB::new([0.0, 0.0, 0.0], [10.0, 5.0, 10.0]);
        
        assert_eq!(aabb.min, [0.0, 0.0, 0.0]);
        assert_eq!(aabb.max, [10.0, 5.0, 10.0]);
    }

    #[test]
    fn test_aabb_from_center_extents() {
        let aabb = FluidAABB::from_center_extents([5.0, 2.5, 5.0], [5.0, 2.5, 5.0]);
        
        assert_eq!(aabb.min, [0.0, 0.0, 0.0]);
        assert_eq!(aabb.max, [10.0, 5.0, 10.0]);
    }

    #[test]
    fn test_aabb_center() {
        let aabb = FluidAABB::new([0.0, 0.0, 0.0], [10.0, 10.0, 10.0]);
        let center = aabb.center();
        
        assert_eq!(center, [5.0, 5.0, 5.0]);
    }

    #[test]
    fn test_aabb_size() {
        let aabb = FluidAABB::new([0.0, 0.0, 0.0], [10.0, 5.0, 20.0]);
        let size = aabb.size();
        
        assert_eq!(size, [10.0, 5.0, 20.0]);
    }

    #[test]
    fn test_aabb_volume() {
        let aabb = FluidAABB::new([0.0, 0.0, 0.0], [2.0, 3.0, 4.0]);
        
        assert_eq!(aabb.volume(), 24.0);
    }

    #[test]
    fn test_aabb_contains_point() {
        let aabb = FluidAABB::new([0.0, 0.0, 0.0], [10.0, 10.0, 10.0]);
        
        assert!(aabb.contains_point([5.0, 5.0, 5.0]));
        assert!(aabb.contains_point([0.0, 0.0, 0.0]));
        assert!(aabb.contains_point([10.0, 10.0, 10.0]));
        assert!(!aabb.contains_point([15.0, 5.0, 5.0]));
        assert!(!aabb.contains_point([-1.0, 5.0, 5.0]));
    }

    #[test]
    fn test_aabb_overlaps() {
        let aabb1 = FluidAABB::new([0.0, 0.0, 0.0], [10.0, 10.0, 10.0]);
        let aabb2 = FluidAABB::new([5.0, 5.0, 5.0], [15.0, 15.0, 15.0]);
        let aabb3 = FluidAABB::new([20.0, 20.0, 20.0], [30.0, 30.0, 30.0]);
        
        assert!(aabb1.overlaps(&aabb2));
        assert!(!aabb1.overlaps(&aabb3));
    }

    #[test]
    fn test_aabb_expand() {
        let mut aabb = FluidAABB::new([0.0, 0.0, 0.0], [10.0, 10.0, 10.0]);
        aabb.expand([15.0, 5.0, 5.0]);
        
        assert_eq!(aabb.max[0], 15.0);
    }

    #[test]
    fn test_aabb_merge() {
        let aabb1 = FluidAABB::new([0.0, 0.0, 0.0], [5.0, 5.0, 5.0]);
        let aabb2 = FluidAABB::new([3.0, 3.0, 3.0], [10.0, 10.0, 10.0]);
        let merged = aabb1.merge(&aabb2);
        
        assert_eq!(merged.min, [0.0, 0.0, 0.0]);
        assert_eq!(merged.max, [10.0, 10.0, 10.0]);
    }

    // ==================== Performance Metrics Tests ====================

    #[test]
    fn test_performance_metrics_total_time() {
        let metrics = FluidPerformanceMetrics {
            physics_time_ms: 2.0,
            render_time_ms: 3.0,
            spawn_time_ms: 0.5,
            ..Default::default()
        };
        
        assert_eq!(metrics.total_time_ms(), 5.5);
    }

    #[test]
    fn test_performance_metrics_grade() {
        let mut metrics = FluidPerformanceMetrics::default();
        
        metrics.physics_time_ms = 1.0;
        assert_eq!(metrics.grade(), 'A');
        
        metrics.physics_time_ms = 3.0;
        assert_eq!(metrics.grade(), 'B');
        
        metrics.physics_time_ms = 6.0;
        assert_eq!(metrics.grade(), 'C');
        
        metrics.physics_time_ms = 10.0;
        assert_eq!(metrics.grade(), 'D');
        
        metrics.physics_time_ms = 15.0;
        assert_eq!(metrics.grade(), 'F');
    }

    #[test]
    fn test_performance_metrics_within_budget() {
        let metrics = FluidPerformanceMetrics {
            physics_time_ms: 5.0,
            render_time_ms: 5.0,
            spawn_time_ms: 1.0,
            ..Default::default()
        };
        
        assert!(metrics.is_within_budget(16.67)); // 60 FPS
        assert!(!metrics.is_within_budget(10.0));
    }

    #[test]
    fn test_performance_metrics_summary() {
        let metrics = FluidPerformanceMetrics {
            active_particles: 50_000,
            physics_time_ms: 2.5,
            render_time_ms: 1.5,
            gpu_memory_bytes: 100 * 1024 * 1024, // 100 MB
            ..Default::default()
        };
        
        let summary = metrics.summary();
        assert!(summary.contains("50000"));
        assert!(summary.contains("2.50"));
    }

    // ==================== Accessibility Tests ====================

    #[test]
    fn test_colorblind_palette_water_colors() {
        let palettes = ColorblindPalette::all_palettes();
        
        for palette in palettes {
            let color = palette.water_color();
            for c in color {
                assert!(c >= 0.0 && c <= 1.0);
            }
        }
    }

    #[test]
    fn test_colorblind_palette_descriptions() {
        let palettes = ColorblindPalette::all_palettes();
        
        for palette in palettes {
            let desc = palette.description();
            assert!(!desc.is_empty());
        }
    }

    #[test]
    fn test_accessibility_settings_default() {
        let settings = AccessibilitySettings::default();
        
        assert_eq!(settings.palette, ColorblindPalette::Standard);
        assert!(settings.show_slider_values);
        assert!(!settings.large_fonts);
        assert!(!settings.reduce_motion);
    }

    // ==================== Preview Hint Tests ====================

    #[test]
    fn test_preview_hint_wave_amplitude() {
        let hint = PreviewHint::wave_amplitude(0.1);
        assert!(hint.effect_description.contains("Calm"));
        
        let hint = PreviewHint::wave_amplitude(0.5);
        assert!(hint.effect_description.contains("Moderate"));
        
        let hint = PreviewHint::wave_amplitude(2.0);
        assert!(hint.effect_description.contains("Large"));
    }

    #[test]
    fn test_preview_hint_particle_count() {
        let hint = PreviewHint::particle_count(5_000);
        assert!(hint.performance_impact == "Low");
        
        let hint = PreviewHint::particle_count(30_000);
        assert!(hint.performance_impact == "Medium");
        
        let hint = PreviewHint::particle_count(100_000);
        assert!(hint.performance_impact == "High");
    }

    #[test]
    fn test_preview_hint_viscosity() {
        let hint = PreviewHint::viscosity(2.0);
        assert!(hint.effect_description.contains("water"));
        
        let hint = PreviewHint::viscosity(15.0);
        assert!(hint.effect_description.contains("oil"));
        
        let hint = PreviewHint::viscosity(40.0);
        assert!(hint.effect_description.contains("honey"));
    }

    // ==================== Batch Operation Tests ====================

    #[test]
    fn test_batch_multiply_viscosity() {
        let mut configs = vec![
            FluidEditorConfig::pool(),
            FluidEditorConfig::ocean(),
        ];
        
        let original = configs[0].physics.viscosity;
        BatchOperation::multiply_viscosity(&mut configs, &[0], 2.0);
        
        assert_eq!(configs[0].physics.viscosity, original * 2.0);
    }

    #[test]
    fn test_batch_set_caustics() {
        let mut configs = vec![
            FluidEditorConfig::pool(),
            FluidEditorConfig::ocean(),
        ];
        
        BatchOperation::set_caustics(&mut configs, &[0, 1], false);
        
        assert!(!configs[0].caustics.enabled);
        assert!(!configs[1].caustics.enabled);
    }

    #[test]
    fn test_batch_set_quality() {
        let mut configs = vec![
            FluidEditorConfig::pool(),
            FluidEditorConfig::ocean(),
        ];
        
        BatchOperation::set_quality(&mut configs, &[0, 1], QualityPreset::Low);
        
        assert_eq!(configs[0].max_particles, 5_000);
        assert_eq!(configs[1].max_particles, 5_000);
    }

    #[test]
    fn test_batch_apply_palette() {
        let mut configs = vec![FluidEditorConfig::pool()];
        
        BatchOperation::apply_palette(&mut configs, &[0], ColorblindPalette::Deuteranopia);
        
        assert_eq!(configs[0].rendering.fluid_color, ColorblindPalette::Deuteranopia.water_color());
    }

    // ==================== Interpolation Tests ====================

    #[test]
    fn test_config_interpolate_zero() {
        let config1 = FluidEditorConfig::pool();
        let config2 = FluidEditorConfig::ocean();
        
        let result = config1.interpolate(&config2, 0.0);
        
        assert_eq!(result.max_particles, config1.max_particles);
    }

    #[test]
    fn test_config_interpolate_one() {
        let config1 = FluidEditorConfig::pool();
        let config2 = FluidEditorConfig::ocean();
        
        let result = config1.interpolate(&config2, 1.0);
        
        assert_eq!(result.max_particles, config2.max_particles);
    }

    #[test]
    fn test_config_interpolate_half() {
        let config1 = FluidEditorConfig::custom();
        let mut config2 = FluidEditorConfig::custom();
        config1.physics.viscosity;
        config2.physics.viscosity = 20.0;
        
        let result = config1.interpolate(&config2, 0.5);
        
        // Should be halfway between
        let expected = (config1.physics.viscosity + config2.physics.viscosity) / 2.0;
        assert!((result.physics.viscosity - expected).abs() < 0.01);
    }

    #[test]
    fn test_config_smooth_transition() {
        let config1 = FluidEditorConfig::pool();
        let config2 = FluidEditorConfig::ocean();
        
        let result = config1.smooth_transition(&config2, 0.5, 1.0);
        
        // Should be smoothstepped
        assert!(result.max_particles > config1.max_particles);
        assert!(result.max_particles < config2.max_particles);
    }

    #[test]
    fn test_config_diff() {
        let config1 = FluidEditorConfig::pool();
        let config2 = FluidEditorConfig::ocean();
        
        let diffs = config1.diff(&config2);
        
        assert!(!diffs.is_empty());
        // Should detect preset difference
        assert!(diffs.iter().any(|d| d.contains("Preset")));
    }

    #[test]
    fn test_config_apply_quality_preset() {
        let mut config = FluidEditorConfig::default();
        
        config.apply_quality_preset(QualityPreset::Low);
        
        assert_eq!(config.max_particles, 5_000);
        assert!(!config.caustics.enabled);
        assert!(!config.god_rays.enabled);
    }

    #[test]
    fn test_config_summary() {
        let config = FluidEditorConfig::ocean();
        let summary = config.summary();
        
        assert!(summary.contains("Ocean"));
        assert!(summary.contains("50K")); // 50,000 particles
    }

    // ==================== Quality Preset Extended Tests ====================

    #[test]
    fn test_quality_preset_all_presets() {
        let presets = QualityPreset::all_presets();
        assert_eq!(presets.len(), 4);
    }

    #[test]
    fn test_quality_preset_god_ray_samples() {
        assert_eq!(QualityPreset::Low.recommended_god_ray_samples(), 8);
        assert_eq!(QualityPreset::Medium.recommended_god_ray_samples(), 16);
        assert_eq!(QualityPreset::High.recommended_god_ray_samples(), 32);
        assert_eq!(QualityPreset::Ultra.recommended_god_ray_samples(), 48);
    }

    #[test]
    fn test_quality_preset_reflection_resolution() {
        assert_eq!(QualityPreset::Low.recommended_reflection_resolution(), 256);
        assert_eq!(QualityPreset::Medium.recommended_reflection_resolution(), 512);
        assert_eq!(QualityPreset::High.recommended_reflection_resolution(), 1024);
        assert_eq!(QualityPreset::Ultra.recommended_reflection_resolution(), 2048);
    }

    // ==================== Scene Placement Tests ====================

    #[test]
    fn test_scene_placement_default() {
        let placement = FluidScenePlacement::default();
        
        assert!(!placement.id.is_empty());
        assert!(placement.active);
        assert!(placement.tags.contains(&"water".to_string()));
    }

    // ==================== Validation System Tests ====================

    #[test]
    fn test_validator_new() {
        let validator = ConfigValidator::new();
        
        assert_eq!(validator.target_fps, 60.0);
        assert!(!validator.strict_mode);
    }

    #[test]
    fn test_validator_strict() {
        let validator = ConfigValidator::strict();
        
        assert!(validator.strict_mode);
        assert!(validator.max_gpu_memory < 512 * 1024 * 1024);
    }

    #[test]
    fn test_validator_valid_config() {
        let validator = ConfigValidator::new();
        // Use a low-particle config to avoid performance warnings
        let mut config = FluidEditorConfig::pool();
        config.max_particles = 10_000;
        
        let issues = validator.validate(&config);
        
        // Low particle count should have no errors
        let errors = issues.iter().filter(|i| i.severity == ValidationSeverity::Error).count();
        assert_eq!(errors, 0);
    }

    #[test]
    fn test_validator_high_gravity() {
        let validator = ConfigValidator::new();
        let mut config = FluidEditorConfig::pool();
        config.max_particles = 5_000; // Low particles to focus on gravity
        config.physics.gravity = [0.0, -100.0, 0.0];
        
        let issues = validator.validate(&config);
        
        // Should warn about high gravity (case insensitive check)
        assert!(issues.iter().any(|i| i.message.to_lowercase().contains("gravity")));
    }

    #[test]
    fn test_validator_invisible_water() {
        let validator = ConfigValidator::new();
        let mut config = FluidEditorConfig::pool();
        config.rendering.fluid_color = [0.2, 0.5, 0.8, 0.0]; // Zero alpha
        
        let issues = validator.validate(&config);
        
        // Should warn about invisible water
        assert!(issues.iter().any(|i| i.message.contains("invisible")));
    }

    #[test]
    fn test_validator_has_errors() {
        let validator = ConfigValidator::new();
        // Use low particle count to ensure no performance errors
        let mut config = FluidEditorConfig::pool();
        config.max_particles = 10_000;
        
        // Low particle pool config should not have errors
        assert!(!validator.has_errors(&config));
    }

    // ==================== Easing Function Tests ====================

    #[test]
    fn test_easing_linear() {
        let ease = EasingFunction::Linear;
        
        assert_eq!(ease.apply(0.0), 0.0);
        assert_eq!(ease.apply(0.5), 0.5);
        assert_eq!(ease.apply(1.0), 1.0);
    }

    #[test]
    fn test_easing_ease_in() {
        let ease = EasingFunction::EaseIn;
        
        assert_eq!(ease.apply(0.0), 0.0);
        assert!(ease.apply(0.5) < 0.5); // Slow start
        assert_eq!(ease.apply(1.0), 1.0);
    }

    #[test]
    fn test_easing_ease_out() {
        let ease = EasingFunction::EaseOut;
        
        assert_eq!(ease.apply(0.0), 0.0);
        assert!(ease.apply(0.5) > 0.5); // Fast start
        assert_eq!(ease.apply(1.0), 1.0);
    }

    #[test]
    fn test_easing_ease_in_out() {
        let ease = EasingFunction::EaseInOut;
        
        assert_eq!(ease.apply(0.0), 0.0);
        assert!((ease.apply(0.5) - 0.5).abs() < 0.1); // Near middle
        assert_eq!(ease.apply(1.0), 1.0);
    }

    #[test]
    fn test_easing_clamp() {
        let ease = EasingFunction::Linear;
        
        // Should clamp out-of-range values
        assert_eq!(ease.apply(-0.5), 0.0);
        assert_eq!(ease.apply(1.5), 1.0);
    }

    #[test]
    fn test_easing_all_functions() {
        let functions = EasingFunction::all_functions();
        assert_eq!(functions.len(), 8);
    }

    #[test]
    fn test_easing_descriptions() {
        for ease in EasingFunction::all_functions() {
            assert!(!ease.description().is_empty());
        }
    }

    // ==================== Config Transition Tests ====================

    #[test]
    fn test_transition_new() {
        let from = FluidEditorConfig::pool();
        let to = FluidEditorConfig::ocean();
        
        let transition = ConfigTransition::new(from.clone(), to.clone(), 1.0);
        
        assert_eq!(transition.duration, 1.0);
        assert_eq!(transition.elapsed, 0.0);
        assert!(!transition.complete);
    }

    #[test]
    fn test_transition_update() {
        let from = FluidEditorConfig::pool();
        let to = FluidEditorConfig::ocean();
        
        let mut transition = ConfigTransition::new(from.clone(), to.clone(), 1.0);
        
        // Update halfway
        let result = transition.update(0.5);
        
        assert!(!transition.complete);
        assert!(result.max_particles > from.max_particles);
        assert!(result.max_particles < to.max_particles);
    }

    #[test]
    fn test_transition_complete() {
        let from = FluidEditorConfig::pool();
        let to = FluidEditorConfig::ocean();
        
        let mut transition = ConfigTransition::new(from.clone(), to.clone(), 1.0);
        
        // Update past end
        let result = transition.update(2.0);
        
        assert!(transition.complete);
        assert_eq!(result.max_particles, to.max_particles);
    }

    #[test]
    fn test_transition_skip() {
        let from = FluidEditorConfig::pool();
        let to = FluidEditorConfig::ocean();
        
        let mut transition = ConfigTransition::new(from.clone(), to.clone(), 1.0);
        transition.skip();
        
        assert!(transition.complete);
        assert_eq!(transition.progress(), 1.0);
    }

    #[test]
    fn test_transition_with_easing() {
        let from = FluidEditorConfig::pool();
        let to = FluidEditorConfig::ocean();
        
        let transition = ConfigTransition::new(from, to, 1.0)
            .with_easing(EasingFunction::EaseOutElastic);
        
        assert_eq!(transition.easing, EasingFunction::EaseOutElastic);
    }

    // ==================== Debug Visualization Tests ====================

    #[test]
    fn test_debug_viz_default() {
        let viz = DebugVisualization::default();
        
        // Default should have some but not all enabled
        assert!(viz.show_bounds);
        assert!(viz.show_emitters);
        assert!(!viz.show_particles);
    }

    #[test]
    fn test_debug_viz_none() {
        let viz = DebugVisualization::none();
        
        assert!(!viz.any_enabled());
    }

    #[test]
    fn test_debug_viz_physics() {
        let viz = DebugVisualization::physics();
        
        assert!(viz.show_particles);
        assert!(viz.show_velocities);
        assert!(viz.show_pressure);
        assert!(viz.any_enabled());
    }

    #[test]
    fn test_debug_viz_rendering() {
        let viz = DebugVisualization::rendering();
        
        assert!(viz.show_normals);
        assert!(viz.show_performance);
    }

    // ==================== Keyboard Shortcuts Tests ====================

    #[test]
    fn test_keyboard_shortcuts_exist() {
        let shortcuts = editor_shortcuts();
        
        assert!(!shortcuts.is_empty());
        assert!(shortcuts.len() >= 10);
    }

    #[test]
    fn test_keyboard_shortcut_display() {
        let shortcuts = editor_shortcuts();
        
        // Find undo shortcut
        let undo = shortcuts.iter().find(|s| s.action == "Undo").unwrap();
        
        assert_eq!(undo.display(), "Ctrl+Z");
    }

    #[test]
    fn test_keyboard_shortcuts_categories() {
        let shortcuts = editor_shortcuts();
        
        let categories: Vec<_> = shortcuts.iter().map(|s| s.category).collect();
        
        assert!(categories.contains(&"Edit"));
        assert!(categories.contains(&"View"));
        assert!(categories.contains(&"Simulation"));
        assert!(categories.contains(&"File"));
    }

    // ==================== Preset Export/Import Tests ====================

    #[test]
    fn test_exported_preset_from_config() {
        let config = FluidEditorConfig::ocean();
        let preset = ExportedPreset::from_config("My Ocean", config);
        
        assert_eq!(preset.name, "My Ocean");
        assert_eq!(preset.version, ExportedPreset::CURRENT_VERSION);
        assert!(preset.created_at > 0);
    }

    #[test]
    fn test_exported_preset_with_metadata() {
        let config = FluidEditorConfig::ocean();
        let preset = ExportedPreset::from_config("Ocean", config)
            .with_author("Test Author")
            .with_description("A nice ocean")
            .with_tags(vec!["ocean".to_string(), "waves".to_string()]);
        
        assert_eq!(preset.author, Some("Test Author".to_string()));
        assert_eq!(preset.description, Some("A nice ocean".to_string()));
        assert_eq!(preset.tags.len(), 2);
    }

    #[test]
    fn test_exported_preset_json_roundtrip() {
        let config = FluidEditorConfig::pool();
        let preset = ExportedPreset::from_config("Pool", config);
        
        let json = preset.to_json().unwrap();
        let restored = ExportedPreset::from_json(&json).unwrap();
        
        assert_eq!(restored.name, "Pool");
        assert_eq!(restored.config.preset, WaterBodyPreset::Pool);
    }

    #[test]
    fn test_exported_preset_toml_roundtrip() {
        let config = FluidEditorConfig::lake();
        let preset = ExportedPreset::from_config("Lake", config);
        
        let toml = preset.to_toml().unwrap();
        let restored = ExportedPreset::from_toml(&toml).unwrap();
        
        assert_eq!(restored.name, "Lake");
        assert_eq!(restored.config.preset, WaterBodyPreset::Lake);
    }

    // ==================== Clipboard Tests ====================

    #[test]
    fn test_clipboard_new() {
        let clipboard = ConfigClipboard::new();
        
        assert!(!clipboard.has_content());
        assert!(clipboard.paste().is_none());
    }

    #[test]
    fn test_clipboard_copy_paste() {
        let mut clipboard = ConfigClipboard::new();
        let config = FluidEditorConfig::ocean();
        
        clipboard.copy(&config);
        
        assert!(clipboard.has_content());
        
        let pasted = clipboard.paste().unwrap();
        assert_eq!(pasted.preset, WaterBodyPreset::Ocean);
    }

    #[test]
    fn test_clipboard_clear() {
        let mut clipboard = ConfigClipboard::new();
        clipboard.copy(&FluidEditorConfig::pool());
        
        clipboard.clear();
        
        assert!(!clipboard.has_content());
    }

    #[test]
    fn test_clipboard_to_system() {
        let mut clipboard = ConfigClipboard::new();
        clipboard.copy(&FluidEditorConfig::river());
        
        let json = clipboard.to_system_clipboard().unwrap();
        
        assert!(json.contains("River"));
    }

    #[test]
    fn test_clipboard_from_system() {
        let config = FluidEditorConfig::lake();
        let json = serde_json::to_string(&config).unwrap();
        
        let restored = ConfigClipboard::from_system_clipboard(&json).unwrap();
        
        assert_eq!(restored.preset, WaterBodyPreset::Lake);
    }

    // ==================== Batch Operation Extended Tests ====================

    #[test]
    fn test_batch_reset_physics() {
        let mut configs = vec![FluidEditorConfig::pool()];
        configs[0].physics.viscosity = 999.0;
        
        BatchOperation::reset_physics(&mut configs, &[0]);
        
        // Should reset to pool defaults
        let expected = FluidEditorConfig::pool();
        assert!((configs[0].physics.viscosity - expected.physics.viscosity).abs() < 0.01);
    }

    #[test]
    fn test_batch_reset_visuals() {
        let mut configs = vec![FluidEditorConfig::ocean()];
        configs[0].caustics.enabled = false;
        configs[0].foam.enabled = false;
        
        BatchOperation::reset_visuals(&mut configs, &[0]);
        
        // Should reset to ocean defaults
        let expected = FluidEditorConfig::ocean();
        assert_eq!(configs[0].caustics.enabled, expected.caustics.enabled);
        assert_eq!(configs[0].foam.enabled, expected.foam.enabled);
    }
}
