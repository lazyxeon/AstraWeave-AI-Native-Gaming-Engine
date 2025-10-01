//! ECS integration for behavior trees

use crate::{BehaviorContext, BehaviorGraph, BehaviorStatus};
use astraweave_ecs::Query;

const SIMULATION_STAGE: &str = "simulation";

/// Component for entities with behavior graphs
pub struct CBehaviorGraph {
    pub graph: BehaviorGraph,
    pub context: BehaviorContext,
    pub status: BehaviorStatus,
    pub running_node: Option<String>, // for debugging
}
/// System to tick behavior graphs
pub fn behavior_tick_system(world: &mut astraweave_ecs::World) {
    // Collect entities with behavior graphs
    let mut to_update = vec![];
    {
        let q = Query::<CBehaviorGraph>::new(world);
        for (entity, bg) in q {
            let status = bg.graph.tick(&bg.context);
            to_update.push((entity, status));
        }
    }

    // Update statuses

    for (entity, status) in to_update {
        if let Some(bg) = world.get_mut::<CBehaviorGraph>(entity) {
            bg.status = status;
            bg.running_node = bg.graph.current_node_name();
        }
    }
}
/// Plugin to add behavior systems
pub struct BehaviorPlugin;

impl astraweave_ecs::Plugin for BehaviorPlugin {
    fn build(&self, app: &mut astraweave_ecs::App) {
        app.add_system(
            SIMULATION_STAGE,
            behavior_tick_system as astraweave_ecs::SystemFn,
        );
    }
}
