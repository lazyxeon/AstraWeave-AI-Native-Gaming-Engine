//! Wave 2 mutation remediation tests — Navigation + Networking panels
//! Covers: NavAreaType, NavLinkType, NavigationTab, NavMeshBakeSettings, NavDebugOptions,
//!         NavAgentConfig, NavObstacle, NetworkRole, ConnectionState, InterestPolicy,
//!         CompressionLevel, ClientInfo, NetworkStats

use aw_editor_lib::panels::navigation_panel::{
    NavAgentConfig, NavAreaType, NavDebugOptions, NavLink, NavLinkType, NavMeshBakeSettings,
    NavObstacle, NavRegion, NavigationTab, PathTestResult,
};
use aw_editor_lib::panels::networking_panel::{
    ClientInfo, CompressionLevel, ConnectionState, InterestPolicy, NetworkRole, NetworkStats,
};

// ═══════════════════════════════════════════════════════════════════════════════════
// NAV AREA TYPE
// ═══════════════════════════════════════════════════════════════════════════════════

#[test]
fn nav_area_type_all_count() {
    assert_eq!(NavAreaType::all().len(), 9);
}

#[test]
fn nav_area_type_all_contains_all_variants() {
    let all = NavAreaType::all();
    assert!(all.contains(&NavAreaType::Walkable));
    assert!(all.contains(&NavAreaType::Road));
    assert!(all.contains(&NavAreaType::Grass));
    assert!(all.contains(&NavAreaType::Water));
    assert!(all.contains(&NavAreaType::Mud));
    assert!(all.contains(&NavAreaType::Ice));
    assert!(all.contains(&NavAreaType::Ladder));
    assert!(all.contains(&NavAreaType::Jump));
    assert!(all.contains(&NavAreaType::Blocked));
}

#[test]
fn nav_area_type_names() {
    assert_eq!(NavAreaType::Walkable.name(), "Walkable");
    assert_eq!(NavAreaType::Road.name(), "Road");
    assert_eq!(NavAreaType::Grass.name(), "Grass");
    assert_eq!(NavAreaType::Water.name(), "Water");
    assert_eq!(NavAreaType::Mud.name(), "Mud");
    assert_eq!(NavAreaType::Ice.name(), "Ice");
    assert_eq!(NavAreaType::Ladder.name(), "Ladder");
    assert_eq!(NavAreaType::Jump.name(), "Jump");
    assert_eq!(NavAreaType::Blocked.name(), "Blocked");
}

#[test]
fn nav_area_type_icons_non_empty() {
    for variant in NavAreaType::all() {
        assert!(!variant.icon().is_empty(), "{:?} icon empty", variant);
    }
}

#[test]
fn nav_area_type_icons_unique() {
    let icons: Vec<_> = NavAreaType::all().iter().map(|v| v.icon()).collect();
    for (i, a) in icons.iter().enumerate() {
        for (j, b) in icons.iter().enumerate() {
            if i != j {
                assert_ne!(a, b, "Duplicate icon at {} and {}", i, j);
            }
        }
    }
}

#[test]
fn nav_area_type_cost_walkable() {
    let cost = NavAreaType::Walkable.cost();
    assert!((cost - 1.0).abs() < f32::EPSILON);
}

#[test]
fn nav_area_type_cost_road() {
    let cost = NavAreaType::Road.cost();
    assert!((cost - 0.5).abs() < f32::EPSILON);
}

#[test]
fn nav_area_type_cost_grass() {
    let cost = NavAreaType::Grass.cost();
    assert!((cost - 1.2).abs() < f32::EPSILON);
}

#[test]
fn nav_area_type_cost_water() {
    let cost = NavAreaType::Water.cost();
    assert!((cost - 3.0).abs() < f32::EPSILON);
}

#[test]
fn nav_area_type_cost_mud() {
    let cost = NavAreaType::Mud.cost();
    assert!((cost - 2.5).abs() < f32::EPSILON);
}

#[test]
fn nav_area_type_cost_ice() {
    let cost = NavAreaType::Ice.cost();
    assert!((cost - 1.1).abs() < f32::EPSILON);
}

