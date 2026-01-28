//! Building Components for Water System
//!
//! Implements water dispensers, drains, gates, and water wheels
//! for building-based water manipulation (inspired by Enshrouded).

use glam::{IVec3, Vec3};
use serde::{Deserialize, Serialize};

use crate::volume_grid::{CellFlags, WaterVolumeGrid};

/// Direction for water emission/flow
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum FlowDirection {
    /// Emit downward (gravity)
    #[default]
    Down,
    /// Emit upward (pressure fountain)
    Up,
    /// Emit in positive X direction
    East,
    /// Emit in negative X direction
    West,
    /// Emit in positive Z direction
    South,
    /// Emit in negative Z direction
    North,
}

impl FlowDirection {
    /// Convert to unit vector
    pub fn to_vec3(&self) -> Vec3 {
        match self {
            FlowDirection::Down => Vec3::new(0.0, -1.0, 0.0),
            FlowDirection::Up => Vec3::new(0.0, 1.0, 0.0),
            FlowDirection::East => Vec3::new(1.0, 0.0, 0.0),
            FlowDirection::West => Vec3::new(-1.0, 0.0, 0.0),
            FlowDirection::South => Vec3::new(0.0, 0.0, 1.0),
            FlowDirection::North => Vec3::new(0.0, 0.0, -1.0),
        }
    }

    /// Convert to integer offset
    pub fn to_ivec3(&self) -> IVec3 {
        match self {
            FlowDirection::Down => IVec3::new(0, -1, 0),
            FlowDirection::Up => IVec3::new(0, 1, 0),
            FlowDirection::East => IVec3::new(1, 0, 0),
            FlowDirection::West => IVec3::new(-1, 0, 0),
            FlowDirection::South => IVec3::new(0, 0, 1),
            FlowDirection::North => IVec3::new(0, 0, -1),
        }
    }
}

/// Water dispenser - generates water at a configurable rate
///
/// Matches Enshrouded's water dispenser with 36 blocks/second flow rate
/// and auto-shutoff when water reaches the dispenser level.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WaterDispenser {
    /// Grid position of the dispenser
    pub position: IVec3,
    /// Flow rate in blocks per second (default: 36.0 like Enshrouded)
    pub flow_rate: f32,
    /// Whether the dispenser is currently active
    pub active: bool,
    /// Auto-shutoff when water reaches dispenser level
    pub auto_shutoff: bool,
    /// Direction of water emission
    pub direction: FlowDirection,
    /// Whether dispenser is infinite (creative) or requires water source
    pub infinite: bool,
    /// Remaining water if not infinite
    pub remaining_water: f32,
}

impl Default for WaterDispenser {
    fn default() -> Self {
        Self {
            position: IVec3::ZERO,
            flow_rate: 36.0, // Enshrouded default
            active: true,
            auto_shutoff: true,
            direction: FlowDirection::Down,
            infinite: true,
            remaining_water: 1000.0,
        }
    }
}

impl WaterDispenser {
    /// Create a new water dispenser at the given position
    pub fn new(position: IVec3) -> Self {
        Self {
            position,
            ..Default::default()
        }
    }

    /// Set flow rate
    pub fn with_flow_rate(mut self, rate: f32) -> Self {
        self.flow_rate = rate;
        self
    }

    /// Set flow direction
    pub fn with_direction(mut self, direction: FlowDirection) -> Self {
        self.direction = direction;
        self
    }

    /// Set auto-shutoff behavior
    pub fn with_auto_shutoff(mut self, enabled: bool) -> Self {
        self.auto_shutoff = enabled;
        self
    }

    /// Set finite water supply
    pub fn with_finite_supply(mut self, amount: f32) -> Self {
        self.infinite = false;
        self.remaining_water = amount;
        self
    }

    /// Update the dispenser, adding water to the grid
    pub fn tick(&mut self, water_grid: &mut WaterVolumeGrid, dt: f32) {
        if !self.active {
            return;
        }

        // Check auto-shutoff
        if self.auto_shutoff {
            let water_at_dispenser = water_grid.get_level(self.position);
            if water_at_dispenser > 0.9 {
                return; // Water has backed up to dispenser level
            }
        }

        // Calculate water to emit
        let mut emit_amount = self.flow_rate * dt;

        // Check finite supply
        if !self.infinite {
            if self.remaining_water <= 0.0 {
                self.active = false;
                return;
            }
            emit_amount = emit_amount.min(self.remaining_water);
            self.remaining_water -= emit_amount;
        }

        // Emit water in the target direction
        let target_pos = self.position + self.direction.to_ivec3();
        water_grid.add_water(target_pos, emit_amount);
    }

