use astraweave_gameplay::*;
use astraweave_physics::PhysicsWorld;
use glam::vec3;

fn main() -> anyhow::Result<()> {
    let mut phys = PhysicsWorld::new(vec3(0.0, -9.81, 0.0));
    let ground = phys.create_ground_plane(vec3(50.0, 0.0, 50.0), 1.0);
    let player_id = phys.add_dynamic_box(
        vec3(0.0, 1.0, 0.0),
        vec3(0.3, 0.5, 0.3),
        2.0,
        astraweave_physics::Layers::DEFAULT,
    );
    let enemy_id = phys.add_dynamic_box(
        vec3(1.2, 1.0, 0.0),
        vec3(0.3, 0.5, 0.3),
        2.0,
        astraweave_physics::Layers::DEFAULT,
    );

    let mut targets = vec![Combatant {
        body: enemy_id,
        stats: Stats::new(80),
        iframes: None,
        parry: Some(Parry {
            window: 0.3,
            active: true,
        }),
    }];

    // sweep from player forward
    let hit = perform_attack_sweep(
        &mut phys,
        player_id,
        vec3(0.0, 1.0, 0.0),
        vec3(1.5, 1.0, 0.0),
        0.2,
        20,
        DamageType::Physical,
        &mut targets,
    );
    match hit {
        Some(h) if h.parried => println!("Parried! No damage."),
        Some(h) => println!("Hit target {} for {} dmg", h.target, h.damage),
        None => println!("Miss"),
    }
    Ok(())
}