#[test]
fn nav_area_type_cost_ladder() {
    let cost = NavAreaType::Ladder.cost();
    assert!((cost - 2.0).abs() < f32::EPSILON);
}

#[test]
fn nav_area_type_cost_jump() {
    let cost = NavAreaType::Jump.cost();
    assert!((cost - 1.5).abs() < f32::EPSILON);
}

#[test]
fn nav_area_type_cost_blocked_is_infinity() {
    assert!(NavAreaType::Blocked.cost().is_infinite());
}

#[test]
fn nav_area_type_display_format() {
    for variant in NavAreaType::all() {
        let s = format!("{}", variant);
        assert!(s.contains(variant.name()), "Display missing name for {:?}", variant);
    }
}

#[test]
fn nav_area_type_color_distinct() {
    let colors: Vec<_> = NavAreaType::all().iter().map(|v| v.color()).collect();
    // At least some colors should differ
    let unique_count = {
        let mut c = colors.clone();
        c.dedup();
        c.len()
    };
    assert!(unique_count > 1);
}

// ═══════════════════════════════════════════════════════════════════════════════════
// NAV LINK TYPE
// ═══════════════════════════════════════════════════════════════════════════════════

#[test]
fn nav_link_type_all_count() {
    assert_eq!(NavLinkType::all().len(), 5);
}

#[test]
fn nav_link_type_names() {
    assert_eq!(NavLinkType::Walk.name(), "Walk");
    assert_eq!(NavLinkType::Jump.name(), "Jump");
    assert_eq!(NavLinkType::Drop.name(), "Drop");
    assert_eq!(NavLinkType::Ladder.name(), "Ladder");
    assert_eq!(NavLinkType::Teleport.name(), "Teleport");
}

#[test]
fn nav_link_type_icons_non_empty() {
    for variant in NavLinkType::all() {
        assert!(!variant.icon().is_empty(), "{:?} icon empty", variant);
    }
}

#[test]
fn nav_link_type_display() {
    for variant in NavLinkType::all() {
        let s = format!("{}", variant);
        assert!(!s.is_empty());
    }
}

// ═══════════════════════════════════════════════════════════════════════════════════
// NAVIGATION TAB
// ═══════════════════════════════════════════════════════════════════════════════════

#[test]
fn navigation_tab_all_count() {
    assert_eq!(NavigationTab::all().len(), 6);
}

#[test]
fn navigation_tab_names() {
    assert_eq!(NavigationTab::Mesh.name(), "Mesh");
    assert_eq!(NavigationTab::Agents.name(), "Agents");
    assert_eq!(NavigationTab::Obstacles.name(), "Obstacles");
    assert_eq!(NavigationTab::Links.name(), "Links");
    assert_eq!(NavigationTab::PathTest.name(), "Path Test");
    assert_eq!(NavigationTab::Settings.name(), "Settings");
}

#[test]
fn navigation_tab_icons_non_empty() {
    for tab in NavigationTab::all() {
        assert!(!tab.icon().is_empty(), "{:?} icon empty", tab);
    }
}

#[test]
fn navigation_tab_display() {
    for tab in NavigationTab::all() {
        let s = format!("{}", tab);
        assert!(s.contains(tab.name()));
    }
}

#[test]
fn navigation_tab_default_is_mesh() {
    assert_eq!(NavigationTab::default(), NavigationTab::Mesh);
}

// ═══════════════════════════════════════════════════════════════════════════════════
// NAV MESH BAKE SETTINGS
// ═══════════════════════════════════════════════════════════════════════════════════

