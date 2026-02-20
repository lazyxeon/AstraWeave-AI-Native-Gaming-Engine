//! Wave 2 Mutation Remediation — Dialogue Editor + Project Settings panels
//!
//! Targets: dialogue_editor_panel.rs (3,589 lines) + project_settings_panel.rs (2,972 lines)
//! Focus: enum Display/name/icon, Default values, boundary helpers, format strings

use aw_editor_lib::panels::dialogue_editor_panel::{
    DialogueChoice, DialogueGraph, DialogueNode, DialogueNodeType, DialogueSpeaker, DialogueTab,
    ExportFormat, IssueSeverity, LayoutAlgorithm, VariableType,
};
use aw_editor_lib::panels::project_settings_panel::{
    AntialiasingMode, AoMode, AudioBackend, AudioSettings, BroadphaseType, BuildConfig,
    CompressionMode, GiMode, InputAction, PhysicsSettings, QualityLevel, ReflectionMode,
    RendererBackend, RenderingSettings, ShadowMode, SettingsTab, TargetPlatform, TextureQuality,
    TonemappingMode,
};

// ═══════════════════════════════════════════════════════════════════════════
// DIALOGUE NODE TYPE — 7 variants × (Display, name, icon, color, is_*)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn dialogue_node_type_default_is_speech() {
    assert_eq!(DialogueNodeType::default(), DialogueNodeType::Speech);
}

#[test]
fn dialogue_node_type_all_returns_7() {
    assert_eq!(DialogueNodeType::all().len(), 7);
}

#[test]
fn dialogue_node_type_names_unique() {
    let names: Vec<&str> = DialogueNodeType::all().iter().map(|v| v.name()).collect();
    assert_eq!(names, vec!["Speech", "Choice", "Condition", "Action", "Random Branch", "Jump", "End"]);
}

#[test]
fn dialogue_node_type_icons_nonempty() {
    for v in DialogueNodeType::all() {
        assert!(!v.icon().is_empty(), "icon empty for {:?}", v);
    }
}

#[test]
fn dialogue_node_type_display_contains_icon_and_name() {
    for v in DialogueNodeType::all() {
        let display = format!("{v}");
        assert!(display.contains(v.name()), "display '{}' missing name '{}'", display, v.name());
    }
}

#[test]
fn dialogue_node_type_color_unique_per_variant() {
    let colors: Vec<_> = DialogueNodeType::all().iter().map(|v| v.color()).collect();
    for i in 0..colors.len() {
        for j in (i + 1)..colors.len() {
            assert_ne!(colors[i], colors[j], "duplicate color at {:?} and {:?}",
                DialogueNodeType::all()[i], DialogueNodeType::all()[j]);
        }
    }
}

#[test]
fn dialogue_node_type_is_branching() {
    assert!(DialogueNodeType::Choice.is_branching());
    assert!(DialogueNodeType::Condition.is_branching());
    assert!(DialogueNodeType::RandomBranch.is_branching());
    assert!(!DialogueNodeType::Speech.is_branching());
    assert!(!DialogueNodeType::Action.is_branching());
    assert!(!DialogueNodeType::Jump.is_branching());
    assert!(!DialogueNodeType::End.is_branching());
}

#[test]
fn dialogue_node_type_is_terminal() {
    assert!(DialogueNodeType::End.is_terminal());
    assert!(!DialogueNodeType::Speech.is_terminal());
    assert!(!DialogueNodeType::Choice.is_terminal());
    assert!(!DialogueNodeType::Condition.is_terminal());
    assert!(!DialogueNodeType::Action.is_terminal());
    assert!(!DialogueNodeType::RandomBranch.is_terminal());
    assert!(!DialogueNodeType::Jump.is_terminal());
}

// ═══════════════════════════════════════════════════════════════════════════
// VARIABLE TYPE — 4 variants
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn variable_type_default_is_boolean() {
    assert_eq!(VariableType::default(), VariableType::Boolean);
}

#[test]
fn variable_type_all_returns_4() {
    assert_eq!(VariableType::all().len(), 4);
}