    /// Check if dispenser has water remaining
    pub fn has_water(&self) -> bool {
        self.infinite || self.remaining_water > 0.0
    }
}

/// Water drain - removes water at or above a specific level
///
/// Allows creating controlled water features like swimming pools
/// with consistent water levels.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WaterDrain {
    /// Grid position of the drain
    pub position: IVec3,
    /// Maximum water level before draining (0.0-1.0)
    /// Water above this level in the drain cell is removed
    pub max_level: f32,
    /// Drain rate in blocks per second
    pub drain_rate: f32,
    /// Whether the drain is active
    pub active: bool,
    /// Total water drained (for statistics)
    pub total_drained: f32,
}

impl Default for WaterDrain {
    fn default() -> Self {
        Self {
            position: IVec3::ZERO,
            max_level: 0.5, // Drain water above 50%
            drain_rate: 36.0,
            active: true,
            total_drained: 0.0,
        }
    }
}

impl WaterDrain {
    /// Create a new drain at the given position
    pub fn new(position: IVec3) -> Self {
        Self {
            position,
            ..Default::default()
        }
    }

    /// Set maximum water level (water above this is drained)
    pub fn with_max_level(mut self, level: f32) -> Self {
        self.max_level = level.clamp(0.0, 1.0);
        self
    }

    /// Set drain rate
    pub fn with_drain_rate(mut self, rate: f32) -> Self {
        self.drain_rate = rate;
        self
    }

    /// Update the drain, removing water from the grid
    pub fn tick(&mut self, water_grid: &mut WaterVolumeGrid, dt: f32) {
        if !self.active {
            return;
        }

        let current_level = water_grid.get_level(self.position);
        if current_level <= self.max_level {
            return; // Water level is acceptable
        }

        // Calculate how much to drain
        let excess = current_level - self.max_level;
        let drain_amount = (self.drain_rate * dt).min(excess);

        let removed = water_grid.remove_water(self.position, drain_amount);
        self.total_drained += removed;
    }
}

/// Water gate - controllable barrier for water flow
///
/// Can be opened/closed to control water passage,
/// enabling dam and irrigation systems.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WaterGate {
    /// Grid position of the gate
    pub position: IVec3,
    /// Current openness (0.0 = closed, 1.0 = fully open)
    pub openness: f32,
    /// Target openness
    pub target_openness: f32,
    /// Opening/closing speed (units per second)
    pub speed: f32,
    /// Whether gate is mechanically linked (auto-control)
    pub linked: bool,
    /// Link group ID for synchronized gates
    pub link_group: u32,
}

impl Default for WaterGate {
    fn default() -> Self {
        Self {
            position: IVec3::ZERO,
            openness: 0.0, // Start closed
            target_openness: 0.0,
            speed: 1.0, // 1 second to fully open/close
            linked: false,
            link_group: 0,
        }
    }
}

impl WaterGate {
    /// Create a new gate at the given position
    pub fn new(position: IVec3) -> Self {
        Self {
            position,
            ..Default::default()
        }
    }

    /// Set the gate to open
    pub fn open(&mut self) {
        self.target_openness = 1.0;
    }

    /// Set the gate to close
    pub fn close(&mut self) {
        self.target_openness = 0.0;
    }

    /// Toggle gate state
    pub fn toggle(&mut self) {
        if self.target_openness > 0.5 {
            self.close();
        } else {
            self.open();
        }
    }

    /// Set specific openness target (0.0-1.0)
    pub fn set_openness(&mut self, target: f32) {
        self.target_openness = target.clamp(0.0, 1.0);
    }

    /// Check if gate is fully open
    pub fn is_open(&self) -> bool {
        self.openness > 0.99
    }

    /// Check if gate is fully closed
    pub fn is_closed(&self) -> bool {
        self.openness < 0.01
    }