#[test]
fn nav_mesh_bake_settings_defaults() {
    let s = NavMeshBakeSettings::default();
    assert!((s.cell_size - 0.3).abs() < f32::EPSILON);
    assert!((s.cell_height - 0.2).abs() < f32::EPSILON);
    assert!((s.agent_height - 2.0).abs() < f32::EPSILON);
    assert!((s.agent_radius - 0.5).abs() < f32::EPSILON);
    assert!((s.agent_max_climb - 0.5).abs() < f32::EPSILON);
    assert!((s.agent_max_slope - 45.0).abs() < f32::EPSILON);
    assert_eq!(s.region_min_size, 8);
    assert_eq!(s.region_merge_size, 20);
    assert!((s.edge_max_len - 12.0).abs() < f32::EPSILON);
    assert!((s.edge_max_error - 1.3).abs() < f32::EPSILON);
    assert_eq!(s.verts_per_poly, 6);
    assert!((s.detail_sample_dist - 6.0).abs() < f32::EPSILON);
    assert!((s.detail_sample_max_error - 1.0).abs() < f32::EPSILON);
}

// ═══════════════════════════════════════════════════════════════════════════════════
// NAV AGENT CONFIG
// ═══════════════════════════════════════════════════════════════════════════════════

#[test]
fn nav_agent_config_defaults() {
    let c = NavAgentConfig::default();
    assert!((c.radius - 0.5).abs() < f32::EPSILON);
    assert!((c.height - 2.0).abs() < f32::EPSILON);
    assert!((c.max_slope - 45.0).abs() < f32::EPSILON);
    assert!((c.step_height - 0.5).abs() < f32::EPSILON);
    assert!((c.max_jump_distance - 2.0).abs() < f32::EPSILON);
    assert!((c.max_fall_distance - 5.0).abs() < f32::EPSILON);
}

// ═══════════════════════════════════════════════════════════════════════════════════
// NAV OBSTACLE
// ═══════════════════════════════════════════════════════════════════════════════════

#[test]
fn nav_obstacle_defaults() {
    let o = NavObstacle::default();
    assert_eq!(o.id, 0);
    assert_eq!(o.name, "Obstacle");
    assert!(!o.is_dynamic);
    assert!(o.carve_when_stationary);
}

// ═══════════════════════════════════════════════════════════════════════════════════
// NAV DEBUG OPTIONS
// ═══════════════════════════════════════════════════════════════════════════════════

#[test]
fn nav_debug_options_defaults() {
    let d = NavDebugOptions::default();
    assert!(d.show_triangles);
    assert!(d.show_edges);
    assert!(!d.show_vertices);
    assert!(d.show_regions);
    assert!(!d.show_connections);
    assert!(d.show_off_mesh_links);
    assert!(d.show_obstacles);
    assert!(d.show_path);
    assert!(!d.wireframe_mode);
}

// ═══════════════════════════════════════════════════════════════════════════════════
// NAV REGION
// ═══════════════════════════════════════════════════════════════════════════════════

#[test]
fn nav_region_fields() {
    let r = NavRegion {
        id: 42,
        area_type: NavAreaType::Road,
        bounds: ((0.0, 0.0, 0.0), (10.0, 5.0, 10.0)),
        triangle_count: 100,
        is_valid: true,
    };
    assert_eq!(r.id, 42);
    assert_eq!(r.area_type, NavAreaType::Road);
    assert_eq!(r.triangle_count, 100);
    assert!(r.is_valid);
}

// ═══════════════════════════════════════════════════════════════════════════════════
// PATH TEST RESULT
// ═══════════════════════════════════════════════════════════════════════════════════

#[test]
fn path_test_result_success() {
    let r = PathTestResult {
        start: (0.0, 0.0, 0.0),
        end: (10.0, 0.0, 10.0),
        path: vec![(0.0, 0.0, 0.0), (5.0, 0.0, 5.0), (10.0, 0.0, 10.0)],
        total_cost: 14.14,
        distance: 14.14,
        success: true,
        computation_time_ms: 0.5,
    };
    assert!(r.success);
    assert_eq!(r.path.len(), 3);
}

#[test]
fn path_test_result_failure() {
    let r = PathTestResult {
        start: (0.0, 0.0, 0.0),
        end: (100.0, 0.0, 100.0),
        path: vec![],
        total_cost: 0.0,
        distance: 0.0,
        success: false,
        computation_time_ms: 1.2,
    };
    assert!(!r.success);
    assert!(r.path.is_empty());
}

// ═══════════════════════════════════════════════════════════════════════════════════
// NETWORK ROLE
// ═══════════════════════════════════════════════════════════════════════════════════

