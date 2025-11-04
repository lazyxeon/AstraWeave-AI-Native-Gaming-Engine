//! Integration tests for behavioral pattern detection and preference profiles

use astraweave_memory::episode::{
    ActionResult, CompanionResponse, Episode as GameEpisode, EpisodeCategory, EpisodeOutcome,
    Observation, PlayerAction,
};
use astraweave_memory::{
    MemoryStorage, PatternDetector, PlaystylePattern, PreferenceProfile, ProfileBuilder,
};

fn create_combat_episode(
    id: &str,
    damage_dealt: f32,
    damage_taken: f32,
    duration_ms: u64,
    companion_action: &str,
    effectiveness: f32,
) -> GameEpisode {
    let mut episode = GameEpisode::new(id.to_string(), EpisodeCategory::Combat);
    episode.outcome = Some(EpisodeOutcome {
        success_rating: 0.85,
        player_satisfaction: 0.8,
        companion_effectiveness: effectiveness,
        duration_ms,
        damage_dealt,
        damage_taken,
        resources_used: 100.0,
        failure_count: 0,
    });

    // Add observation with companion response
    episode.add_observation(Observation::new(
        0,
        Some(PlayerAction {
            action_type: "melee_attack".to_string(),
            target: Some("enemy".to_string()),
            parameters: serde_json::json!({}),
        }),
        Some(CompanionResponse {
            action_type: companion_action.to_string(),
            result: ActionResult::Success,
            effectiveness,
        }),
        serde_json::json!({
            "player_health": 90.0,
            "enemy_count": 2
        }),
    ));

    episode
}

fn create_dialogue_episode(id: &str, companion_action: &str, effectiveness: f32) -> GameEpisode {
    let mut episode = GameEpisode::new(id.to_string(), EpisodeCategory::Dialogue);
    episode.outcome = Some(EpisodeOutcome {
        success_rating: 0.75,
        player_satisfaction: 0.85,
        companion_effectiveness: effectiveness,
        duration_ms: 5000,
        damage_dealt: 0.0,
        damage_taken: 0.0,
        resources_used: 10.0,
        failure_count: 0,
    });

    // Add multiple observations for analytical pattern
    for i in 0..6 {
        episode.add_observation(Observation::new(
            i * 1000,
            None,
            Some(CompanionResponse {
                action_type: companion_action.to_string(),
                result: ActionResult::Success,
                effectiveness,
            }),
            serde_json::json!({}),
        ));
    }

    episode
}

#[test]
fn test_pattern_detection_from_storage() {
    let mut storage = MemoryStorage::in_memory().unwrap();

    // Store aggressive combat episodes
    for i in 0..5 {
        let episode = create_combat_episode(
            &format!("aggressive_{}", i),
            500.0, // High damage dealt
            75.0,  // Accepts damage
            8000,
            "offensive_buff",
            0.85,
        );
        storage.store_memory(&episode.to_memory().unwrap()).unwrap();
    }

    // Store cautious combat episodes
    for i in 0..3 {
        let episode = create_combat_episode(
            &format!("cautious_{}", i),
            200.0, // Low damage
            20.0,  // Low damage taken
            15000,
            "defensive_shield",
            0.75,
        );
        storage.store_memory(&episode.to_memory().unwrap()).unwrap();
    }

    let detector = PatternDetector::new();
    let patterns = detector.detect_playstyle_patterns(&storage).unwrap();

    // Should detect both aggressive and cautious patterns
    assert!(patterns.len() >= 1);
    assert!(patterns
        .iter()
        .any(|p| p.pattern == PlaystylePattern::Aggressive));
}

#[test]
fn test_pattern_confidence_threshold() {
    let mut storage = MemoryStorage::in_memory().unwrap();

    // Store only 3 episodes (below minimum of 5)
    for i in 0..3 {
        let episode =
            create_combat_episode(&format!("ep_{}", i), 400.0, 50.0, 10000, "support", 0.8);
        storage.store_memory(&episode.to_memory().unwrap()).unwrap();
    }

    let detector = PatternDetector::new();
    let patterns = detector.detect_playstyle_patterns(&storage).unwrap();

    // Should return empty because below minimum threshold
    assert_eq!(patterns.len(), 0);
}

#[test]
fn test_social_pattern_detection() {
    let mut storage = MemoryStorage::in_memory().unwrap();

    // Store dialogue episodes
    for i in 0..6 {
        let episode =
            create_dialogue_episode(&format!("dialogue_{}", i), "conversational_support", 0.9);
        storage.store_memory(&episode.to_memory().unwrap()).unwrap();
    }

    let detector = PatternDetector::new();
    let patterns = detector.detect_playstyle_patterns(&storage).unwrap();

    // Should detect social pattern
    assert!(patterns
        .iter()
        .any(|p| p.pattern == PlaystylePattern::Social));

    // May also detect analytical (many observations per episode)
    assert!(patterns.len() >= 1);
}

