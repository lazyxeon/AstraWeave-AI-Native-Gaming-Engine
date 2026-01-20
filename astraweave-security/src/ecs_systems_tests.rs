// ECS Systems Integration Tests
// Tests for input_validation_system, telemetry_collection_system, and anomaly_detection_system

#[cfg(test)]
#[allow(clippy::module_inception)]
mod ecs_systems_tests {
    use super::super::*;
    use astraweave_ecs::World;

    // ============================================================================
    // Helper Functions
    // ============================================================================

    fn create_test_world() -> World {
        World::new()
    }

    fn create_anti_cheat_component(
        player_id: &str,
        trust_score: f32,
        anomalies: Vec<String>,
    ) -> CAntiCheat {
        CAntiCheat {
            player_id: player_id.to_string(),
            trust_score,
            last_validation: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            anomaly_flags: anomalies,
        }
    }

    fn create_telemetry_data() -> TelemetryData {
        TelemetryData {
            events: Vec::new(),
            anomaly_count: 0,
            session_start: std::time::Instant::now(),
        }
    }

    // ============================================================================
    // Suite 1: input_validation_system Tests (5 tests)
    // ============================================================================

    #[test]
    fn test_input_validation_clean_player() {
        let mut world = create_test_world();
        world.insert_resource(create_telemetry_data());

        // Create clean player (no anomalies, high trust)
        let entity = world.spawn();
        let anti_cheat = create_anti_cheat_component("player1", 0.95, vec![]);
        world.insert(entity, anti_cheat);

        // Run input validation system
        input_validation_system(&mut world);

        // Trust score should remain high (smoothed with 0.9 weight on old value)
        let anti_cheat = world.get::<CAntiCheat>(entity).unwrap();
        assert!(
            anti_cheat.trust_score >= 0.90,
            "Clean player should maintain high trust score, got {}",
            anti_cheat.trust_score
        );
        assert!(
            anti_cheat.anomaly_flags.is_empty(),
            "Clean player should have no anomaly flags"
        );
    }

    #[test]
    fn test_input_validation_rapid_input_detection() {
        let mut world = create_test_world();
        world.insert_resource(create_telemetry_data());

        // Create player with rapid input anomaly
        let entity = world.spawn();
        let anti_cheat =
            create_anti_cheat_component("player2", 0.90, vec!["rapid_input".to_string()]);
        world.insert(entity, anti_cheat);

        // Run input validation system
        input_validation_system(&mut world);

        // Trust score should decrease (0.8 penalty applied)
        let anti_cheat = world.get::<CAntiCheat>(entity).unwrap();
        assert!(
            anti_cheat.trust_score < 0.90,
            "Rapid input should reduce trust score, got {}",
            anti_cheat.trust_score
        );
        assert!(
            anti_cheat
                .anomaly_flags
                .contains(&"rapid_input".to_string()),
            "Rapid input anomaly should be flagged"
        );
    }

    #[test]
    fn test_input_validation_multiple_anomalies() {
        let mut world = create_test_world();
        world.insert_resource(create_telemetry_data());

        // Create player with multiple anomalies
        let entity = world.spawn();
        let anomalies = vec![
            "rapid_input".to_string(),
            "impossible_movement".to_string(),
            "memory_tamper".to_string(),
        ];
        let anti_cheat = create_anti_cheat_component("player3", 0.85, anomalies.clone());
        world.insert(entity, anti_cheat);

        // Run input validation system
        input_validation_system(&mut world);

        // Trust score reduced by multiplicative penalties: 0.85 * 0.8 * 0.5 * 0.3 = 0.102
        // Then smoothed: (0.85 * 0.9) + (0.102 * 0.1) ≈ 0.775
        let anti_cheat = world.get::<CAntiCheat>(entity).unwrap();
        assert!(
            anti_cheat.trust_score < 0.85 && anti_cheat.trust_score > 0.70,
            "Multiple anomalies should reduce trust score with smoothing, got {}",
            anti_cheat.trust_score
        );
        assert_eq!(
            anti_cheat.anomaly_flags.len(),
            6,
            "Anomalies should accumulate (3 original + 3 validation)"
        );
    }

