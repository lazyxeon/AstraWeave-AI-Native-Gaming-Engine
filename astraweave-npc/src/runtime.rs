use std::collections::HashMap;
use anyhow::Result;
use glam::{Vec3, vec3};

use astraweave_physics::{PhysicsWorld, BodyId};
use astraweave_audio::AudioEngine;

use crate::{NpcMode, NpcAction, NpcWorldView, profile::NpcProfile, llm::LlmAdapter};

pub type NpcId = u64;

pub trait CommandSink {
    fn move_character(&mut self, body: BodyId, dir: Vec3, speed: f32);
    fn say(&mut self, speaker: &str, text: &str);
    fn open_shop(&mut self, npc_id: NpcId);
    fn call_guards(&mut self, pos: Vec3, reason: &str);
    fn give_quest(&mut self, npc_id: NpcId, quest_id: &str);
}

pub struct EngineCommandSink<'a> {
    pub phys: &'a mut PhysicsWorld,
    pub audio: &'a mut AudioEngine,
}

impl<'a> CommandSink for EngineCommandSink<'a> {
    fn move_character(&mut self, body: BodyId, dir: Vec3, speed: f32) {
        let v = if dir.length_squared() > 1e-4 { dir.normalize() * speed } else { Vec3::ZERO };
        // character vertical handled 0 here
        self.phys.control_character(body, vec3(v.x, 0.0, v.z), 1.0/60.0, false);
    }

    fn say(&mut self, _speaker: &str, text: &str) {
        // Play a voice beep scaled to text length (your audio crate can do VO/TTS later)
        self.audio.play_voice_beep(text.len());
        println!("{}: {}", _speaker, text);
    }

    fn open_shop(&mut self, _npc_id: NpcId) {
        println!("[Shop] Opened shop UI (placeholder)");
    }

    fn call_guards(&mut self, pos: Vec3, reason: &str) {
        println!("[Guards] Called to {:?} because {}", pos, reason);
        let _ = self.audio.play_sfx_3d_beep(10, pos, 600.0, 0.25, 0.5);
    }

    fn give_quest(&mut self, _npc_id: NpcId, quest_id: &str) {
        println!("[Quest] Offered quest {}", quest_id);
    }
}

pub struct Npc {
    pub id: NpcId,
    pub profile: NpcProfile,
    pub body: BodyId,
    pub mode: NpcMode,
    pub pending: Vec<NpcAction>,
    pub cooldown_talk: f32,
}

pub struct NpcManager {
    next_id: NpcId,
    npcs: HashMap<NpcId, Npc>,
    planner: Box<dyn LlmAdapter>,
}

impl NpcManager {
    pub fn new(planner: Box<dyn LlmAdapter>) -> Self {
        Self { next_id: 1, npcs: HashMap::new(), planner }
    }

    pub fn spawn_from_profile(&mut self, phys: &mut PhysicsWorld, prof: NpcProfile) -> NpcId {
        // convert home to spawn pos; capsule half extents similar to character
        let pos = prof.home_vec3();
        let body = phys.add_character(pos, vec3(0.4, 0.9, 0.4));
        let id = self.alloc_id();
        self.npcs.insert(id, Npc {
            id, profile: prof, body, mode: NpcMode::Idle, pending: vec![], cooldown_talk: 0.0
        });
        id
    }

    pub fn update(&mut self, dt: f32, glue: &mut dyn CommandSink, views: &HashMap<NpcId, NpcWorldView>) {
        // Collect actions to execute to avoid borrowing issues
        let mut actions_to_execute = Vec::new();
        
        for (_id, npc) in self.npcs.iter_mut() {
            // cooldowns
            npc.cooldown_talk = (npc.cooldown_talk - dt).max(0.0);

            // execute one pending action per tick to keep things readable
            if let Some(act) = npc.pending.first().cloned() {
                actions_to_execute.push((npc.id, npc.body, npc.profile.persona.display_name.clone(), act));
                npc.pending.remove(0);
            } else {
                // idle micro-behavior: guards patrol slowly; merchants idle
                match npc.profile.role {
                    crate::profile::Role::Guard => {
                        if let Some(view) = views.get(&npc.id) {
                            if let Some(pd) = view.player_dist {
                                if pd < 2.0 {
                                    // step aside a bit
                                    let dir = (view.self_pos - view.player_pos.unwrap()).normalize_or_zero();
                                    glue.move_character(npc.body, dir, 0.6);
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        
        // Execute collected actions
        for (npc_id, body, display_name, act) in actions_to_execute {
            Self::execute_action(glue, npc_id, body, &display_name, &act);
        }
    }

    pub fn handle_player_utterance(&mut self, npc_id: NpcId, view: &NpcWorldView, utter: &str) -> Result<()> {
        if let Some(npc) = self.npcs.get_mut(&npc_id) {
            if npc.cooldown_talk > 0.0 { return Ok(()); }
            let plan = self.planner.plan_dialogue_and_behaviour(&npc.profile, view, Some(utter))?;
            npc.pending.extend(plan.actions);
            npc.mode = NpcMode::Conversing;
            npc.cooldown_talk = 0.5;
        }
        Ok(())
    }

    fn execute_action(glue: &mut dyn CommandSink, npc_id: NpcId, body: BodyId, display_name: &str, act: &NpcAction) {
        match act {
            NpcAction::Say { text } => glue.say(display_name, text),
            NpcAction::MoveTo { pos, speed } => {
                // move in direct line toward pos (simple demo; pathfinding can be added)
                // direction = (pos - current). normalized
                // For now, we cannot query position from CommandSink, so move toward target directly
                let dir = Vec3::new(pos.x - 0.0, 0.0, pos.z - 0.0); // placeholder calculation
                glue.move_character(body, dir, *speed);
            }
            NpcAction::Emote { kind } => {
                println!("{} emotes {:?}", display_name, kind);
            }
            NpcAction::OpenShop => glue.open_shop(npc_id),
            NpcAction::GiveQuest { id } => glue.give_quest(npc_id, id),
            NpcAction::CallGuards { reason } => {
                // For now, use a placeholder position since we can't query body position via CommandSink
                let placeholder_pos = Vec3::new(0.0, 0.0, 0.0);
                glue.call_guards(placeholder_pos, reason);
            }
        }
    }


    fn body_pos(&self, _glue: &dyn CommandSink, _body: BodyId) -> Option<Vec3> {
        // For now, we cannot query position via CommandSink.
        // In your integration, pull from PhysicsWorld or World. For demo, return None to skip direction calc (handled by move_character).
        None
    }

    fn alloc_id(&mut self) -> NpcId { let id = self.next_id; self.next_id += 1; id }
}
