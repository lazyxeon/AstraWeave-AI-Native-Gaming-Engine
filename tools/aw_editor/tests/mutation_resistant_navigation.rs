//! Mutation-resistant tests for Navigation Panel system
//!
//! These tests target boundary conditions, comparison operators, and boolean return paths
//! to achieve high mutation testing kill rates (90%+).

use aw_editor_lib::panels::{NavAreaType, NavLinkType, NavigationTab};
use std::collections::HashSet;

// ============================================================================
// NAV AREA TYPE TESTS
// ============================================================================

mod nav_area_type_tests {
    use super::*;

    #[test]
    fn test_all_count() {
        assert_eq!(NavAreaType::all().len(), 9);
    }

    // Test all variant name() uniqueness
    #[test]
    fn test_all_names_unique() {
        let names: Vec<&str> = NavAreaType::all().iter().map(|c| c.name()).collect();
        let unique: HashSet<_> = names.iter().collect();
        assert_eq!(names.len(), unique.len());
    }

    // Test all variant icon() uniqueness
    #[test]
    fn test_all_icons_unique() {
        let icons: Vec<&str> = NavAreaType::all().iter().map(|c| c.icon()).collect();
        let unique: HashSet<_> = icons.iter().collect();
        assert_eq!(icons.len(), unique.len());
    }

    // Test each variant name
    #[test]
    fn test_walkable_name() {
        assert_eq!(NavAreaType::Walkable.name(), "Walkable");
    }

    #[test]
    fn test_road_name() {
        assert_eq!(NavAreaType::Road.name(), "Road");
    }

    #[test]
    fn test_water_name() {
        assert_eq!(NavAreaType::Water.name(), "Water");
    }

    #[test]
    fn test_grass_name() {
        assert_eq!(NavAreaType::Grass.name(), "Grass");
    }

    #[test]
    fn test_mud_name() {
        assert_eq!(NavAreaType::Mud.name(), "Mud");
    }

    #[test]
    fn test_ice_name() {
        assert_eq!(NavAreaType::Ice.name(), "Ice");
    }

    #[test]
    fn test_ladder_name() {
        assert_eq!(NavAreaType::Ladder.name(), "Ladder");
    }

    #[test]
    fn test_jump_name() {
        assert_eq!(NavAreaType::Jump.name(), "Jump");
    }

    #[test]
    fn test_blocked_name() {
        assert_eq!(NavAreaType::Blocked.name(), "Blocked");
    }

    // Test cost() values
    #[test]
    fn test_walkable_cost() {
        assert_eq!(NavAreaType::Walkable.cost(), 1.0);
    }

    #[test]
    fn test_road_cost_is_lowest() {
        assert_eq!(NavAreaType::Road.cost(), 0.5);
        assert!(NavAreaType::Road.cost() < NavAreaType::Walkable.cost());
    }

    #[test]
    fn test_water_cost_is_high() {
        assert_eq!(NavAreaType::Water.cost(), 3.0);
        assert!(NavAreaType::Water.cost() > NavAreaType::Walkable.cost());
    }

    #[test]
    fn test_mud_cost_is_expensive() {
        assert_eq!(NavAreaType::Mud.cost(), 2.5);
        assert!(NavAreaType::Mud.cost() > NavAreaType::Walkable.cost());
    }

    #[test]
    fn test_blocked_cost_is_infinity() {
        assert!(NavAreaType::Blocked.cost().is_infinite());
    }

    #[test]
    fn test_grass_cost() {
        assert_eq!(NavAreaType::Grass.cost(), 1.2);
    }

    #[test]
    fn test_ice_cost() {
        assert_eq!(NavAreaType::Ice.cost(), 1.1);
    }

    #[test]
    fn test_ladder_cost() {
        assert_eq!(NavAreaType::Ladder.cost(), 2.0);
    }

    #[test]
    fn test_jump_cost() {
        assert_eq!(NavAreaType::Jump.cost(), 1.5);
    }

    // Test Display trait
    #[test]
    fn test_display_contains_icon() {
        let display = format!("{}", NavAreaType::Walkable);
        assert!(display.contains("🚶"));
    }

    #[test]
    fn test_display_contains_name() {
        let display = format!("{}", NavAreaType::Road);
        assert!(display.contains("Road"));
    }

    // Test default
    #[test]
    fn test_default_is_walkable() {
        assert_eq!(NavAreaType::default(), NavAreaType::Walkable);
    }
}

// ============================================================================
// NAV LINK TYPE TESTS
// ============================================================================

mod nav_link_type_tests {
    use super::*;

    #[test]
    fn test_all_count() {
        assert_eq!(NavLinkType::all().len(), 5);
    }