#[test]
fn variable_type_names() {
    let names: Vec<&str> = VariableType::all().iter().map(|v| v.name()).collect();
    assert_eq!(names, vec!["Boolean", "Integer", "Float", "String"]);
}

#[test]
fn variable_type_icons_nonempty() {
    for v in VariableType::all() {
        assert!(!v.icon().is_empty());
    }
}

#[test]
fn variable_type_display_contains_name() {
    for v in VariableType::all() {
        let d = format!("{v}");
        assert!(d.contains(v.name()));
    }
}

#[test]
fn variable_type_is_numeric() {
    assert!(!VariableType::Boolean.is_numeric());
    assert!(VariableType::Integer.is_numeric());
    assert!(VariableType::Float.is_numeric());
    assert!(!VariableType::String.is_numeric());
}

// ═══════════════════════════════════════════════════════════════════════════
// DIALOGUE TAB — 9 variants
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn dialogue_tab_default_is_graph() {
    assert_eq!(DialogueTab::default(), DialogueTab::Graph);
}

#[test]
fn dialogue_tab_all_returns_9() {
    assert_eq!(DialogueTab::all().len(), 9);
}

#[test]
fn dialogue_tab_names() {
    let all = DialogueTab::all();
    assert_eq!(all[0].name(), "Graph");
    assert_eq!(all[1].name(), "Nodes");
    assert_eq!(all[2].name(), "Speakers");
    assert_eq!(all[3].name(), "Variables");
    assert_eq!(all[4].name(), "Localization");
    assert_eq!(all[5].name(), "Preview");
    assert_eq!(all[6].name(), "Validation");
    assert_eq!(all[7].name(), "Export");
    assert_eq!(all[8].name(), "Templates");
}

#[test]
fn dialogue_tab_icons_nonempty() {
    for v in DialogueTab::all() {
        assert!(!v.icon().is_empty());
    }
}

