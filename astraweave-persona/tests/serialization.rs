// Test for deterministic persona/memory serialization
use astraweave_memory::{CompanionProfile, Skill, Fact};
use std::io::Write;
use std::fs::File;
use tempfile::tempdir;
use zip::write::SimpleFileOptions;

#[test]
fn test_persona_serialization_roundtrip() {
    let mut p = CompanionProfile::new_default();
    p.persona.tone = "serious".into();
    p.skills.push(Skill {
        name: "Stealth".into(),
        level: 3,
        notes: "Prefers shadows".into(),
    });
    p.sign();
    let s = serde_json::to_string(&p).unwrap();
    let p2: CompanionProfile = serde_json::from_str(&s).unwrap();
    assert!(p2.verify());
    assert_eq!(p2.persona.tone, "serious");
    assert_eq!(p2.skills[0].name, "Stealth");
}

#[test]
fn test_load_persona_zip_basic() {
    let dir = tempdir().unwrap();
    let zip_path = dir.path().join("test_persona.zip");
    
    // Create a test zip file
    {
        let file = File::create(&zip_path).unwrap();
        let mut zip = zip::ZipWriter::new(file);
        
        let manifest = r#"
tone = "friendly"
risk = "low"
humor = "high"
voice = "TestBot"
"#;
        
        zip.start_file("persona_manifest.toml", SimpleFileOptions::default()).unwrap();
        zip.write_all(manifest.as_bytes()).unwrap();
        zip.finish().unwrap();
    }
    
    let profile = astraweave_persona::load_persona_zip(zip_path.to_str().unwrap()).unwrap();
    
    assert_eq!(profile.persona.tone, "friendly");
    assert_eq!(profile.persona.risk, "low");
    assert_eq!(profile.persona.humor, "high");
    assert_eq!(profile.persona.voice, "TestBot");
    assert_eq!(profile.persona.name, "TestBot");
}

#[test]
fn test_load_persona_zip_with_skills() {
    let dir = tempdir().unwrap();
    let zip_path = dir.path().join("test_persona_skills.zip");
    
    {
        let file = File::create(&zip_path).unwrap();
        let mut zip = zip::ZipWriter::new(file);
        
        let manifest = r#"
tone = "serious"
risk = "medium"
humor = "low"
voice = "Warrior"

[[skills]]
name = "Combat"
level = 5
notes = "Expert fighter"

[[skills]]
name = "Tactics"
level = 3
notes = "Strategic thinker"
"#;
        
        zip.start_file("persona_manifest.toml", SimpleFileOptions::default()).unwrap();
        zip.write_all(manifest.as_bytes()).unwrap();
        zip.finish().unwrap();
    }
    
    let profile = astraweave_persona::load_persona_zip(zip_path.to_str().unwrap()).unwrap();
    
    assert_eq!(profile.skills.len(), 2);
    assert_eq!(profile.skills[0].name, "Combat");
    assert_eq!(profile.skills[0].level, 5);
    assert_eq!(profile.skills[0].notes, "Expert fighter");
    assert_eq!(profile.skills[1].name, "Tactics");
    assert_eq!(profile.skills[1].level, 3);
}

#[test]
fn test_load_persona_zip_with_facts() {
    let dir = tempdir().unwrap();
    let zip_path = dir.path().join("test_persona_facts.zip");
    
    {
        let file = File::create(&zip_path).unwrap();
        let mut zip = zip::ZipWriter::new(file);
        
        let manifest = r#"
tone = "curious"
risk = "high"
humor = "medium"
voice = "Explorer"

[[facts]]
k = "origin"
v = "Mars Colony"
t = "background"

[[facts]]
k = "favorite_color"
v = "blue"
t = "preference"
"#;
        
        zip.start_file("persona_manifest.toml", SimpleFileOptions::default()).unwrap();
        zip.write_all(manifest.as_bytes()).unwrap();
        zip.finish().unwrap();
    }
    
    let profile = astraweave_persona::load_persona_zip(zip_path.to_str().unwrap()).unwrap();
    
    assert_eq!(profile.facts.len(), 2);
    assert_eq!(profile.facts[0].k, "origin");
    assert_eq!(profile.facts[0].v, "Mars Colony");
    assert_eq!(profile.facts[0].t, "background");
}

#[test]
fn test_load_persona_zip_with_prefs_json() {
    let dir = tempdir().unwrap();
    let zip_path = dir.path().join("test_persona_prefs.zip");
    
    {
        let file = File::create(&zip_path).unwrap();
        let mut zip = zip::ZipWriter::new(file);
        
        let manifest = r#"
tone = "helpful"
risk = "low"
humor = "low"
voice = "Assistant"
prefs_json = '{"theme": "dark", "language": "en"}'
"#;
        
        zip.start_file("persona_manifest.toml", SimpleFileOptions::default()).unwrap();
        zip.write_all(manifest.as_bytes()).unwrap();
        zip.finish().unwrap();
    }
    
    let profile = astraweave_persona::load_persona_zip(zip_path.to_str().unwrap()).unwrap();
    
    assert!(profile.player_prefs.is_object());
    assert_eq!(profile.player_prefs["theme"], "dark");
    assert_eq!(profile.player_prefs["language"], "en");
}

