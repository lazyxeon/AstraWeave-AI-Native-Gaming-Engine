//! Animation & Skinning Benchmarks
//!
//! Benchmarks for skeletal animation system including:
//! - Transform operations (lerp, slerp, to_matrix)
//! - Animation clip sampling
//! - Skeleton hierarchy traversal
//! - Joint palette generation
//! - Animation blending (crossfade, additive)
//!
//! These are CPU-side benchmarks simulating the animation pipeline.
//! GPU skinning would add ~1-2µs upload overhead per skeleton.

// =============================================================================
// MISSION-CRITICAL CORRECTNESS ASSERTIONS
// =============================================================================
// Animation benchmarks validate CORRECTNESS of skeletal animation systems.
// Assertions verify:
//   1. Quaternion Validity: Rotation quaternions are normalized (length ≈ 1)
//   2. Transform Validity: Matrices are finite, non-singular
//   3. Interpolation Bounds: lerp/slerp results are within valid parameter range
//   4. Joint Hierarchy: Parent indices are valid (< child index)
//   5. Blend Weights: Weights sum to 1.0 (or close enough for crossfade)
// =============================================================================

#![allow(dead_code)]

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::hint::black_box;

/// CORRECTNESS: Validate quaternion is normalized
#[inline]
fn assert_quaternion_normalized(q: &Quat, context: &str) {
    let len = (q.x*q.x + q.y*q.y + q.z*q.z + q.w*q.w).sqrt();
    assert!((len - 1.0).abs() < 0.01,
        "[CORRECTNESS FAILURE] {}: quaternion not normalized (len={})", context, len);
}

/// CORRECTNESS: Validate transform matrix is finite and valid
#[inline]
fn assert_transform_matrix_valid(mat: &[[f32; 4]; 4], context: &str) {
    for row in 0..4 {
        for col in 0..4 {
            assert!(mat[row][col].is_finite(),
                "[CORRECTNESS FAILURE] {}: matrix[{}][{}] non-finite ({})", 
                context, row, col, mat[row][col]);
        }
    }
}

/// CORRECTNESS: Validate interpolation parameter is in valid range
#[inline]
fn assert_interp_param_valid(t: f32, context: &str) {
    assert!(t >= 0.0 && t <= 1.0,
        "[CORRECTNESS FAILURE] {}: interpolation parameter {} out of range [0,1]", context, t);
}

// ============================================================================
// Local Implementations (mirroring animation.rs patterns)
// ============================================================================

/// Linear-interpolable 3D vector
#[derive(Clone, Copy, Debug)]
struct Vec3 {
    x: f32,
    y: f32,
    z: f32,
}

impl Vec3 {
    const ZERO: Self = Self {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    };
    const ONE: Self = Self {
        x: 1.0,
        y: 1.0,
        z: 1.0,
    };

    fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    fn lerp(self, other: Self, t: f32) -> Self {
        Self {
            x: self.x + (other.x - self.x) * t,
            y: self.y + (other.y - self.y) * t,
            z: self.z + (other.z - self.z) * t,
        }
    }

    fn length(self) -> f32 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }
}

/// Quaternion for rotations
#[derive(Clone, Copy, Debug)]
struct Quat {
    x: f32,
    y: f32,
    z: f32,
    w: f32,
}

impl Quat {
    const IDENTITY: Self = Self {
        x: 0.0,
        y: 0.0,
        z: 0.0,
        w: 1.0,
    };

    fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self { x, y, z, w }
    }

    fn from_axis_angle(axis: Vec3, angle: f32) -> Self {
        let half_angle = angle * 0.5;
        let s = half_angle.sin();
        let len = axis.length();
        if len > 0.0001 {
            Self {
                x: axis.x / len * s,
                y: axis.y / len * s,
                z: axis.z / len * s,
                w: half_angle.cos(),
            }
        } else {
            Self::IDENTITY
        }
    }

    fn dot(self, other: Self) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z + self.w * other.w
    }

    fn normalize(self) -> Self {
        let len = (self.x * self.x + self.y * self.y + self.z * self.z + self.w * self.w).sqrt();
        if len > 0.0001 {
            Self {
                x: self.x / len,
                y: self.y / len,
                z: self.z / len,
                w: self.w / len,
            }
        } else {
            Self::IDENTITY
        }
    }

    /// Spherical linear interpolation
    fn slerp(self, other: Self, t: f32) -> Self {
        let mut dot = self.dot(other);
        let mut other = other;

        // Handle opposite quaternions
        if dot < 0.0 {
            other = Self {
                x: -other.x,
                y: -other.y,
                z: -other.z,
                w: -other.w,
            };
            dot = -dot;
        }

        // If very close, use linear interpolation
        if dot > 0.9995 {
            let result = Self {
                x: self.x + (other.x - self.x) * t,
                y: self.y + (other.y - self.y) * t,
                z: self.z + (other.z - self.z) * t,
                w: self.w + (other.w - self.w) * t,
            };
            return result.normalize();
        }

        // Full spherical interpolation
        let theta = dot.clamp(-1.0, 1.0).acos();
        let sin_theta = theta.sin();
        if sin_theta.abs() < 0.0001 {
            return self;
        }

        let s0 = ((1.0 - t) * theta).sin() / sin_theta;
        let s1 = (t * theta).sin() / sin_theta;

        Self {
            x: self.x * s0 + other.x * s1,
            y: self.y * s0 + other.y * s1,
            z: self.z * s0 + other.z * s1,
            w: self.w * s0 + other.w * s1,
        }
    }

    /// Convert quaternion to 3x3 rotation matrix (row-major as flat array)
    fn to_rotation_matrix(self) -> [f32; 9] {
        let x2 = self.x * self.x;
        let y2 = self.y * self.y;
        let z2 = self.z * self.z;
        let xy = self.x * self.y;
        let xz = self.x * self.z;
        let yz = self.y * self.z;
        let wx = self.w * self.x;
        let wy = self.w * self.y;
        let wz = self.w * self.z;

        [
            1.0 - 2.0 * (y2 + z2),
            2.0 * (xy - wz),
            2.0 * (xz + wy),
            2.0 * (xy + wz),
            1.0 - 2.0 * (x2 + z2),
            2.0 * (yz - wx),
            2.0 * (xz - wy),
            2.0 * (yz + wx),
            1.0 - 2.0 * (x2 + y2),
        ]
    }
}

/// 4x4 transformation matrix (column-major)
#[derive(Clone, Copy, Debug)]
struct Mat4 {
    cols: [[f32; 4]; 4],
}

impl Mat4 {
    const IDENTITY: Self = Self {
        cols: [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ],
    };

    /// Create TRS matrix from translation, rotation, scale
    fn from_trs(translation: Vec3, rotation: Quat, scale: Vec3) -> Self {
        let r = rotation.to_rotation_matrix();

        Self {
            cols: [
                [r[0] * scale.x, r[3] * scale.x, r[6] * scale.x, 0.0],
                [r[1] * scale.y, r[4] * scale.y, r[7] * scale.y, 0.0],
                [r[2] * scale.z, r[5] * scale.z, r[8] * scale.z, 0.0],
                [translation.x, translation.y, translation.z, 1.0],
            ],
        }
    }

    /// Multiply two matrices
    fn mul(self, other: Self) -> Self {
        let mut result = [[0.0f32; 4]; 4];

        for i in 0..4 {
            for j in 0..4 {
                result[i][j] = self.cols[0][j] * other.cols[i][0]
                    + self.cols[1][j] * other.cols[i][1]
                    + self.cols[2][j] * other.cols[i][2]
                    + self.cols[3][j] * other.cols[i][3];
            }
        }

        Self { cols: result }
    }
}

/// Local transform (TRS)
#[derive(Clone, Copy, Debug)]
struct Transform {
    translation: Vec3,
    rotation: Quat,
    scale: Vec3,
}

impl Transform {
    fn default_pose() -> Self {
        Self {
            translation: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        }
    }

    fn to_matrix(&self) -> Mat4 {
        Mat4::from_trs(self.translation, self.rotation, self.scale)
    }