    #[test]
    fn test_input_validation_multiple_players() {
        let mut world = create_test_world();
        world.insert_resource(create_telemetry_data());

        // Create 3 players with different profiles
        let entity1 = world.spawn();
        world.insert(
            entity1,
            create_anti_cheat_component("player1", 0.95, vec![]),
        );

        let entity2 = world.spawn();
        world.insert(
            entity2,
            create_anti_cheat_component("player2", 0.80, vec!["rapid_input".to_string()]),
        );

        let entity3 = world.spawn();
        world.insert(
            entity3,
            create_anti_cheat_component("player3", 0.60, vec![]),
        );

        // Run input validation system
        input_validation_system(&mut world);

        // All players should have updated timestamps
        let anti_cheat1 = world.get::<CAntiCheat>(entity1).unwrap();
        let anti_cheat2 = world.get::<CAntiCheat>(entity2).unwrap();
        let anti_cheat3 = world.get::<CAntiCheat>(entity3).unwrap();

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        assert!(
            now - anti_cheat1.last_validation < 2,
            "Player 1 should have recent validation timestamp"
        );
        assert!(
            now - anti_cheat2.last_validation < 2,
            "Player 2 should have recent validation timestamp"
        );
        assert!(
            now - anti_cheat3.last_validation < 2,
            "Player 3 should have recent validation timestamp"
        );
    }

    #[test]
    fn test_input_validation_trust_score_smoothing() {
        let mut world = create_test_world();
        world.insert_resource(create_telemetry_data());

        // Create player with high initial trust
        let entity = world.spawn();
        let anti_cheat = create_anti_cheat_component("player4", 1.0, vec![]);
        world.insert(entity, anti_cheat);

        // Run validation multiple times (clean player)
        for _ in 0..5 {
            input_validation_system(&mut world);
        }

        // Trust score should remain very high due to smoothing (0.9 weight on old value)
        let anti_cheat = world.get::<CAntiCheat>(entity).unwrap();
        assert!(
            anti_cheat.trust_score >= 0.99,
            "Clean player should maintain trust score near 1.0 with smoothing, got {}",
            anti_cheat.trust_score
        );
    }

    // ============================================================================
    // Suite 2: telemetry_collection_system Tests (5 tests)
    // ============================================================================

    #[test]
    fn test_telemetry_collection_event_limit() {
        let mut world = create_test_world();
        let mut telemetry = create_telemetry_data();

        // Add 1500 events (exceeds 1000 limit)
        for i in 0..1500 {
            telemetry.events.push(TelemetryEvent {
                timestamp: i as u64,
                event_type: format!("event_{}", i),
                severity: TelemetrySeverity::Info,
                data: serde_json::json!({"index": i}),
            });
        }

        world.insert_resource(telemetry);

        // Run telemetry collection system
        telemetry_collection_system(&mut world);

        // Should keep only last 1000 events
        let telemetry = world.get_resource::<TelemetryData>().unwrap();
        assert_eq!(
            telemetry.events.len(),
            1000,
            "Telemetry should keep only last 1000 events"
        );

        // Verify we kept the most recent events (500-1499)
        assert_eq!(
            telemetry.events[0].data["index"].as_u64().unwrap(),
            500,
            "Should keep events starting from index 500"
        );
        assert_eq!(
            telemetry.events[999].data["index"].as_u64().unwrap(),
            1499,
            "Should keep events up to index 1499"
        );
    }

    #[test]
    fn test_telemetry_collection_no_events() {
        let mut world = create_test_world();
        world.insert_resource(create_telemetry_data());

        // Run telemetry collection system with no events
        telemetry_collection_system(&mut world);

        // Should not crash or error
        let telemetry = world.get_resource::<TelemetryData>().unwrap();
        assert_eq!(telemetry.events.len(), 0, "Should have no events");
        assert_eq!(telemetry.anomaly_count, 0, "Should have no anomalies");
    }