    /// Update gate animation
    pub fn tick(&mut self, dt: f32) {
        let diff = self.target_openness - self.openness;
        let change = diff.signum() * self.speed * dt;

        if change.abs() >= diff.abs() {
            self.openness = self.target_openness;
        } else {
            self.openness += change;
        }

        self.openness = self.openness.clamp(0.0, 1.0);
    }

    /// Get flow multiplier for water passing through
    ///
    /// Returns a value from 0.0 (blocked) to 1.0 (full flow)
    pub fn flow_multiplier(&self) -> f32 {
        // Non-linear for more realistic gate behavior
        // Closed gate still allows tiny seepage
        self.openness.powf(0.5) * 0.99 + 0.01 * self.openness
    }

    /// Apply gate effect to water grid
    ///
    /// Reduces flow through the gate cell based on openness
    pub fn apply_to_grid(&self, water_grid: &mut WaterVolumeGrid) {
        if let Some(cell) = water_grid.get_cell_mut(self.position) {
            // When gate is closed, mark cell as having restricted flow
            if self.is_closed() {
                cell.flags.insert(CellFlags::GATE);
            } else {
                cell.flags.remove(CellFlags::GATE);
            }
        }
    }
}

/// Water wheel - generates mechanical power from water flow
///
/// Can be used to power machinery, grinders, etc.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WaterWheel {
    /// Grid position of the water wheel
    pub position: IVec3,
    /// Wheel orientation (axis of rotation)
    pub axis: WheelAxis,
    /// Current rotation angle (radians)
    pub rotation: f32,
    /// Current rotation speed (radians per second)
    pub rotation_speed: f32,
    /// Power output (arbitrary units, 0-100)
    pub power_output: f32,
    /// Maximum power output
    pub max_power: f32,
    /// Minimum flow rate to start turning
    pub min_flow_rate: f32,
    /// Efficiency multiplier
    pub efficiency: f32,
    /// Inertia (resistance to speed changes)
    pub inertia: f32,
}

/// Axis of rotation for water wheel
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum WheelAxis {
    /// Rotates around X axis (water flows in Z direction)
    #[default]
    X,
    /// Rotates around Z axis (water flows in X direction)
    Z,
}

impl Default for WaterWheel {
    fn default() -> Self {
        Self {
            position: IVec3::ZERO,
            axis: WheelAxis::X,
            rotation: 0.0,
            rotation_speed: 0.0,
            power_output: 0.0,
            max_power: 100.0,
            min_flow_rate: 0.1,
            efficiency: 0.8,
            inertia: 0.5,
        }
    }
}

impl WaterWheel {
    /// Create a new water wheel at the given position
    pub fn new(position: IVec3) -> Self {
        Self {
            position,
            ..Default::default()
        }
    }

    /// Set wheel axis
    pub fn with_axis(mut self, axis: WheelAxis) -> Self {
        self.axis = axis;
        self
    }

    /// Update wheel based on water flow
    pub fn tick(&mut self, water_grid: &WaterVolumeGrid, dt: f32) {
        // Sample flow at wheel position
        let flow_rate = water_grid.get_flow_rate(self.position);

        // Check adjacent cells for water level
        let water_level = water_grid.get_level(self.position);
        let effective_flow = flow_rate * water_level;

        if effective_flow >= self.min_flow_rate {
            // Calculate target rotation speed based on flow
            let target_speed = effective_flow * 10.0 * self.efficiency;

            // Apply inertia
            let speed_diff = target_speed - self.rotation_speed;
            self.rotation_speed += speed_diff * (1.0 - self.inertia) * dt * 5.0;

            // Calculate power output
            self.power_output = (self.rotation_speed * self.efficiency * 10.0).min(self.max_power);
        } else {
            // Slow down due to friction
            self.rotation_speed *= (1.0 - dt * 0.5).max(0.0);
            self.power_output = 0.0;
        }

        // Update rotation angle
        self.rotation += self.rotation_speed * dt;
        if self.rotation > std::f32::consts::TAU {
            self.rotation -= std::f32::consts::TAU;
        }
    }

    /// Get current power output (0-100)
    pub fn power(&self) -> f32 {
        self.power_output
    }