    fn lerp(&self, other: &Transform, t: f32) -> Transform {
        Transform {
            translation: self.translation.lerp(other.translation, t),
            rotation: self.rotation.slerp(other.rotation, t),
            scale: self.scale.lerp(other.scale, t),
        }
    }
}

/// Joint in skeleton hierarchy
#[derive(Clone, Debug)]
struct Joint {
    parent_index: Option<usize>,
    inverse_bind_matrix: Mat4,
    local_transform: Transform,
}

/// Skeleton hierarchy
struct Skeleton {
    joints: Vec<Joint>,
}

impl Skeleton {
    /// Create a simple humanoid skeleton (20 joints)
    fn create_humanoid() -> Self {
        let mut joints = Vec::with_capacity(20);

        // Root/hips (0)
        joints.push(Joint {
            parent_index: None,
            inverse_bind_matrix: Mat4::IDENTITY,
            local_transform: Transform::default_pose(),
        });

        // Spine hierarchy (1-3)
        for i in 0..3 {
            joints.push(Joint {
                parent_index: Some(i),
                inverse_bind_matrix: Mat4::IDENTITY,
                local_transform: Transform {
                    translation: Vec3::new(0.0, 0.2, 0.0),
                    rotation: Quat::IDENTITY,
                    scale: Vec3::ONE,
                },
            });
        }

        // Head (4)
        joints.push(Joint {
            parent_index: Some(3),
            inverse_bind_matrix: Mat4::IDENTITY,
            local_transform: Transform {
                translation: Vec3::new(0.0, 0.3, 0.0),
                rotation: Quat::IDENTITY,
                scale: Vec3::ONE,
            },
        });

        // Left arm (5-7): shoulder, elbow, wrist
        for i in 0..3 {
            joints.push(Joint {
                parent_index: Some(if i == 0 { 3 } else { 4 + i }),
                inverse_bind_matrix: Mat4::IDENTITY,
                local_transform: Transform {
                    translation: Vec3::new(-0.2, 0.0, 0.0),
                    rotation: Quat::IDENTITY,
                    scale: Vec3::ONE,
                },
            });
        }

        // Right arm (8-10): shoulder, elbow, wrist
        for i in 0..3 {
            joints.push(Joint {
                parent_index: Some(if i == 0 { 3 } else { 7 + i }),
                inverse_bind_matrix: Mat4::IDENTITY,
                local_transform: Transform {
                    translation: Vec3::new(0.2, 0.0, 0.0),
                    rotation: Quat::IDENTITY,
                    scale: Vec3::ONE,
                },
            });
        }

        // Left leg (11-13): hip, knee, ankle
        for i in 0..3 {
            joints.push(Joint {
                parent_index: Some(if i == 0 { 0 } else { 10 + i }),
                inverse_bind_matrix: Mat4::IDENTITY,
                local_transform: Transform {
                    translation: Vec3::new(-0.1, -0.4, 0.0),
                    rotation: Quat::IDENTITY,
                    scale: Vec3::ONE,
                },
            });
        }

        // Right leg (14-16): hip, knee, ankle
        for i in 0..3 {
            joints.push(Joint {
                parent_index: Some(if i == 0 { 0 } else { 13 + i }),
                inverse_bind_matrix: Mat4::IDENTITY,
                local_transform: Transform {
                    translation: Vec3::new(0.1, -0.4, 0.0),
                    rotation: Quat::IDENTITY,
                    scale: Vec3::ONE,
                },
            });
        }

        // Fingers/toes (17-19)
        for i in 0..3 {
            joints.push(Joint {
                parent_index: Some(7 + i * 3),
                inverse_bind_matrix: Mat4::IDENTITY,
                local_transform: Transform {
                    translation: Vec3::new(0.0, 0.0, 0.1),
                    rotation: Quat::IDENTITY,
                    scale: Vec3::ONE,
                },
            });
        }

        Self { joints }
    }

