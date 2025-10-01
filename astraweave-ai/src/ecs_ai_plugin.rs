//! ECS AI planning plugin: registers a minimal planning system into the ai_planning stage.
use astraweave_core::{ActionStep, CAmmo, CCooldowns, CDesiredPos, CPos, CTeam, CompanionState, EnemyState, IVec2, PlayerState, World, WorldSnapshot, PerceptionConfig, build_snapshot};
use astraweave_core::ecs_events::{Events, AiPlannedEvent, AiPlanningFailedEvent};
use astraweave_core::ecs_bridge::EntityBridge;
use astraweave_ecs as ecs;

use crate::orchestrator::{Orchestrator, RuleOrchestrator};

pub struct AiPlanningPlugin;

// Helper: map a legacy companion id to an ECS entity.
// 1) find the closest ECS entity on team 1 by position relative to snap.me
// 2) if an EntityBridge exists prefer the bridge mapping for the legacy id
// 3) return None if no candidate ECS companion was found
fn map_legacy_companion_to_ecs(
    positions: &std::collections::BTreeMap<ecs::Entity, IVec2>,
    teams: &std::collections::BTreeMap<ecs::Entity, u8>,
    snap: &WorldSnapshot,
    comp: astraweave_core::Entity,
    world: &ecs::World,
) -> Option<ecs::Entity> {
    // find closest companion by Manhattan distance to the legacy companion position
    let closest = positions
        .iter()
        .filter(|(e, _)| teams.get(e).copied() == Some(1))
        .map(|(e, p)| (*e, p))
        .min_by_key(|(_, p)| (p.x - snap.me.pos.x).abs() + (p.y - snap.me.pos.y).abs())
        .map(|(e, _)| e)?;

    // prefer bridge mapping from legacy id -> ecs entity when present
    let mapped = world
        .resource::<EntityBridge>()
        .and_then(|b| b.get(&comp))
        .unwrap_or(closest);
    Some(mapped)
}

fn sys_ai_planning(world: &mut ecs::World) {
    // Build snapshots and set desired positions per companion entity
    // Prefer legacy World + perception builder when available; fallback to ECS-only snapshot
    // Ensure AiPlannedEvent resource exists
    if world.resource::<Events<AiPlannedEvent>>().is_none() {
        world.insert_resource(Events::<AiPlannedEvent>::default());
    }

    // Cache ECS component views
    let mut positions: std::collections::BTreeMap<ecs::Entity, IVec2> = std::collections::BTreeMap::new();
    let mut teams: std::collections::BTreeMap<ecs::Entity, u8> = std::collections::BTreeMap::new();
    {
        let q = ecs::Query::<CPos>::new(world);
        for (e, p) in q { positions.insert(e, IVec2 { x: p.pos.x, y: p.pos.y }); }
    }
    {
        let q = ecs::Query::<CTeam>::new(world);
        for (e, t) in q { teams.insert(e, t.id); }
    }

    let orch = RuleOrchestrator;
    let mut updates: Vec<(ecs::Entity, CDesiredPos)> = vec![];
    let mut planned_events: Vec<AiPlannedEvent> = vec![];
    let mut failed_events: Vec<AiPlanningFailedEvent> = vec![];

    // Try legacy world path
    if let Some(w) = world.resource::<World>() {
        // pick first player and companion if present
        let player_opt = w.all_of_team(0).first().copied();
        let comp_opt = w.all_of_team(1).first().copied();
        if let (Some(player), Some(comp)) = (player_opt, comp_opt) {
            let enemies = w.enemies_of(1);
            let snap = build_snapshot(
                w,
                player,
                comp,
                &enemies,
                None,
                &PerceptionConfig { los_max: 10 },
            );
            let plan = orch.propose_plan(&snap);
            if let Some(ActionStep::MoveTo { x, y }) = plan.steps.iter().find_map(|s| {
                if let ActionStep::MoveTo { x, y } = s { Some(ActionStep::MoveTo { x: *x, y: *y }) } else { None }
            }) {
                if let Some(mapped) = map_legacy_companion_to_ecs(&positions, &teams, &snap, comp, world) {
                        updates.push((mapped, CDesiredPos { pos: IVec2 { x, y } }));
                    planned_events.push(AiPlannedEvent { entity: mapped, target: IVec2 { x, y } });
                }
            } else {
                // No valid move found
                if let Some(mapped) = map_legacy_companion_to_ecs(&positions, &teams, &snap, comp, world) {
                    failed_events.push(AiPlanningFailedEvent {
                        entity: mapped,
                        reason: "No valid actions in plan".to_string(),
                    });
                }
            }
            // Early return after legacy-based planning for now (single companion minimal)
            if !updates.is_empty() {
                for (e, d) in &updates { world.insert(*e, *d); }
                if let Some(ev) = world.resource_mut::<Events<AiPlannedEvent>>() {
                    let mut w = ev.writer();
                    for pe in planned_events { w.send(pe); }
                }
                if let Some(ev) = world.resource_mut::<Events<AiPlanningFailedEvent>>() {
                    let mut w = ev.writer();
                    for fe in failed_events { w.send(fe); }
                }
                return;
            }
        }
    }

    // Fallback: ECS-only snapshot composition
    let player = PlayerState { hp: 100, pos: IVec2 { x: 0, y: 0 }, stance: "stand".into(), orders: vec![] };
    let enemies: Vec<EnemyState> = positions
        .iter()
        .filter_map(|(e, pos)| {
            let team_id = teams.get(e).copied().unwrap_or(0);
            if team_id == 2 {
                Some(EnemyState { id: 0, pos: *pos, hp: 50, cover: "low".into(), last_seen: 0.0 })
            } else { None }
        })
        .collect();
    for (e, pos) in &positions {
        if teams.get(e).copied() != Some(1) { continue; }
        let ammo = world.get::<CAmmo>(*e).map(|a| a.rounds).unwrap_or(0);
        let cds_map = world.get::<CCooldowns>(*e).map(|c| c.map.clone()).unwrap_or_default();
        let cooldowns: std::collections::BTreeMap<String, f32> = cds_map.into_iter().map(|(k, v)| (k.to_string(), v)).collect();
        let me = CompanionState { ammo, cooldowns, morale: 1.0, pos: *pos };
        let snap = WorldSnapshot { t: 0.0, player: player.clone(), me, enemies: enemies.clone(), pois: vec![], obstacles: vec![], objective: None };
        let plan = orch.propose_plan(&snap);
        if let Some(ActionStep::MoveTo { x, y }) = plan.steps.iter().find_map(|s| {
            if let ActionStep::MoveTo { x, y } = s { Some(ActionStep::MoveTo { x: *x, y: *y }) } else { None }
        }) {
            updates.push((*e, CDesiredPos { pos: IVec2 { x, y } }));
            planned_events.push(AiPlannedEvent { entity: *e, target: IVec2 { x, y } });
        } else {
            failed_events.push(AiPlanningFailedEvent {
                entity: *e,
                reason: "No valid move action found".to_string(),
            });
        }
    }
    for (e, d) in updates { world.insert(e, d); }
    if let Some(ev) = world.resource_mut::<Events<AiPlannedEvent>>() {
        let mut w = ev.writer();
        for pe in planned_events { w.send(pe); }
    }
    if let Some(ev) = world.resource_mut::<Events<AiPlanningFailedEvent>>() {
        let mut w = ev.writer();
        for fe in failed_events { w.send(fe); }
    }
}