    // Test all variant name() uniqueness
    #[test]
    fn test_all_names_unique() {
        let names: Vec<&str> = NavLinkType::all().iter().map(|c| c.name()).collect();
        let unique: HashSet<_> = names.iter().collect();
        assert_eq!(names.len(), unique.len());
    }

    // Test all variant icon() uniqueness
    #[test]
    fn test_all_icons_unique() {
        let icons: Vec<&str> = NavLinkType::all().iter().map(|c| c.icon()).collect();
        let unique: HashSet<_> = icons.iter().collect();
        assert_eq!(icons.len(), unique.len());
    }

    // Test each variant name
    #[test]
    fn test_walk_name() {
        assert_eq!(NavLinkType::Walk.name(), "Walk");
    }

    #[test]
    fn test_jump_name() {
        assert_eq!(NavLinkType::Jump.name(), "Jump");
    }

    #[test]
    fn test_drop_name() {
        assert_eq!(NavLinkType::Drop.name(), "Drop");
    }

    #[test]
    fn test_ladder_name() {
        assert_eq!(NavLinkType::Ladder.name(), "Ladder");
    }

    #[test]
    fn test_teleport_name() {
        assert_eq!(NavLinkType::Teleport.name(), "Teleport");
    }

    // Test Display trait
    #[test]
    fn test_display_contains_icon() {
        let display = format!("{}", NavLinkType::Jump);
        assert!(display.contains("🦘"));
    }

    #[test]
    fn test_display_contains_name() {
        let display = format!("{}", NavLinkType::Teleport);
        assert!(display.contains("Teleport"));
    }

    // Test default
    #[test]
    fn test_default_is_walk() {
        assert_eq!(NavLinkType::default(), NavLinkType::Walk);
    }
}

// ============================================================================
// NAVIGATION TAB TESTS
// ============================================================================

mod navigation_tab_tests {
    use super::*;

    #[test]
    fn test_all_count() {
        assert_eq!(NavigationTab::all().len(), 6);
    }

    // Test all variant name() uniqueness
    #[test]
    fn test_all_names_unique() {
        let names: Vec<&str> = NavigationTab::all().iter().map(|c| c.name()).collect();
        let unique: HashSet<_> = names.iter().collect();
        assert_eq!(names.len(), unique.len());
    }

    // Test all variant icon() uniqueness
    #[test]
    fn test_all_icons_unique() {
        let icons: Vec<&str> = NavigationTab::all().iter().map(|c| c.icon()).collect();
        let unique: HashSet<_> = icons.iter().collect();
        assert_eq!(icons.len(), unique.len());
    }

    // Test each variant name
    #[test]
    fn test_mesh_name() {
        assert_eq!(NavigationTab::Mesh.name(), "Mesh");
    }

    #[test]
    fn test_agents_name() {
        assert_eq!(NavigationTab::Agents.name(), "Agents");
    }

    #[test]
    fn test_obstacles_name() {
        assert_eq!(NavigationTab::Obstacles.name(), "Obstacles");
    }

    #[test]
    fn test_links_name() {
        assert_eq!(NavigationTab::Links.name(), "Links");
    }

    #[test]
    fn test_path_test_name() {
        assert_eq!(NavigationTab::PathTest.name(), "Path Test");
    }

    #[test]
    fn test_settings_name() {
        assert_eq!(NavigationTab::Settings.name(), "Settings");
    }

    // Test Display trait
    #[test]
    fn test_display_contains_icon() {
        let display = format!("{}", NavigationTab::Mesh);
        assert!(display.contains("🗺️"));
    }

    #[test]
    fn test_display_contains_name() {
        let display = format!("{}", NavigationTab::Settings);
        assert!(display.contains("Settings"));
    }

    // Test default
    #[test]
    fn test_default_is_mesh() {
        assert_eq!(NavigationTab::default(), NavigationTab::Mesh);
    }
}

// ============================================================================
// NAV AGENT CONFIG TESTS
// ============================================================================

mod nav_agent_config_tests {
    use aw_editor_lib::panels::NavAgentConfig;

    #[test]
    fn test_default_name() {
        let config = NavAgentConfig::default();
        assert_eq!(config.name, "Default Agent");
    }

    #[test]
    fn test_default_radius() {
        let config = NavAgentConfig::default();
        assert_eq!(config.radius, 0.5);
    }

    #[test]
    fn test_default_height() {
        let config = NavAgentConfig::default();
        assert_eq!(config.height, 2.0);
    }

    #[test]
    fn test_default_max_slope() {
        let config = NavAgentConfig::default();
        assert_eq!(config.max_slope, 45.0);
    }

