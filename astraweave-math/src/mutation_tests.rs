// Copyright 2025 AstraWeave Contributors
// SPDX-License-Identifier: MIT

//! # Mutation Testing Suite for astraweave-math
//!
//! This module contains comprehensive mutation tests designed to detect
//! common mutation categories:
//!
//! 1. **Boundary Condition Mutations** - Tests that catch < vs <=, > vs >=, etc.
//! 2. **Comparison Operator Mutations** - Tests that catch == vs !=, sign inversions
//! 3. **Boolean Return Path Mutations** - Tests that catch logic inversions, early returns
//!
//! These tests ensure that mutants in critical SIMD math operations are killed
//! by our test suite, providing confidence in mathematical correctness.

#[cfg(test)]
mod boundary_condition_tests {
    //! Tests designed to kill boundary condition mutants (< vs <=, > vs >=)
    //!
    //! These tests use exact boundary values where off-by-one mutations
    //! would cause test failures.

    use glam::{Mat4, Quat, Vec3};

    use crate::simd_mat::{inverse_simd, mul_simd, transform_point_simd, transpose_simd};
    use crate::simd_movement::update_positions_simd;
    use crate::simd_quat::{mul_quat_simd, normalize_quat_simd, slerp_simd};
    use crate::simd_vec::{cross_simd, dot_simd, length_simd, length_squared_simd, normalize_simd};

    // ========================================================================
    // Vec3 SIMD Boundary Tests
    // ========================================================================

    #[test]
    fn test_normalize_zero_vector_boundary() {
        // Exact zero vector - critical boundary
        let zero = Vec3::ZERO;
        let result = normalize_simd(zero);
        // Zero vector should normalize to zero (not NaN or panic)
        assert!(result.x.is_finite(), "Zero vector normalize should not produce NaN");
        assert!(result.y.is_finite(), "Zero vector normalize should not produce NaN");
        assert!(result.z.is_finite(), "Zero vector normalize should not produce NaN");
    }

    #[test]
    fn test_normalize_near_zero_boundary() {
        // Near-zero but not exactly zero - tests epsilon handling
        let near_zero = Vec3::new(1e-10, 0.0, 0.0);
        let result = normalize_simd(near_zero);
        // Should either normalize or return safe value
        assert!(result.x.is_finite(), "Near-zero normalize should be finite");
    }

    #[test]
    fn test_normalize_unit_vector_boundary() {
        // Exact unit vector - result should be identical
        let unit = Vec3::new(1.0, 0.0, 0.0);
        let result = normalize_simd(unit);
        assert!((result.x - 1.0).abs() < 1e-6, "Unit X should remain 1.0");
        assert!((result.y - 0.0).abs() < 1e-6, "Unit Y should remain 0.0");
        assert!((result.z - 0.0).abs() < 1e-6, "Unit Z should remain 0.0");
    }

    #[test]
    fn test_normalize_maximum_component_boundary() {
        // Large magnitude vector - tests handling without overflow
        // Use moderately large values that won't cause precision issues
        let large = Vec3::new(1e6, 1e6, 1e6);
        let result = normalize_simd(large);
        let expected_component = 1.0 / 3.0_f32.sqrt();
        assert!(
            (result.x - expected_component).abs() < 1e-4,
            "Large vector should normalize correctly: got {}, expected {}",
            result.x,
            expected_component
        );
    }

    #[test]
    fn test_length_zero_vector_boundary() {
        let zero = Vec3::ZERO;
        let result = length_simd(zero);
        assert_eq!(result, 0.0, "Zero vector length must be exactly 0.0");
    }

    #[test]
    fn test_length_unit_axis_boundaries() {
        // Each axis unit vector should have length exactly 1.0
        let unit_x = Vec3::new(1.0, 0.0, 0.0);
        let unit_y = Vec3::new(0.0, 1.0, 0.0);
        let unit_z = Vec3::new(0.0, 0.0, 1.0);

        assert!((length_simd(unit_x) - 1.0).abs() < 1e-6, "Unit X length should be 1.0");
        assert!((length_simd(unit_y) - 1.0).abs() < 1e-6, "Unit Y length should be 1.0");
        assert!((length_simd(unit_z) - 1.0).abs() < 1e-6, "Unit Z length should be 1.0");
    }

    #[test]
    fn test_length_squared_preserves_precision() {
        // length_squared should NOT take sqrt - test precision
        let v = Vec3::new(3.0, 4.0, 0.0);
        let len_sq = length_squared_simd(v);
        assert_eq!(len_sq, 25.0, "3² + 4² should be exactly 25");
    }

    #[test]
    fn test_dot_product_zero_boundary() {
        // Perpendicular vectors - dot product exactly 0.0
        let a = Vec3::new(1.0, 0.0, 0.0);
        let b = Vec3::new(0.0, 1.0, 0.0);
        let result = dot_simd(a, b);
        assert_eq!(result, 0.0, "Perpendicular vectors dot product must be 0.0");
    }

    #[test]
    fn test_dot_product_parallel_boundary() {
        // Parallel vectors - dot product equals product of lengths
        let a = Vec3::new(3.0, 0.0, 0.0);
        let b = Vec3::new(4.0, 0.0, 0.0);
        let result = dot_simd(a, b);
        assert_eq!(result, 12.0, "Parallel vectors: 3 * 4 = 12");
    }

    #[test]
    fn test_cross_product_parallel_boundary() {
        // Parallel vectors - cross product is exactly zero
        let a = Vec3::new(1.0, 0.0, 0.0);
        let b = Vec3::new(2.0, 0.0, 0.0);
        let result = cross_simd(a, b);
        assert_eq!(result, Vec3::ZERO, "Parallel vectors cross product must be zero");
    }

    // ========================================================================
    // Quaternion SIMD Boundary Tests
    // ========================================================================

