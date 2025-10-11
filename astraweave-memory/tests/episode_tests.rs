//! Integration tests for episode recording and memory system.

use astraweave_memory::{
    ActionResult, CompanionResponse, EpisodeCategory, EpisodeOutcome, EpisodeRecorder,
    GameEpisode as Episode, MemoryType, Observation, PlayerAction,
};

#[test]
fn test_end_to_end_episode_workflow() {
    // Create recorder
    let mut recorder = EpisodeRecorder::new();

    // Start combat episode
    let episode_id = recorder.start_episode("companion_alpha".to_string(), EpisodeCategory::Combat);
    assert!(!episode_id.is_empty());

    // Record series of combat observations
    for i in 0..5 {
        let obs = Observation::new(
            i * 1000,
            Some(PlayerAction {
                action_type: "melee_strike".to_string(),
                target: Some("orc_warrior".to_string()),
                parameters: serde_json::json!({
                    "damage": 45 + i * 5,
                    "crit": i % 2 == 0
                }),
            }),
            Some(CompanionResponse {
                action_type: "defensive_stance".to_string(),
                result: ActionResult::Success,
                effectiveness: 0.85,
            }),
            serde_json::json!({
                "player_health": 1.0 - (i as f64 * 0.1),
                "companion_health": 0.9,
                "enemy_count": 3 - (i / 2),
                "location": "Darkwood Forest"
            }),
        );

        recorder.record_observation("companion_alpha", obs);
    }

    // Tag the episode
    recorder.tag_active_episode("companion_alpha", "forest_encounter".to_string());
    recorder.tag_active_episode("companion_alpha", "aggressive_playstyle".to_string());

    // Complete with outcome
    let outcome = EpisodeOutcome {
        success_rating: 0.95,
        player_satisfaction: 0.9,
        companion_effectiveness: 0.85,
        duration_ms: 45_000,
        damage_dealt: 275.0,
        damage_taken: 50.0,
        resources_used: 30.0,
        failure_count: 0,
    };

    let completed_episode = recorder
        .end_episode("companion_alpha", outcome)
        .expect("Should return completed episode");

    // Verify episode state
    assert!(completed_episode.is_complete());
    assert_eq!(completed_episode.observations.len(), 5);
    assert_eq!(completed_episode.tags.len(), 2);
    assert_eq!(completed_episode.category, EpisodeCategory::Combat);

    // Verify recorder state
    assert_eq!(recorder.active_count(), 0);
    assert!(!recorder.has_active_episode("companion_alpha"));
}

#[test]
fn test_episode_to_memory_conversion() {
    let mut episode = Episode::new("combat_001".to_string(), EpisodeCategory::Combat);
    episode.add_tag("boss_fight".to_string());
    episode.add_tag("cathedral".to_string());

    // Add observations with varying player health
    for i in 0..10 {
        let obs = Observation::new(
            i * 2000,
            Some(PlayerAction {
                action_type: if i < 5 {
                    "ranged_attack".to_string()
                } else {
                    "melee_attack".to_string()
                },
                target: Some("corrupted_bishop".to_string()),
                parameters: serde_json::json!({}),
            }),
            Some(CompanionResponse {
                action_type: "healing_aura".to_string(),
                result: ActionResult::Success,
                effectiveness: 0.9,
            }),
            serde_json::json!({
                "player_health": 1.0 - (i as f64 * 0.05),
                "location": "Cathedral of Light"
            }),
        );

        episode.add_observation(obs);
    }

    let outcome = EpisodeOutcome {
        success_rating: 0.85,
        player_satisfaction: 0.8,
        companion_effectiveness: 0.9,
        duration_ms: 180_000,
        damage_dealt: 1200.0,
        damage_taken: 300.0,
        resources_used: 500.0,
        failure_count: 1,
    };

    episode.complete(outcome);

    // Convert to memory
    let memory = episode.to_memory().expect("Conversion should succeed");

    // Verify memory properties
    assert_eq!(memory.id, "combat_001");
    assert_eq!(memory.memory_type, MemoryType::Episodic);
    assert!(memory.content.text.contains("Combat episode"));
    assert_eq!(memory.metadata.tags.len(), 2);
    assert!(memory.metadata.tags.contains(&"boss_fight".to_string()));

    // Verify importance scoring
    let quality = episode
        .outcome
        .as_ref()
        .expect("Should have outcome")
        .quality_score();
    assert!((memory.metadata.importance - quality).abs() < 0.01);

    // Verify emotional context
    let emotional = memory
        .content
        .emotional_context
        .expect("Should have emotional context");
    assert_eq!(emotional.primary_emotion, "triumphant"); // 95% success = triumphant (>80%)
    assert!((emotional.intensity - 0.8).abs() < 0.01); // player_satisfaction is 0.8
    assert!(emotional.valence > 0.0); // Positive outcome (95% > 50%)

    // Verify spatial-temporal context
    assert!(memory.content.context.duration.is_some());
    // Note: Duration comes from outcome.duration_ms since episode completes instantly in tests
    let duration = memory.content.context.duration.expect("Should have duration");
    assert!(duration <= 180_000); // May be actual elapsed time (small) or outcome duration
    assert_eq!(memory.content.context.participants.len(), 2); // player + companion

    // Verify episode data is preserved in JSON
    let episode_data: Episode = serde_json::from_value(memory.content.data.clone())
        .expect("Should deserialize episode from memory data");
    assert_eq!(episode_data.observations.len(), 10);
    assert!(episode_data.is_complete());
}