    #[test]
    fn test_default_step_height() {
        let config = NavAgentConfig::default();
        assert_eq!(config.step_height, 0.5);
    }

    #[test]
    fn test_default_max_jump_distance() {
        let config = NavAgentConfig::default();
        assert_eq!(config.max_jump_distance, 2.0);
    }

    #[test]
    fn test_default_max_fall_distance() {
        let config = NavAgentConfig::default();
        assert_eq!(config.max_fall_distance, 5.0);
    }

    #[test]
    fn test_custom_values() {
        let config = NavAgentConfig {
            name: "Custom Agent".into(),
            radius: 1.0,
            height: 3.0,
            max_slope: 60.0,
            step_height: 1.0,
            max_jump_distance: 5.0,
            max_fall_distance: 10.0,
        };
        assert_eq!(config.name, "Custom Agent");
        assert_eq!(config.radius, 1.0);
        assert_eq!(config.height, 3.0);
        assert_eq!(config.max_slope, 60.0);
        assert_eq!(config.step_height, 1.0);
    }
}

// ============================================================================
// NAV MESH BAKE SETTINGS TESTS
// ============================================================================

mod nav_mesh_bake_settings_tests {
    use aw_editor_lib::panels::NavMeshBakeSettings;

    #[test]
    fn test_default_cell_size() {
        let settings = NavMeshBakeSettings::default();
        assert_eq!(settings.cell_size, 0.3);
    }

    #[test]
    fn test_default_cell_height() {
        let settings = NavMeshBakeSettings::default();
        assert_eq!(settings.cell_height, 0.2);
    }

    #[test]
    fn test_default_agent_height() {
        let settings = NavMeshBakeSettings::default();
        assert_eq!(settings.agent_height, 2.0);
    }

    #[test]
    fn test_default_agent_radius() {
        let settings = NavMeshBakeSettings::default();
        assert_eq!(settings.agent_radius, 0.5);
    }

    #[test]
    fn test_default_agent_max_climb() {
        let settings = NavMeshBakeSettings::default();
        assert_eq!(settings.agent_max_climb, 0.5);
    }

    #[test]
    fn test_default_agent_max_slope() {
        let settings = NavMeshBakeSettings::default();
        assert_eq!(settings.agent_max_slope, 45.0);
    }

    #[test]
    fn test_default_region_min_size() {
        let settings = NavMeshBakeSettings::default();
        assert_eq!(settings.region_min_size, 8);
    }

    #[test]
    fn test_default_region_merge_size() {
        let settings = NavMeshBakeSettings::default();
        assert_eq!(settings.region_merge_size, 20);
    }

    #[test]
    fn test_default_edge_max_len() {
        let settings = NavMeshBakeSettings::default();
        assert_eq!(settings.edge_max_len, 12.0);
    }

    #[test]
    fn test_default_verts_per_poly() {
        let settings = NavMeshBakeSettings::default();
        assert_eq!(settings.verts_per_poly, 6);
    }
}

// ============================================================================
// NAV OBSTACLE TESTS
// ============================================================================

mod nav_obstacle_tests {
    use aw_editor_lib::panels::NavObstacle;

    #[test]
    fn test_default_id() {
        let obstacle = NavObstacle::default();
        assert_eq!(obstacle.id, 0);
    }

    #[test]
    fn test_default_name() {
        let obstacle = NavObstacle::default();
        assert_eq!(obstacle.name, "Obstacle");
    }

    #[test]
    fn test_default_position() {
        let obstacle = NavObstacle::default();
        assert_eq!(obstacle.position, (0.0, 0.0, 0.0));
    }

    #[test]
    fn test_default_size() {
        let obstacle = NavObstacle::default();
        assert_eq!(obstacle.size, (1.0, 1.0, 1.0));
    }

    #[test]
    fn test_default_is_dynamic() {
        let obstacle = NavObstacle::default();
        assert!(!obstacle.is_dynamic);
    }

    #[test]
    fn test_default_carve_when_stationary() {
        let obstacle = NavObstacle::default();
        assert!(obstacle.carve_when_stationary);
    }

    #[test]
    fn test_custom_values() {
        let obstacle = NavObstacle {
            id: 42,
            name: "Custom Obstacle".into(),
            position: (10.0, 5.0, 0.0),
            size: (2.0, 3.0, 4.0),
            is_dynamic: true,
            carve_when_stationary: false,
        };
        assert_eq!(obstacle.id, 42);
        assert_eq!(obstacle.name, "Custom Obstacle");
        assert!(obstacle.is_dynamic);
        assert!(!obstacle.carve_when_stationary);
    }
}