    #[test]
    fn test_slerp_t_zero_boundary() {
        // t = 0.0: result must be exactly a
        let a = Quat::from_rotation_x(0.0);
        let b = Quat::from_rotation_x(std::f32::consts::PI);
        let result = slerp_simd(a, b, 0.0);
        assert!(
            (result.x - a.x).abs() < 1e-6,
            "slerp(a, b, 0.0) must equal a"
        );
        assert!(
            (result.y - a.y).abs() < 1e-6,
            "slerp(a, b, 0.0) must equal a"
        );
        assert!(
            (result.z - a.z).abs() < 1e-6,
            "slerp(a, b, 0.0) must equal a"
        );
        assert!(
            (result.w - a.w).abs() < 1e-6,
            "slerp(a, b, 0.0) must equal a"
        );
    }

    #[test]
    fn test_slerp_t_one_boundary() {
        // t = 1.0: result must be exactly b (or -b if shortest path taken)
        let a = Quat::from_rotation_x(0.0);
        let b = Quat::from_rotation_x(std::f32::consts::PI);
        let result = slerp_simd(a, b, 1.0);
        // slerp may take shortest path, so check both b and -b
        let matches_b = (result.x - b.x).abs() < 1e-5
            && (result.y - b.y).abs() < 1e-5
            && (result.z - b.z).abs() < 1e-5
            && (result.w - b.w).abs() < 1e-5;
        let matches_neg_b = (result.x + b.x).abs() < 1e-5
            && (result.y + b.y).abs() < 1e-5
            && (result.z + b.z).abs() < 1e-5
            && (result.w + b.w).abs() < 1e-5;
        assert!(
            matches_b || matches_neg_b,
            "slerp(a, b, 1.0) must equal b or -b: got {:?}, expected {:?}",
            result,
            b
        );
    }

    #[test]
    fn test_slerp_t_half_boundary() {
        // t = 0.5: result must be exactly halfway
        let a = Quat::IDENTITY;
        let b = Quat::from_rotation_z(std::f32::consts::FRAC_PI_2); // 90 degrees
        let result = slerp_simd(a, b, 0.5);
        // At 0.5, should be 45 degree rotation
        let expected = Quat::from_rotation_z(std::f32::consts::FRAC_PI_4);
        assert!(
            (result.x - expected.x).abs() < 1e-5,
            "slerp midpoint X: got {}, expected {}",
            result.x,
            expected.x
        );
        assert!(
            (result.w - expected.w).abs() < 1e-5,
            "slerp midpoint W: got {}, expected {}",
            result.w,
            expected.w
        );
    }

    #[test]
    fn test_normalize_quat_identity_boundary() {
        // Identity quaternion is already normalized
        let identity = Quat::IDENTITY;
        let result = normalize_quat_simd(identity);
        assert!(
            (result.x - identity.x).abs() < 1e-6,
            "Identity quat normalization should preserve X"
        );
        assert!(
            (result.w - identity.w).abs() < 1e-6,
            "Identity quat normalization should preserve W"
        );
    }

    #[test]
    fn test_normalize_quat_unnormalized_boundary() {
        // Scaled quaternion - should normalize to unit length
        let scaled = Quat::from_xyzw(0.0, 0.0, 0.0, 2.0); // Length 2
        let result = normalize_quat_simd(scaled);
        let length = (result.x * result.x
            + result.y * result.y
            + result.z * result.z
            + result.w * result.w)
            .sqrt();
        assert!(
            (length - 1.0).abs() < 1e-6,
            "Normalized quat length should be 1.0"
        );
    }

    #[test]
    fn test_mul_quat_identity_boundary() {
        // Multiply by identity should return original
        let q = Quat::from_rotation_y(1.5);
        let identity = Quat::IDENTITY;

        let left_id = mul_quat_simd(identity, q);
        let right_id = mul_quat_simd(q, identity);

        assert!(
            (left_id.x - q.x).abs() < 1e-6,
            "Identity * Q should equal Q"
        );
        assert!(
            (right_id.x - q.x).abs() < 1e-6,
            "Q * Identity should equal Q"
        );
    }

    // ========================================================================
    // Matrix SIMD Boundary Tests
    // ========================================================================

    #[test]
    fn test_mul_identity_boundary() {
        // Identity * M = M
        let m = Mat4::from_scale(Vec3::new(2.0, 3.0, 4.0));
        let result = mul_simd(Mat4::IDENTITY, m);
        assert!(
            (result.col(0).x - m.col(0).x).abs() < 1e-6,
            "Identity * M should equal M"
        );
    }

    #[test]
    fn test_transpose_identity_boundary() {
        // Identity transpose equals identity
        let result = transpose_simd(Mat4::IDENTITY);
        assert!(
            (result.col(0).x - 1.0).abs() < 1e-6,
            "Identity transpose [0][0] should be 1.0"
        );
        assert!(
            (result.col(1).y - 1.0).abs() < 1e-6,
            "Identity transpose [1][1] should be 1.0"
        );
    }

    #[test]
    fn test_inverse_identity_boundary() {
        // Identity inverse equals identity
        let result = inverse_simd(Mat4::IDENTITY);
        assert!(
            (result.col(0).x - 1.0).abs() < 1e-6,
            "Identity inverse [0][0] should be 1.0"
        );
    }

    #[test]
    fn test_transform_point_origin_boundary() {
        // Transform origin by any matrix still depends on translation
        let transform = Mat4::from_translation(Vec3::new(5.0, 0.0, 0.0));
        let origin = Vec3::ZERO;
        let result = transform_point_simd(transform, origin);
        assert!(
            (result.x - 5.0).abs() < 1e-6,
            "Origin transformed by translation should move"
        );
    }

    // ========================================================================
    // Movement SIMD Boundary Tests
    // ========================================================================

