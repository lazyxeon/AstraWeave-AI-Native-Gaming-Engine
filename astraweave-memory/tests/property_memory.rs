// Property-based test for dynamic memory update
use astraweave_memory::CompanionProfile;

#[test]
fn test_dynamic_memory_update() {
    let mut profile = CompanionProfile::new_default();
    // Add a new episode and distill
    profile.episodes.push(astraweave_memory::Episode {
        title: "Rescued Player".into(),
        summary: "Saved the player from danger.".into(),
        tags: vec!["heroic".into()],
        ts: "2025-09-23T12:00:00Z".into(),
    });
    profile.distill();
    assert!(profile.facts.iter().any(|f| f.k.contains("Rescued Player")));
    // Update preferences
    profile.persona.likes.push("quiet places".into());
    assert!(profile.persona.likes.contains(&"quiet places".to_string()));
}
