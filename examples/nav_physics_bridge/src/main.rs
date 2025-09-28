use anyhow::Result;
use astraweave_nav::{NavMesh, Triangle};
use astraweave_physics::PhysicsWorld;
use glam::Vec3;

fn main() -> Result<()> {
    // Bake a simple flat navmesh (two triangles making a square)
    let tris = vec![
        Triangle {
            a: Vec3::new(0.0, 0.0, 0.0),
            b: Vec3::new(0.0, 0.0, 5.0),
            c: Vec3::new(5.0, 0.0, 0.0),
        },
        Triangle {
            a: Vec3::new(5.0, 0.0, 0.0),
            b: Vec3::new(0.0, 0.0, 5.0),
            c: Vec3::new(5.0, 0.0, 5.0),
        },
    ];
    let nav = NavMesh::bake(&tris, 0.5, 80.0);
    let start = Vec3::new(0.5, 0.0, 0.5);
    let goal = Vec3::new(4.5, 0.0, 4.5);
    let path = nav.find_path(start, goal);
    assert!(path.len() >= 2, "path should have at least start and goal");
    println!("Path waypoints: {:?}", path);

    // Physics world with ground
    let mut pw = PhysicsWorld::new(Vec3::new(0.0, -9.8, 0.0));
    let _ground = pw.create_ground_plane(Vec3::new(20.0, 0.5, 20.0), 0.9);
    let char_id = pw.add_character(Vec3::new(start.x, 1.0, start.z), Vec3::new(0.4, 0.9, 0.4));

    // Move along path at constant speed
    let mut cur_wp = 1usize;
    let speed = 2.0; // m/s
    for _ in 0..(10 * 60) {
        // 10 seconds @60Hz
        let pos = pw.body_transform(char_id).unwrap();
        let p = Vec3::new(pos.w_axis.x, 0.0, pos.w_axis.z);
        let target = if cur_wp < path.len() {
            path[cur_wp]
        } else {
            goal
        };
        let to = (target - p).clamp_length_max(1.0);
        let dir = if to.length() > 1e-3 {
            to.normalize()
        } else {
            Vec3::ZERO
        };
        pw.control_character(
            char_id,
            Vec3::new(dir.x * speed, 0.0, dir.z * speed),
            1.0 / 60.0,
            false,
        );
        pw.step();
        if p.distance(target) < 0.15 && cur_wp < path.len() {
            cur_wp += 1;
        }
    }
    let final_pos = pw.body_transform(char_id).unwrap().w_axis;
    let final2d = Vec3::new(final_pos.x, 0.0, final_pos.z);
    println!("Final pos: {:?}", final2d);
    assert!(
        final2d.distance(goal) < 0.5,
        "character should reach near goal"
    );
    Ok(())
}