    #[test]
    fn test_update_positions_zero_dt_boundary() {
        // dt = 0: positions should not change
        let mut positions = vec![Vec3::new(1.0, 2.0, 3.0)];
        let velocities = vec![Vec3::new(100.0, 200.0, 300.0)];
        update_positions_simd(&mut positions, &velocities, 0.0);
        assert_eq!(positions[0], Vec3::new(1.0, 2.0, 3.0), "Zero dt should not move");
    }

    #[test]
    fn test_update_positions_zero_velocity_boundary() {
        // Zero velocity: positions should not change
        let mut positions = vec![Vec3::new(1.0, 2.0, 3.0)];
        let velocities = vec![Vec3::ZERO];
        update_positions_simd(&mut positions, &velocities, 1.0);
        assert_eq!(positions[0], Vec3::new(1.0, 2.0, 3.0), "Zero velocity should not move");
    }

    #[test]
    fn test_update_positions_single_entity_boundary() {
        // Single entity - tests non-batch path
        let mut positions = vec![Vec3::ZERO];
        let velocities = vec![Vec3::new(10.0, 0.0, 0.0)];
        update_positions_simd(&mut positions, &velocities, 0.1);
        assert!((positions[0].x - 1.0).abs() < 1e-6, "10 * 0.1 = 1.0");
    }

    #[test]
    fn test_update_positions_batch_boundary() {
        // Exactly 4 entities - tests batch boundary
        let mut positions = vec![Vec3::ZERO; 4];
        let velocities = vec![Vec3::new(1.0, 0.0, 0.0); 4];
        update_positions_simd(&mut positions, &velocities, 1.0);
        for (i, pos) in positions.iter().enumerate() {
            assert!((pos.x - 1.0).abs() < 1e-6, "Entity {} should move by 1.0", i);
        }
    }

    #[test]
    fn test_update_positions_batch_plus_remainder() {
        // 5 entities = 1 batch of 4 + 1 remainder
        let mut positions = vec![Vec3::ZERO; 5];
        let velocities = vec![Vec3::new(2.0, 0.0, 0.0); 5];
        update_positions_simd(&mut positions, &velocities, 0.5);
        for (i, pos) in positions.iter().enumerate() {
            assert!((pos.x - 1.0).abs() < 1e-6, "Entity {} should move by 1.0", i);
        }
    }
}

#[cfg(test)]
mod comparison_operator_tests {
    //! Tests designed to kill comparison operator mutants (== vs !=, < vs >)
    //!
    //! These tests verify correct comparison behavior by testing both positive
    //! and negative cases explicitly.

    use glam::{Mat4, Quat, Vec3};

    use crate::simd_mat::{inverse_simd, mul_simd, transpose_simd};
    use crate::simd_quat::{dot_quat_simd, mul_quat_simd, normalize_quat_simd, slerp_simd};
    use crate::simd_vec::{cross_simd, dot_simd, length_simd, normalize_simd};

    // ========================================================================
    // Vec3 Comparison Tests
    // ========================================================================

    #[test]
    fn test_dot_product_positive_vs_negative() {
        // Parallel same direction: positive
        let a = Vec3::new(1.0, 0.0, 0.0);
        let b = Vec3::new(1.0, 0.0, 0.0);
        assert!(dot_simd(a, b) > 0.0, "Same direction dot should be positive");

        // Parallel opposite direction: negative
        let c = Vec3::new(-1.0, 0.0, 0.0);
        assert!(dot_simd(a, c) < 0.0, "Opposite direction dot should be negative");
    }

    #[test]
    fn test_dot_product_magnitude_comparison() {
        // Longer vectors have larger dot products
        let a = Vec3::new(2.0, 0.0, 0.0);
        let b_short = Vec3::new(1.0, 0.0, 0.0);
        let b_long = Vec3::new(3.0, 0.0, 0.0);

        let dot_short = dot_simd(a, b_short);
        let dot_long = dot_simd(a, b_long);

        assert!(dot_long > dot_short, "Longer vector should produce larger dot product");
    }

    #[test]
    fn test_cross_product_direction_comparison() {
        // Cross product direction follows right-hand rule
        let x = Vec3::new(1.0, 0.0, 0.0);
        let y = Vec3::new(0.0, 1.0, 0.0);

        let cross_xy = cross_simd(x, y);
        let cross_yx = cross_simd(y, x);

        // X × Y = +Z
        assert!(cross_xy.z > 0.0, "X × Y should point in +Z direction");
        // Y × X = -Z
        assert!(cross_yx.z < 0.0, "Y × X should point in -Z direction");
    }

    #[test]
    fn test_normalize_preserves_direction() {
        // Positive components should stay positive
        let positive = Vec3::new(3.0, 4.0, 5.0);
        let result = normalize_simd(positive);
        assert!(result.x > 0.0, "Positive X should stay positive after normalize");
        assert!(result.y > 0.0, "Positive Y should stay positive after normalize");
        assert!(result.z > 0.0, "Positive Z should stay positive after normalize");

        // Negative components should stay negative
        let negative = Vec3::new(-3.0, -4.0, -5.0);
        let result_neg = normalize_simd(negative);
        assert!(result_neg.x < 0.0, "Negative X should stay negative after normalize");
    }

    #[test]
    fn test_length_comparison_invariant() {
        // Length should be invariant to sign
        let positive = Vec3::new(3.0, 4.0, 0.0);
        let negative = Vec3::new(-3.0, -4.0, 0.0);
        let mixed = Vec3::new(-3.0, 4.0, 0.0);

        let len_pos = length_simd(positive);
        let len_neg = length_simd(negative);
        let len_mix = length_simd(mixed);

        assert!(
            (len_pos - len_neg).abs() < 1e-6,
            "Length should be sign-invariant"
        );
        assert!(
            (len_pos - len_mix).abs() < 1e-6,
            "Length should be sign-invariant"
        );
    }

