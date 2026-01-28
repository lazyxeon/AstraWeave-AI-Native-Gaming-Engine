use serde::{Deserialize, Serialize};

/// Minimal persona and companion types to satisfy cross-crate references in tests and examples.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Episode {
    pub title: String,
    pub summary: String,
    pub tags: Vec<String>,
    pub ts: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Persona {
    pub name: String,
    pub likes: Vec<String>,
    pub dislikes: Vec<String>,
    // Additional fields expected by consumers
    pub tone: String,
    pub risk: String,
    pub humor: String,
    pub voice: String,
    pub backstory: String,
    pub goals: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Fact {
    pub k: String,
    pub v: String,
    /// Optional type/metadata for the fact
    pub t: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Skill {
    pub name: String,
    pub level: u8,
    /// Additional notes about the skill
    pub notes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompanionProfile {
    pub id: String,
    pub persona: Persona,
    pub episodes: Vec<Episode>,
    pub facts: Vec<Fact>,
    /// Player preferences (free-form JSON)
    pub player_prefs: serde_json::Value,
    /// Companion skills
    pub skills: Vec<Skill>,
}

impl CompanionProfile {
    pub fn new_default() -> Self {
        Self {
            id: "companion_default".to_string(),
            persona: Persona::default(),
            episodes: Vec::new(),
            facts: Vec::new(),
            player_prefs: serde_json::Value::Null,
            skills: Vec::new(),
        }
    }

    pub fn distill(&mut self) {
        // naive distill: turn episodes into facts
        for e in &self.episodes {
            self.facts.push(Fact {
                k: e.title.clone(),
                v: e.summary.clone(),
                t: "".to_string(),
            });
        }
    }

    pub fn load_from_file(_p: &str) -> anyhow::Result<Self> {
        Ok(CompanionProfile::new_default())
    }

    /// Sign the profile (placeholder for integrity/signature logic)
    pub fn sign(&mut self) {
        // no-op placeholder; in production this would compute a signature
        // or integrity hash for the profile
    }

    /// Save the profile to a file (placeholder)
    pub fn save_to_file(&self, _p: &str) -> anyhow::Result<()> {
        // In tests/examples we don't actually write to disk; return Ok
        Ok(())
    }

    /// Verify the profile integrity (placeholder)
    pub fn verify(&self) -> bool {
        // Simplified: always true for the example
        true
    }

    /// Check if profile has any episodes
    pub fn has_episodes(&self) -> bool {
        !self.episodes.is_empty()
    }

    /// Count the number of episodes
    pub fn episode_count(&self) -> usize {
        self.episodes.len()
    }

    /// Check if profile has any facts
    pub fn has_facts(&self) -> bool {
        !self.facts.is_empty()
    }

    /// Count the number of facts
    pub fn fact_count(&self) -> usize {
        self.facts.len()
    }

    /// Check if profile has any skills
    pub fn has_skills(&self) -> bool {
        !self.skills.is_empty()
    }

    /// Count the number of skills
    pub fn skill_count(&self) -> usize {
        self.skills.len()
    }

    /// Add an episode to the profile
    pub fn add_episode(&mut self, episode: Episode) {
        self.episodes.push(episode);
    }

    /// Add a fact to the profile
    pub fn add_fact(&mut self, fact: Fact) {
        self.facts.push(fact);
    }

    /// Add a skill to the profile
    pub fn add_skill(&mut self, skill: Skill) {
        self.skills.push(skill);
    }

    /// Get a skill by name
    pub fn get_skill(&self, name: &str) -> Option<&Skill> {
        self.skills.iter().find(|s| s.name == name)
    }

    /// Get a fact by key
    pub fn get_fact(&self, key: &str) -> Option<&Fact> {
        self.facts.iter().find(|f| f.k == key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ====================================================================
    // Episode Tests
    // ====================================================================

    #[test]
    fn test_episode_default() {
        let episode = Episode::default();
        assert!(episode.title.is_empty());
        assert!(episode.summary.is_empty());
        assert!(episode.tags.is_empty());
        assert!(episode.ts.is_empty());
    }

    #[test]
    fn test_episode_with_values() {
        let episode = Episode {
            title: "Battle of Dawn".to_string(),
            summary: "A fierce battle at dawn".to_string(),
            tags: vec!["combat".to_string(), "victory".to_string()],
            ts: "2025-01-01T00:00:00Z".to_string(),
        };
        assert_eq!(episode.title, "Battle of Dawn");
        assert_eq!(episode.tags.len(), 2);
    }

    #[test]
    fn test_episode_clone() {
        let episode = Episode {
            title: "Test".to_string(),
            summary: "Summary".to_string(),
            tags: vec!["tag1".to_string()],
            ts: "now".to_string(),
        };
        let cloned = episode.clone();
        assert_eq!(episode.title, cloned.title);
        assert_eq!(episode.summary, cloned.summary);
    }

    #[test]
    fn test_episode_debug() {
        let episode = Episode::default();
        let debug_str = format!("{:?}", episode);
        assert!(debug_str.contains("Episode"));
    }

    // ====================================================================
    // Persona Tests
    // ====================================================================

    #[test]
    fn test_persona_default() {
        let persona = Persona::default();
        assert!(persona.name.is_empty());
        assert!(persona.likes.is_empty());
        assert!(persona.dislikes.is_empty());
        assert!(persona.tone.is_empty());
    }

    #[test]
    fn test_persona_with_values() {
        let persona = Persona {
            name: "Elara".to_string(),
            likes: vec!["music".to_string(), "adventure".to_string()],
            dislikes: vec!["cowardice".to_string()],
            tone: "friendly".to_string(),
            risk: "moderate".to_string(),
            humor: "witty".to_string(),
            voice: "melodic".to_string(),
            backstory: "Born in the mountains...".to_string(),
            goals: vec!["Find the artifact".to_string()],
        };
        assert_eq!(persona.name, "Elara");
        assert_eq!(persona.likes.len(), 2);
        assert_eq!(persona.dislikes.len(), 1);
    }

    #[test]
    fn test_persona_clone() {
        let persona = Persona {
            name: "Clone Test".to_string(),
            likes: vec!["testing".to_string()],
            ..Default::default()
        };
        let cloned = persona.clone();
        assert_eq!(persona.name, cloned.name);
    }

    // ====================================================================
    // Fact Tests
    // ====================================================================

    #[test]
    fn test_fact_default() {
        let fact = Fact::default();
        assert!(fact.k.is_empty());
        assert!(fact.v.is_empty());
        assert!(fact.t.is_empty());
    }

    #[test]
    fn test_fact_with_values() {
        let fact = Fact {
            k: "location".to_string(),
            v: "Enchanted Forest".to_string(),
            t: "place".to_string(),
        };
        assert_eq!(fact.k, "location");
        assert_eq!(fact.v, "Enchanted Forest");
        assert_eq!(fact.t, "place");
    }

    #[test]
    fn test_fact_clone() {
        let fact = Fact {
            k: "key".to_string(),
            v: "value".to_string(),
            t: "type".to_string(),
        };
        let cloned = fact.clone();
        assert_eq!(fact.k, cloned.k);
    }

    // ====================================================================
    // Skill Tests
    // ====================================================================

    #[test]
    fn test_skill_default() {
        let skill = Skill::default();
        assert!(skill.name.is_empty());
        assert_eq!(skill.level, 0);
        assert!(skill.notes.is_empty());
    }

    #[test]
    fn test_skill_with_values() {
        let skill = Skill {
            name: "Swordsmanship".to_string(),
            level: 75,
            notes: "Expert level fighter".to_string(),
        };
        assert_eq!(skill.name, "Swordsmanship");
        assert_eq!(skill.level, 75);
    }

    #[test]
    fn test_skill_clone() {
        let skill = Skill {
            name: "Archery".to_string(),
            level: 50,
            notes: "Intermediate".to_string(),
        };
        let cloned = skill.clone();
        assert_eq!(skill.name, cloned.name);
        assert_eq!(skill.level, cloned.level);
    }

    // ====================================================================
    // CompanionProfile Construction Tests
    // ====================================================================

    #[test]
    fn test_companion_profile_new_default() {
        let profile = CompanionProfile::new_default();
        assert_eq!(profile.id, "companion_default");
        assert!(profile.episodes.is_empty());
        assert!(profile.facts.is_empty());
        assert!(profile.skills.is_empty());
    }

    #[test]
    fn test_companion_profile_with_custom_id() {
        let mut profile = CompanionProfile::new_default();
        profile.id = "custom_companion".to_string();
        assert_eq!(profile.id, "custom_companion");
    }

    #[test]
    fn test_companion_profile_clone() {
        let profile = CompanionProfile::new_default();
        let cloned = profile.clone();
        assert_eq!(profile.id, cloned.id);
    }

    #[test]
    fn test_companion_profile_debug() {
        let profile = CompanionProfile::new_default();
        let debug_str = format!("{:?}", profile);
        assert!(debug_str.contains("CompanionProfile"));
    }

    // ====================================================================
    // CompanionProfile Episode Tests
    // ====================================================================

    #[test]
    fn test_companion_profile_has_episodes_empty() {
        let profile = CompanionProfile::new_default();
        assert!(!profile.has_episodes());
    }

    #[test]
    fn test_companion_profile_has_episodes_with_content() {
        let mut profile = CompanionProfile::new_default();
        profile.add_episode(Episode {
            title: "Test Episode".to_string(),
            ..Default::default()
        });
        assert!(profile.has_episodes());
    }

    #[test]
    fn test_companion_profile_episode_count() {
        let mut profile = CompanionProfile::new_default();
        assert_eq!(profile.episode_count(), 0);

        profile.add_episode(Episode::default());
        assert_eq!(profile.episode_count(), 1);

        profile.add_episode(Episode::default());
        assert_eq!(profile.episode_count(), 2);
    }

    #[test]
    fn test_companion_profile_add_episode() {
        let mut profile = CompanionProfile::new_default();
        let episode = Episode {
            title: "New Episode".to_string(),
            summary: "Something happened".to_string(),
            tags: vec!["important".to_string()],
            ts: "today".to_string(),
        };

        profile.add_episode(episode);
        assert_eq!(profile.episode_count(), 1);
        assert_eq!(profile.episodes[0].title, "New Episode");
    }

    // ====================================================================
    // CompanionProfile Fact Tests
    // ====================================================================

    #[test]
    fn test_companion_profile_has_facts_empty() {
        let profile = CompanionProfile::new_default();
        assert!(!profile.has_facts());
    }

    #[test]
    fn test_companion_profile_has_facts_with_content() {
        let mut profile = CompanionProfile::new_default();
        profile.add_fact(Fact {
            k: "key".to_string(),
            v: "value".to_string(),
            t: "".to_string(),
        });
        assert!(profile.has_facts());
    }

    #[test]
    fn test_companion_profile_fact_count() {
        let mut profile = CompanionProfile::new_default();
        assert_eq!(profile.fact_count(), 0);

        profile.add_fact(Fact::default());
        assert_eq!(profile.fact_count(), 1);
    }

    #[test]
    fn test_companion_profile_add_fact() {
        let mut profile = CompanionProfile::new_default();
        let fact = Fact {
            k: "player_name".to_string(),
            v: "Hero".to_string(),
            t: "meta".to_string(),
        };

        profile.add_fact(fact);
        assert_eq!(profile.fact_count(), 1);
        assert_eq!(profile.facts[0].k, "player_name");
    }

    #[test]
    fn test_companion_profile_get_fact() {
        let mut profile = CompanionProfile::new_default();
        profile.add_fact(Fact {
            k: "test_key".to_string(),
            v: "test_value".to_string(),
            t: "".to_string(),
        });

        let fact = profile.get_fact("test_key");
        assert!(fact.is_some());
        assert_eq!(fact.unwrap().v, "test_value");
    }

    #[test]
    fn test_companion_profile_get_fact_nonexistent() {
        let profile = CompanionProfile::new_default();
        assert!(profile.get_fact("missing").is_none());
    }

    // ====================================================================
    // CompanionProfile Skill Tests
    // ====================================================================

    #[test]
    fn test_companion_profile_has_skills_empty() {
        let profile = CompanionProfile::new_default();
        assert!(!profile.has_skills());
    }

    #[test]
    fn test_companion_profile_has_skills_with_content() {
        let mut profile = CompanionProfile::new_default();
        profile.add_skill(Skill {
            name: "Combat".to_string(),
            level: 50,
            notes: "".to_string(),
        });
        assert!(profile.has_skills());
    }

    #[test]
    fn test_companion_profile_skill_count() {
        let mut profile = CompanionProfile::new_default();
        assert_eq!(profile.skill_count(), 0);

        profile.add_skill(Skill::default());
        assert_eq!(profile.skill_count(), 1);
    }

    #[test]
    fn test_companion_profile_add_skill() {
        let mut profile = CompanionProfile::new_default();
        let skill = Skill {
            name: "Magic".to_string(),
            level: 80,
            notes: "Fire specialist".to_string(),
        };

        profile.add_skill(skill);
        assert_eq!(profile.skill_count(), 1);
        assert_eq!(profile.skills[0].name, "Magic");
    }

    #[test]
    fn test_companion_profile_get_skill() {
        let mut profile = CompanionProfile::new_default();
        profile.add_skill(Skill {
            name: "Stealth".to_string(),
            level: 65,
            notes: "Shadow walker".to_string(),
        });

        let skill = profile.get_skill("Stealth");
        assert!(skill.is_some());
        assert_eq!(skill.unwrap().level, 65);
    }

    #[test]
    fn test_companion_profile_get_skill_nonexistent() {
        let profile = CompanionProfile::new_default();
        assert!(profile.get_skill("NonExistent").is_none());
    }

    // ====================================================================
    // CompanionProfile Distill Tests
    // ====================================================================

    #[test]
    fn test_companion_profile_distill_empty() {
        let mut profile = CompanionProfile::new_default();
        profile.distill();
        assert!(!profile.has_facts()); // No episodes to distill
    }

    #[test]
    fn test_companion_profile_distill_with_episodes() {
        let mut profile = CompanionProfile::new_default();
        profile.add_episode(Episode {
            title: "First Battle".to_string(),
            summary: "Won against goblins".to_string(),
            tags: vec![],
            ts: "".to_string(),
        });
        profile.add_episode(Episode {
            title: "Quest Complete".to_string(),
            summary: "Found the treasure".to_string(),
            tags: vec![],
            ts: "".to_string(),
        });

        profile.distill();
        assert_eq!(profile.fact_count(), 2);
        assert!(profile.get_fact("First Battle").is_some());
        assert!(profile.get_fact("Quest Complete").is_some());
    }

    #[test]
    fn test_companion_profile_distill_preserves_episode_info() {
        let mut profile = CompanionProfile::new_default();
        profile.add_episode(Episode {
            title: "Title".to_string(),
            summary: "Summary text".to_string(),
            tags: vec![],
            ts: "".to_string(),
        });

        profile.distill();
        let fact = profile.get_fact("Title").unwrap();
        assert_eq!(fact.v, "Summary text");
    }

    // ====================================================================
    // CompanionProfile Load/Save/Verify Tests
    // ====================================================================

    #[test]
    fn test_companion_profile_load_from_file() {
        let result = CompanionProfile::load_from_file("any_path");
        assert!(result.is_ok());
        let profile = result.unwrap();
        assert_eq!(profile.id, "companion_default");
    }

    #[test]
    fn test_companion_profile_save_to_file() {
        let profile = CompanionProfile::new_default();
        let result = profile.save_to_file("test_path");
        assert!(result.is_ok());
    }

    #[test]
    fn test_companion_profile_sign() {
        let mut profile = CompanionProfile::new_default();
        profile.sign(); // Should not panic
    }

    #[test]
    fn test_companion_profile_verify() {
        let profile = CompanionProfile::new_default();
        assert!(profile.verify());
    }

    // ====================================================================
    // Serialization Tests
    // ====================================================================

    #[test]
    fn test_episode_serialization() {
        let episode = Episode {
            title: "Serialize Me".to_string(),
            summary: "A test".to_string(),
            tags: vec!["test".to_string()],
            ts: "2025".to_string(),
        };

        let json = serde_json::to_string(&episode).unwrap();
        let deserialized: Episode = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.title, "Serialize Me");
    }

    #[test]
    fn test_persona_serialization() {
        let persona = Persona {
            name: "Test".to_string(),
            likes: vec!["coding".to_string()],
            dislikes: vec![],
            tone: "calm".to_string(),
            risk: "low".to_string(),
            humor: "dry".to_string(),
            voice: "quiet".to_string(),
            backstory: "Story".to_string(),
            goals: vec!["finish".to_string()],
        };

        let json = serde_json::to_string(&persona).unwrap();
        let deserialized: Persona = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.name, "Test");
    }

    #[test]
    fn test_fact_serialization() {
        let fact = Fact {
            k: "key".to_string(),
            v: "value".to_string(),
            t: "type".to_string(),
        };

        let json = serde_json::to_string(&fact).unwrap();
        let deserialized: Fact = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.k, "key");
    }

    #[test]
    fn test_skill_serialization() {
        let skill = Skill {
            name: "Swimming".to_string(),
            level: 42,
            notes: "Good form".to_string(),
        };

        let json = serde_json::to_string(&skill).unwrap();
        let deserialized: Skill = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.name, "Swimming");
        assert_eq!(deserialized.level, 42);
    }

    #[test]
    fn test_companion_profile_serialization() {
        let mut profile = CompanionProfile::new_default();
        profile.add_episode(Episode {
            title: "Test".to_string(),
            ..Default::default()
        });
        profile.add_fact(Fact {
            k: "k".to_string(),
            v: "v".to_string(),
            t: "".to_string(),
        });
        profile.add_skill(Skill {
            name: "Skill".to_string(),
            level: 10,
            notes: "".to_string(),
        });

        let json = serde_json::to_string(&profile).unwrap();
        let deserialized: CompanionProfile = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.id, "companion_default");
        assert_eq!(deserialized.episode_count(), 1);
        assert_eq!(deserialized.fact_count(), 1);
        assert_eq!(deserialized.skill_count(), 1);
    }

    // ====================================================================
    // Edge Case Tests
    // ====================================================================

    #[test]
    fn test_profile_multiple_skills_same_name() {
        let mut profile = CompanionProfile::new_default();
        profile.add_skill(Skill {
            name: "Combat".to_string(),
            level: 10,
            notes: "First".to_string(),
        });
        profile.add_skill(Skill {
            name: "Combat".to_string(),
            level: 20,
            notes: "Second".to_string(),
        });

        // get_skill returns first match
        let skill = profile.get_skill("Combat");
        assert!(skill.is_some());
        assert_eq!(skill.unwrap().level, 10);
    }

    #[test]
    fn test_profile_unicode_content() {
        let mut profile = CompanionProfile::new_default();
        profile.persona.name = "日本語キャラクター".to_string();
        profile.add_episode(Episode {
            title: "冒険の始まり".to_string(),
            summary: "🎮 ゲーム開始!".to_string(),
            tags: vec!["日本語".to_string()],
            ts: "".to_string(),
        });

        assert_eq!(profile.persona.name, "日本語キャラクター");
        assert_eq!(profile.episodes[0].title, "冒険の始まり");
    }

    #[test]
    fn test_profile_empty_strings() {
        let mut profile = CompanionProfile::new_default();
        profile.add_fact(Fact {
            k: "".to_string(),
            v: "".to_string(),
            t: "".to_string(),
        });

        let fact = profile.get_fact("");
        assert!(fact.is_some());
    }

    #[test]
    fn test_profile_player_prefs_json() {
        let mut profile = CompanionProfile::new_default();
        profile.player_prefs =
            serde_json::json!({"difficulty": "hard", "volume": 75, "hints": false});

        if let serde_json::Value::Object(map) = &profile.player_prefs {
            assert!(map.contains_key("difficulty"));
            assert!(map.contains_key("volume"));
        } else {
            panic!("Expected object");
        }
    }
}