#[test]
fn test_action_sequence_detection() {
    let mut storage = MemoryStorage::in_memory().unwrap();

    // Create episode with repeating action sequence
    for ep_idx in 0..5 {
        let mut episode =
            GameEpisode::new(format!("seq_episode_{}", ep_idx), EpisodeCategory::Combat);
        episode.outcome = Some(EpisodeOutcome {
            success_rating: 0.8,
            player_satisfaction: 0.75,
            companion_effectiveness: 0.85,
            duration_ms: 10000,
            damage_dealt: 400.0,
            damage_taken: 50.0,
            resources_used: 100.0,
            failure_count: 0,
        });

        // Repeated sequence: attack -> dodge -> attack
        for i in 0..3 {
            let action_type = if i == 1 { "dodge" } else { "melee_attack" };
            episode.add_observation(Observation::new(
                (i * 1000) as u64,
                Some(PlayerAction {
                    action_type: action_type.to_string(),
                    target: if i != 1 {
                        Some("enemy".to_string())
                    } else {
                        None
                    },
                    parameters: serde_json::json!({}),
                }),
                None,
                serde_json::json!({}),
            ));
        }

        storage.store_memory(&episode.to_memory().unwrap()).unwrap();
    }

    let detector = PatternDetector::new();
    let sequences = detector.detect_action_sequences(&storage, 3).unwrap();

    // Should detect the repeated attack-dodge-attack pattern
    assert!(!sequences.is_empty());
    assert!(sequences
        .iter()
        .any(|seq| seq.sequence.contains(&"melee_attack".to_string())
            && seq.sequence.contains(&"dodge".to_string())));
}

#[test]
fn test_companion_effectiveness_analysis() {
    let mut storage = MemoryStorage::in_memory().unwrap();

    // Combat episodes with high effectiveness
    for i in 0..5 {
        let episode = create_combat_episode(
            &format!("combat_{}", i),
            400.0,
            50.0,
            10000,
            "combat_support",
            0.9, // High effectiveness
        );
        storage.store_memory(&episode.to_memory().unwrap()).unwrap();
    }

    // Dialogue episodes with lower effectiveness
    for i in 0..3 {
        let episode = create_dialogue_episode(
            &format!("dialogue_{}", i),
            "dialogue_support",
            0.65, // Lower effectiveness
        );
        storage.store_memory(&episode.to_memory().unwrap()).unwrap();
    }

    let detector = PatternDetector::new();
    let effectiveness = detector.analyze_companion_effectiveness(&storage).unwrap();

    assert!(effectiveness.contains_key(&EpisodeCategory::Combat));
    assert!(effectiveness.contains_key(&EpisodeCategory::Dialogue));

    // Combat should have higher effectiveness
    assert!(effectiveness[&EpisodeCategory::Combat] > effectiveness[&EpisodeCategory::Dialogue]);
}

#[test]
fn test_category_distribution() {
    let mut storage = MemoryStorage::in_memory().unwrap();

    // Store varied categories
    for i in 0..10 {
        let episode =
            create_combat_episode(&format!("combat_{}", i), 400.0, 50.0, 10000, "support", 0.8);
        storage.store_memory(&episode.to_memory().unwrap()).unwrap();
    }

    for i in 0..5 {
        let episode = create_dialogue_episode(&format!("dialogue_{}", i), "support", 0.8);
        storage.store_memory(&episode.to_memory().unwrap()).unwrap();
    }

    let detector = PatternDetector::new();
    let distribution = detector.get_category_distribution(&storage).unwrap();

    assert_eq!(distribution.get(&EpisodeCategory::Combat), Some(&10));
    assert_eq!(distribution.get(&EpisodeCategory::Dialogue), Some(&5));
    assert_eq!(distribution.len(), 2);
}

#[test]
fn test_preference_profile_building() {
    let mut storage = MemoryStorage::in_memory().unwrap();

    // Store diverse episodes
    for i in 0..15 {
        let category = if i < 10 {
            EpisodeCategory::Combat
        } else {
            EpisodeCategory::Dialogue
        };

        let episode = if i < 10 {
            create_combat_episode(
                &format!("ep_{}", i),
                400.0,
                60.0, // Increased to ensure Aggressive pattern matches (> 50.0)
                10000,
                "healing_spell",
                0.85,
            )
        } else {
            create_dialogue_episode(&format!("ep_{}", i), "supportive_comment", 0.75)
        };

        storage.store_memory(&episode.to_memory().unwrap()).unwrap();
    }

    let builder = ProfileBuilder::new();
    let profile = builder.build_profile(&storage).unwrap();

    assert_eq!(profile.episode_count, 15);
    assert!(profile.learning_confidence > 0.0);

    // Debug output
    println!("Episode count: {}", profile.episode_count);
    println!("Learning confidence: {}", profile.learning_confidence);
    println!("Converged: {}", profile.converged);
    println!("Patterns: {}", profile.dominant_patterns.len());
    println!("Categories: {}", profile.preferred_categories.len());

    assert!(
        profile.converged,
        "Profile should converge with 15 episodes (confidence: {})",
        profile.learning_confidence
    );
    assert!(!profile.dominant_patterns.is_empty());
    assert!(!profile.preferred_categories.is_empty());
}