    // ========================================================================
    // Quaternion Comparison Tests
    // ========================================================================

    #[test]
    fn test_quat_dot_sign_comparison() {
        // Same quaternion: dot = 1.0
        let q = Quat::from_rotation_y(1.0);
        let dot_same = dot_quat_simd(q, q);
        assert!(dot_same > 0.0, "Same quaternion dot should be positive");

        // Opposite quaternion: dot = -1.0
        let neg_q = Quat::from_xyzw(-q.x, -q.y, -q.z, -q.w);
        let dot_opp = dot_quat_simd(q, neg_q);
        assert!(dot_opp < 0.0, "Opposite quaternion dot should be negative");
    }

    #[test]
    fn test_slerp_interpolation_monotonic() {
        // Slerp should be monotonically increasing with t
        let a = Quat::IDENTITY;
        let b = Quat::from_rotation_z(std::f32::consts::PI);

        let t_25 = slerp_simd(a, b, 0.25);
        let t_50 = slerp_simd(a, b, 0.50);
        let t_75 = slerp_simd(a, b, 0.75);

        // Distance from a should increase monotonically
        let dist_25 = (1.0 - dot_quat_simd(a, t_25)).abs();
        let dist_50 = (1.0 - dot_quat_simd(a, t_50)).abs();
        let dist_75 = (1.0 - dot_quat_simd(a, t_75)).abs();

        assert!(dist_50 > dist_25, "t=0.5 should be farther from a than t=0.25");
        assert!(dist_75 > dist_50, "t=0.75 should be farther from a than t=0.5");
    }

    #[test]
    fn test_quat_multiply_order_matters() {
        // Quaternion multiplication is NOT commutative
        let q1 = Quat::from_rotation_x(std::f32::consts::FRAC_PI_2);
        let q2 = Quat::from_rotation_y(std::f32::consts::FRAC_PI_2);

        let q1_then_q2 = mul_quat_simd(q1, q2);
        let q2_then_q1 = mul_quat_simd(q2, q1);

        // Results should be different (non-commutative)
        let diff = (q1_then_q2.x - q2_then_q1.x).abs()
            + (q1_then_q2.y - q2_then_q1.y).abs()
            + (q1_then_q2.z - q2_then_q1.z).abs()
            + (q1_then_q2.w - q2_then_q1.w).abs();

        assert!(diff > 0.01, "Quaternion multiplication should be non-commutative");
    }

    #[test]
    fn test_normalize_quat_length_comparison() {
        // Unnormalized quaternion should have different length before vs after
        let unnorm = Quat::from_xyzw(2.0, 0.0, 0.0, 2.0);
        let len_before = (unnorm.x * unnorm.x
            + unnorm.y * unnorm.y
            + unnorm.z * unnorm.z
            + unnorm.w * unnorm.w)
            .sqrt();

        let normed = normalize_quat_simd(unnorm);
        let len_after = (normed.x * normed.x
            + normed.y * normed.y
            + normed.z * normed.z
            + normed.w * normed.w)
            .sqrt();

        assert!(len_before > len_after, "Unnormalized should be longer before normalize");
        assert!((len_after - 1.0).abs() < 1e-6, "Normalized length should be 1.0");
    }

    // ========================================================================
    // Matrix Comparison Tests
    // ========================================================================

    #[test]
    fn test_matrix_multiply_order_matters() {
        // Matrix multiplication is NOT commutative for different transform types
        // Use translation + rotation which definitely don't commute
        let translate = Mat4::from_translation(Vec3::new(5.0, 0.0, 0.0));
        let rotate = Mat4::from_rotation_z(std::f32::consts::FRAC_PI_2); // 90 degrees

        let translate_then_rotate = mul_simd(rotate, translate); // rotate applied after translate
        let rotate_then_translate = mul_simd(translate, rotate); // translate applied after rotate

        // Transform a test point to verify order matters
        let test_point = Vec3::new(1.0, 0.0, 0.0);
        let p1 = translate_then_rotate.transform_point3(test_point);
        let p2 = rotate_then_translate.transform_point3(test_point);

        let diff = (p1.x - p2.x).abs() + (p1.y - p2.y).abs() + (p1.z - p2.z).abs();
        assert!(diff > 0.1, "Matrix multiplication should be non-commutative: diff = {}", diff);
    }

    #[test]
    fn test_transpose_involution() {
        // (M^T)^T = M
        let m = Mat4::from_cols(
            glam::Vec4::new(1.0, 2.0, 3.0, 4.0),
            glam::Vec4::new(5.0, 6.0, 7.0, 8.0),
            glam::Vec4::new(9.0, 10.0, 11.0, 12.0),
            glam::Vec4::new(13.0, 14.0, 15.0, 16.0),
        );

        let t1 = transpose_simd(m);
        let t2 = transpose_simd(t1);

        assert!(
            (t2.col(0).x - m.col(0).x).abs() < 1e-6,
            "Double transpose should equal original"
        );
    }

    #[test]
    fn test_inverse_determinant_sign() {
        // Invertible matrix should have non-zero determinant
        let invertible = Mat4::from_scale(Vec3::new(2.0, 3.0, 4.0));
        let det = invertible.determinant();
        assert!(det.abs() > 1e-6, "Invertible matrix determinant should be non-zero");

        // Inverse should also be invertible
        let inv = inverse_simd(invertible);
        let inv_det = inv.determinant();
        assert!(inv_det.abs() > 1e-6, "Inverse determinant should be non-zero");
    }
}

#[cfg(test)]
mod boolean_return_path_tests {
    //! Tests designed to kill boolean return path mutants
    //!
    //! These tests verify that functions correctly handle edge cases and
    //! return appropriate values for degenerate inputs.

    use glam::{Mat4, Quat, Vec3};