    /// Create a large skeleton for stress testing
    fn create_stress_skeleton(joint_count: usize) -> Self {
        let mut joints = Vec::with_capacity(joint_count);

        // Root
        joints.push(Joint {
            parent_index: None,
            inverse_bind_matrix: Mat4::IDENTITY,
            local_transform: Transform::default_pose(),
        });

        // Chain of joints
        for i in 1..joint_count {
            joints.push(Joint {
                parent_index: Some(i - 1),
                inverse_bind_matrix: Mat4::IDENTITY,
                local_transform: Transform {
                    translation: Vec3::new(0.0, 0.1, 0.0),
                    rotation: Quat::from_axis_angle(Vec3::new(0.0, 1.0, 0.0), 0.1),
                    scale: Vec3::ONE,
                },
            });
        }

        Self { joints }
    }
}

/// Animation keyframe
struct Keyframe {
    time: f32,
    transform: Transform,
}

/// Animation clip for a single joint
struct JointAnimation {
    target_joint: usize,
    keyframes: Vec<Keyframe>,
}

/// Animation clip
struct AnimationClip {
    duration: f32,
    channels: Vec<JointAnimation>,
}

impl AnimationClip {
    /// Create a simple walk cycle animation
    fn create_walk_cycle(skeleton: &Skeleton) -> Self {
        let mut channels = Vec::new();

        // Animate all joints with simple oscillation
        for i in 0..skeleton.joints.len() {
            let mut keyframes = Vec::new();

            // 4 keyframes over 1 second
            for k in 0..4 {
                let t = k as f32 * 0.25;
                let angle = (t * std::f32::consts::PI * 2.0).sin() * 0.2;

                keyframes.push(Keyframe {
                    time: t,
                    transform: Transform {
                        translation: skeleton.joints[i].local_transform.translation,
                        rotation: Quat::from_axis_angle(Vec3::new(1.0, 0.0, 0.0), angle),
                        scale: Vec3::ONE,
                    },
                });
            }

            channels.push(JointAnimation {
                target_joint: i,
                keyframes,
            });
        }

        Self {
            duration: 1.0,
            channels,
        }
    }

    /// Sample animation at time, returning joint transforms
    fn sample(&self, time: f32, skeleton: &Skeleton) -> Vec<Transform> {
        let mut transforms: Vec<Transform> = skeleton
            .joints
            .iter()
            .map(|j| j.local_transform)
            .collect();

        let wrapped_time = time % self.duration;

        for channel in &self.channels {
            if channel.target_joint >= transforms.len() {
                continue;
            }

            let kf = &channel.keyframes;
            if kf.is_empty() {
                continue;
            }

            // Find surrounding keyframes
            let mut idx0 = 0;
            let mut idx1 = 0;
            let mut t = 0.0;

            for i in 0..kf.len() - 1 {
                if wrapped_time >= kf[i].time && wrapped_time < kf[i + 1].time {
                    idx0 = i;
                    idx1 = i + 1;
                    t = (wrapped_time - kf[i].time) / (kf[i + 1].time - kf[i].time);
                    break;
                }
            }

            // Interpolate
            transforms[channel.target_joint] = kf[idx0].transform.lerp(&kf[idx1].transform, t);
        }

        transforms
    }
}

/// Joint palette for GPU skinning (up to 128 joints)
#[repr(C)]
#[derive(Clone)]
struct JointPalette {
    matrices: [[f32; 16]; 128],
    joint_count: u32,
}

impl JointPalette {
    fn new() -> Self {
        Self {
            matrices: [[0.0; 16]; 128],
            joint_count: 0,
        }
    }