    #[test]
    fn test_telemetry_collection_multiple_event_types() {
        let mut world = create_test_world();
        let mut telemetry = create_telemetry_data();

        // Add different event types
        telemetry.events.push(TelemetryEvent {
            timestamp: 100,
            event_type: "input_anomaly".to_string(),
            severity: TelemetrySeverity::Warning,
            data: serde_json::json!({"player_id": "player1"}),
        });

        telemetry.events.push(TelemetryEvent {
            timestamp: 101,
            event_type: "systemic_anomaly".to_string(),
            severity: TelemetrySeverity::Critical,
            data: serde_json::json!({"total_players": 10}),
        });

        telemetry.events.push(TelemetryEvent {
            timestamp: 102,
            event_type: "login".to_string(),
            severity: TelemetrySeverity::Info,
            data: serde_json::json!({"player_id": "player2"}),
        });

        world.insert_resource(telemetry);

        // Run telemetry collection system
        telemetry_collection_system(&mut world);

        // All events should be preserved
        let telemetry = world.get_resource::<TelemetryData>().unwrap();
        assert_eq!(telemetry.events.len(), 3, "All events should be preserved");
        assert_eq!(
            telemetry.events[0].event_type, "input_anomaly",
            "First event should be input_anomaly"
        );
        assert_eq!(
            telemetry.events[1].event_type, "systemic_anomaly",
            "Second event should be systemic_anomaly"
        );
        assert_eq!(
            telemetry.events[2].event_type, "login",
            "Third event should be login"
        );
    }

    #[test]
    fn test_telemetry_collection_session_duration_tracking() {
        let mut world = create_test_world();
        let telemetry = create_telemetry_data();
        world.insert_resource(telemetry);

        // Run telemetry collection system
        telemetry_collection_system(&mut world);

        // Session should have started (Instant is not zero)
        let telemetry = world.get_resource::<TelemetryData>().unwrap();
        let duration = telemetry.session_start.elapsed().as_millis();
        assert!(
            duration < 100,
            "Session should have just started (< 100ms elapsed)"
        );
    }

    #[test]
    fn test_telemetry_collection_anomaly_count_preserved() {
        let mut world = create_test_world();
        let mut telemetry = create_telemetry_data();
        telemetry.anomaly_count = 42;
        world.insert_resource(telemetry);

        // Run telemetry collection system
        telemetry_collection_system(&mut world);

        // Anomaly count should be preserved (not modified by this system)
        let telemetry = world.get_resource::<TelemetryData>().unwrap();
        assert_eq!(
            telemetry.anomaly_count, 42,
            "Anomaly count should be preserved"
        );
    }

    // ============================================================================
    // Suite 3: anomaly_detection_system Tests (5 tests)
    // ============================================================================

    #[test]
    fn test_anomaly_detection_count_anomalies() {
        let mut world = create_test_world();
        world.insert_resource(create_telemetry_data());

        // Create players with different anomaly counts
        let entity1 = world.spawn();
        world.insert(
            entity1,
            create_anti_cheat_component("player1", 0.80, vec!["rapid_input".to_string()]),
        );

        let entity2 = world.spawn();
        world.insert(
            entity2,
            create_anti_cheat_component(
                "player2",
                0.60,
                vec![
                    "rapid_input".to_string(),
                    "impossible_movement".to_string(),
                    "memory_tamper".to_string(),
                ],
            ),
        );

        let entity3 = world.spawn();
        world.insert(
            entity3,
            create_anti_cheat_component("player3", 0.95, vec![]),
        );

        // Run anomaly detection system
        anomaly_detection_system(&mut world);

        // Should count all anomalies (1 + 3 + 0 = 4)
        let telemetry = world.get_resource::<TelemetryData>().unwrap();
        assert_eq!(
            telemetry.anomaly_count, 4,
            "Should count all anomalies across all players"
        );
    }