    use crate::simd_mat::{inverse_simd, mul_simd, transform_point_simd, transform_points_batch};
    use crate::simd_movement::update_positions_simd;
    use crate::simd_quat::{normalize_batch as normalize_quat_batch, normalize_quat_simd, slerp_batch, slerp_simd};
    use crate::simd_vec::{length_simd, normalize_simd};

    // ========================================================================
    // Vec3 Boolean Path Tests
    // ========================================================================

    #[test]
    fn test_normalize_returns_valid_for_any_input() {
        // All inputs should return finite, non-NaN results
        let test_cases = [
            Vec3::ZERO,
            Vec3::new(1e-20, 0.0, 0.0),  // Near zero
            Vec3::new(1e20, 0.0, 0.0),   // Large
            Vec3::ONE,
            Vec3::NEG_ONE,
        ];

        for (i, v) in test_cases.iter().enumerate() {
            let result = normalize_simd(*v);
            assert!(
                result.x.is_finite() && result.y.is_finite() && result.z.is_finite(),
                "normalize test case {} should return finite values",
                i
            );
        }
    }

    #[test]
    fn test_length_always_non_negative() {
        // Length should never be negative
        let test_cases = [
            Vec3::ZERO,
            Vec3::new(-5.0, -5.0, -5.0),
            Vec3::new(1e30, 0.0, 0.0),
        ];

        for (i, v) in test_cases.iter().enumerate() {
            let len = length_simd(*v);
            assert!(len >= 0.0, "length test case {} should be non-negative", i);
        }
    }

    // ========================================================================
    // Quaternion Boolean Path Tests
    // ========================================================================

    #[test]
    fn test_normalize_quat_handles_degenerate() {
        // Near-zero quaternion edge case (exact zero may produce NaN in some implementations)
        let near_zero_quat = Quat::from_xyzw(1e-10, 0.0, 0.0, 1e-10);
        let result = normalize_quat_simd(near_zero_quat);
        // Should return some valid quaternion (ideally normalized)
        let is_valid = result.x.is_finite()
            && result.y.is_finite()
            && result.z.is_finite()
            && result.w.is_finite();
        assert!(
            is_valid,
            "Near-zero quat normalize should produce finite values, got {:?}",
            result
        );
    }

    #[test]
    fn test_slerp_handles_identical_quaternions() {
        // When a == b, slerp should return a/b for any t
        let q = Quat::from_rotation_x(1.0);
        let result = slerp_simd(q, q, 0.5);
        assert!(
            (result.x - q.x).abs() < 1e-6 && (result.w - q.w).abs() < 1e-6,
            "slerp of identical quaternions should return same quaternion"
        );
    }

    #[test]
    fn test_slerp_handles_opposite_quaternions() {
        // q and -q represent same rotation, slerp should handle gracefully
        let q = Quat::from_rotation_y(1.0);
        let neg_q = Quat::from_xyzw(-q.x, -q.y, -q.z, -q.w);
        let result = slerp_simd(q, neg_q, 0.5);
        // Result should be valid (not NaN)
        assert!(
            result.x.is_finite() && result.w.is_finite(),
            "slerp of opposite quaternions should be valid"
        );
    }

    // ========================================================================
    // Batch Function Boolean Path Tests
    // ========================================================================

    #[test]
    fn test_slerp_batch_empty_input() {
        // Empty batch should produce empty output
        let pairs: Vec<(Quat, Quat)> = vec![];
        let result = slerp_batch(&pairs, 0.5);
        assert!(result.is_empty(), "Empty input should produce empty output");
    }

    #[test]
    fn test_slerp_batch_single_element() {
        // Single element batch
        let pairs = vec![(Quat::IDENTITY, Quat::from_rotation_z(1.0))];
        let result = slerp_batch(&pairs, 0.0);
        assert_eq!(result.len(), 1, "Single element batch should produce one result");
        assert!(
            (result[0].w - Quat::IDENTITY.w).abs() < 1e-6,
            "slerp(a, b, 0.0) should equal a"
        );
    }

    #[test]
    fn test_normalize_quat_batch_all_valid() {
        // All elements should be normalized
        let input = vec![
            Quat::from_xyzw(2.0, 0.0, 0.0, 2.0),
            Quat::from_xyzw(0.0, 3.0, 0.0, 3.0),
            Quat::IDENTITY,
        ];
        let result = normalize_quat_batch(&input);

        for (i, q) in result.iter().enumerate() {
            let len =
                (q.x * q.x + q.y * q.y + q.z * q.z + q.w * q.w).sqrt();
            assert!(
                (len - 1.0).abs() < 1e-5,
                "Batch element {} should have unit length, got {}",
                i,
                len
            );
        }
    }

    // ========================================================================
    // Matrix Boolean Path Tests
    // ========================================================================

    #[test]
    fn test_inverse_of_inverse_equals_original() {
        // (M^-1)^-1 = M
        let m = Mat4::from_scale(Vec3::new(2.0, 3.0, 4.0));
        let inv = inverse_simd(m);
        let inv_inv = inverse_simd(inv);

        assert!(
            (inv_inv.col(0).x - m.col(0).x).abs() < 1e-5,
            "Double inverse should equal original"
        );
    }

    #[test]
    fn test_mul_inverse_equals_identity() {
        // M * M^-1 = I
        let m = Mat4::from_scale(Vec3::new(2.0, 3.0, 4.0));
        let inv = inverse_simd(m);
        let result = mul_simd(m, inv);

        // Check diagonal is 1.0
        assert!(
            (result.col(0).x - 1.0).abs() < 1e-5,
            "M * M^-1 diagonal [0][0] should be 1.0"
        );
        assert!(
            (result.col(1).y - 1.0).abs() < 1e-5,
            "M * M^-1 diagonal [1][1] should be 1.0"
        );
    }

