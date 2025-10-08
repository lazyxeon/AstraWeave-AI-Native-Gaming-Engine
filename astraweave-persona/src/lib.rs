use anyhow::{anyhow, Result};
// persona types live in the `persona` module of `astraweave_memory`
use astraweave_memory::persona::{CompanionProfile, Fact, Persona, Skill};
use serde::Deserialize;
use std::io::Read;

// Phase 2 LLM Integration
pub mod llm_persona;
pub use llm_persona::*;

#[derive(Deserialize)]
struct Manifest {
    tone: String,
    risk: String,
    humor: String,
    voice: String,
    #[serde(default)]
    prefs_json: Option<String>,
    #[serde(default)]
    skills: Option<Vec<SkillEntry>>,
    #[serde(default)]
    facts: Option<Vec<FactEntry>>,
}
#[derive(Deserialize)]
struct SkillEntry {
    name: String,
    level: u8,
    notes: String,
}
#[derive(Deserialize)]
struct FactEntry {
    k: String,
    v: String,
    t: String,
}

pub fn load_persona_zip(path: &str) -> Result<CompanionProfile> {
    let file = std::fs::File::open(path)?;
    let mut zip = zip::ZipArchive::new(file)?;
    let mut manifest_txt = String::new();
    {
        let mut mf = zip
            .by_name("persona_manifest.toml")
            .map_err(|_| anyhow!("persona_manifest.toml missing"))?;
        mf.read_to_string(&mut manifest_txt)?;
    }
    let m: Manifest = toml::from_str(&manifest_txt)?;
    let mut p = CompanionProfile::new_default();
    // Populate persona fields
    p.persona.name = m.voice.clone();
    p.persona.tone = m.tone.clone();
    p.persona.risk = m.risk.clone();
    p.persona.humor = m.humor.clone();
    p.persona.voice = m.voice.clone();
    p.persona.backstory = "Loaded from manifest".to_string();

    // Player preferences (optional JSON)
    if let Some(js) = m.prefs_json {
        p.player_prefs = serde_json::from_str(&js).unwrap_or(serde_json::Value::Null);
    }

    // Skills
    if let Some(sk) = m.skills {
        p.skills = sk
            .into_iter()
            .map(|s| Skill {
                name: s.name,
                level: s.level,
                notes: s.notes,
            })
            .collect();
    }

    // Facts
    if let Some(fs) = m.facts {
        p.facts = fs
            .into_iter()
            .map(|f| Fact {
                k: f.k,
                v: f.v,
                t: f.t,
            })
            .collect();
    }

    // Sign the profile (no-op placeholder)
    p.sign();
    Ok(p)
}