    /// Check if wheel is generating power
    pub fn is_active(&self) -> bool {
        self.power_output > 0.1
    }

    /// Get rotation in degrees (for display)
    pub fn rotation_degrees(&self) -> f32 {
        self.rotation.to_degrees()
    }

    /// Get RPM (rotations per minute)
    pub fn rpm(&self) -> f32 {
        self.rotation_speed * 60.0 / std::f32::consts::TAU
    }
}

/// Manages all water building components in a scene
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct WaterBuildingManager {
    /// All water dispensers
    pub dispensers: Vec<WaterDispenser>,
    /// All water drains
    pub drains: Vec<WaterDrain>,
    /// All water gates
    pub gates: Vec<WaterGate>,
    /// All water wheels
    pub wheels: Vec<WaterWheel>,
}

impl WaterBuildingManager {
    /// Create a new empty manager
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a dispenser
    pub fn add_dispenser(&mut self, dispenser: WaterDispenser) -> usize {
        let id = self.dispensers.len();
        self.dispensers.push(dispenser);
        id
    }

    /// Add a drain
    pub fn add_drain(&mut self, drain: WaterDrain) -> usize {
        let id = self.drains.len();
        self.drains.push(drain);
        id
    }

    /// Add a gate
    pub fn add_gate(&mut self, gate: WaterGate) -> usize {
        let id = self.gates.len();
        self.gates.push(gate);
        id
    }

    /// Add a water wheel
    pub fn add_wheel(&mut self, wheel: WaterWheel) -> usize {
        let id = self.wheels.len();
        self.wheels.push(wheel);
        id
    }

    /// Remove dispenser at position
    pub fn remove_dispenser_at(&mut self, pos: IVec3) -> bool {
        if let Some(idx) = self.dispensers.iter().position(|d| d.position == pos) {
            self.dispensers.remove(idx);
            true
        } else {
            false
        }
    }

    /// Remove drain at position
    pub fn remove_drain_at(&mut self, pos: IVec3) -> bool {
        if let Some(idx) = self.drains.iter().position(|d| d.position == pos) {
            self.drains.remove(idx);
            true
        } else {
            false
        }
    }

    /// Update all building components
    pub fn tick(&mut self, water_grid: &mut WaterVolumeGrid, dt: f32) {
        // Update dispensers
        for dispenser in &mut self.dispensers {
            dispenser.tick(water_grid, dt);
        }

        // Update drains
        for drain in &mut self.drains {
            drain.tick(water_grid, dt);
        }

        // Update gates
        for gate in &mut self.gates {
            gate.tick(dt);
            gate.apply_to_grid(water_grid);
        }

        // Update wheels
        for wheel in &mut self.wheels {
            wheel.tick(water_grid, dt);
        }
    }

    /// Open all gates in a link group
    pub fn open_gate_group(&mut self, group: u32) {
        for gate in &mut self.gates {
            if gate.linked && gate.link_group == group {
                gate.open();
            }
        }
    }

    /// Close all gates in a link group
    pub fn close_gate_group(&mut self, group: u32) {
        for gate in &mut self.gates {
            if gate.linked && gate.link_group == group {
                gate.close();
            }
        }
    }

    /// Get total power output from all water wheels
    pub fn total_power(&self) -> f32 {
        self.wheels.iter().map(|w| w.power()).sum()
    }

    /// Get total water drained
    pub fn total_drained(&self) -> f32 {
        self.drains.iter().map(|d| d.total_drained).sum()
    }

    /// Get statistics
    pub fn stats(&self) -> WaterBuildingStats {
        WaterBuildingStats {
            dispenser_count: self.dispensers.len(),
            active_dispensers: self.dispensers.iter().filter(|d| d.active).count(),
            drain_count: self.drains.len(),
            active_drains: self.drains.iter().filter(|d| d.active).count(),
            gate_count: self.gates.len(),
            open_gates: self.gates.iter().filter(|g| g.is_open()).count(),
            wheel_count: self.wheels.len(),
            active_wheels: self.wheels.iter().filter(|w| w.is_active()).count(),
            total_power: self.total_power(),
            total_drained: self.total_drained(),
        }
    }
}