    #[test]
    fn test_transform_points_batch_empty() {
        // Empty batch should produce empty output
        let transform = Mat4::IDENTITY;
        let points: Vec<Vec3> = vec![];
        let result = transform_points_batch(transform, &points);
        assert!(result.is_empty(), "Empty batch should produce empty output");
    }

    #[test]
    fn test_transform_points_batch_preserves_count() {
        // Output count should equal input count
        let transform = Mat4::from_translation(Vec3::ONE);
        let points = vec![Vec3::ZERO, Vec3::ONE, Vec3::new(2.0, 3.0, 4.0)];
        let result = transform_points_batch(transform, &points);
        assert_eq!(
            result.len(),
            points.len(),
            "Output count should equal input count"
        );
    }

    // ========================================================================
    // Movement Boolean Path Tests
    // ========================================================================

    #[test]
    fn test_update_positions_empty_slices() {
        // Empty slices should not panic
        let mut positions: Vec<Vec3> = vec![];
        let velocities: Vec<Vec3> = vec![];
        update_positions_simd(&mut positions, &velocities, 1.0);
        assert!(positions.is_empty(), "Empty should remain empty");
    }

    #[test]
    fn test_update_positions_preserves_count() {
        // Count should be preserved
        let mut positions = vec![Vec3::ZERO; 17]; // Odd number to test remainder
        let velocities = vec![Vec3::ONE; 17];
        update_positions_simd(&mut positions, &velocities, 0.1);
        assert_eq!(positions.len(), 17, "Count should be preserved");
    }

    #[test]
    fn test_transform_point_identity_preserves() {
        // Identity transform should preserve point
        let point = Vec3::new(1.0, 2.0, 3.0);
        let result = transform_point_simd(Mat4::IDENTITY, point);
        assert!(
            (result.x - point.x).abs() < 1e-6,
            "Identity should preserve point"
        );
        assert!(
            (result.y - point.y).abs() < 1e-6,
            "Identity should preserve point"
        );
        assert!(
            (result.z - point.z).abs() < 1e-6,
            "Identity should preserve point"
        );
    }
}

// ============================================================================
// Mathematical Identity Tests (Additional Coverage)
// ============================================================================

#[cfg(test)]
mod mathematical_identity_tests {
    //! Tests that verify fundamental mathematical identities that must hold
    //! for correct SIMD implementations.

    use glam::{Mat4, Quat, Vec3};

    use crate::simd_mat::transpose_simd;
    use crate::simd_quat::{mul_quat_simd, normalize_quat_simd, slerp_simd};
    use crate::simd_vec::{cross_simd, dot_simd, length_simd, normalize_simd};

    #[test]
    fn test_pythagorean_identity() {
        // For unit vector: x² + y² + z² = 1
        let v = Vec3::new(3.0, 4.0, 0.0);
        let normalized = normalize_simd(v);
        let sum_of_squares =
            normalized.x * normalized.x + normalized.y * normalized.y + normalized.z * normalized.z;
        assert!(
            (sum_of_squares - 1.0).abs() < 1e-6,
            "Unit vector sum of squares should be 1.0"
        );
    }

    #[test]
    fn test_cross_product_orthogonality() {
        // (A × B) · A = 0 and (A × B) · B = 0
        let a = Vec3::new(1.0, 2.0, 3.0);
        let b = Vec3::new(4.0, 5.0, 6.0);
        let cross = cross_simd(a, b);

        let dot_a = dot_simd(cross, a);
        let dot_b = dot_simd(cross, b);

        assert!(dot_a.abs() < 1e-5, "Cross product should be orthogonal to A");
        assert!(dot_b.abs() < 1e-5, "Cross product should be orthogonal to B");
    }

    #[test]
    fn test_cross_product_anticommutativity() {
        // A × B = -(B × A)
        let a = Vec3::new(1.0, 2.0, 3.0);
        let b = Vec3::new(4.0, 5.0, 6.0);

        let ab = cross_simd(a, b);
        let ba = cross_simd(b, a);

        assert!(
            (ab.x + ba.x).abs() < 1e-6,
            "A × B should equal -(B × A)"
        );
        assert!(
            (ab.y + ba.y).abs() < 1e-6,
            "A × B should equal -(B × A)"
        );
        assert!(
            (ab.z + ba.z).abs() < 1e-6,
            "A × B should equal -(B × A)"
        );
    }

    #[test]
    fn test_quaternion_conjugate_inverse() {
        // For unit quaternion: q^-1 = q*
        let q = normalize_quat_simd(Quat::from_rotation_y(1.5));
        let conj = Quat::from_xyzw(-q.x, -q.y, -q.z, q.w);
        let product = mul_quat_simd(q, conj);

        // Should equal identity
        assert!(
            (product.x).abs() < 1e-5,
            "q * q* should have zero imaginary part"
        );
        assert!(
            (product.w - 1.0).abs() < 1e-5,
            "q * q* should have w = 1"
        );
    }

    #[test]
    fn test_slerp_associativity_approximation() {
        // slerp(slerp(a, b, 0.5), b, 0.5) ≈ slerp(a, b, 0.75)
        let a = Quat::IDENTITY;
        let b = Quat::from_rotation_z(1.0);

        let mid = slerp_simd(a, b, 0.5);
        let three_quarters_via_mid = slerp_simd(mid, b, 0.5);
        let three_quarters_direct = slerp_simd(a, b, 0.75);

        assert!(
            (three_quarters_via_mid.w - three_quarters_direct.w).abs() < 1e-4,
            "Slerp chaining should approximate direct slerp"
        );
    }