#[test]
fn test_multiple_companion_episodes() {
    let mut recorder = EpisodeRecorder::new();

    // Start episodes for 3 different companions
    let id1 = recorder.start_episode("warrior_companion".to_string(), EpisodeCategory::Combat);
    let id2 = recorder.start_episode("rogue_companion".to_string(), EpisodeCategory::Exploration);
    let id3 = recorder.start_episode("mage_companion".to_string(), EpisodeCategory::Dialogue);

    assert_eq!(recorder.active_count(), 3);
    assert_ne!(id1, id2);
    assert_ne!(id2, id3);

    // Record observations for each
    for i in 0..3 {
        recorder.record_observation(
            "warrior_companion",
            Observation::new(
                i * 1000,
                None,
                None,
                serde_json::json!({"companion_type": "warrior"}),
            ),
        );

        recorder.record_observation(
            "rogue_companion",
            Observation::new(
                i * 1000,
                None,
                None,
                serde_json::json!({"companion_type": "rogue"}),
            ),
        );

        recorder.record_observation(
            "mage_companion",
            Observation::new(
                i * 1000,
                None,
                None,
                serde_json::json!({"companion_type": "mage"}),
            ),
        );
    }

    // Verify each has observations
    assert_eq!(
        recorder
            .get_active_episode("warrior_companion")
            .expect("Should exist")
            .observations
            .len(),
        3
    );
    assert_eq!(
        recorder
            .get_active_episode("rogue_companion")
            .expect("Should exist")
            .observations
            .len(),
        3
    );
    assert_eq!(
        recorder
            .get_active_episode("mage_companion")
            .expect("Should exist")
            .observations
            .len(),
        3
    );

    // Complete warrior episode
    let outcome = EpisodeOutcome {
        success_rating: 0.9,
        player_satisfaction: 0.85,
        companion_effectiveness: 0.8,
        duration_ms: 30_000,
        damage_dealt: 200.0,
        damage_taken: 50.0,
        resources_used: 40.0,
        failure_count: 0,
    };

    let warrior_episode = recorder
        .end_episode("warrior_companion", outcome)
        .expect("Should return episode");

    assert_eq!(warrior_episode.category, EpisodeCategory::Combat);
    assert_eq!(recorder.active_count(), 2);

    // Abort rogue episode
    let rogue_episode = recorder
        .abort_episode("rogue_companion")
        .expect("Should return episode");

    assert!(!rogue_episode.is_complete());
    assert_eq!(recorder.active_count(), 1);

    // Verify only mage remains
    let active_ids = recorder.active_companion_ids();
    assert_eq!(active_ids.len(), 1);
    assert!(active_ids.contains(&"mage_companion".to_string()));
}

#[test]
fn test_episode_analysis_helpers() {
    let mut episode = Episode::new("analysis_test".to_string(), EpisodeCategory::Combat);

    // Add varied actions
    let action_types = vec![
        "melee_attack",
        "melee_attack",
        "melee_attack",
        "dodge",
        "ranged_attack",
        "use_potion",
        "melee_attack",
    ];

    for (i, action_type) in action_types.iter().enumerate() {
        let obs = Observation::new(
            i as u64 * 1000,
            Some(PlayerAction {
                action_type: action_type.to_string(),
                target: Some("enemy".to_string()),
                parameters: serde_json::json!({}),
            }),
            None,
            serde_json::json!({
                "player_health": 1.0 - (i as f64 * 0.1)
            }),
        );

        episode.add_observation(obs);
    }

    // Test action counting
    assert_eq!(episode.count_actions("melee"), 4);
    assert_eq!(episode.count_actions("ranged"), 1);
    assert_eq!(episode.count_actions("dodge"), 1);
    assert_eq!(episode.count_actions("potion"), 1);

    // Test action diversity
    assert_eq!(episode.action_diversity(), 4); // melee, dodge, ranged, potion

    // Test average health
    let avg_health = episode.average_player_health().expect("Should have average");
    // Expected: (1.0 + 0.9 + 0.8 + 0.7 + 0.6 + 0.5 + 0.4) / 7 = 0.7
    assert!((avg_health - 0.7).abs() < 0.01);
}

