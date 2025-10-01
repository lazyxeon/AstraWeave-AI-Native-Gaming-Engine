//! ECS adapter: integrate a minimal ECS app/schedule while bridging existing World.
use astraweave_ecs as ecs;

use crate::{World, CPos, CHealth, CTeam, CAmmo, CCooldowns, CDesiredPos, IVec2};
use crate::ecs_events::{Events, MovedEvent};
use crate::ecs_bridge::EntityBridge;

#[derive(Clone, Copy)]
struct Dt(pub f32);

fn sim_cooldowns(world_compat: &mut World, dt: f32) {
    world_compat.tick(dt);
}

fn sys_sim(world: &mut ecs::World) {
    let dt = world.resource::<Dt>().map(|d| d.0).unwrap_or(0.016);
    if let Some(w) = world.resource_mut::<World>() {
        sim_cooldowns(w, dt);
    }
    // Phase 1: mirror basic cooldown decay into ECS components if present
    let dt = dt;
    world.each_mut::<CCooldowns>(|_, cds| {
        for v in cds.map.values_mut() { *v = (*v - dt).max(0.0); }
    });
}

fn sys_move(world: &mut ecs::World) {
    // Move entities one step toward desired pos (cardinal-only 4-neighborhood) per tick
    // Deterministic order by BTreeMap underlying storage
    // Note: no collision here—Phase 1 minimal behavior
    // Read positions and desired goals, mutate positions
    // We purposely run after sim (cooldowns)
    use std::collections::BTreeMap;
    let goals: BTreeMap<ecs::Entity, CDesiredPos> = {
        let mut m = BTreeMap::new();
        let q = ecs::Query::<CDesiredPos>::new(&*world);
        for (e, g) in q { m.insert(e, *g); }
        m
    };
    let mut moved: Vec<(ecs::Entity, IVec2, IVec2)> = vec![];
    world.each_mut::<CPos>(|e, p| {
        if let Some(goal) = goals.get(&e) {
            let mut dx = (goal.pos.x - p.pos.x).signum();
            let mut dy = (goal.pos.y - p.pos.y).signum();
            // Cardinal-only behavior: prefer moving along X this tick; if we move in X,
            // do not also move in Y (prevents diagonal movement).
            if dx != 0 { dy = 0; }
            if dx != 0 || dy != 0 {
                let from = IVec2 { x: p.pos.x, y: p.pos.y };
                if dx != 0 {
                    p.pos.x += dx;
                } else if dy != 0 {
                    p.pos.y += dy;
                }
                moved.push((e, from, IVec2 { x: p.pos.x, y: p.pos.y }));
            }
        }
    });
    if let Some(ev) = world.resource_mut::<Events<MovedEvent>>() {
        let mut w = ev.writer();
    for (e, from, to) in moved { w.send(MovedEvent { entity: e, from, to }); }
    }
}

fn sys_refresh_los(world: &mut ecs::World) {
    // Example LOS cache refresh placeholder: for now, no persistent cache type.
    // In Phase 1 we show how to call helpers; a later step would store a cache component/resource.
    // Using obstacles from legacy world if present
    if let Some(w) = world.resource::<World>() {
        let _ = &w.obstacles; // no-op to show access; real cache omitted for minimal footprint
    }
}

fn sys_bridge_sync(world: &mut ecs::World) {
    // Ensure any mapped ECS entities carry a CLegacyId component and
    // remove CLegacyId from entities not present in the bridge.
    use std::collections::BTreeSet;

    // Collect all ecs entities referenced by the bridge
    let mut referenced = BTreeSet::new();
    if let Some(bridge) = world.resource::<EntityBridge>() {
        for ecs_e in bridge.ecs_entities() { referenced.insert(ecs_e); }
    }

    // Add CLegacyId to referenced entities if missing
    for &e in referenced.iter() {
        if world.get::<crate::CLegacyId>(e).is_none() {
            if let Some(bridge) = world.resource::<EntityBridge>() {
                if let Some(legacy) = bridge.get_by_ecs(&e) {
                    world.insert(e, crate::CLegacyId { id: legacy });
                }
            }
        }
    }

    // Note: Phase 1 `astraweave_ecs::World` does not provide a component removal
    // API. Removing CLegacyId entries would require extending the ECS. For now
    // we only ensure referenced entities have the CLegacyId component. Stale
    // CLegacyId components (if any) will remain until a future ECS API adds
    // removal support.
}

// EntityBridge is defined in `crate::ecs_bridge` for cross-crate access.