/// Statistics about water building components
#[derive(Clone, Copy, Debug)]
pub struct WaterBuildingStats {
    pub dispenser_count: usize,
    pub active_dispensers: usize,
    pub drain_count: usize,
    pub active_drains: usize,
    pub gate_count: usize,
    pub open_gates: usize,
    pub wheel_count: usize,
    pub active_wheels: usize,
    pub total_power: f32,
    pub total_drained: f32,
}

impl std::fmt::Display for WaterBuildingStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Water Buildings: {} dispensers ({} active), {} drains, {} gates ({} open), {} wheels ({} active, {:.1} power)",
            self.dispenser_count,
            self.active_dispensers,
            self.drain_count,
            self.gate_count,
            self.open_gates,
            self.wheel_count,
            self.active_wheels,
            self.total_power
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::UVec3;

    fn create_test_grid() -> WaterVolumeGrid {
        WaterVolumeGrid::new(UVec3::new(16, 16, 16), 1.0, Vec3::ZERO)
    }

    // ==================== FlowDirection Tests ====================

    #[test]
    fn test_flow_direction_to_vec3() {
        assert_eq!(FlowDirection::Down.to_vec3(), Vec3::new(0.0, -1.0, 0.0));
        assert_eq!(FlowDirection::Up.to_vec3(), Vec3::new(0.0, 1.0, 0.0));
        assert_eq!(FlowDirection::East.to_vec3(), Vec3::new(1.0, 0.0, 0.0));
        assert_eq!(FlowDirection::West.to_vec3(), Vec3::new(-1.0, 0.0, 0.0));
        assert_eq!(FlowDirection::South.to_vec3(), Vec3::new(0.0, 0.0, 1.0));
        assert_eq!(FlowDirection::North.to_vec3(), Vec3::new(0.0, 0.0, -1.0));
    }

    #[test]
    fn test_flow_direction_to_ivec3() {
        assert_eq!(FlowDirection::Down.to_ivec3(), IVec3::new(0, -1, 0));
        assert_eq!(FlowDirection::Up.to_ivec3(), IVec3::new(0, 1, 0));
        assert_eq!(FlowDirection::East.to_ivec3(), IVec3::new(1, 0, 0));
        assert_eq!(FlowDirection::West.to_ivec3(), IVec3::new(-1, 0, 0));
        assert_eq!(FlowDirection::South.to_ivec3(), IVec3::new(0, 0, 1));
        assert_eq!(FlowDirection::North.to_ivec3(), IVec3::new(0, 0, -1));
    }

    #[test]
    fn test_flow_direction_default() {
        assert_eq!(FlowDirection::default(), FlowDirection::Down);
    }

    // ==================== WaterDispenser Tests ====================

    #[test]
    fn test_dispenser_finite_supply_depletes() {
        let mut grid = create_test_grid();
        let mut dispenser = WaterDispenser::new(IVec3::new(8, 8, 8))
            .with_finite_supply(10.0)
            .with_auto_shutoff(false);

        assert!(!dispenser.infinite);
        assert!(dispenser.has_water());

        // Tick multiple times to deplete supply
        for _ in 0..20 {
            dispenser.tick(&mut grid, 1.0);
        }

        // Should have run out and become inactive
        assert!(!dispenser.has_water());
        assert!(!dispenser.active);
    }

    #[test]
    fn test_dispenser_direction() {
        let mut grid = create_test_grid();
        let mut dispenser = WaterDispenser::new(IVec3::new(8, 8, 8))
            .with_direction(FlowDirection::East);

        dispenser.tick(&mut grid, 1.0);

        // Should have created water to the east
        let water_east = grid.get_level(IVec3::new(9, 8, 8));
        assert!(water_east > 0.0);
    }

    #[test]
    fn test_dispenser_inactive() {
        let mut grid = create_test_grid();
        let mut dispenser = WaterDispenser::new(IVec3::new(8, 8, 8));
        dispenser.active = false;

        dispenser.tick(&mut grid, 1.0);

        // Should not have created any water
        let water_below = grid.get_level(IVec3::new(8, 7, 8));
        assert_eq!(water_below, 0.0);
    }

    #[test]
    fn test_dispenser_default() {
        let dispenser = WaterDispenser::default();
        assert_eq!(dispenser.position, IVec3::ZERO);
        assert_eq!(dispenser.flow_rate, 36.0);
        assert!(dispenser.active);
        assert!(dispenser.auto_shutoff);
        assert!(dispenser.infinite);
    }

    // ==================== WaterDrain Tests ====================

    #[test]
    fn test_drain_inactive() {
        let mut grid = create_test_grid();
        grid.set_level(IVec3::new(8, 8, 8), 1.0);

        let mut drain = WaterDrain::new(IVec3::new(8, 8, 8));
        drain.active = false;

        drain.tick(&mut grid, 1.0);

        // Should not have drained
        let level = grid.get_level(IVec3::new(8, 8, 8));
        assert_eq!(level, 1.0);
    }

    #[test]
    fn test_drain_tracks_total() {
        let mut grid = create_test_grid();
        grid.set_level(IVec3::new(8, 8, 8), 1.0);

        let mut drain = WaterDrain::new(IVec3::new(8, 8, 8))
            .with_max_level(0.0)
            .with_drain_rate(100.0);

        drain.tick(&mut grid, 1.0);

        assert!(drain.total_drained > 0.0);
    }

    #[test]
    fn test_drain_default() {
        let drain = WaterDrain::default();
        assert_eq!(drain.position, IVec3::ZERO);
        assert_eq!(drain.max_level, 0.5);
        assert_eq!(drain.drain_rate, 36.0);
        assert!(drain.active);
        assert_eq!(drain.total_drained, 0.0);
    }

    // ==================== WaterGate Tests ====================

    #[test]
    fn test_gate_toggle() {
        let mut gate = WaterGate::new(IVec3::new(8, 8, 8));
        assert!(gate.is_closed());

        // Toggle should open
        gate.toggle();
        assert_eq!(gate.target_openness, 1.0);

        // Simulate full opening
        gate.openness = 1.0;

        // Toggle should close
        gate.toggle();
        assert_eq!(gate.target_openness, 0.0);
    }

    #[test]
    fn test_gate_set_openness() {
        let mut gate = WaterGate::new(IVec3::new(8, 8, 8));

        gate.set_openness(0.5);
        assert_eq!(gate.target_openness, 0.5);

        // Should clamp to 0-1
        gate.set_openness(2.0);
        assert_eq!(gate.target_openness, 1.0);

        gate.set_openness(-0.5);
        assert_eq!(gate.target_openness, 0.0);
    }

    #[test]
    fn test_gate_tick_clamps() {
        let mut gate = WaterGate::new(IVec3::new(8, 8, 8));
        gate.openness = 0.95;
        gate.target_openness = 1.0;
        gate.speed = 10.0; // Fast

        gate.tick(1.0);

        // Should clamp to 1.0, not overshoot
        assert_eq!(gate.openness, 1.0);
    }

    #[test]
    fn test_gate_apply_to_grid() {
        let mut grid = create_test_grid();
        let mut gate = WaterGate::new(IVec3::new(8, 8, 8));

        // Closed gate should set GATE flag
        gate.apply_to_grid(&mut grid);

        // Open the gate
        gate.openness = 1.0;
        gate.apply_to_grid(&mut grid);
    }

    #[test]
    fn test_gate_default() {
        let gate = WaterGate::default();
        assert_eq!(gate.position, IVec3::ZERO);
        assert_eq!(gate.openness, 0.0);
        assert_eq!(gate.target_openness, 0.0);
        assert_eq!(gate.speed, 1.0);
        assert!(!gate.linked);
        assert_eq!(gate.link_group, 0);
    }

    // ==================== WaterWheel Tests ====================

    #[test]
    fn test_wheel_with_axis() {
        let wheel = WaterWheel::new(IVec3::new(8, 8, 8)).with_axis(WheelAxis::Z);
        assert_eq!(wheel.axis, WheelAxis::Z);
    }

    #[test]
    fn test_wheel_rpm() {
        let mut wheel = WaterWheel::new(IVec3::new(8, 8, 8));
        wheel.rotation_speed = std::f32::consts::TAU; // 1 full rotation per second

        let rpm = wheel.rpm();
        assert!((rpm - 60.0).abs() < 0.01); // Should be 60 RPM
    }

    #[test]
    fn test_wheel_rotation_degrees() {
        let mut wheel = WaterWheel::new(IVec3::new(8, 8, 8));
        wheel.rotation = std::f32::consts::PI; // Half rotation

        let degrees = wheel.rotation_degrees();
        assert!((degrees - 180.0).abs() < 0.01);
    }

    #[test]
    fn test_wheel_default() {
        let wheel = WaterWheel::default();
        assert_eq!(wheel.position, IVec3::ZERO);
        assert_eq!(wheel.axis, WheelAxis::X);
        assert_eq!(wheel.rotation, 0.0);
        assert_eq!(wheel.max_power, 100.0);
    }

    #[test]
    fn test_wheel_axis_default() {
        assert_eq!(WheelAxis::default(), WheelAxis::X);
    }

    // ==================== WaterBuildingManager Tests ====================

    #[test]
    fn test_manager_remove_dispenser() {
        let mut manager = WaterBuildingManager::new();
        manager.add_dispenser(WaterDispenser::new(IVec3::new(1, 1, 1)));
        manager.add_dispenser(WaterDispenser::new(IVec3::new(2, 2, 2)));

        assert_eq!(manager.dispensers.len(), 2);

        // Remove existing dispenser
        assert!(manager.remove_dispenser_at(IVec3::new(1, 1, 1)));
        assert_eq!(manager.dispensers.len(), 1);

        // Remove non-existent dispenser
        assert!(!manager.remove_dispenser_at(IVec3::new(5, 5, 5)));
        assert_eq!(manager.dispensers.len(), 1);
    }

    #[test]
    fn test_manager_remove_drain() {
        let mut manager = WaterBuildingManager::new();
        manager.add_drain(WaterDrain::new(IVec3::new(1, 1, 1)));
        manager.add_drain(WaterDrain::new(IVec3::new(2, 2, 2)));

        assert_eq!(manager.drains.len(), 2);

        assert!(manager.remove_drain_at(IVec3::new(2, 2, 2)));
        assert_eq!(manager.drains.len(), 1);

        assert!(!manager.remove_drain_at(IVec3::new(9, 9, 9)));
    }

    #[test]
    fn test_manager_close_gate_group() {
        let mut manager = WaterBuildingManager::new();

        let mut gate1 = WaterGate::new(IVec3::new(4, 8, 8));
        gate1.linked = true;
        gate1.link_group = 1;
        gate1.openness = 1.0;
        gate1.target_openness = 1.0;

        let mut gate2 = WaterGate::new(IVec3::new(8, 8, 8));
        gate2.linked = true;
        gate2.link_group = 1;
        gate2.openness = 1.0;
        gate2.target_openness = 1.0;

        manager.add_gate(gate1);
        manager.add_gate(gate2);

        manager.close_gate_group(1);

        assert_eq!(manager.gates[0].target_openness, 0.0);
        assert_eq!(manager.gates[1].target_openness, 0.0);
    }

    #[test]
    fn test_manager_total_drained() {
        let mut manager = WaterBuildingManager::new();

        let mut drain1 = WaterDrain::new(IVec3::new(1, 1, 1));
        drain1.total_drained = 50.0;

        let mut drain2 = WaterDrain::new(IVec3::new(2, 2, 2));
        drain2.total_drained = 30.0;

        manager.add_drain(drain1);
        manager.add_drain(drain2);

        assert_eq!(manager.total_drained(), 80.0);
    }

    // ==================== WaterBuildingStats Tests ====================

    #[test]
    fn test_stats_display() {
        let stats = WaterBuildingStats {
            dispenser_count: 2,
            active_dispensers: 1,
            drain_count: 3,
            active_drains: 2,
            gate_count: 4,
            open_gates: 1,
            wheel_count: 2,
            active_wheels: 1,
            total_power: 50.0,
            total_drained: 100.0,
        };

        let display = format!("{}", stats);
        assert!(display.contains("2 dispensers"));
        assert!(display.contains("1 active"));
        assert!(display.contains("50.0 power"));
    }

    // ==================== Original Tests ====================

    #[test]
    fn test_dispenser_creates_water() {
        let mut grid = create_test_grid();
        let mut dispenser = WaterDispenser::new(IVec3::new(8, 8, 8));

        // Tick for 1 second
        dispenser.tick(&mut grid, 1.0);

        // Should have created water below
        let water_below = grid.get_level(IVec3::new(8, 7, 8));
        assert!(water_below > 0.0);
    }

    #[test]
    fn test_dispenser_auto_shutoff() {
        let mut grid = create_test_grid();

        // Fill the dispenser cell
        grid.set_level(IVec3::new(8, 8, 8), 1.0);

        let mut dispenser = WaterDispenser::new(IVec3::new(8, 8, 8)).with_auto_shutoff(true);

        let initial_below = grid.get_level(IVec3::new(8, 7, 8));
        dispenser.tick(&mut grid, 1.0);
        let after_below = grid.get_level(IVec3::new(8, 7, 8));

        // Should not have added more water (auto-shutoff triggered)
        assert_eq!(initial_below, after_below);
    }

    #[test]
    fn test_drain_removes_water() {
        let mut grid = create_test_grid();
        grid.set_level(IVec3::new(8, 8, 8), 1.0);

        let mut drain = WaterDrain::new(IVec3::new(8, 8, 8)).with_max_level(0.5);

        drain.tick(&mut grid, 1.0);

        // Should have drained to max level
        let level = grid.get_level(IVec3::new(8, 8, 8));
        assert!(level <= 0.5 + 0.01); // Small epsilon for float comparison
    }

    #[test]
    fn test_gate_opens_closes() {
        let mut gate = WaterGate::new(IVec3::new(8, 8, 8));
        assert!(gate.is_closed());

        gate.open();
        for _ in 0..20 {
            gate.tick(0.1);
        }
        assert!(gate.is_open());

        gate.close();
        for _ in 0..20 {
            gate.tick(0.1);
        }
        assert!(gate.is_closed());
    }

    #[test]
    fn test_gate_flow_multiplier() {
        let mut gate = WaterGate::new(IVec3::new(8, 8, 8));

        // Closed gate has very low flow
        assert!(gate.flow_multiplier() < 0.02);

        gate.openness = 1.0;
        // Open gate has full flow
        assert!(gate.flow_multiplier() > 0.98);
    }

    #[test]
    fn test_water_wheel_no_flow() {
        let grid = create_test_grid();
        let mut wheel = WaterWheel::new(IVec3::new(8, 8, 8));

        wheel.tick(&grid, 1.0);

        // No water = no power
        assert_eq!(wheel.power(), 0.0);
        assert!(!wheel.is_active());
    }

    #[test]
    fn test_building_manager() {
        let mut grid = create_test_grid();
        let mut manager = WaterBuildingManager::new();

        manager.add_dispenser(WaterDispenser::new(IVec3::new(8, 12, 8)));
        manager.add_drain(WaterDrain::new(IVec3::new(8, 4, 8)));
        manager.add_gate(WaterGate::new(IVec3::new(8, 8, 8)));
        manager.add_wheel(WaterWheel::new(IVec3::new(4, 8, 8)));

        let stats = manager.stats();
        assert_eq!(stats.dispenser_count, 1);
        assert_eq!(stats.drain_count, 1);
        assert_eq!(stats.gate_count, 1);
        assert_eq!(stats.wheel_count, 1);

        // Tick the manager
        manager.tick(&mut grid, 0.1);
    }

    #[test]
    fn test_gate_groups() {
        let mut manager = WaterBuildingManager::new();

        let mut gate1 = WaterGate::new(IVec3::new(4, 8, 8));
        gate1.linked = true;
        gate1.link_group = 1;

        let mut gate2 = WaterGate::new(IVec3::new(8, 8, 8));
        gate2.linked = true;
        gate2.link_group = 1;

        let mut gate3 = WaterGate::new(IVec3::new(12, 8, 8));
        gate3.linked = true;
        gate3.link_group = 2;

        manager.add_gate(gate1);
        manager.add_gate(gate2);
        manager.add_gate(gate3);

        // Open group 1
        manager.open_gate_group(1);

        assert_eq!(manager.gates[0].target_openness, 1.0);
        assert_eq!(manager.gates[1].target_openness, 1.0);
        assert_eq!(manager.gates[2].target_openness, 0.0); // Different group
    }
}
