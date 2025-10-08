use serde::{Serialize, Deserialize};

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
            self.facts.push(Fact { k: e.title.clone(), v: e.summary.clone(), t: "".to_string() });
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
}
