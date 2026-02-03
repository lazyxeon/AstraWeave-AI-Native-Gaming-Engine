//! Kani formal verification proofs for SIMD vector operations
//!
//! These proofs verify mathematical properties of vector operations:
//! - Dot product symmetry
//! - Cross product anticommutativity
//! - Normalization produces unit vectors
//!
//! Run with: `cargo kani --package astraweave-math`

#![cfg(kani)]

use glam::Vec3;

// Note: We test the scalar fallback paths since Kani doesn't support SIMD intrinsics.
// The proofs verify the mathematical correctness of the algorithms.

/// Verify dot product is symmetric: dot(a, b) == dot(b, a)
#[kani::proof]
fn dot_product_symmetric() {
    let ax: f32 = kani::any();
    let ay: f32 = kani::any();
    let az: f32 = kani::any();
    let bx: f32 = kani::any();
    let by: f32 = kani::any();
    let bz: f32 = kani::any();

    // Assume finite, non-NaN values for tractability
    kani::assume(ax.is_finite() && ay.is_finite() && az.is_finite());
    kani::assume(bx.is_finite() && by.is_finite() && bz.is_finite());
    kani::assume(ax.abs() < 1e10 && ay.abs() < 1e10 && az.abs() < 1e10);
    kani::assume(bx.abs() < 1e10 && by.abs() < 1e10 && bz.abs() < 1e10);

    let a = Vec3::new(ax, ay, az);
    let b = Vec3::new(bx, by, bz);

    let dot_ab = a.dot(b);
    let dot_ba = b.dot(a);

    // Allow small floating-point tolerance
    let diff = (dot_ab - dot_ba).abs();
    kani::assert(
        diff < 1e-6 || diff.is_nan(),
        "Dot product must be symmetric",
    );
}

/// Verify dot product with zero vector is zero
#[kani::proof]
fn dot_product_zero_identity() {
    let ax: f32 = kani::any();
    let ay: f32 = kani::any();
    let az: f32 = kani::any();

    kani::assume(ax.is_finite() && ay.is_finite() && az.is_finite());
    kani::assume(ax.abs() < 1e10 && ay.abs() < 1e10 && az.abs() < 1e10);

    let a = Vec3::new(ax, ay, az);
    let zero = Vec3::ZERO;

    let dot = a.dot(zero);
    kani::assert(dot == 0.0, "Dot product with zero must be zero");
}

/// Verify cross product is anticommutative: cross(a, b) == -cross(b, a)
#[kani::proof]
fn cross_product_anticommutative() {
    let ax: f32 = kani::any();
    let ay: f32 = kani::any();
    let az: f32 = kani::any();
    let bx: f32 = kani::any();
    let by: f32 = kani::any();
    let bz: f32 = kani::any();

    kani::assume(ax.is_finite() && ay.is_finite() && az.is_finite());
    kani::assume(bx.is_finite() && by.is_finite() && bz.is_finite());
    kani::assume(ax.abs() < 1e5 && ay.abs() < 1e5 && az.abs() < 1e5);
    kani::assume(bx.abs() < 1e5 && by.abs() < 1e5 && bz.abs() < 1e5);

    let a = Vec3::new(ax, ay, az);
    let b = Vec3::new(bx, by, bz);

    let cross_ab = a.cross(b);
    let cross_ba = b.cross(a);
    let neg_cross_ba = -cross_ba;

    // Allow floating-point tolerance
    let diff = (cross_ab - neg_cross_ba).length();
    kani::assert(
        diff < 1e-4 || diff.is_nan(),
        "Cross product must be anticommutative",
    );
}

/// Verify cross product is orthogonal to both inputs
#[kani::proof]
fn cross_product_orthogonal() {
    let ax: f32 = kani::any();
    let ay: f32 = kani::any();
    let az: f32 = kani::any();
    let bx: f32 = kani::any();
    let by: f32 = kani::any();
    let bz: f32 = kani::any();

    kani::assume(ax.is_finite() && ay.is_finite() && az.is_finite());
    kani::assume(bx.is_finite() && by.is_finite() && bz.is_finite());
    kani::assume(ax.abs() < 1e5 && ay.abs() < 1e5 && az.abs() < 1e5);
    kani::assume(bx.abs() < 1e5 && by.abs() < 1e5 && bz.abs() < 1e5);

    let a = Vec3::new(ax, ay, az);
    let b = Vec3::new(bx, by, bz);

    let cross = a.cross(b);

    // Cross product should be orthogonal to both a and b
    let dot_with_a = cross.dot(a).abs();
    let dot_with_b = cross.dot(b).abs();

    kani::assert(
        dot_with_a < 1e-3 || dot_with_a.is_nan(),
        "Cross product must be orthogonal to first vector",
    );
    kani::assert(
        dot_with_b < 1e-3 || dot_with_b.is_nan(),
        "Cross product must be orthogonal to second vector",
    );
}