#[test]
fn network_role_all_count() {
    assert_eq!(NetworkRole::all().len(), 4);
}

#[test]
fn network_role_names() {
    assert_eq!(NetworkRole::Server.name(), "Server");
    assert_eq!(NetworkRole::Client.name(), "Client");
    assert_eq!(NetworkRole::ListenServer.name(), "Listen Server");
    assert_eq!(NetworkRole::Offline.name(), "Offline");
}

#[test]
fn network_role_icons_non_empty() {
    for role in NetworkRole::all() {
        assert!(!role.icon().is_empty(), "{:?} icon empty", role);
    }
}

#[test]
fn network_role_is_server() {
    assert!(NetworkRole::Server.is_server());
    assert!(!NetworkRole::Client.is_server());
    assert!(NetworkRole::ListenServer.is_server());
    assert!(!NetworkRole::Offline.is_server());
}

#[test]
fn network_role_is_online() {
    assert!(NetworkRole::Server.is_online());
    assert!(NetworkRole::Client.is_online());
    assert!(NetworkRole::ListenServer.is_online());
    assert!(!NetworkRole::Offline.is_online());
}

#[test]
fn network_role_description_non_empty() {
    for role in NetworkRole::all() {
        assert!(!role.description().is_empty(), "{:?} description empty", role);
    }
}

#[test]
fn network_role_display() {
    for role in NetworkRole::all() {
        let s = format!("{}", role);
        assert!(s.contains(role.name()));
    }
}

// ═══════════════════════════════════════════════════════════════════════════════════
// CONNECTION STATE
// ═══════════════════════════════════════════════════════════════════════════════════

#[test]
fn connection_state_all_count() {
    assert_eq!(ConnectionState::all().len(), 5);
}

#[test]
fn connection_state_names() {
    assert_eq!(ConnectionState::Disconnected.name(), "Disconnected");
    assert_eq!(ConnectionState::Connecting.name(), "Connecting");
    assert_eq!(ConnectionState::Connected.name(), "Connected");
    assert_eq!(ConnectionState::Reconnecting.name(), "Reconnecting");
    assert_eq!(ConnectionState::Error.name(), "Error");
}

#[test]
fn connection_state_is_active() {
    assert!(!ConnectionState::Disconnected.is_active());
    assert!(ConnectionState::Connecting.is_active());
    assert!(ConnectionState::Connected.is_active());
    assert!(ConnectionState::Reconnecting.is_active());
    assert!(!ConnectionState::Error.is_active());
}

#[test]
fn connection_state_is_stable() {
    assert!(!ConnectionState::Disconnected.is_stable());
    assert!(!ConnectionState::Connecting.is_stable());
    assert!(ConnectionState::Connected.is_stable());
    assert!(!ConnectionState::Reconnecting.is_stable());
    assert!(!ConnectionState::Error.is_stable());
}

#[test]
fn connection_state_icons_non_empty() {
    for state in ConnectionState::all() {
        assert!(!state.icon().is_empty(), "{:?} icon empty", state);
    }
}

#[test]
fn connection_state_display() {
    for state in ConnectionState::all() {
        let s = format!("{}", state);
        assert!(s.contains(state.name()));
    }
}

// ═══════════════════════════════════════════════════════════════════════════════════
// INTEREST POLICY
// ═══════════════════════════════════════════════════════════════════════════════════

#[test]
fn interest_policy_all_count() {
    assert_eq!(InterestPolicy::all().len(), 5);
}

#[test]
fn interest_policy_names() {
    assert_eq!(InterestPolicy::Full.name(), "Full");
    assert_eq!(InterestPolicy::Radius.name(), "Radius");
    assert_eq!(InterestPolicy::FieldOfView.name(), "Field of View");
    assert_eq!(InterestPolicy::FieldOfViewWithLOS.name(), "FOV + LOS");
    assert_eq!(InterestPolicy::Custom.name(), "Custom");
}