#[test]
fn test_load_persona_zip_missing_manifest() {
    let dir = tempdir().unwrap();
    let zip_path = dir.path().join("test_persona_empty.zip");
    
    {
        let file = File::create(&zip_path).unwrap();
        let mut zip = zip::ZipWriter::new(file);
        
        // Create zip without manifest
        zip.start_file("other_file.txt", SimpleFileOptions::default()).unwrap();
        zip.write_all(b"test").unwrap();
        zip.finish().unwrap();
    }
    
    let result = astraweave_persona::load_persona_zip(zip_path.to_str().unwrap());
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("persona_manifest.toml missing"));
}

#[test]
fn test_load_persona_zip_invalid_path() {
    let result = astraweave_persona::load_persona_zip("nonexistent_file.zip");
    assert!(result.is_err());
}

#[test]
fn test_load_persona_zip_invalid_toml() {
    let dir = tempdir().unwrap();
    let zip_path = dir.path().join("test_persona_invalid.zip");
    
    {
        let file = File::create(&zip_path).unwrap();
        let mut zip = zip::ZipWriter::new(file);
        
        let manifest = "invalid toml content ][[ }";
        
        zip.start_file("persona_manifest.toml", SimpleFileOptions::default()).unwrap();
        zip.write_all(manifest.as_bytes()).unwrap();
        zip.finish().unwrap();
    }
    
    let result = astraweave_persona::load_persona_zip(zip_path.to_str().unwrap());
    assert!(result.is_err());
}

#[test]
fn test_load_persona_zip_signature() {
    let dir = tempdir().unwrap();
    let zip_path = dir.path().join("test_persona_sig.zip");
    
    {
        let file = File::create(&zip_path).unwrap();
        let mut zip = zip::ZipWriter::new(file);
        
        let manifest = r#"
tone = "test"
risk = "test"
humor = "test"
voice = "Test"
"#;
        
        zip.start_file("persona_manifest.toml", SimpleFileOptions::default()).unwrap();
        zip.write_all(manifest.as_bytes()).unwrap();
        zip.finish().unwrap();
    }
    
    let profile = astraweave_persona::load_persona_zip(zip_path.to_str().unwrap()).unwrap();
    
    // Profile should be signed after loading
    assert!(profile.verify());
}

#[test]
fn test_companion_profile_with_multiple_skills() {
    let mut p = CompanionProfile::new_default();
    p.skills.push(Skill {
        name: "Hacking".into(),
        level: 8,
        notes: "Elite hacker".into(),
    });
    p.skills.push(Skill {
        name: "Stealth".into(),
        level: 6,
        notes: "Silent movement".into(),
    });
    p.skills.push(Skill {
        name: "Combat".into(),
        level: 4,
        notes: "Basic combat training".into(),
    });
    
    assert_eq!(p.skills.len(), 3);
    assert_eq!(p.skills[0].level, 8);
    assert_eq!(p.skills[2].name, "Combat");
}

#[test]
fn test_companion_profile_with_multiple_facts() {
    let mut p = CompanionProfile::new_default();
    p.facts.push(Fact {
        k: "birthplace".into(),
        v: "New Tokyo".into(),
        t: "location".into(),
    });
    p.facts.push(Fact {
        k: "age".into(),
        v: "28".into(),
        t: "personal".into(),
    });
    
    assert_eq!(p.facts.len(), 2);
    assert_eq!(p.facts[1].k, "age");
}

#[test]
fn test_persona_default_values() {
    let p = CompanionProfile::new_default();
    
    // Check that defaults are set
    assert!(p.skills.is_empty());
    assert!(p.facts.is_empty());
    assert_eq!(p.player_prefs, serde_json::Value::Null);
}

#[test]
fn test_load_persona_zip_complete_profile() {
    let dir = tempdir().unwrap();
    let zip_path = dir.path().join("test_persona_complete.zip");
    
    {
        let file = File::create(&zip_path).unwrap();
        let mut zip = zip::ZipWriter::new(file);
        
        let manifest = r#"
tone = "adventurous"
risk = "high"
humor = "medium"
voice = "Captain"
prefs_json = '{"difficulty": "hard"}'

[[skills]]
name = "Navigation"
level = 9
notes = "Expert navigator"

[[skills]]
name = "Diplomacy"
level = 7
notes = "Skilled negotiator"

[[facts]]
k = "ship_name"
v = "Normandy"
t = "asset"

[[facts]]
k = "crew_size"
v = "50"
t = "stat"
"#;
        
        zip.start_file("persona_manifest.toml", SimpleFileOptions::default()).unwrap();
        zip.write_all(manifest.as_bytes()).unwrap();
        zip.finish().unwrap();
    }
    
    let profile = astraweave_persona::load_persona_zip(zip_path.to_str().unwrap()).unwrap();
    
    // Verify all components
    assert_eq!(profile.persona.tone, "adventurous");
    assert_eq!(profile.persona.risk, "high");
    assert_eq!(profile.skills.len(), 2);
    assert_eq!(profile.facts.len(), 2);
    assert_eq!(profile.player_prefs["difficulty"], "hard");
    assert!(profile.verify());
}