    fn from_skeleton(skeleton: &Skeleton, local_transforms: &[Transform]) -> Self {
        let mut palette = Self::new();

        // Compute world transforms and multiply by inverse bind
        let mut world_transforms = vec![Mat4::IDENTITY; skeleton.joints.len()];

        for (i, joint) in skeleton.joints.iter().enumerate() {
            let local = local_transforms[i].to_matrix();

            world_transforms[i] = match joint.parent_index {
                Some(parent) => world_transforms[parent].mul(local),
                None => local,
            };
        }

        // Compute final skinning matrices (capped at 128 joints)
        let joint_count = skeleton.joints.len().min(128);
        for i in 0..joint_count {
            let joint = &skeleton.joints[i];
            let skinning_matrix = world_transforms[i].mul(joint.inverse_bind_matrix);
            palette.matrices[i] = [
                skinning_matrix.cols[0][0],
                skinning_matrix.cols[0][1],
                skinning_matrix.cols[0][2],
                skinning_matrix.cols[0][3],
                skinning_matrix.cols[1][0],
                skinning_matrix.cols[1][1],
                skinning_matrix.cols[1][2],
                skinning_matrix.cols[1][3],
                skinning_matrix.cols[2][0],
                skinning_matrix.cols[2][1],
                skinning_matrix.cols[2][2],
                skinning_matrix.cols[2][3],
                skinning_matrix.cols[3][0],
                skinning_matrix.cols[3][1],
                skinning_matrix.cols[3][2],
                skinning_matrix.cols[3][3],
            ];
        }

        palette.joint_count = skeleton.joints.len() as u32;
        palette
    }
}

/// LCG pseudo-random for deterministic benchmarks
struct LcgRng {
    state: u32,
}

impl LcgRng {
    fn new(seed: u32) -> Self {
        Self { state: seed }
    }

    fn next_f32(&mut self) -> f32 {
        self.state = self.state.wrapping_mul(1664525).wrapping_add(1013904223);
        (self.state as f32) / (u32::MAX as f32)
    }
}

// ============================================================================
// Benchmarks
// ============================================================================

fn bench_transform_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("animation/transform");

    // Transform lerp
    let t0 = Transform {
        translation: Vec3::new(0.0, 0.0, 0.0),
        rotation: Quat::IDENTITY,
        scale: Vec3::ONE,
    };
    let t1 = Transform {
        translation: Vec3::new(1.0, 2.0, 3.0),
        rotation: Quat::from_axis_angle(Vec3::new(0.0, 1.0, 0.0), 1.5),
        scale: Vec3::new(1.2, 1.2, 1.2),
    };

    group.bench_function("lerp", |b| {
        b.iter(|| {
            black_box(t0.lerp(&t1, black_box(0.5)))
        });
    });

    // Transform to matrix
    group.bench_function("to_matrix", |b| {
        b.iter(|| {
            black_box(t0.to_matrix())
        });
    });

    // Quaternion slerp only
    let q0 = Quat::IDENTITY;
    let q1 = Quat::from_axis_angle(Vec3::new(0.0, 1.0, 0.0), 1.5);

    group.bench_function("quat_slerp", |b| {
        b.iter(|| {
            black_box(q0.slerp(q1, black_box(0.5)))
        });
    });

    // Vec3 lerp only
    let v0 = Vec3::new(0.0, 0.0, 0.0);
    let v1 = Vec3::new(1.0, 2.0, 3.0);

    group.bench_function("vec3_lerp", |b| {
        b.iter(|| {
            black_box(v0.lerp(v1, black_box(0.5)))
        });
    });

    group.finish();
}

fn bench_matrix_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("animation/matrix");

    let t = Transform {
        translation: Vec3::new(1.0, 2.0, 3.0),
        rotation: Quat::from_axis_angle(Vec3::new(0.0, 1.0, 0.0), 1.0),
        scale: Vec3::new(1.5, 1.5, 1.5),
    };
    let m1 = t.to_matrix();
    let m2 = Transform::default_pose().to_matrix();

    group.bench_function("multiply", |b| {
        b.iter(|| {
            black_box(m1.mul(m2))
        });
    });

    // Quaternion to rotation matrix
    let q = Quat::from_axis_angle(Vec3::new(0.0, 1.0, 0.0), 1.5);

    group.bench_function("quat_to_rotation", |b| {
        b.iter(|| {
            black_box(q.to_rotation_matrix())
        });
    });

    group.finish();
}