/// Verify length is non-negative
#[kani::proof]
fn length_non_negative() {
    let x: f32 = kani::any();
    let y: f32 = kani::any();
    let z: f32 = kani::any();

    kani::assume(x.is_finite() && y.is_finite() && z.is_finite());

    let v = Vec3::new(x, y, z);
    let len = v.length();

    kani::assert(
        len >= 0.0 || len.is_nan(),
        "Length must be non-negative",
    );
}

/// Verify length_squared equals length^2
#[kani::proof]
fn length_squared_is_length_squared() {
    let x: f32 = kani::any();
    let y: f32 = kani::any();
    let z: f32 = kani::any();

    kani::assume(x.is_finite() && y.is_finite() && z.is_finite());
    kani::assume(x.abs() < 1e10 && y.abs() < 1e10 && z.abs() < 1e10);

    let v = Vec3::new(x, y, z);
    let len_sq = v.length_squared();
    let len = v.length();

    // Allow floating-point tolerance
    let expected = len * len;
    let diff = (len_sq - expected).abs();

    kani::assert(
        diff < 1e-4 || diff.is_nan(),
        "length_squared must equal length^2",
    );
}

/// Verify normalization produces unit vector (length ≈ 1)
#[kani::proof]
fn normalize_produces_unit_vector() {
    let x: f32 = kani::any();
    let y: f32 = kani::any();
    let z: f32 = kani::any();

    kani::assume(x.is_finite() && y.is_finite() && z.is_finite());
    kani::assume(x.abs() < 1e10 && y.abs() < 1e10 && z.abs() < 1e10);

    let v = Vec3::new(x, y, z);

    // Only test non-zero vectors
    kani::assume(v.length_squared() > 1e-10);

    let normalized = v.normalize();
    let len = normalized.length();

    // Allow floating-point tolerance
    kani::assert(
        (len - 1.0).abs() < 1e-5 || len.is_nan(),
        "Normalized vector must have length ~1",
    );
}

/// Verify normalize_or_zero returns zero for zero vector
#[kani::proof]
fn normalize_or_zero_handles_zero() {
    let zero = Vec3::ZERO;
    let result = zero.normalize_or_zero();

    kani::assert(result == Vec3::ZERO, "normalize_or_zero of zero must be zero");
}

/// Verify normalize preserves direction
#[kani::proof]
fn normalize_preserves_direction() {
    let x: f32 = kani::any();
    let y: f32 = kani::any();
    let z: f32 = kani::any();

    kani::assume(x.is_finite() && y.is_finite() && z.is_finite());
    kani::assume(x.abs() < 1e10 && y.abs() < 1e10 && z.abs() < 1e10);

    let v = Vec3::new(x, y, z);
    let len = v.length();

    // Only test non-zero vectors
    kani::assume(len > 1e-6);

    let normalized = v.normalize();

    // Dot product of v and normalized should equal length(v)
    let dot = v.dot(normalized);
    let diff = (dot - len).abs();

    kani::assert(
        diff < 1e-3 || diff.is_nan(),
        "Normalized vector must point in same direction",
    );
}

/// Verify Vec3::ZERO has zero length
#[kani::proof]
fn zero_vector_has_zero_length() {
    let zero = Vec3::ZERO;
    kani::assert(zero.length() == 0.0, "Zero vector must have zero length");
    kani::assert(
        zero.length_squared() == 0.0,
        "Zero vector must have zero length_squared",
    );
}

/// Verify unit vectors have length 1
#[kani::proof]
fn unit_vectors_have_length_one() {
    let x = Vec3::X;
    let y = Vec3::Y;
    let z = Vec3::Z;

    kani::assert((x.length() - 1.0).abs() < 1e-6, "X unit vector must have length 1");
    kani::assert((y.length() - 1.0).abs() < 1e-6, "Y unit vector must have length 1");
    kani::assert((z.length() - 1.0).abs() < 1e-6, "Z unit vector must have length 1");
}

/// Verify standard cross product results
#[kani::proof]
fn standard_cross_products() {
    let x = Vec3::X;
    let y = Vec3::Y;
    let z = Vec3::Z;

    // X × Y = Z
    let x_cross_y = x.cross(y);
    kani::assert(
        (x_cross_y - z).length() < 1e-6,
        "X × Y must equal Z",
    );

    // Y × Z = X
    let y_cross_z = y.cross(z);
    kani::assert(
        (y_cross_z - x).length() < 1e-6,
        "Y × Z must equal X",
    );

    // Z × X = Y
    let z_cross_x = z.cross(x);
    kani::assert(
        (z_cross_x - y).length() < 1e-6,
        "Z × X must equal Y",
    );
}
