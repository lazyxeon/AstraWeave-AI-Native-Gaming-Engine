use astraweave_nav::Triangle;
use glam::{vec3, Vec3};

/// Very simple “island room” generator: a floor + a ramp plateau.
pub fn generate_island_room() -> Vec<Triangle> {
    vec![
        tri(
            vec3(-4.0, 0.0, -4.0),
            vec3(4.0, 0.0, -4.0),
            vec3(4.0, 0.0, 4.0),
        ),
        tri(
            vec3(-4.0, 0.0, -4.0),
            vec3(4.0, 0.0, 4.0),
            vec3(-4.0, 0.0, 4.0),
        ),
        // ramp
        tri(
            vec3(1.5, 0.0, -1.0),
            vec3(4.0, 0.6, -1.0),
            vec3(4.0, 0.6, 1.0),
        ),
        tri(
            vec3(1.5, 0.0, -1.0),
            vec3(4.0, 0.6, 1.0),
            vec3(1.5, 0.0, 1.0),
        ),
        // plateau
        tri(
            vec3(4.0, 0.6, -1.0),
            vec3(6.5, 0.6, -1.0),
            vec3(6.5, 0.6, 1.0),
        ),
        tri(
            vec3(4.0, 0.6, -1.0),
            vec3(6.5, 0.6, 1.0),
            vec3(4.0, 0.6, 1.0),
        ),
    ]
}

#[inline]
fn tri(a: Vec3, b: Vec3, c: Vec3) -> Triangle {
    Triangle { a, b, c }
}
