// Test for deterministic persona/memory serialization
use astraweave_memory::{CompanionProfile, Skill};
use astraweave_persona::*;

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