impl ecs::Plugin for AiPlanningPlugin {
    fn build(&self, app: &mut ecs::App) {
        app.world.insert_resource(Events::<AiPlanningFailedEvent>::default());
        app.schedule.add_system("ai_planning", sys_ai_planning as ecs::SystemFn);
    }
}

/// Convenience: build an ECS app with core systems and the AI planning plugin installed.
pub fn build_app_with_ai(legacy_world: astraweave_core::World, dt: f32) -> ecs::App {
    let mut app = astraweave_core::ecs_adapter::build_app(legacy_world, dt);
    app = app.add_plugin(AiPlanningPlugin);
    app
}

#[cfg(test)]
mod tests {
    use super::*;
    use astraweave_core::{World, IVec2, Team};
    use anyhow::anyhow;
    use anyhow::Result;

    #[test]
    fn ai_plugin_sets_desired_position_for_companion() -> Result<()> {
        // Legacy world can be empty for this test
        let w = World::new();
        let mut app = build_app_with_ai(w, 0.016);
        // Spawn two ECS entities: a companion and an enemy
        let ally = app.world.spawn();
    app.world.insert(ally, CPos { pos: IVec2 { x: 0, y: 0 } });
        app.world.insert(ally, CTeam { id: 1 });
        app.world.insert(ally, CAmmo { rounds: 10 });
        app.world.insert(ally, CCooldowns { map: std::collections::BTreeMap::new() });

        let enemy = app.world.spawn();
    app.world.insert(enemy, CPos { pos: IVec2 { x: 3, y: 0 } });
        app.world.insert(enemy, CTeam { id: 2 });

        app = app.run_fixed(1);
    let d = app.world.get::<CDesiredPos>(ally).ok_or_else(|| anyhow!("desired pos set"))?;
        // Expect to move towards enemy along +x axis
        assert!(d.pos.x >= 1 && d.pos.y == 0);

        // Event should be published
    let evs = app.world.resource_mut::<Events<AiPlannedEvent>>().ok_or_else(|| anyhow!("Events<AiPlannedEvent> resource missing"))?;
        let mut rdr = evs.reader();
        let v: Vec<_> = rdr.drain().collect();
        assert_eq!(v.len(), 1);
        assert_eq!(v[0].entity, ally);
        Ok(())
    }
}
