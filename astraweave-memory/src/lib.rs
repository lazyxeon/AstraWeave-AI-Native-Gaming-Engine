use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use semver::Version;
use std::sync::OnceLock;

const CURRENT_VERSION: &str = "1.1.0";

static CURRENT_VERSION_PARSED: OnceLock<Version> = OnceLock::new();
static SKILL_MIGRATION_VERSION_PARSED: OnceLock<Version> = OnceLock::new();

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Persona {
    pub tone: String,
    pub risk: String,
    pub humor: String,
    pub voice: String,
    pub backstory: String,
    pub likes: Vec<String>,
    pub dislikes: Vec<String>,
    pub goals: Vec<String>,
}

impl Default for Persona {
    fn default() -> Self {
        Self {
            tone: "neutral".into(),
            risk: "low".into(),
            humor: "none".into(),
            voice: "default".into(),
            backstory: "".into(),
            likes: vec![],
            dislikes: vec![],
            goals: vec![],
        }
    }
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
            version: "1.1.0".into(),
            persona: Persona {
                tone: "dry".into(),
                risk: "medium".into(),
                humor: "light".into(),
                voice: "v01".into(),
                backstory: "Raised in the shadow of the old forest, loyal to the player.".into(),
                likes: vec!["stealth missions".into(), "ancient ruins".into()],
                dislikes: vec!["loud noises".into()],
                goals: vec!["protect the player".into(), "uncover lost secrets".into()],
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
        let mut p: Self = serde_json::from_str(&s)?;
        p.migrate()?;
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

    /// Guardrail: ensure version matches expected major.minor (patch allowed to vary)
    pub fn version_compatible(&self, expected_major: u32, expected_minor: u32) -> bool {
        let parts: Vec<&str> = self.version.split('.').collect();
        if parts.len() < 2 {
            return false;
        }
        let Ok(maj) = parts[0].parse::<u32>() else {
            return false;
        };
        let Ok(min) = parts[1].parse::<u32>() else {
            return false;
        };
        maj == expected_major && min == expected_minor
    }
}

impl CompanionProfile {
    /// Migrate profile to latest version
    pub fn migrate(&mut self) -> anyhow::Result<()> {
        let current_version = CURRENT_VERSION_PARSED.get_or_init(|| {
            Version::parse(CURRENT_VERSION).expect("Failed to parse CURRENT_VERSION constant")
        });
        let profile_version = Version::parse(&self.version)?;

        // If profile version is already at or above current, no migration needed
        if profile_version >= *current_version {
            return Ok(());
        }

        // Run skill-seeding migration only for versions < 1.1.0 and when skills is empty
        let skill_migration_version = SKILL_MIGRATION_VERSION_PARSED.get_or_init(|| {
            Version::parse("1.1.0").expect("Failed to parse skill migration version")
        });
        if profile_version < *skill_migration_version && self.skills.is_empty() {
            self.skills.push(Skill {
                name: "stealth".into(),
                level: 1,
                notes: "Basic training".into(),
            });
        }

        self.version = CURRENT_VERSION.to_string();
        self.sign();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_and_verify() {
        let mut p = CompanionProfile::new_default();
        p.episodes.push(Episode {
            title: "A".into(),
            summary: "Did thing".into(),
            tags: vec!["x".into()],
            ts: "t0".into(),
        });
        p.distill();
        p.sign();
        assert!(p.verify());
        let s = serde_json::to_string(&p).unwrap();
        let p2: CompanionProfile = serde_json::from_str(&s).unwrap();
        assert!(p2.verify());
        assert!(p2.version_compatible(1, 1));
    }

    #[test]
    fn migration_works() {
        let mut p = CompanionProfile {
            version: "1.0.0".into(),
            persona: Persona::default(),
            player_prefs: serde_json::json!({}),
            facts: vec![],
            episodes: vec![],
            skills: vec![],
            signature: None,
        };
        p.migrate().unwrap();
        assert_eq!(p.version, "1.1.0");
        assert!(!p.skills.is_empty());
        assert!(p.verify());
    }

    #[test]
    fn migration_noop_when_skills_present() {
        let existing_skill = Skill {
            name: "combat".into(),
            level: 2,
            notes: "Experienced".into(),
        };
        let mut p = CompanionProfile {
            version: "1.0.0".into(),
            persona: Persona::default(),
            player_prefs: serde_json::json!({}),
            facts: vec![],
            episodes: vec![],
            skills: vec![existing_skill.clone()],
            signature: None,
        };
        p.migrate().unwrap();
        assert_eq!(p.version, "1.1.0");
        assert_eq!(p.skills.len(), 1);
        assert_eq!(p.skills[0].name, "combat");
        assert_eq!(p.skills[0].level, 2);
        assert_eq!(p.skills[0].notes, "Experienced");
        assert!(p.verify());
    }

    #[test]
    fn migration_from_other_versions() {
        // Test migrating from 0.9.0 (should add skill if empty)
        let mut p_old = CompanionProfile {
            version: "0.9.0".into(),
            persona: Persona::default(),
            player_prefs: serde_json::json!({}),
            facts: vec![],
            episodes: vec![],
            skills: vec![],
            signature: None,
        };
        p_old.migrate().unwrap();
        assert_eq!(p_old.version, "1.1.0");
        assert!(!p_old.skills.is_empty());
        assert_eq!(p_old.skills[0].name, "stealth");

        // Test migrating from 1.0.1 (should add skill if empty)
        let mut p_patch = CompanionProfile {
            version: "1.0.1".into(),
            persona: Persona::default(),
            player_prefs: serde_json::json!({}),
            facts: vec![],
            episodes: vec![],
            skills: vec![],
            signature: None,
        };
        p_patch.migrate().unwrap();
        // Test migrating from 2.0.0 (future version, should not add skill)
        let mut p_future = CompanionProfile {
            version: "2.0.0".into(),
            persona: Persona::default(),
            player_prefs: serde_json::json!({}),
            facts: vec![],
            episodes: vec![],
            skills: vec![],
            signature: None,
        };
        // Future versions should not be downgraded
        let result = p_future.migrate();
        assert!(result.is_ok());
        assert_eq!(p_future.version, "2.0.0");
        assert!(p_future.skills.is_empty()); // No skill added for future versions
    }

    #[test]
    fn load_from_file_applies_migrations_integration() {
        use tempfile::NamedTempFile;
        use std::io::Write;

        let legacy_profile = CompanionProfile {
            version: "1.0.0".into(),
            persona: Persona::default(),
            player_prefs: serde_json::json!({}),
            facts: vec![],
            episodes: vec![],
            skills: vec![],
            signature: None,
        };
        let json = serde_json::to_string_pretty(&legacy_profile).unwrap();

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(json.as_bytes()).unwrap();
        let path = temp_file.path().to_str().unwrap();

        let loaded = CompanionProfile::load_from_file(path).unwrap();
        assert_eq!(loaded.version, "1.1.0");
        assert!(!loaded.skills.is_empty());
        assert_eq!(loaded.skills[0].name, "stealth");
        assert!(loaded.verify());
    }
}
