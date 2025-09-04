use anyhow::{Result, anyhow};
use serde::Deserialize;
use std::io::Read;
use astraweave_memory::{CompanionProfile, Persona, Fact, Skill};

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
struct SkillEntry { name:String, level:u8, notes:String }
#[derive(Deserialize)]
struct FactEntry  { k:String, v:String, t:String }

pub fn load_persona_zip(path: &str) -> Result<CompanionProfile> {
    let file = std::fs::File::open(path)?;
    let mut zip = zip::ZipArchive::new(file)?;
    let mut manifest_txt = String::new();
    {
        let mut mf = zip.by_name("persona_manifest.toml")
            .map_err(|_| anyhow!("persona_manifest.toml missing"))?;
        mf.read_to_string(&mut manifest_txt)?;
    }
    let m: Manifest = toml::from_str(&manifest_txt)?;
    let mut p = CompanionProfile::new_default();
    p.persona = Persona { tone: m.tone, risk: m.risk, humor: m.humor, voice: m.voice };
    if let Some(js) = m.prefs_json {
        p.player_prefs = serde_json::from_str(&js)?;
    }
    if let Some(sk) = m.skills {
        p.skills = sk.into_iter().map(|s| Skill{ name:s.name, level:s.level, notes:s.notes }).collect();
    }
    if let Some(fs) = m.facts {
        p.facts = fs.into_iter().map(|f| Fact{ k:f.k, v:f.v, t:f.t }).collect();
    }
    p.sign();
    Ok(p)
}