#[test]
fn interest_policy_is_spatial() {
    assert!(!InterestPolicy::Full.is_spatial());
    assert!(InterestPolicy::Radius.is_spatial());
    assert!(InterestPolicy::FieldOfView.is_spatial());
    assert!(InterestPolicy::FieldOfViewWithLOS.is_spatial());
    assert!(!InterestPolicy::Custom.is_spatial());
}

#[test]
fn interest_policy_has_visibility_check() {
    assert!(!InterestPolicy::Full.has_visibility_check());
    assert!(!InterestPolicy::Radius.has_visibility_check());
    assert!(InterestPolicy::FieldOfView.has_visibility_check());
    assert!(InterestPolicy::FieldOfViewWithLOS.has_visibility_check());
    assert!(!InterestPolicy::Custom.has_visibility_check());
}

#[test]
fn interest_policy_description_non_empty() {
    for p in InterestPolicy::all() {
        assert!(!p.description().is_empty(), "{:?} description empty", p);
    }
}

#[test]
fn interest_policy_display() {
    for p in InterestPolicy::all() {
        let s = format!("{}", p);
        assert!(s.contains(p.name()));
    }
}

// ═══════════════════════════════════════════════════════════════════════════════════
// COMPRESSION LEVEL
// ═══════════════════════════════════════════════════════════════════════════════════

#[test]
fn compression_level_all_count() {
    assert_eq!(CompressionLevel::all().len(), 4);
}

#[test]
fn compression_level_names() {
    assert_eq!(CompressionLevel::None.name(), "None");
    assert_eq!(CompressionLevel::Fast.name(), "Fast");
    assert_eq!(CompressionLevel::Balanced.name(), "Balanced");
    assert_eq!(CompressionLevel::Maximum.name(), "Maximum");
}

#[test]
fn compression_level_cpu_cost_monotonic() {
    let costs: Vec<u8> = CompressionLevel::all().iter().map(|c| c.cpu_cost()).collect();
    for w in costs.windows(2) {
        assert!(w[0] <= w[1], "CPU cost not monotonic: {:?}", costs);
    }
}

#[test]
fn compression_level_cpu_cost_values() {
    assert_eq!(CompressionLevel::None.cpu_cost(), 0);
    assert_eq!(CompressionLevel::Fast.cpu_cost(), 1);
    assert_eq!(CompressionLevel::Balanced.cpu_cost(), 2);
    assert_eq!(CompressionLevel::Maximum.cpu_cost(), 4);
}

#[test]
fn compression_level_description_non_empty() {
    for c in CompressionLevel::all() {
        assert!(!c.description().is_empty(), "{:?} description empty", c);
    }
}

#[test]
fn compression_level_default_is_fast() {
    assert_eq!(CompressionLevel::default(), CompressionLevel::Fast);
}

#[test]
fn compression_level_display() {
    for c in CompressionLevel::all() {
        let s = format!("{}", c);
        assert!(s.contains(c.name()));
    }
}

// ═══════════════════════════════════════════════════════════════════════════════════
// CLIENT INFO DEFAULTS
// ═══════════════════════════════════════════════════════════════════════════════════

#[test]
fn client_info_defaults() {
    let c = ClientInfo::default();
    assert_eq!(c.id, 0);
    assert!(c.name.is_empty());
    assert!(c.address.is_empty());
    assert_eq!(c.ping_ms, 0);
    assert!((c.packet_loss_percent - 0.0).abs() < f32::EPSILON);
    assert_eq!(c.state, ConnectionState::default());
    assert!(c.player_entity_id.is_none());
    assert_eq!(c.bytes_sent, 0);
    assert_eq!(c.bytes_received, 0);
}

// ═══════════════════════════════════════════════════════════════════════════════════
// NETWORK STATS DEFAULTS
// ═══════════════════════════════════════════════════════════════════════════════════

#[test]
fn network_stats_defaults() {
    let s = NetworkStats::default();
    assert!((s.bytes_sent_per_sec - 0.0).abs() < f32::EPSILON);
    assert!((s.bytes_received_per_sec - 0.0).abs() < f32::EPSILON);
    assert_eq!(s.current_tick, 0);
    assert_eq!(s.entities_replicated, 0);
    assert_eq!(s.entities_filtered, 0);
}