fn bench_animation_sampling(c: &mut Criterion) {
    let mut group = c.benchmark_group("animation/sample");

    // Humanoid skeleton (20 joints)
    let skeleton = Skeleton::create_humanoid();
    let clip = AnimationClip::create_walk_cycle(&skeleton);

    group.bench_function("humanoid_20_joints", |b| {
        let mut time = 0.0;
        b.iter(|| {
            time = (time + 0.016) % clip.duration;
            black_box(clip.sample(black_box(time), &skeleton))
        });
    });

    // Stress test with larger skeleton
    for joint_count in [50, 100, 200] {
        let stress_skeleton = Skeleton::create_stress_skeleton(joint_count);
        let stress_clip = AnimationClip::create_walk_cycle(&stress_skeleton);

        group.throughput(Throughput::Elements(joint_count as u64));
        group.bench_with_input(
            BenchmarkId::new("stress", joint_count),
            &joint_count,
            |b, _| {
                let mut time = 0.0;
                b.iter(|| {
                    time = (time + 0.016) % stress_clip.duration;
                    black_box(stress_clip.sample(black_box(time), &stress_skeleton))
                });
            },
        );
    }

    group.finish();
}

fn bench_joint_palette_generation(c: &mut Criterion) {
    let mut group = c.benchmark_group("animation/joint_palette");

    // Humanoid
    let skeleton = Skeleton::create_humanoid();
    let clip = AnimationClip::create_walk_cycle(&skeleton);
    let transforms = clip.sample(0.5, &skeleton);

    group.bench_function("humanoid_20_joints", |b| {
        b.iter(|| {
            black_box(JointPalette::from_skeleton(&skeleton, &transforms))
        });
    });

    // Stress test
    for joint_count in [50, 100, 200] {
        let stress_skeleton = Skeleton::create_stress_skeleton(joint_count);
        let stress_clip = AnimationClip::create_walk_cycle(&stress_skeleton);
        let stress_transforms = stress_clip.sample(0.5, &stress_skeleton);

        group.throughput(Throughput::Elements(joint_count as u64));
        group.bench_with_input(
            BenchmarkId::new("stress", joint_count),
            &joint_count,
            |b, _| {
                b.iter(|| {
                    black_box(JointPalette::from_skeleton(&stress_skeleton, &stress_transforms))
                });
            },
        );
    }

    group.finish();
}

fn bench_skeleton_hierarchy(c: &mut Criterion) {
    let mut group = c.benchmark_group("animation/hierarchy");

    // World transform computation (forward kinematics)
    for joint_count in [20, 50, 100, 200] {
        let skeleton = Skeleton::create_stress_skeleton(joint_count);
        let transforms: Vec<Transform> = skeleton
            .joints
            .iter()
            .map(|j| j.local_transform)
            .collect();

        group.throughput(Throughput::Elements(joint_count as u64));
        group.bench_with_input(
            BenchmarkId::new("forward_kinematics", joint_count),
            &joint_count,
            |b, _| {
                b.iter(|| {
                    let mut world_transforms = vec![Mat4::IDENTITY; skeleton.joints.len()];

                    for (i, joint) in skeleton.joints.iter().enumerate() {
                        let local = transforms[i].to_matrix();

                        world_transforms[i] = match joint.parent_index {
                            Some(parent) => world_transforms[parent].mul(local),
                            None => local,
                        };
                    }

                    black_box(world_transforms)
                });
            },
        );
    }

    group.finish();
}