/// Build a minimal ECS app with stages and a single simulation system that
/// bridges into the legacy `World` struct for Phase 1.
pub fn build_app(legacy_world: World, dt: f32) -> ecs::App {
    let mut app = ecs::App::new();
    // Insert base resources first
    app.world.insert_resource(Dt(dt));
    app.world.insert_resource(Events::<MovedEvent>::default());
    app.world.insert_resource(EntityBridge::default());

    // Auto-populate ECS entities and the entity bridge from the provided legacy World
    // using the owned `legacy_world` to avoid borrowing app.world while also mutating it.
    for legacy in legacy_world.entities() {
        let e = app.world.spawn();
        // Mirror pose if present
            if let Some(p) = legacy_world.pose(legacy) { app.world.insert(e, CPos { pos: IVec2 { x: p.pos.x, y: p.pos.y } }); }
        if let Some(h) = legacy_world.health(legacy) { app.world.insert(e, CHealth { hp: h.hp }); }
        if let Some(t) = legacy_world.team(legacy) { app.world.insert(e, CTeam { id: t.id }); }
        if let Some(a) = legacy_world.ammo(legacy) { app.world.insert(e, CAmmo { rounds: a.rounds }); }
        if let Some(cds) = legacy_world.cooldowns(legacy) {
            // convert HashMap<String,f32> -> BTreeMap<CooldownKey,f32> for CCooldowns
            let map: crate::cooldowns::Map = cds.map.iter()
                .map(|(k, v)| (crate::cooldowns::CooldownKey::from(k.as_str()), *v))
                .collect();
            app.world.insert(e, CCooldowns { map });
        }
    // populate bridge
    if let Some(bridge) = app.world.resource_mut::<EntityBridge>() { bridge.insert_pair(legacy, e); }
    }

    // Now insert the legacy world as a resource so systems can access it.
    app.world.insert_resource::<World>(legacy_world);
    app.add_system("simulation", sys_sim as ecs::SystemFn);
    app.add_system("simulation", sys_move as ecs::SystemFn);
    // Bridge sync runs after simulation so mappings are reflected into components
    app.add_system("sync", sys_bridge_sync as ecs::SystemFn);
    // AI planning system is registered from astraweave-ai crate to avoid a dependency cycle.
    app.add_system("perception", sys_refresh_los as ecs::SystemFn);
    app
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::IVec2;
    #[test]
    fn ecs_drives_legacy_world_tick() {
        let mut w = World::new();
    let _e = w.spawn("ally", IVec2 { x: 0, y: 0 }, crate::Team { id: 1 }, 100, 5);
        let app = build_app(w, 0.010).run_fixed(5);
        let w2 = app.world.resource::<World>().unwrap();
        assert!((w2.t - 0.050).abs() < 1e-6);
    }

    #[test]
    fn ecs_components_update_cooldowns() {
        let w = World::new();
        let mut app = build_app(w, 0.020);
        // Insert an entity with cooldowns component
        let e = app.world.spawn();
    app.world.insert(e, CCooldowns { map: std::collections::BTreeMap::from([(crate::cooldowns::CooldownKey::from("throw:smoke"), 0.05)]) });
        // Run 2 ticks => cd should reduce to ~0.01
        app = app.run_fixed(2);
        let mut val = 0.0;
    app.world.each_mut::<CCooldowns>(|_, cds| { val = *cds.map.get(&crate::cooldowns::CooldownKey::from("throw:smoke")).unwrap(); });
        assert!(val <= 0.02 && val >= 0.009);
    }

    #[test]
    fn simple_movement_toward_goal() {
        let w = World::new();
        let mut app = build_app(w, 0.016);
        let e = app.world.spawn();
        // For Phase 1 tests we demonstrate populating the entity bridge when
        // creating ECS entities that correspond to legacy world entities.
        if let Some(bridge) = app.world.resource_mut::<EntityBridge>() {
            // Use a synthetic legacy id 1 for test purposes
            bridge.insert_pair(1, e);
        }
        app.world.insert(e, CPos { pos: IVec2 { x: 0, y: 0 } });
    app.world.insert(e, CDesiredPos { pos: IVec2 { x: 2, y: 0 } });
        app = app.run_fixed(3);
    let p = app.world.get::<CPos>(e).unwrap();
    assert_eq!((p.pos.x,p.pos.y), (2,0));
    }

    #[test]
    fn movement_emits_events() {
        let w = World::new();
        let mut app = build_app(w, 0.016);
        let e = app.world.spawn();
        if let Some(bridge) = app.world.resource_mut::<EntityBridge>() {
            bridge.insert_pair(1, e);
        }
        app.world.insert(e, CPos { pos: IVec2 { x: 0, y: 0 } });
    app.world.insert(e, CDesiredPos { pos: IVec2 { x: 1, y: 0 } });
        app = app.run_fixed(1);
        let evs = app.world.resource_mut::<Events<MovedEvent>>().unwrap();
        let mut rdr = evs.reader();
        let collected: Vec<_> = rdr.drain().collect();
        assert_eq!(collected.len(), 1);
        assert_eq!(collected[0].entity, e);
        assert_eq!(collected[0].from, IVec2 { x: 0, y: 0 });
        assert_eq!(collected[0].to, IVec2 { x: 1, y: 0 });
    }
}