#[test]
fn test_outcome_quality_scoring() {
    // Test high quality outcome
    let high_quality = EpisodeOutcome {
        success_rating: 1.0,
        player_satisfaction: 1.0,
        companion_effectiveness: 1.0,
        duration_ms: 30_000,
        damage_dealt: 500.0,
        damage_taken: 10.0,
        resources_used: 50.0,
        failure_count: 0,
    };

    let quality = high_quality.quality_score();
    assert!(quality > 0.95, "Expected very high quality, got {}", quality);

    // Test low quality outcome
    let low_quality = EpisodeOutcome {
        success_rating: 0.2,
        player_satisfaction: 0.3,
        companion_effectiveness: 0.1,
        duration_ms: 120_000,
        damage_dealt: 50.0,
        damage_taken: 300.0,
        resources_used: 200.0,
        failure_count: 5,
    };

    let quality = low_quality.quality_score();
    assert!(quality < 0.3, "Expected low quality, got {}", quality);

    // Test mixed outcome
    let mixed = EpisodeOutcome {
        success_rating: 0.6,
        player_satisfaction: 0.5,
        companion_effectiveness: 0.7,
        duration_ms: 60_000,
        damage_dealt: 200.0,
        damage_taken: 100.0,
        resources_used: 150.0,
        failure_count: 1,
    };

    let quality = mixed.quality_score();
    assert!(
        quality > 0.4 && quality < 0.7,
        "Expected medium quality, got {}",
        quality
    );
}

#[test]
fn test_graceful_shutdown_scenario() {
    let mut recorder = EpisodeRecorder::new();

    // Start multiple episodes
    recorder.start_episode("companion_1".to_string(), EpisodeCategory::Combat);
    recorder.start_episode("companion_2".to_string(), EpisodeCategory::Exploration);
    recorder.start_episode("companion_3".to_string(), EpisodeCategory::Dialogue);

    // Add observations
    for _ in 0..5 {
        recorder.record_observation(
            "companion_1",
            Observation::new(1000, None, None, serde_json::json!({})),
        );
    }

    assert_eq!(recorder.active_count(), 3);

    // Simulate graceful shutdown
    let completed = recorder.complete_all_episodes();

    assert_eq!(completed.len(), 3);
    assert!(completed.iter().all(|ep| ep.is_complete()));
    assert_eq!(recorder.active_count(), 0);

    // Verify default outcomes were applied
    for episode in completed {
        let outcome = episode.outcome.expect("Should have outcome");
        assert_eq!(outcome.success_rating, 0.5); // Neutral
        assert_eq!(outcome.player_satisfaction, 0.5);
    }
}

#[test]
fn test_episode_emotional_context_mapping() {
    // Test triumphant emotion (>80% success)
    let mut ep_triumphant = Episode::new("triumph".to_string(), EpisodeCategory::Combat);
    ep_triumphant.complete(EpisodeOutcome {
        success_rating: 0.95,
        player_satisfaction: 0.9,
        companion_effectiveness: 0.85,
        duration_ms: 30_000,
        damage_dealt: 500.0,
        damage_taken: 20.0,
        resources_used: 50.0,
        failure_count: 0,
    });

    let mem = ep_triumphant.to_memory().expect("Should convert");
    let emotion = mem.content.emotional_context.expect("Should have emotion");
    assert_eq!(emotion.primary_emotion, "triumphant");

    // Test frustrated emotion (20-40% success)
    let mut ep_frustrated = Episode::new("frustrate".to_string(), EpisodeCategory::Combat);
    ep_frustrated.complete(EpisodeOutcome {
        success_rating: 0.3,
        player_satisfaction: 0.2,
        companion_effectiveness: 0.4,
        duration_ms: 90_000,
        damage_dealt: 100.0,
        damage_taken: 200.0,
        resources_used: 150.0,
        failure_count: 3,
    });

    let mem = ep_frustrated.to_memory().expect("Should convert");
    let emotion = mem.content.emotional_context.expect("Should have emotion");
    assert_eq!(emotion.primary_emotion, "frustrated");

    // Test satisfied emotion (60-80% success)
    let mut ep_satisfied = Episode::new("satisfy".to_string(), EpisodeCategory::Combat);
    ep_satisfied.complete(EpisodeOutcome {
        success_rating: 0.7,
        player_satisfaction: 0.75,
        companion_effectiveness: 0.65,
        duration_ms: 45_000,
        damage_dealt: 300.0,
        damage_taken: 80.0,
        resources_used: 100.0,
        failure_count: 1,
    });

    let mem = ep_satisfied.to_memory().expect("Should convert");
    let emotion = mem.content.emotional_context.expect("Should have emotion");
    assert_eq!(emotion.primary_emotion, "satisfied");
}

#[test]
fn test_action_result_multipliers() {
    assert_eq!(ActionResult::Success.success_multiplier(), 1.0);
    assert_eq!(ActionResult::Partial.success_multiplier(), 0.5);
    assert_eq!(ActionResult::Interrupted.success_multiplier(), 0.25);
    assert_eq!(ActionResult::Failure.success_multiplier(), 0.0);
}

#[test]
fn test_episode_category_display() {
    assert_eq!(format!("{}", EpisodeCategory::Combat), "Combat");
    assert_eq!(format!("{}", EpisodeCategory::Dialogue), "Dialogue");
    assert_eq!(format!("{}", EpisodeCategory::Exploration), "Exploration");
    assert_eq!(format!("{}", EpisodeCategory::Puzzle), "Puzzle");
    assert_eq!(format!("{}", EpisodeCategory::Quest), "Quest");
    assert_eq!(format!("{}", EpisodeCategory::Social), "Social");
}