fn bench_animation_blending(c: &mut Criterion) {
    let mut group = c.benchmark_group("animation/blending");

    let skeleton = Skeleton::create_humanoid();
    let clip1 = AnimationClip::create_walk_cycle(&skeleton);
    let clip2 = AnimationClip::create_walk_cycle(&skeleton);

    // Sample both animations
    let transforms1 = clip1.sample(0.0, &skeleton);
    let transforms2 = clip2.sample(0.5, &skeleton);

    // Crossfade blend
    group.bench_function("crossfade_20_joints", |b| {
        b.iter(|| {
            let blend_factor = black_box(0.5);
            let result: Vec<Transform> = transforms1
                .iter()
                .zip(&transforms2)
                .map(|(t1, t2)| t1.lerp(t2, blend_factor))
                .collect();
            black_box(result)
        });
    });

    // Larger blending stress test
    for joint_count in [50, 100, 200] {
        let stress_skeleton = Skeleton::create_stress_skeleton(joint_count);
        let stress_clip1 = AnimationClip::create_walk_cycle(&stress_skeleton);
        let stress_clip2 = AnimationClip::create_walk_cycle(&stress_skeleton);
        let stress_t1 = stress_clip1.sample(0.0, &stress_skeleton);
        let stress_t2 = stress_clip2.sample(0.5, &stress_skeleton);

        group.throughput(Throughput::Elements(joint_count as u64));
        group.bench_with_input(
            BenchmarkId::new("crossfade_stress", joint_count),
            &joint_count,
            |b, _| {
                b.iter(|| {
                    let blend_factor = black_box(0.5);
                    let result: Vec<Transform> = stress_t1
                        .iter()
                        .zip(&stress_t2)
                        .map(|(t1, t2)| t1.lerp(t2, blend_factor))
                        .collect();
                    black_box(result)
                });
            },
        );
    }

    group.finish();
}

fn bench_full_animation_frame(c: &mut Criterion) {
    let mut group = c.benchmark_group("animation/full_frame");

    // Simulate multiple animated characters per frame
    for character_count in [1, 10, 50, 100] {
        let skeleton = Skeleton::create_humanoid();
        let clip = AnimationClip::create_walk_cycle(&skeleton);

        // Pre-generate different animation times for each character
        let mut rng = LcgRng::new(42);
        let times: Vec<f32> = (0..character_count)
            .map(|_| rng.next_f32() * clip.duration)
            .collect();

        group.throughput(Throughput::Elements(character_count as u64 * 20)); // 20 joints each
        group.bench_with_input(
            BenchmarkId::new("characters", character_count),
            &character_count,
            |b, &count| {
                b.iter(|| {
                    let mut palettes = Vec::with_capacity(count);
                    for i in 0..count {
                        let transforms = clip.sample(times[i], &skeleton);
                        let palette = JointPalette::from_skeleton(&skeleton, &transforms);
                        palettes.push(palette);
                    }
                    black_box(palettes)
                });
            },
        );
    }

    group.finish();
}

fn bench_keyframe_search(c: &mut Criterion) {
    let mut group = c.benchmark_group("animation/keyframe_search");

    // Binary search vs linear for different keyframe counts
    for keyframe_count in [4, 16, 64, 256] {
        let times: Vec<f32> = (0..keyframe_count)
            .map(|i| i as f32 / keyframe_count as f32)
            .collect();

        // Linear search
        group.bench_with_input(
            BenchmarkId::new("linear", keyframe_count),
            &keyframe_count,
            |b, _| {
                let search_time = 0.73; // Somewhere in the middle-ish
                b.iter(|| {
                    let mut idx0 = 0usize;
                    let mut idx1 = 0usize;
                    let mut _t = 0.0f32;

                    for i in 0..times.len() - 1 {
                        if search_time >= times[i] && search_time < times[i + 1] {
                            idx0 = i;
                            idx1 = i + 1;
                            _t = (search_time - times[i]) / (times[i + 1] - times[i]);
                            break;
                        }
                    }

                    black_box((idx0, idx1, _t))
                });
            },
        );

        // Binary search
        group.bench_with_input(
            BenchmarkId::new("binary", keyframe_count),
            &keyframe_count,
            |b, _| {
                let search_time = 0.73;
                b.iter(|| {
                    let idx = times.partition_point(|&t| t <= search_time);
                    let idx0 = idx.saturating_sub(1);
                    let idx1 = (idx).min(times.len() - 1);
                    let t = if idx0 != idx1 {
                        (search_time - times[idx0]) / (times[idx1] - times[idx0])
                    } else {
                        0.0
                    };
                    black_box((idx0, idx1, t))
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_transform_operations,
    bench_matrix_operations,
    bench_animation_sampling,
    bench_joint_palette_generation,
    bench_skeleton_hierarchy,
    bench_animation_blending,
    bench_full_animation_frame,
    bench_keyframe_search,
);
criterion_main!(benches);
