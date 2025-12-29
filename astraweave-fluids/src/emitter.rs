//! Fluid Emitter Module
//!
//! Provides mesh-based and point-based fluid emitters for runtime particle spawning.

/// A fluid emitter that spawns particles from mesh surfaces or points.
pub struct FluidEmitter {
    /// World-space transform for the emitter
    pub transform: [[f32; 4]; 4],
    /// Emission rate in particles per second
    pub rate: f32,
    /// Base velocity for emitted particles (local space)
    pub velocity: [f32; 3],
    /// Random velocity jitter magnitude
    pub jitter: f32,
    /// Particle color for emitted particles
    pub color: [f32; 4],
    /// Emitter shape
    pub shape: EmitterShape,
    /// Internal accumulator for emission timing
    accumulator: f32,
    /// Whether the emitter is currently active
    pub enabled: bool,
}

/// Shape of the fluid emitter source
pub enum EmitterShape {
    /// Single point emitter
    Point,
    /// Spherical volume emitter
    Sphere { radius: f32 },
    /// Axis-aligned box volume emitter
    Box { half_extents: [f32; 3] },
    /// Mesh surface emitter (vertices + normals)
    Mesh {
        vertices: Vec<[f32; 3]>,
        normals: Vec<[f32; 3]>,
    },
}

impl Default for FluidEmitter {
    fn default() -> Self {
        Self {
            transform: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
            rate: 100.0,
            velocity: [0.0, -1.0, 0.0],
            jitter: 0.1,
            color: [0.2, 0.5, 0.8, 1.0],
            shape: EmitterShape::Point,
            accumulator: 0.0,
            enabled: true,
        }
    }
}

impl FluidEmitter {
    /// Create a new point emitter at the given position.
    pub fn point(position: [f32; 3], rate: f32) -> Self {
        let mut emitter = Self::default();
        emitter.transform[3] = [position[0], position[1], position[2], 1.0];
        emitter.rate = rate;
        emitter
    }

    /// Create a new spherical volume emitter.
    pub fn sphere(center: [f32; 3], radius: f32, rate: f32) -> Self {
        let mut emitter = Self::default();
        emitter.transform[3] = [center[0], center[1], center[2], 1.0];
        emitter.shape = EmitterShape::Sphere { radius };
        emitter.rate = rate;
        emitter
    }

    /// Create a new mesh surface emitter.
    pub fn from_mesh(vertices: Vec<[f32; 3]>, normals: Vec<[f32; 3]>, rate: f32) -> Self {
        let mut emitter = Self::default();
        emitter.shape = EmitterShape::Mesh { vertices, normals };
        emitter.rate = rate;
        emitter
    }

    /// Update the emitter and return particles to spawn.
    /// Returns (positions, velocities, colors) for spawning.
    pub fn tick(&mut self, dt: f32) -> (Vec<[f32; 3]>, Vec<[f32; 3]>, Vec<[f32; 4]>) {
        if !self.enabled {
            return (Vec::new(), Vec::new(), Vec::new());
        }

        self.accumulator += dt * self.rate;
        let count = self.accumulator as usize;
        self.accumulator -= count as f32;

        let mut positions = Vec::with_capacity(count);
        let mut velocities = Vec::with_capacity(count);
        let mut colors = Vec::with_capacity(count);

        let origin = [
            self.transform[3][0],
            self.transform[3][1],
            self.transform[3][2],
        ];

        for i in 0..count {
            let (pos, normal) = self.sample_shape(i as u32);

            // Apply jitter
            let jitter_seed = (i as f32 * 12.9898).sin() * 43758.5453;
            let jx = (jitter_seed.fract() - 0.5) * self.jitter;
            let jy = ((jitter_seed * 2.0).fract() - 0.5) * self.jitter;
            let jz = ((jitter_seed * 3.0).fract() - 0.5) * self.jitter;

            positions.push([origin[0] + pos[0], origin[1] + pos[1], origin[2] + pos[2]]);

            velocities.push([
                self.velocity[0] * normal[0] + jx,
                self.velocity[1] * normal[1] + jy,
                self.velocity[2] * normal[2] + jz,
            ]);

            colors.push(self.color);
        }

        (positions, velocities, colors)
    }

    fn sample_shape(&self, seed: u32) -> ([f32; 3], [f32; 3]) {
        match &self.shape {
            EmitterShape::Point => ([0.0, 0.0, 0.0], [0.0, -1.0, 0.0]),
            EmitterShape::Sphere { radius } => {
                // Random point on sphere surface
                let theta = (seed as f32 * 0.618033) * std::f32::consts::TAU;
                let phi = ((seed as f32 * 0.414213).fract() - 0.5) * std::f32::consts::PI;
                let x = phi.cos() * theta.cos() * radius;
                let y = phi.sin() * radius;
                let z = phi.cos() * theta.sin() * radius;
                let n = [theta.cos() * phi.cos(), phi.sin(), theta.sin() * phi.cos()];
                ([x, y, z], n)
            }
            EmitterShape::Box { half_extents } => {
                // Random point in box volume
                let fx = ((seed as f32 * 0.123456).fract() - 0.5) * 2.0;
                let fy = ((seed as f32 * 0.654321).fract() - 0.5) * 2.0;
                let fz = ((seed as f32 * 0.987654).fract() - 0.5) * 2.0;
                (
                    [
                        fx * half_extents[0],
                        fy * half_extents[1],
                        fz * half_extents[2],
                    ],
                    [0.0, -1.0, 0.0],
                )
            }
            EmitterShape::Mesh { vertices, normals } => {
                if vertices.is_empty() {
                    return ([0.0, 0.0, 0.0], [0.0, -1.0, 0.0]);
                }
                let idx = seed as usize % vertices.len();
                (
                    vertices[idx],
                    normals.get(idx).copied().unwrap_or([0.0, -1.0, 0.0]),
                )
            }
        }
    }
}

/// A drain that removes particles from a region
pub struct FluidDrain {
    /// Center position of the drain
    pub position: [f32; 3],
    /// Radius of the drain sphere
    pub radius: f32,
    /// Whether the drain is currently active
    pub enabled: bool,
}

impl FluidDrain {
    pub fn new(position: [f32; 3], radius: f32) -> Self {
        Self {
            position,
            radius,
            enabled: true,
        }
    }
}
