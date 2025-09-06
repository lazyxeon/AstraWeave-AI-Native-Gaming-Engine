use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Persona {
    pub tone: String,
    pub risk: String,
    pub humor: String,
    pub voice: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Fact {
    pub k: String,
    pub v: String,
    pub t: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Episode {
    pub title: String,
    pub summary: String,
    pub tags: Vec<String>,
    pub ts: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Skill {
    pub name: String,
    pub level: u8,
    pub notes: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CompanionProfile {
    pub version: String,
    pub persona: Persona,
    pub player_prefs: serde_json::Value,
    pub facts: Vec<Fact>,
    pub episodes: Vec<Episode>,
    pub skills: Vec<Skill>,
    pub signature: Option<String>,
}

impl CompanionProfile {
    pub fn new_default() -> Self {
        Self {
            version: "1.0.0".into(),
            persona: Persona {
                tone: "dry".into(),
                risk: "medium".into(),
                humor: "light".into(),
                voice: "v01".into(),
            },
            player_prefs: serde_json::json!({"stealth_bias":0.5,"loot_greed":0.2}),
            facts: vec![],
            episodes: vec![],
            skills: vec![],
            signature: None,
        }
    }

    pub fn distill(&mut self) {
        // naive: convert older episodes into facts and truncate
        let mut new_facts = vec![];
        for e in self.episodes.drain(..).take(10) {
            new_facts.push(Fact {
                k: format!("ep:{}", e.title),
                v: e.summary,
                t: e.ts,
            });
        }
        self.facts.extend(new_facts);
    }

    pub fn sign(&mut self) {
        // simple content hash as "signature" (not cryptographically signed)
        let mut hasher = Sha256::new();
        let mut clone = self.clone();
        clone.signature = None;
        let bytes = serde_json::to_vec(&clone).unwrap();
        hasher.update(bytes);
        let out = hasher.finalize();
        self.signature = Some(hex::encode(out));
    }

    pub fn save_to_file(&self, path: &str) -> anyhow::Result<()> {
        let s = serde_json::to_string_pretty(self)?;
        std::fs::write(path, s)?;
        Ok(())
    }

    pub fn load_from_file(path: &str) -> anyhow::Result<Self> {
        let s = std::fs::read_to_string(path)?;
        let p: Self = serde_json::from_str(&s)?;
        Ok(p)
    }

    pub fn verify(&self) -> bool {
        if let Some(sig) = &self.signature {
            let mut hasher = Sha256::new();
            let mut clone = self.clone();
            clone.signature = None;
            let bytes = serde_json::to_vec(&clone).unwrap();
            hasher.update(bytes);
            let out = hasher.finalize();
            return *sig == hex::encode(out);
        }
        false
    }
}