    #[test]
    fn test_anomaly_detection_low_trust_players() {
        let mut world = create_test_world();
        world.insert_resource(create_telemetry_data());

        // Create 4 players: 2 low trust (< 0.5), 2 high trust
        let entity1 = world.spawn();
        world.insert(
            entity1,
            create_anti_cheat_component("player1", 0.30, vec![]),
        );

        let entity2 = world.spawn();
        world.insert(
            entity2,
            create_anti_cheat_component("player2", 0.40, vec![]),
        );

        let entity3 = world.spawn();
        world.insert(
            entity3,
            create_anti_cheat_component("player3", 0.80, vec![]),
        );

        let entity4 = world.spawn();
        world.insert(
            entity4,
            create_anti_cheat_component("player4", 0.90, vec![]),
        );

        // Run anomaly detection system
        anomaly_detection_system(&mut world);

        // Should NOT trigger systemic anomaly (2/4 = 50%, not > 50%)
        let telemetry = world.get_resource::<TelemetryData>().unwrap();
        let systemic_events: Vec<_> = telemetry
            .events
            .iter()
            .filter(|e| e.event_type == "systemic_anomaly")
            .collect();

        assert_eq!(
            systemic_events.len(),
            0,
            "Should not trigger systemic anomaly at exactly 50%"
        );
    }

    #[test]
    fn test_anomaly_detection_systemic_anomaly_trigger() {
        let mut world = create_test_world();
        world.insert_resource(create_telemetry_data());

        // Create 4 players: 3 low trust (< 0.5), 1 high trust (> 50% low trust)
        let entity1 = world.spawn();
        world.insert(
            entity1,
            create_anti_cheat_component("player1", 0.30, vec![]),
        );

        let entity2 = world.spawn();
        world.insert(
            entity2,
            create_anti_cheat_component("player2", 0.40, vec![]),
        );

        let entity3 = world.spawn();
        world.insert(
            entity3,
            create_anti_cheat_component("player3", 0.20, vec![]),
        );

        let entity4 = world.spawn();
        world.insert(
            entity4,
            create_anti_cheat_component("player4", 0.90, vec![]),
        );

        // Run anomaly detection system
        anomaly_detection_system(&mut world);

        // Should trigger systemic anomaly (3/4 = 75% > 50%)
        let telemetry = world.get_resource::<TelemetryData>().unwrap();
        let systemic_events: Vec<_> = telemetry
            .events
            .iter()
            .filter(|e| e.event_type == "systemic_anomaly")
            .collect();

        assert_eq!(
            systemic_events.len(),
            1,
            "Should trigger systemic anomaly when > 50% low trust"
        );
        assert_eq!(
            systemic_events[0].severity,
            TelemetrySeverity::Critical,
            "Systemic anomaly should be Critical severity"
        );

        // Verify event data
        let data = &systemic_events[0].data;
        assert_eq!(
            data["low_trust_players"].as_u64().unwrap(),
            3,
            "Should report 3 low trust players"
        );
        assert_eq!(
            data["total_players"].as_u64().unwrap(),
            4,
            "Should report 4 total players"
        );
    }

    #[test]
    fn test_anomaly_detection_no_players() {
        let mut world = create_test_world();
        world.insert_resource(create_telemetry_data());

        // Run anomaly detection system with no players
        anomaly_detection_system(&mut world);

        // Should not crash or error
        let telemetry = world.get_resource::<TelemetryData>().unwrap();
        assert_eq!(telemetry.anomaly_count, 0, "Should have no anomalies");
        assert_eq!(telemetry.events.len(), 0, "Should have no events");
    }

    #[test]
    fn test_anomaly_detection_edge_case_boundary() {
        let mut world = create_test_world();
        world.insert_resource(create_telemetry_data());

        // Create exactly 0.5 threshold: 1 player with trust_score = 0.5
        let entity = world.spawn();
        world.insert(entity, create_anti_cheat_component("player1", 0.5, vec![]));

        // Run anomaly detection system
        anomaly_detection_system(&mut world);

        // Player with trust_score = 0.5 should NOT be counted as low trust (< 0.5)
        let telemetry = world.get_resource::<TelemetryData>().unwrap();
        let systemic_events: Vec<_> = telemetry
            .events
            .iter()
            .filter(|e| e.event_type == "systemic_anomaly")
            .collect();

        assert_eq!(
            systemic_events.len(),
            0,
            "Player with trust_score = 0.5 should not trigger systemic anomaly"
        );
    }
}