#[test]
fn dialogue_tab_display_contains_name() {
    for v in DialogueTab::all() {
        let d = format!("{v}");
        assert!(d.contains(v.name()));
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// LAYOUT ALGORITHM — 4 variants
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn layout_algorithm_default_is_hierarchical() {
    assert_eq!(LayoutAlgorithm::default(), LayoutAlgorithm::Hierarchical);
}

#[test]
fn layout_algorithm_all_count() {
    assert_eq!(LayoutAlgorithm::all().len(), 4);
}

#[test]
fn layout_algorithm_names() {
    let names: Vec<&str> = LayoutAlgorithm::all().iter().map(|v| v.name()).collect();
    assert_eq!(names, vec!["Hierarchical", "Radial", "Force Directed", "Tree"]);
}

#[test]
fn layout_algorithm_icons_nonempty() {
    for v in LayoutAlgorithm::all() {
        assert!(!v.icon().is_empty());
    }
}

#[test]
fn layout_algorithm_descriptions_nonempty() {
    for v in LayoutAlgorithm::all() {
        assert!(!v.description().is_empty(), "description empty for {:?}", v);
    }
}

#[test]
fn layout_algorithm_display_contains_name() {
    for v in LayoutAlgorithm::all() {
        let d = format!("{v}");
        assert!(d.contains(v.name()));
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// EXPORT FORMAT — 5 variants
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn export_format_default_is_json() {
    assert_eq!(ExportFormat::default(), ExportFormat::Json);
}

#[test]
fn export_format_all_count() {
    assert_eq!(ExportFormat::all().len(), 5);
}

#[test]
fn export_format_names() {
    assert_eq!(ExportFormat::Json.name(), "JSON");
    assert_eq!(ExportFormat::Yarn.name(), "Yarn");
    assert_eq!(ExportFormat::Ink.name(), "Ink");
    assert_eq!(ExportFormat::Xml.name(), "XML");
    assert_eq!(ExportFormat::Csv.name(), "CSV");
}

#[test]
fn export_format_extensions() {
    assert_eq!(ExportFormat::Json.extension(), "json");
    assert_eq!(ExportFormat::Yarn.extension(), "yarn");
    assert_eq!(ExportFormat::Ink.extension(), "ink");
    assert_eq!(ExportFormat::Xml.extension(), "xml");
    assert_eq!(ExportFormat::Csv.extension(), "csv");
}

#[test]
fn export_format_is_text_format() {
    for v in ExportFormat::all() {
        assert!(v.is_text_format(), "is_text_format false for {:?}", v);
    }
}

#[test]
fn export_format_display_contains_name() {
    for v in ExportFormat::all() {
        let d = format!("{v}");
        assert!(d.contains(v.name()));
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// ISSUE SEVERITY — 3 variants
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn issue_severity_default_is_info() {
    assert_eq!(IssueSeverity::default(), IssueSeverity::Info);
}

#[test]
fn issue_severity_all_count() {
    assert_eq!(IssueSeverity::all().len(), 3);
}

#[test]
fn issue_severity_names() {
    assert_eq!(IssueSeverity::Info.name(), "Info");
    assert_eq!(IssueSeverity::Warning.name(), "Warning");
    assert_eq!(IssueSeverity::Error.name(), "Error");
}

#[test]
fn issue_severity_icons_nonempty() {
    for v in IssueSeverity::all() {
        assert!(!v.icon().is_empty());
    }
}

#[test]
fn issue_severity_colors_unique() {
    let error_c = IssueSeverity::Error.color();
    let warn_c = IssueSeverity::Warning.color();
    let info_c = IssueSeverity::Info.color();
    assert_ne!(error_c, warn_c);
    assert_ne!(error_c, info_c);
    assert_ne!(warn_c, info_c);
}

#[test]
fn issue_severity_display_contains_name() {
    for v in IssueSeverity::all() {
        let d = format!("{v}");
        assert!(d.contains(v.name()));
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// DIALOGUE DEFAULTS — structs
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn dialogue_choice_default_values() {
    let c = DialogueChoice::default();
    assert_eq!(c.text, "Continue");
    assert_eq!(c.target_node_id, None);
    assert!(c.condition.is_empty());
    assert!(!c.is_default);
}

#[test]
fn dialogue_node_default_values() {
    let n = DialogueNode::default();
    assert_eq!(n.id, 0);
    assert_eq!(n.node_type, DialogueNodeType::Speech);
    assert_eq!(n.speaker_id, None);
    assert!(n.text.is_empty());
    assert!(n.choices.is_empty());
    assert_eq!(n.position, (0.0, 0.0));
    assert!(n.notes.is_empty());
}

#[test]
fn dialogue_speaker_default_values() {
    let s = DialogueSpeaker::default();
    assert!(s.id.is_empty());
    assert_eq!(s.name, "Unknown");
    assert!(s.portrait.is_empty());
    assert!(s.voice_id.is_empty());
}

#[test]
fn dialogue_graph_default_values() {
    let g = DialogueGraph::default();
    assert_eq!(g.id, 0);
    assert_eq!(g.name, "New Dialogue");
    assert!(g.description.is_empty());
    assert_eq!(g.start_node_id, None);
    assert!(g.nodes.is_empty());
    assert!(g.speakers.is_empty());
    assert!(g.variables.is_empty());
}

// ═══════════════════════════════════════════════════════════════════════════
// TARGET PLATFORM — 9 variants
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn target_platform_all_returns_9() {
    assert_eq!(TargetPlatform::all().len(), 9);
}

#[test]
fn target_platform_names() {
    assert_eq!(TargetPlatform::Windows.name(), "Windows");
    assert_eq!(TargetPlatform::Linux.name(), "Linux");
    assert_eq!(TargetPlatform::MacOS.name(), "macOS");
    assert_eq!(TargetPlatform::Android.name(), "Android");
    assert_eq!(TargetPlatform::Ios.name(), "iOS");
    assert_eq!(TargetPlatform::WebAssembly.name(), "WebAssembly");
    assert_eq!(TargetPlatform::PlayStation.name(), "PlayStation");
    assert_eq!(TargetPlatform::Xbox.name(), "Xbox");
    assert_eq!(TargetPlatform::NintendoSwitch.name(), "Nintendo Switch");
}

#[test]
fn target_platform_icons_nonempty() {
    for v in TargetPlatform::all() {
        assert!(!v.icon().is_empty());
    }
}

#[test]
fn target_platform_is_desktop() {
    assert!(TargetPlatform::Windows.is_desktop());
    assert!(TargetPlatform::Linux.is_desktop());
    assert!(TargetPlatform::MacOS.is_desktop());
    assert!(!TargetPlatform::Android.is_desktop());
    assert!(!TargetPlatform::Ios.is_desktop());
    assert!(!TargetPlatform::WebAssembly.is_desktop());
    assert!(!TargetPlatform::PlayStation.is_desktop());
    assert!(!TargetPlatform::Xbox.is_desktop());
    assert!(!TargetPlatform::NintendoSwitch.is_desktop());
}

#[test]
fn target_platform_is_mobile() {
    assert!(TargetPlatform::Android.is_mobile());
    assert!(TargetPlatform::Ios.is_mobile());
    assert!(!TargetPlatform::Windows.is_mobile());
    assert!(!TargetPlatform::Linux.is_mobile());
    assert!(!TargetPlatform::MacOS.is_mobile());
    assert!(!TargetPlatform::WebAssembly.is_mobile());
    assert!(!TargetPlatform::PlayStation.is_mobile());
    assert!(!TargetPlatform::Xbox.is_mobile());
    assert!(!TargetPlatform::NintendoSwitch.is_mobile());
}

#[test]
fn target_platform_is_console() {
    assert!(TargetPlatform::PlayStation.is_console());
    assert!(TargetPlatform::Xbox.is_console());
    assert!(TargetPlatform::NintendoSwitch.is_console());
    assert!(!TargetPlatform::Windows.is_console());
    assert!(!TargetPlatform::Android.is_console());
    assert!(!TargetPlatform::WebAssembly.is_console());
}

#[test]
fn target_platform_display_contains_name() {
    for v in TargetPlatform::all() {
        let d = format!("{v}");
        assert!(d.contains(v.name()));
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// TEXTURE QUALITY — 4 variants
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn texture_quality_default_is_high() {
    assert_eq!(TextureQuality::default(), TextureQuality::High);
}

#[test]
fn texture_quality_all_count() {
    assert_eq!(TextureQuality::all().len(), 4);
}

#[test]
fn texture_quality_names() {
    assert_eq!(TextureQuality::Low.name(), "Low");
    assert_eq!(TextureQuality::Medium.name(), "Medium");
    assert_eq!(TextureQuality::High.name(), "High");
    assert_eq!(TextureQuality::Ultra.name(), "Ultra");
}

#[test]
fn texture_quality_display_matches_name() {
    for v in TextureQuality::all() {
        assert_eq!(format!("{v}"), v.name());
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// ANTIALIASING MODE — 7 variants
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn antialiasing_default_is_smaa() {
    assert_eq!(AntialiasingMode::default(), AntialiasingMode::Smaa);
}

#[test]
fn antialiasing_all_count() {
    assert_eq!(AntialiasingMode::all().len(), 7);
}

#[test]
fn antialiasing_names() {
    assert_eq!(AntialiasingMode::None.name(), "None");
    assert_eq!(AntialiasingMode::Fxaa.name(), "FXAA");
    assert_eq!(AntialiasingMode::Smaa.name(), "SMAA");
    assert_eq!(AntialiasingMode::Taa.name(), "TAA");
    assert_eq!(AntialiasingMode::Msaa2x.name(), "MSAA 2x");
    assert_eq!(AntialiasingMode::Msaa4x.name(), "MSAA 4x");
    assert_eq!(AntialiasingMode::Msaa8x.name(), "MSAA 8x");
}

#[test]
fn antialiasing_is_msaa() {
    assert!(!AntialiasingMode::None.is_msaa());
    assert!(!AntialiasingMode::Fxaa.is_msaa());
    assert!(!AntialiasingMode::Smaa.is_msaa());
    assert!(!AntialiasingMode::Taa.is_msaa());
    assert!(AntialiasingMode::Msaa2x.is_msaa());
    assert!(AntialiasingMode::Msaa4x.is_msaa());
    assert!(AntialiasingMode::Msaa8x.is_msaa());
}

#[test]
fn antialiasing_display_matches_name() {
    for v in AntialiasingMode::all() {
        assert_eq!(format!("{v}"), v.name());
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// BROADPHASE TYPE — 3 variants
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn broadphase_default_is_sap() {
    assert_eq!(BroadphaseType::default(), BroadphaseType::Sap);
}

#[test]
fn broadphase_all_count() {
    assert_eq!(BroadphaseType::all().len(), 3);
}

#[test]
fn broadphase_names() {
    assert_eq!(BroadphaseType::Sap.name(), "SAP");
    assert_eq!(BroadphaseType::DynamicAabb.name(), "Dynamic AABB");
    assert_eq!(BroadphaseType::Quadtree.name(), "Quadtree");
}

#[test]
fn broadphase_display_matches_name() {
    for v in BroadphaseType::all() {
        assert_eq!(format!("{v}"), v.name());
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// AUDIO BACKEND — 5 variants
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn audio_backend_default_is_auto() {
    assert_eq!(AudioBackend::default(), AudioBackend::Auto);
}

#[test]
fn audio_backend_all_count() {
    assert_eq!(AudioBackend::all().len(), 5);
}

#[test]
fn audio_backend_names() {
    assert_eq!(AudioBackend::Auto.name(), "Auto");
    assert_eq!(AudioBackend::Wasapi.name(), "WASAPI");
    assert_eq!(AudioBackend::CoreAudio.name(), "Core Audio");
    assert_eq!(AudioBackend::Alsa.name(), "ALSA");
    assert_eq!(AudioBackend::PulseAudio.name(), "PulseAudio");
}

// ═══════════════════════════════════════════════════════════════════════════
// RENDERER BACKEND — 6 variants
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn renderer_backend_default_is_auto() {
    assert_eq!(RendererBackend::default(), RendererBackend::Auto);
}

#[test]
fn renderer_backend_all_count() {
    assert_eq!(RendererBackend::all().len(), 6);
}

#[test]
fn renderer_backend_names() {
    assert_eq!(RendererBackend::Auto.name(), "Auto");
    assert_eq!(RendererBackend::Vulkan.name(), "Vulkan");
    assert_eq!(RendererBackend::DirectX12.name(), "DirectX 12");
    assert_eq!(RendererBackend::Metal.name(), "Metal");
    assert_eq!(RendererBackend::OpenGL.name(), "OpenGL");
    assert_eq!(RendererBackend::WebGpu.name(), "WebGPU");
}

// ═══════════════════════════════════════════════════════════════════════════
// TONEMAPPING MODE — 5 variants
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn tonemapping_default_is_aces() {
    assert_eq!(TonemappingMode::default(), TonemappingMode::Aces);
}

#[test]
fn tonemapping_all_count() {
    assert_eq!(TonemappingMode::all().len(), 5);
}

#[test]
fn tonemapping_names() {
    assert_eq!(TonemappingMode::None.name(), "None");
    assert_eq!(TonemappingMode::Reinhard.name(), "Reinhard");
    assert_eq!(TonemappingMode::Aces.name(), "ACES");
    assert_eq!(TonemappingMode::AgX.name(), "AgX");
    assert_eq!(TonemappingMode::Filmic.name(), "Filmic");
}

// ═══════════════════════════════════════════════════════════════════════════
// AO MODE — 4 variants
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn ao_mode_default_is_hbao() {
    assert_eq!(AoMode::default(), AoMode::Hbao);
}

#[test]
fn ao_mode_all_count() {
    assert_eq!(AoMode::all().len(), 4);
}

#[test]
fn ao_mode_names() {
    assert_eq!(AoMode::None.name(), "None");
    assert_eq!(AoMode::Ssao.name(), "SSAO");
    assert_eq!(AoMode::Hbao.name(), "HBAO+");
    assert_eq!(AoMode::Gtao.name(), "GTAO");
}

// ═══════════════════════════════════════════════════════════════════════════
// GI MODE (project_settings) — 4 variants
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn gi_mode_default_is_none() {
    assert_eq!(GiMode::default(), GiMode::None);
}

#[test]
fn gi_mode_all_count() {
    assert_eq!(GiMode::all().len(), 4);
}

#[test]
fn gi_mode_names() {
    assert_eq!(GiMode::None.name(), "None");
    assert_eq!(GiMode::ScreenSpace.name(), "Screen Space");
    assert_eq!(GiMode::Lumen.name(), "Lumen");
    assert_eq!(GiMode::PathTraced.name(), "Path Traced");
}

#[test]
fn gi_mode_is_raytraced() {
    assert!(!GiMode::None.is_raytraced());
    assert!(!GiMode::ScreenSpace.is_raytraced());
    assert!(GiMode::Lumen.is_raytraced());
    assert!(GiMode::PathTraced.is_raytraced());
}

// ═══════════════════════════════════════════════════════════════════════════
// REFLECTION MODE — 4 variants
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn reflection_mode_default_is_screen_space() {
    assert_eq!(ReflectionMode::default(), ReflectionMode::ScreenSpace);
}

#[test]
fn reflection_mode_all_count() {
    assert_eq!(ReflectionMode::all().len(), 4);
}

#[test]
fn reflection_mode_names() {
    assert_eq!(ReflectionMode::None.name(), "None");
    assert_eq!(ReflectionMode::ScreenSpace.name(), "Screen Space");
    assert_eq!(ReflectionMode::Raytraced.name(), "Raytraced");
    assert_eq!(ReflectionMode::Hybrid.name(), "Hybrid");
}

// ═══════════════════════════════════════════════════════════════════════════
// SHADOW MODE — 4 variants
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn shadow_mode_default_is_soft() {
    assert_eq!(ShadowMode::default(), ShadowMode::SoftShadows);
}

#[test]
fn shadow_mode_all_count() {
    assert_eq!(ShadowMode::all().len(), 4);
}

#[test]
fn shadow_mode_names() {
    assert_eq!(ShadowMode::None.name(), "None");
    assert_eq!(ShadowMode::HardShadows.name(), "Hard Shadows");
    assert_eq!(ShadowMode::SoftShadows.name(), "Soft Shadows");
    assert_eq!(ShadowMode::Raytraced.name(), "Raytraced");
}

// ═══════════════════════════════════════════════════════════════════════════
// COMPRESSION MODE — 3 variants
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn compression_mode_default_is_fast() {
    assert_eq!(CompressionMode::default(), CompressionMode::Fast);
}

#[test]
fn compression_mode_all_count() {
    assert_eq!(CompressionMode::all().len(), 3);
}

#[test]
fn compression_mode_names() {
    assert_eq!(CompressionMode::None.name(), "None");
    assert_eq!(CompressionMode::Fast.name(), "Fast");
    assert_eq!(CompressionMode::Best.name(), "Best");
}

// ═══════════════════════════════════════════════════════════════════════════
// STRUCT DEFAULTS — QualityLevel, PhysicsSettings, AudioSettings, etc.
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn quality_level_default_values() {
    let q = QualityLevel::default();
    assert_eq!(q.name, "Medium");
    assert_eq!(q.shadow_resolution, 2048);
    assert_eq!(q.shadow_cascades, 4);
    assert_eq!(q.texture_quality, TextureQuality::High);
    assert_eq!(q.antialiasing, AntialiasingMode::Smaa);
    assert!(q.vsync);
    assert_eq!(q.max_fps, 60);
    assert!((q.lod_bias - 1.0).abs() < f32::EPSILON);
    assert!((q.particle_density - 1.0).abs() < f32::EPSILON);
}

#[test]
fn physics_settings_default_values() {
    let p = PhysicsSettings::default();
    assert!((p.gravity[1] - (-9.81)).abs() < 0.001);
    assert!((p.fixed_timestep - 1.0 / 60.0).abs() < 0.001);
    assert_eq!(p.max_substeps, 4);
    assert_eq!(p.broadphase, BroadphaseType::Sap);
    assert!((p.default_friction - 0.5).abs() < f32::EPSILON);
    assert!((p.default_restitution - 0.3).abs() < f32::EPSILON);
    assert!((p.sleep_threshold - 0.1).abs() < f32::EPSILON);
    assert!(p.enable_ccd);
}

#[test]
fn audio_settings_default_values() {
    let a = AudioSettings::default();
    assert!((a.master_volume - 1.0).abs() < f32::EPSILON);
    assert!((a.music_volume - 0.8).abs() < f32::EPSILON);
    assert!((a.sfx_volume - 1.0).abs() < f32::EPSILON);
    assert!((a.voice_volume - 1.0).abs() < f32::EPSILON);
    assert!((a.ambient_volume - 0.7).abs() < f32::EPSILON);
    assert_eq!(a.max_simultaneous_sounds, 64);
    assert!((a.doppler_factor - 1.0).abs() < f32::EPSILON);
    assert_eq!(a.audio_backend, AudioBackend::Auto);
}

#[test]
fn rendering_settings_default_values() {
    let r = RenderingSettings::default();
    assert_eq!(r.renderer_backend, RendererBackend::Auto);
    assert!(r.hdr_enabled);
    assert!(r.bloom_enabled);
    assert!((r.bloom_intensity - 0.5).abs() < f32::EPSILON);
    assert_eq!(r.tonemapping, TonemappingMode::Aces);
    assert_eq!(r.ambient_occlusion, AoMode::Hbao);
    assert_eq!(r.global_illumination, GiMode::ScreenSpace);
    assert_eq!(r.reflection_mode, ReflectionMode::ScreenSpace);
    assert_eq!(r.shadow_mode, ShadowMode::SoftShadows);
}

#[test]
fn build_config_default_values() {
    let b = BuildConfig::default();
    assert_eq!(b.platform, TargetPlatform::Windows);
    assert!(b.enabled);
    assert!(b.development_build);
    assert_eq!(b.compression, CompressionMode::Fast);
    assert!(b.icon_path.is_empty());
    assert_eq!(b.app_name, "My Game");
    assert_eq!(b.version, "1.0.0");
    assert_eq!(b.company, "My Company");
}

#[test]
fn input_action_default_values() {
    let a = InputAction::default();
    assert!(a.name.is_empty());
    assert!(a.primary_key.is_empty());
    assert!(a.secondary_key.is_empty());
    assert!(a.gamepad_button.is_empty());
    assert!((a.dead_zone - 0.1).abs() < f32::EPSILON);
}

// ═══════════════════════════════════════════════════════════════════════════
// CROSS-CUTTING: Enum Display round-trip & uniqueness
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn all_dialogue_enum_names_unique() {
    let all_names: Vec<&str> = DialogueNodeType::all().iter().map(|v| v.name()).collect();
    let set: std::collections::HashSet<&&str> = all_names.iter().collect();
    assert_eq!(all_names.len(), set.len());
}

#[test]
fn all_target_platform_names_unique() {
    let names: Vec<&str> = TargetPlatform::all().iter().map(|v| v.name()).collect();
    let set: std::collections::HashSet<&&str> = names.iter().collect();
    assert_eq!(names.len(), set.len());
}

#[test]
fn all_antialiasing_names_unique() {
    let names: Vec<&str> = AntialiasingMode::all().iter().map(|v| v.name()).collect();
    let set: std::collections::HashSet<&&str> = names.iter().collect();
    assert_eq!(names.len(), set.len());
}

#[test]
fn export_format_icon_nonempty() {
    for v in ExportFormat::all() {
        assert!(!v.icon().is_empty());
    }
}

#[test]
fn settings_tab_default() {
    assert_eq!(SettingsTab::default(), SettingsTab::Project);
}