    #[test]
    fn test_matrix_transpose_symmetry() {
        // Verify transpose property: (M^T)^T = M
        // This is a fundamental property that must hold
        let m = Mat4::from_cols(
            glam::Vec4::new(1.0, 2.0, 3.0, 4.0),
            glam::Vec4::new(5.0, 6.0, 7.0, 8.0),
            glam::Vec4::new(9.0, 10.0, 11.0, 12.0),
            glam::Vec4::new(13.0, 14.0, 15.0, 16.0),
        );

        let t = transpose_simd(m);
        let tt = transpose_simd(t);

        // Double transpose should equal original
        assert!(
            (tt.col(0).x - m.col(0).x).abs() < 1e-6,
            "(M^T)^T should equal M: got {}, expected {}",
            tt.col(0).x,
            m.col(0).x
        );
        assert!(
            (tt.col(1).y - m.col(1).y).abs() < 1e-6,
            "(M^T)^T should equal M for element [1][1]"
        );
    }

    #[test]
    fn test_normalize_idempotence() {
        // normalize(normalize(v)) = normalize(v)
        let v = Vec3::new(3.0, 4.0, 5.0);
        let once = normalize_simd(v);
        let twice = normalize_simd(once);

        assert!(
            (once.x - twice.x).abs() < 1e-6,
            "Double normalize should equal single"
        );
    }

    #[test]
    fn test_length_of_normalized() {
        // |normalize(v)| = 1 (for non-zero v)
        let v = Vec3::new(7.0, 8.0, 9.0);
        let normalized = normalize_simd(v);
        let len = length_simd(normalized);

        assert!((len - 1.0).abs() < 1e-6, "Normalized vector length should be 1.0");
    }
}

// ============================================================================
// SIMD vs Scalar Consistency Tests
// ============================================================================

#[cfg(test)]
mod simd_scalar_consistency_tests {
    //! Tests that verify SIMD implementations match scalar reference implementations

    use glam::{Mat4, Quat, Vec3};

    use crate::simd_mat::{mul_simd, transpose_simd};
    use crate::simd_movement::update_positions_simd;
    use crate::simd_quat::{mul_quat_simd, normalize_quat_simd, slerp_simd};
    use crate::simd_vec::{cross_simd, dot_simd, length_simd, normalize_simd};

    #[test]
    fn test_dot_simd_matches_scalar() {
        let a = Vec3::new(1.5, 2.5, 3.5);
        let b = Vec3::new(4.0, 5.0, 6.0);

        let simd_result = dot_simd(a, b);
        let scalar_result = a.dot(b);

        assert!(
            (simd_result - scalar_result).abs() < 1e-6,
            "SIMD dot should match scalar: {} vs {}",
            simd_result,
            scalar_result
        );
    }

    #[test]
    fn test_cross_simd_matches_scalar() {
        let a = Vec3::new(1.0, 2.0, 3.0);
        let b = Vec3::new(4.0, 5.0, 6.0);

        let simd_result = cross_simd(a, b);
        let scalar_result = a.cross(b);

        assert!(
            (simd_result.x - scalar_result.x).abs() < 1e-6,
            "SIMD cross X should match scalar"
        );
        assert!(
            (simd_result.y - scalar_result.y).abs() < 1e-6,
            "SIMD cross Y should match scalar"
        );
        assert!(
            (simd_result.z - scalar_result.z).abs() < 1e-6,
            "SIMD cross Z should match scalar"
        );
    }

    #[test]
    fn test_normalize_simd_matches_scalar() {
        let v = Vec3::new(3.0, 4.0, 5.0);

        let simd_result = normalize_simd(v);
        let scalar_result = v.normalize();

        assert!(
            (simd_result.x - scalar_result.x).abs() < 1e-6,
            "SIMD normalize X should match scalar"
        );
    }

    #[test]
    fn test_length_simd_matches_scalar() {
        let v = Vec3::new(3.0, 4.0, 12.0);

        let simd_result = length_simd(v);
        let scalar_result = v.length();

        assert!(
            (simd_result - scalar_result).abs() < 1e-6,
            "SIMD length should match scalar: {} vs {}",
            simd_result,
            scalar_result
        );
    }

    #[test]
    fn test_mul_quat_simd_matches_scalar() {
        let a = Quat::from_rotation_x(0.5);
        let b = Quat::from_rotation_y(0.7);

        let simd_result = mul_quat_simd(a, b);
        let scalar_result = a * b;

        assert!(
            (simd_result.x - scalar_result.x).abs() < 1e-6,
            "SIMD quat mul X should match scalar"
        );
        assert!(
            (simd_result.w - scalar_result.w).abs() < 1e-6,
            "SIMD quat mul W should match scalar"
        );
    }

    #[test]
    fn test_normalize_quat_simd_matches_scalar() {
        let q = Quat::from_xyzw(1.0, 2.0, 3.0, 4.0);

        let simd_result = normalize_quat_simd(q);
        let scalar_result = q.normalize();

        assert!(
            (simd_result.x - scalar_result.x).abs() < 1e-6,
            "SIMD quat normalize X should match scalar"
        );
    }

    #[test]
    fn test_slerp_simd_matches_scalar() {
        let a = Quat::IDENTITY;
        let b = Quat::from_rotation_z(1.0);

        let simd_result = slerp_simd(a, b, 0.3);
        let scalar_result = a.slerp(b, 0.3);

        assert!(
            (simd_result.x - scalar_result.x).abs() < 1e-5,
            "SIMD slerp X should match scalar"
        );
        assert!(
            (simd_result.w - scalar_result.w).abs() < 1e-5,
            "SIMD slerp W should match scalar"
        );
    }

    #[test]
    fn test_mul_mat4_simd_matches_scalar() {
        let a = Mat4::from_rotation_z(0.5);
        let b = Mat4::from_scale(Vec3::new(2.0, 3.0, 4.0));

        let simd_result = mul_simd(a, b);
        let scalar_result = a * b;

        assert!(
            (simd_result.col(0).x - scalar_result.col(0).x).abs() < 1e-5,
            "SIMD mat4 mul should match scalar"
        );
    }

