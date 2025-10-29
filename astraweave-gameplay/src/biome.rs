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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_island_room_triangle_count() {
        let triangles = generate_island_room();
        assert_eq!(
            triangles.len(),
            6,
            "Island room should have 6 triangles (2 floor + 2 ramp + 2 plateau)"
        );
    }

    #[test]
    fn test_generate_island_room_floor_triangles() {
        let triangles = generate_island_room();
        
        // First floor triangle
        let floor_tri1 = &triangles[0];
        assert_eq!(floor_tri1.a, vec3(-4.0, 0.0, -4.0));
        assert_eq!(floor_tri1.b, vec3(4.0, 0.0, -4.0));
        assert_eq!(floor_tri1.c, vec3(4.0, 0.0, 4.0));
        
        // Second floor triangle
        let floor_tri2 = &triangles[1];
        assert_eq!(floor_tri2.a, vec3(-4.0, 0.0, -4.0));
        assert_eq!(floor_tri2.b, vec3(4.0, 0.0, 4.0));
        assert_eq!(floor_tri2.c, vec3(-4.0, 0.0, 4.0));
    }

    #[test]
    fn test_generate_island_room_ramp_geometry() {
        let triangles = generate_island_room();
        
        // Ramp triangles should be at indices 2 and 3
        let ramp_tri1 = &triangles[2];
        let ramp_tri2 = &triangles[3];
        
        // Ramp starts at y=0.0 and rises to y=0.6
        assert_eq!(ramp_tri1.a.y, 0.0, "Ramp base should be at ground level");
        assert_eq!(ramp_tri1.b.y, 0.6, "Ramp top should be at plateau level");
        assert_eq!(ramp_tri1.c.y, 0.6, "Ramp top should be at plateau level");
        
        assert_eq!(ramp_tri2.a.y, 0.0);
        assert_eq!(ramp_tri2.b.y, 0.6);
        assert_eq!(ramp_tri2.c.y, 0.0);
    }

    #[test]
    fn test_generate_island_room_plateau_geometry() {
        let triangles = generate_island_room();
        
        // Plateau triangles should be at indices 4 and 5
        let plateau_tri1 = &triangles[4];
        let plateau_tri2 = &triangles[5];
        
        // All plateau vertices should be at y=0.6
        assert_eq!(plateau_tri1.a.y, 0.6, "Plateau should be elevated");
        assert_eq!(plateau_tri1.b.y, 0.6);
        assert_eq!(plateau_tri1.c.y, 0.6);
        
        assert_eq!(plateau_tri2.a.y, 0.6);
        assert_eq!(plateau_tri2.b.y, 0.6);
        assert_eq!(plateau_tri2.c.y, 0.6);
    }

    #[test]
    fn test_generate_island_room_floor_coverage() {
        let triangles = generate_island_room();
        let floor_tri1 = &triangles[0];
        let floor_tri2 = &triangles[1];
        
        // Floor should cover an 8x8 area (from -4 to +4 in x and z)
        let min_x = floor_tri1.a.x.min(floor_tri1.b.x).min(floor_tri1.c.x)
            .min(floor_tri2.a.x).min(floor_tri2.b.x).min(floor_tri2.c.x);
        let max_x = floor_tri1.a.x.max(floor_tri1.b.x).max(floor_tri1.c.x)
            .max(floor_tri2.a.x).max(floor_tri2.b.x).max(floor_tri2.c.x);
        
        assert_eq!(min_x, -4.0, "Floor should start at x=-4");
        assert_eq!(max_x, 4.0, "Floor should end at x=4");
    }

    #[test]
    fn test_tri_helper_function() {
        let a = vec3(1.0, 2.0, 3.0);
        let b = vec3(4.0, 5.0, 6.0);
        let c = vec3(7.0, 8.0, 9.0);
        
        let triangle = tri(a, b, c);
        
        assert_eq!(triangle.a, a);
        assert_eq!(triangle.b, b);
        assert_eq!(triangle.c, c);
    }
}