#[test]
fn test_profile_convergence() {
    let mut storage = MemoryStorage::in_memory().unwrap();

    // Store only 10 episodes
    for i in 0..10 {
        let episode =
            create_combat_episode(&format!("ep_{}", i), 400.0, 50.0, 10000, "support", 0.8);
        storage.store_memory(&episode.to_memory().unwrap()).unwrap();
    }

    let builder = ProfileBuilder::new();
    let profile = builder.build_profile(&storage).unwrap();

    assert_eq!(profile.episode_count, 10);
    assert!(!profile.converged); // Should not converge with only 10 episodes

    // Add more episodes
    for i in 10..20 {
        let episode =
            create_combat_episode(&format!("ep_{}", i), 400.0, 50.0, 10000, "support", 0.8);
        storage.store_memory(&episode.to_memory().unwrap()).unwrap();
    }

    let profile2 = builder.build_profile(&storage).unwrap();
    assert_eq!(profile2.episode_count, 20);
    assert!(profile2.learning_confidence > profile.learning_confidence);
}

#[test]
fn test_action_recommendation() {
    let mut storage = MemoryStorage::in_memory().unwrap();

    // Store episodes with consistent companion action
    for i in 0..10 {
        let episode = create_combat_episode(
            &format!("ep_{}", i),
            400.0,
            50.0,
            10000,
            "healing_spell", // Consistent action
            0.9,             // High effectiveness
        );
        storage.store_memory(&episode.to_memory().unwrap()).unwrap();
    }

    let builder = ProfileBuilder::new();
    let profile = builder.build_profile(&storage).unwrap();

    // Should have learned healing_spell preference
    assert!(!profile.optimal_responses.is_empty());

    let satisfaction = builder.predict_satisfaction(&profile, "healing_spell");
    assert!(satisfaction > 0.7); // High predicted satisfaction
}

#[test]
fn test_category_preference_detection() {
    let mut storage = MemoryStorage::in_memory().unwrap();

    // Combat episodes with high quality
    for i in 0..8 {
        let mut episode = create_combat_episode(
            &format!("combat_{}", i),
            400.0,
            50.0,
            10000,
            "support",
            0.85,
        );
        if let Some(ref mut outcome) = episode.outcome {
            outcome.success_rating = 0.9;
            outcome.player_satisfaction = 0.9;
        }
        storage.store_memory(&episode.to_memory().unwrap()).unwrap();
    }

    // Dialogue episodes with lower quality
    for i in 0..4 {
        let mut episode = create_dialogue_episode(&format!("dialogue_{}", i), "support", 0.65);
        if let Some(ref mut outcome) = episode.outcome {
            outcome.success_rating = 0.6;
            outcome.player_satisfaction = 0.6;
        }
        storage.store_memory(&episode.to_memory().unwrap()).unwrap();
    }

    let builder = ProfileBuilder::new();
    let profile = builder.build_profile(&storage).unwrap();

    // Combat should be preferred (higher quality and frequency)
    let preferred = ProfileBuilder::get_preferred_category(&profile);
    assert_eq!(preferred, Some(EpisodeCategory::Combat));
}

#[test]
fn test_primary_pattern_identification() {
    let mut storage = MemoryStorage::in_memory().unwrap();

    // Store mostly aggressive combat (duration > 10s to avoid Efficient classification)
    for i in 0..10 {
        let episode = create_combat_episode(
            &format!("aggressive_{}", i),
            550.0, // High damage
            80.0,  // Accepts damage
            15000, // Longer duration to ensure not classified as Efficient
            "offensive_buff",
            0.85,
        );
        storage.store_memory(&episode.to_memory().unwrap()).unwrap();
    }

    let builder = ProfileBuilder::new();
    let profile = builder.build_profile(&storage).unwrap();

    let primary = ProfileBuilder::get_primary_pattern(&profile);
    assert_eq!(primary, Some(PlaystylePattern::Aggressive));
}

#[test]
fn test_learning_confidence_growth() {
    let mut storage = MemoryStorage::in_memory().unwrap();

    let builder = ProfileBuilder::new();

    // Start with 5 episodes
    for i in 0..5 {
        let episode =
            create_combat_episode(&format!("ep_{}", i), 400.0, 50.0, 10000, "support", 0.8);
        storage.store_memory(&episode.to_memory().unwrap()).unwrap();
    }

    let profile1 = builder.build_profile(&storage).unwrap();
    let confidence1 = profile1.learning_confidence;

    // Add more episodes
    for i in 5..15 {
        let episode =
            create_combat_episode(&format!("ep_{}", i), 400.0, 50.0, 10000, "support", 0.8);
        storage.store_memory(&episode.to_memory().unwrap()).unwrap();
    }

    let profile2 = builder.build_profile(&storage).unwrap();
    let confidence2 = profile2.learning_confidence;

    // Confidence should increase with more episodes
    assert!(confidence2 > confidence1);
}
