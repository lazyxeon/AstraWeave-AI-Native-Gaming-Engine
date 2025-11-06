use std::collections::{HashMap, HashSet};

use astraweave_ecs::World;
use tracing::info;

use crate::veilweaver_slice::{
    DecisionPromptSpec, EncounterCompleteSpec, EncounterTriggerSpec, TriggerZoneSpec,
    VeilweaverSliceMetadata, WeaveAnchorSpec,
};

use astraweave_core::ecs_events::Events;
use astraweave_ecs::Entity;

pub struct AnchorStabilizedEvent {
    pub anchor_id: String,
}

pub struct TriggerVolumeEvent {
    pub trigger_id: String,
    pub entering: bool,
    pub entity: Option<Entity>,
}

#[derive(Default)]
pub struct WeaveTutorialState {
    pub anchors: HashMap<String, AnchorStatus>,
    pub trigger_zones: HashMap<String, TriggerZoneSpec>,
    pub decision_prompts: HashMap<String, DecisionPromptSpec>,
    pub encounter_triggers: HashMap<String, EncounterTriggerSpec>,
    pub encounter_completes: HashMap<String, EncounterCompleteSpec>,
    pub active_triggers: HashSet<String>,
    pub anchor_sequence: Vec<String>,
    initialized: bool,
}

#[derive(Clone)]
pub struct AnchorStatus {
    pub spec: WeaveAnchorSpec,
    pub stabilized: bool,
    pub activation_order: Option<usize>,
}

impl WeaveTutorialState {
    pub fn from_metadata(meta: VeilweaverSliceMetadata) -> Self {
        let anchors = meta
            .anchors
            .into_iter()
            .map(|anchor| {
                (
                    anchor.anchor_id.clone(),
                    AnchorStatus {
                        spec: anchor,
                        stabilized: false,
                        activation_order: None,
                    },
                )
            })
            .collect();
        let trigger_zones = meta
            .trigger_zones
            .into_iter()
            .map(|trigger| (trigger.trigger_id.clone(), trigger))
            .collect();
        let decision_prompts = meta
            .decision_prompts
            .into_iter()
            .map(|prompt| (prompt.trigger_id.clone(), prompt))
            .collect();
        let encounter_triggers = meta
            .encounter_triggers
            .into_iter()
            .map(|trigger| (trigger.trigger_id.clone(), trigger))
            .collect();
        let encounter_completes = meta
            .encounter_completes
            .into_iter()
            .map(|complete| (complete.trigger_id.clone(), complete))
            .collect();

        Self {
            anchors,
            trigger_zones,
            decision_prompts,
            encounter_triggers,
            encounter_completes,
            active_triggers: HashSet::new(),
            anchor_sequence: Vec::new(),
            initialized: false,
        }
    }

    pub fn mark_anchor_stabilized(&mut self, anchor_id: &str) {
        if let Some(anchor) = self.anchors.get_mut(anchor_id) {
            if !anchor.stabilized {
                anchor.stabilized = true;
                anchor.activation_order = Some(self.anchor_sequence.len());
                self.anchor_sequence.push(anchor_id.to_string());
                info!("Anchor stabilized", id = %anchor_id, order = ?anchor.activation_order);
            }
        }
    }

    pub fn register_trigger_activation(&mut self, trigger_id: &str) {
        self.active_triggers.insert(trigger_id.to_string());
        info!("Trigger activated", id = %trigger_id);
    }

    pub fn register_trigger_release(&mut self, trigger_id: &str) {
        if self.active_triggers.remove(trigger_id) {
            info!("Trigger cleared", id = %trigger_id);
        }
    }

    pub fn anchors_remaining(&self) -> usize {
        self.anchors.values().filter(|a| !a.stabilized).count()
    }
}

pub fn tutorial_anchor_sync(world: &mut World) {
    if let Some(mut state) = world.get_resource_mut::<WeaveTutorialState>() {
        if !state.initialized {
            info!(
                anchors = state.anchors.len(),
                triggers = state.trigger_zones.len(),
                prompts = state.decision_prompts.len(),
                "Veilweaver tutorial state initialized"
            );
            state.initialized = true;
        }
    }
}

pub fn tutorial_trigger_system(world: &mut World) {
    if let (Some(mut state), Some(mut reader)) = (
        world.get_resource_mut::<WeaveTutorialState>(),
        world
            .get_resource_mut::<Events<TriggerVolumeEvent>>()
            .map(|events| events.reader()),
    ) {
        for ev in reader.drain() {
            if ev.entering {
                state.register_trigger_activation(&ev.trigger_id);
            } else {
                state.register_trigger_release(&ev.trigger_id);
            }
        }
    }
}

pub fn tutorial_anchor_events(world: &mut World) {
    if let (Some(mut state), Some(mut reader)) = (
        world.get_resource_mut::<WeaveTutorialState>(),
        world
            .get_resource_mut::<Events<AnchorStabilizedEvent>>()
            .map(|events| events.reader()),
    ) {
        for ev in reader.drain() {
            state.mark_anchor_stabilized(&ev.anchor_id);
        }
    }
}