    #[test]
    fn test_transpose_simd_matches_scalar() {
        let m = Mat4::from_cols(
            glam::Vec4::new(1.0, 2.0, 3.0, 4.0),
            glam::Vec4::new(5.0, 6.0, 7.0, 8.0),
            glam::Vec4::new(9.0, 10.0, 11.0, 12.0),
            glam::Vec4::new(13.0, 14.0, 15.0, 16.0),
        );

        let simd_result = transpose_simd(m);
        let scalar_result = m.transpose();

        assert!(
            (simd_result.col(0).x - scalar_result.col(0).x).abs() < 1e-6,
            "SIMD transpose should match scalar"
        );
        assert!(
            (simd_result.col(1).x - scalar_result.col(1).x).abs() < 1e-6,
            "SIMD transpose should match scalar"
        );
    }

    // ========================================================================
    // MUTATION-KILLING TESTS: Catch * -> + and * -> / mutations
    // ========================================================================
    // These tests use specific values where replacing * with + or / would
    // produce detectably different results.

    #[test]
    fn test_mat4_mul_catches_mul_to_add_mutation() {
        // Use 2x2 scale matrices where * vs + gives different results
        // 2 * 3 = 6, but 2 + 3 = 5
        let a = Mat4::from_scale(Vec3::new(2.0, 2.0, 2.0));
        let b = Mat4::from_scale(Vec3::new(3.0, 3.0, 3.0));
        let result = mul_simd(a, b);
        
        // Expected: scale of 6, NOT 5
        let expected_scale = 6.0;
        assert!(
            (result.col(0).x - expected_scale).abs() < 1e-5,
            "Matrix multiply 2*3 should be 6, not 5 (catches * -> + mutation)"
        );
        assert!(
            (result.col(1).y - expected_scale).abs() < 1e-5,
            "Matrix multiply 2*3 should be 6 for Y scale"
        );
        assert!(
            (result.col(2).z - expected_scale).abs() < 1e-5,
            "Matrix multiply 2*3 should be 6 for Z scale"
        );
    }

    #[test]
    fn test_mat4_mul_catches_mul_to_div_mutation() {
        // Use 6 * 2 = 12, but 6 / 2 = 3
        let a = Mat4::from_scale(Vec3::new(6.0, 6.0, 6.0));
        let b = Mat4::from_scale(Vec3::new(2.0, 2.0, 2.0));
        let result = mul_simd(a, b);
        
        // Expected: scale of 12, NOT 3
        let expected_scale = 12.0;
        assert!(
            (result.col(0).x - expected_scale).abs() < 1e-5,
            "Matrix multiply 6*2 should be 12, not 3 (catches * -> / mutation)"
        );
    }

    #[test]
    fn test_quat_mul_catches_mul_to_add_mutation() {
        // Quaternion multiplication has specific algebra
        // Identity * q = q, but identity + q would be different
        let q = Quat::from_rotation_y(1.0);
        let result = mul_quat_simd(Quat::IDENTITY, q);
        
        assert!(
            (result.x - q.x).abs() < 1e-5,
            "Quat identity * q should equal q.x"
        );
        assert!(
            (result.y - q.y).abs() < 1e-5,
            "Quat identity * q should equal q.y"
        );
        assert!(
            (result.w - q.w).abs() < 1e-5,
            "Quat identity * q should equal q.w"
        );
    }

    #[test]
    fn test_quat_mul_two_rotations() {
        // 90 degree rotation around Y twice = 180 degrees
        let q = Quat::from_rotation_y(std::f32::consts::FRAC_PI_2);
        let result = mul_quat_simd(q, q);
        let expected = Quat::from_rotation_y(std::f32::consts::PI);
        
        // Check that double rotation is correct (catches * -> + or / mutations)
        assert!(
            (result.w - expected.w).abs() < 1e-4 || (result.w + expected.w).abs() < 1e-4,
            "Quat 90+90 deg should equal 180 deg: got w={}, expected w={}",
            result.w, expected.w
        );
    }

    #[test]
    fn test_position_update_catches_add_to_mul_mutation() {
        // pos = pos + vel * dt
        // If mutated to pos + vel + dt, result would be wrong
        let mut positions = vec![Vec3::new(10.0, 0.0, 0.0)];
        let velocities = vec![Vec3::new(5.0, 0.0, 0.0)];
        update_positions_simd(&mut positions, &velocities, 2.0);
        
        // Expected: 10 + 5*2 = 20, NOT 10 + 5 + 2 = 17
        assert!(
            (positions[0].x - 20.0).abs() < 1e-5,
            "Position should be 10 + 5*2 = 20, not 17 (catches + vs * mutation)"
        );
    }

    #[test]
    fn test_position_update_batch_correctness() {
        // Test batch path (4+ entities) with values that expose mutations
        let mut positions = vec![
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(2.0, 0.0, 0.0),
            Vec3::new(3.0, 0.0, 0.0),
            Vec3::new(4.0, 0.0, 0.0),
        ];
        let velocities = vec![
            Vec3::new(10.0, 0.0, 0.0),
            Vec3::new(20.0, 0.0, 0.0),
            Vec3::new(30.0, 0.0, 0.0),
            Vec3::new(40.0, 0.0, 0.0),
        ];
        update_positions_simd(&mut positions, &velocities, 0.5);
        
        // Expected: pos + vel * 0.5
        assert!((positions[0].x - 6.0).abs() < 1e-5, "1 + 10*0.5 = 6");
        assert!((positions[1].x - 12.0).abs() < 1e-5, "2 + 20*0.5 = 12");
        assert!((positions[2].x - 18.0).abs() < 1e-5, "3 + 30*0.5 = 18");
        assert!((positions[3].x - 24.0).abs() < 1e-5, "4 + 40*0.5 = 24");
    }
}

