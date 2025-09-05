use anyhow::Result;
use glam::Vec3;
use rand::Rng;

use crate::{NpcWorldView, NpcPlan, NpcAction, EmoteKind, profile::{NpcProfile, Role}};

/// Adapter trait so you can swap in a real LLM / Inworld-like service.
pub trait LlmAdapter: Send + Sync {
    fn plan_dialogue_and_behaviour(
        &self,
        profile: &NpcProfile,
        view: &NpcWorldView,
        player_utterance: Option<&str>,
    ) -> Result<NpcPlan>;
}

/// A simple mock planner with hand-written heuristics.
/// Replace with a real adapter later.
pub struct MockLlm;

impl LlmAdapter for MockLlm {
    fn plan_dialogue_and_behaviour(
        &self,
        profile: &NpcProfile,
        view: &NpcWorldView,
        player_utterance: Option<&str>,
    ) -> Result<NpcPlan> {
        let mut rng = rand::thread_rng();
        let mut actions = vec![];

        match profile.role {
            Role::Merchant => {
                if let Some(u) = player_utterance {
                    if u.to_lowercase().contains("buy") || u.to_lowercase().contains("shop") {
                        actions.push(NpcAction::Say { text: "Take a look—finest wares this side of the veil.".into() });
                        actions.push(NpcAction::OpenShop);
                    } else {
                        actions.push(NpcAction::Say { text: "Greetings, traveler. Looking for supplies?".into() });
                        actions.push(NpcAction::Emote { kind: EmoteKind::Nod });
                    }
                } else {
                    // idle/working
                    if rng.gen::<f32>() < 0.3 {
                        actions.push(NpcAction::Emote { kind: EmoteKind::Wave });
                    }
                }
            }
            Role::Guard => {
                if view.nearby_threat {
                    actions.push(NpcAction::Say { text: "Stay back—this area isn’t safe.".into() });
                    actions.push(NpcAction::CallGuards { reason: "Threat detected".into() });
                } else if let Some(u) = player_utterance {
                    if u.to_lowercase().contains("danger") || u.to_lowercase().contains("help") {
                        actions.push(NpcAction::Say { text: "I’ll alert the watch. Keep your distance.".into() });
                        actions.push(NpcAction::CallGuards { reason: "Player reported danger".into() });
                    } else {
                        actions.push(NpcAction::Say { text: "Move along. Keep the peace.".into() });
                    }
                } else {
                    if rng.gen::<f32>() < 0.5 {
                        // tiny patrol step
                        let step = view.self_pos + Vec3::new(rng.gen_range(-1.0..1.0), 0.0, rng.gen_range(-1.0..1.0));
                        actions.push(NpcAction::MoveTo { pos: step, speed: 1.2 });
                    }
                }
            }
            Role::Civilian => {
                if let Some(u) = player_utterance {
                    if u.to_lowercase().contains("hello") {
                        actions.push(NpcAction::Say { text: "Oh! Hello there.".into() });
                        actions.push(NpcAction::Emote { kind: EmoteKind::Wave });
                    } else {
                        actions.push(NpcAction::Say { text: "Sorry—busy day.".into() });
                    }
                }
            }
            Role::QuestGiver => {
                if let Some(u) = player_utterance {
                    if u.to_lowercase().contains("quest") || u.to_lowercase().contains("work") {
                        actions.push(NpcAction::Say { text: "There is something you can do...".into() });
                        actions.push(NpcAction::GiveQuest { id: "q_tutorial".into() });
                    } else {
                        actions.push(NpcAction::Say { text: "The threads whisper to those who listen.".into() });
                    }
                }
            }
        }

        Ok(NpcPlan { actions })
    }
}
