//! Wave 2 mutation remediation tests — Entity + Performance panels
//! Covers: EntityArchetype, PrefabAction, ValidationSeverity, EntityFilter, EntityStats,
//!         PerfCategory, MetricUnit, PerfMetric, SubsystemTiming, MemoryStats,
//!         GpuStats, AlertSeverity

use aw_editor_lib::panels::entity_panel::{
    EntityArchetype, EntityFilter, EntityStats, PrefabAction, ValidationSeverity,
};
use aw_editor_lib::panels::performance_panel::{
    AlertSeverity, GpuStats, MemoryStats, MetricUnit, PerfCategory, PerfMetric, SubsystemTiming,
};
use astraweave_core::Entity;

// ═══════════════════════════════════════════════════════════════════════════════════
// ENTITY ARCHETYPE
// ═══════════════════════════════════════════════════════════════════════════════════

#[test]
fn entity_archetype_all_count() {
    assert_eq!(EntityArchetype::all().len(), 9);
}

#[test]
fn entity_archetype_names() {
    assert_eq!(EntityArchetype::Player.name(), "Player");
    assert_eq!(EntityArchetype::Companion.name(), "Companion");
    assert_eq!(EntityArchetype::Enemy.name(), "Enemy");
    assert_eq!(EntityArchetype::Boss.name(), "Boss");
    assert_eq!(EntityArchetype::NPC.name(), "NPC");
    assert_eq!(EntityArchetype::Prop.name(), "Prop");
    assert_eq!(EntityArchetype::Trigger.name(), "Trigger");
    assert_eq!(EntityArchetype::Light.name(), "Light");
    assert_eq!(EntityArchetype::Camera.name(), "Camera");
}

#[test]
fn entity_archetype_icons_non_empty() {
    for arch in EntityArchetype::all() {
        assert!(!arch.icon().is_empty(), "{:?} icon empty", arch);
    }
}

#[test]
fn entity_archetype_is_combatant() {
    assert!(EntityArchetype::Player.is_combatant());
    assert!(EntityArchetype::Companion.is_combatant());
    assert!(EntityArchetype::Enemy.is_combatant());
    assert!(EntityArchetype::Boss.is_combatant());
    assert!(!EntityArchetype::NPC.is_combatant());
    assert!(!EntityArchetype::Prop.is_combatant());
    assert!(!EntityArchetype::Trigger.is_combatant());
    assert!(!EntityArchetype::Light.is_combatant());
    assert!(!EntityArchetype::Camera.is_combatant());
}

#[test]
fn entity_archetype_default_health_values() {
    assert_eq!(EntityArchetype::Player.default_health(), 100);
    assert_eq!(EntityArchetype::Companion.default_health(), 80);
    assert_eq!(EntityArchetype::Enemy.default_health(), 50);
    assert_eq!(EntityArchetype::Boss.default_health(), 500);
    assert_eq!(EntityArchetype::NPC.default_health(), 100);
    assert_eq!(EntityArchetype::Prop.default_health(), 10);
    assert_eq!(EntityArchetype::Trigger.default_health(), 1);
    assert_eq!(EntityArchetype::Light.default_health(), 1);
    assert_eq!(EntityArchetype::Camera.default_health(), 1);
}

#[test]
fn entity_archetype_default_damage_values() {
    assert_eq!(EntityArchetype::Player.default_damage(), 25);
    assert_eq!(EntityArchetype::Companion.default_damage(), 20);
    assert_eq!(EntityArchetype::Enemy.default_damage(), 15);
    assert_eq!(EntityArchetype::Boss.default_damage(), 50);
    assert_eq!(EntityArchetype::NPC.default_damage(), 0);
    assert_eq!(EntityArchetype::Prop.default_damage(), 0);
    assert_eq!(EntityArchetype::Trigger.default_damage(), 0);
    assert_eq!(EntityArchetype::Light.default_damage(), 0);
    assert_eq!(EntityArchetype::Camera.default_damage(), 0);
}

#[test]
fn entity_archetype_display() {
    for arch in EntityArchetype::all() {
        let s = format!("{}", arch);
        assert!(s.contains(arch.name()));
    }
}

#[test]
fn entity_archetype_combatant_has_positive_damage() {
    for arch in EntityArchetype::all() {
        if arch.is_combatant() {
            assert!(arch.default_damage() > 0, "{:?} combatant with 0 damage", arch);
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════════
// PREFAB ACTION
// ═══════════════════════════════════════════════════════════════════════════════════

#[test]
fn prefab_action_names() {
    let e = 1u32;
    assert_eq!(PrefabAction::RevertToOriginal(e).name(), "Revert to Original");
    assert_eq!(PrefabAction::ApplyChangesToFile(e).name(), "Apply Changes");
    assert_eq!(PrefabAction::RevertAllToOriginal(e).name(), "Revert All");
    assert_eq!(PrefabAction::ApplyAllChangesToFile(e).name(), "Apply All Changes");
}

#[test]
fn prefab_action_icons_non_empty() {
    let e = 1u32;
    assert!(!PrefabAction::RevertToOriginal(e).icon().is_empty());
    assert!(!PrefabAction::ApplyChangesToFile(e).icon().is_empty());
    assert!(!PrefabAction::RevertAllToOriginal(e).icon().is_empty());
    assert!(!PrefabAction::ApplyAllChangesToFile(e).icon().is_empty());
}

#[test]
fn prefab_action_is_bulk() {
    let e = 1u32;
    assert!(!PrefabAction::RevertToOriginal(e).is_bulk_action());
    assert!(!PrefabAction::ApplyChangesToFile(e).is_bulk_action());
    assert!(PrefabAction::RevertAllToOriginal(e).is_bulk_action());
    assert!(PrefabAction::ApplyAllChangesToFile(e).is_bulk_action());
}

#[test]
fn prefab_action_display() {
    let e = 1u32;
    let s = format!("{}", PrefabAction::RevertToOriginal(e));
    assert!(s.contains("Revert to Original"));
}

// ═══════════════════════════════════════════════════════════════════════════════════
// VALIDATION SEVERITY
// ═══════════════════════════════════════════════════════════════════════════════════

#[test]
fn validation_severity_all_count() {
    assert_eq!(ValidationSeverity::all().len(), 3);
}

#[test]
fn validation_severity_names() {
    assert_eq!(ValidationSeverity::Error.name(), "Error");
    assert_eq!(ValidationSeverity::Warning.name(), "Warning");
    assert_eq!(ValidationSeverity::Info.name(), "Info");
}

#[test]
fn validation_severity_is_serious() {
    assert!(ValidationSeverity::Error.is_serious());
    assert!(ValidationSeverity::Warning.is_serious());
    assert!(!ValidationSeverity::Info.is_serious());
}

#[test]
fn validation_severity_icons_non_empty() {
    for sev in ValidationSeverity::all() {
        assert!(!sev.icon().is_empty(), "{:?} icon empty", sev);
    }
}

#[test]
fn validation_severity_display() {
    for sev in ValidationSeverity::all() {
        let s = format!("{}", sev);
        assert!(s.contains(sev.name()));
    }
}

// ═══════════════════════════════════════════════════════════════════════════════════
// ENTITY FILTER
// ═══════════════════════════════════════════════════════════════════════════════════

#[test]
fn entity_filter_defaults() {
    let f = EntityFilter::default();
    assert!(f.query.is_empty());
    assert!(f.archetype.is_none());
    assert!(f.team_id.is_none());
    assert!(f.health_range.is_none());
    assert!(!f.favorites_only);
}

// ═══════════════════════════════════════════════════════════════════════════════════
// ENTITY STATS
// ═══════════════════════════════════════════════════════════════════════════════════

#[test]
fn entity_stats_defaults() {
    let s = EntityStats::default();
    assert_eq!(s.total_count, 0);
    assert!(s.by_archetype.is_empty());
    assert_eq!(s.total_health, 0);
    assert!((s.avg_health - 0.0).abs() < f32::EPSILON);
    assert!(s.component_usage.is_empty());
}

// ═══════════════════════════════════════════════════════════════════════════════════
// PERF CATEGORY
// ═══════════════════════════════════════════════════════════════════════════════════

#[test]
fn perf_category_all_count() {
    assert_eq!(PerfCategory::all().len(), 10);
}

#[test]
fn perf_category_names() {
    assert_eq!(PerfCategory::Frame.name(), "Frame");
    assert_eq!(PerfCategory::Cpu.name(), "CPU");
    assert_eq!(PerfCategory::Gpu.name(), "GPU");
    assert_eq!(PerfCategory::Memory.name(), "Memory");
    assert_eq!(PerfCategory::Physics.name(), "Physics");
    assert_eq!(PerfCategory::Ai.name(), "AI");
    assert_eq!(PerfCategory::Rendering.name(), "Rendering");
    assert_eq!(PerfCategory::Audio.name(), "Audio");
    assert_eq!(PerfCategory::Network.name(), "Network");
    assert_eq!(PerfCategory::Scripting.name(), "Scripting");
}

#[test]
fn perf_category_icons_non_empty() {
    for cat in PerfCategory::all() {
        assert!(!cat.icon().is_empty(), "{:?} icon empty", cat);
    }
}

#[test]
fn perf_category_display() {
    for cat in PerfCategory::all() {
        let s = format!("{}", cat);
        assert!(s.contains(cat.name()));
    }
}

// ═══════════════════════════════════════════════════════════════════════════════════
// METRIC UNIT
// ═══════════════════════════════════════════════════════════════════════════════════

#[test]
fn metric_unit_all_count() {
    assert_eq!(MetricUnit::all().len(), 11);
}

#[test]
fn metric_unit_suffix_values() {
    assert_eq!(MetricUnit::Milliseconds.suffix(), "ms");
    assert_eq!(MetricUnit::Microseconds.suffix(), "µs");
    assert_eq!(MetricUnit::Nanoseconds.suffix(), "ns");
    assert_eq!(MetricUnit::Percent.suffix(), "%");
    assert_eq!(MetricUnit::Bytes.suffix(), "B");
    assert_eq!(MetricUnit::Kilobytes.suffix(), "KB");
    assert_eq!(MetricUnit::Megabytes.suffix(), "MB");
    assert_eq!(MetricUnit::Gigabytes.suffix(), "GB");
    assert_eq!(MetricUnit::Count.suffix(), "");
    assert_eq!(MetricUnit::PerSecond.suffix(), "/s");
    assert_eq!(MetricUnit::Fps.suffix(), "FPS");
}

#[test]
fn metric_unit_is_time_unit() {
    assert!(MetricUnit::Milliseconds.is_time_unit());
    assert!(MetricUnit::Microseconds.is_time_unit());
    assert!(MetricUnit::Nanoseconds.is_time_unit());
    assert!(!MetricUnit::Percent.is_time_unit());
    assert!(!MetricUnit::Bytes.is_time_unit());
    assert!(!MetricUnit::Fps.is_time_unit());
}

#[test]
fn metric_unit_is_memory_unit() {
    assert!(MetricUnit::Bytes.is_memory_unit());
    assert!(MetricUnit::Kilobytes.is_memory_unit());
    assert!(MetricUnit::Megabytes.is_memory_unit());
    assert!(MetricUnit::Gigabytes.is_memory_unit());
    assert!(!MetricUnit::Milliseconds.is_memory_unit());
    assert!(!MetricUnit::Percent.is_memory_unit());
    assert!(!MetricUnit::Fps.is_memory_unit());
}

#[test]
fn metric_unit_format_fps() {
    let s = MetricUnit::Fps.format(60.0);
    assert!(s.contains("60"));
    assert!(s.contains("FPS"));
}

#[test]
fn metric_unit_format_percent() {
    let s = MetricUnit::Percent.format(99.5);
    assert!(s.contains("99.5"));
    assert!(s.contains("%"));
}

#[test]
fn metric_unit_format_megabytes() {
    let s = MetricUnit::Megabytes.format(256.0);
    assert!(s.contains("256"));
    assert!(s.contains("MB"));
}

#[test]
fn metric_unit_display_is_suffix() {
    for unit in MetricUnit::all() {
        assert_eq!(format!("{}", unit), unit.suffix());
    }
}

// ═══════════════════════════════════════════════════════════════════════════════════
// PERF METRIC
// ═══════════════════════════════════════════════════════════════════════════════════

#[test]
fn perf_metric_new() {
    let m = PerfMetric::new("Frame Time", PerfCategory::Frame, MetricUnit::Milliseconds);
    assert_eq!(m.name, "Frame Time");
    assert_eq!(m.category, PerfCategory::Frame);
    assert_eq!(m.unit, MetricUnit::Milliseconds);
    assert!((m.value - 0.0).abs() < f64::EPSILON);
    assert!(m.budget.is_none());
    assert!(m.history.is_empty());
}

#[test]
fn perf_metric_with_budget() {
    let m = PerfMetric::new("FPS", PerfCategory::Frame, MetricUnit::Fps).with_budget(16.67);
    assert_eq!(m.budget, Some(16.67));
}

#[test]
fn perf_metric_push() {
    let mut m = PerfMetric::new("test", PerfCategory::Cpu, MetricUnit::Milliseconds);
    m.push(5.0);
    m.push(15.0);
    m.push(10.0);
    assert!((m.value - 10.0).abs() < f64::EPSILON);
    assert_eq!(m.history.len(), 3);
}

#[test]
fn perf_metric_average() {
    let mut m = PerfMetric::new("test", PerfCategory::Cpu, MetricUnit::Milliseconds);
    m.push(10.0);
    m.push(20.0);
    m.push(30.0);
    assert!((m.average() - 20.0).abs() < f64::EPSILON);
}

#[test]
fn perf_metric_average_empty() {
    let m = PerfMetric::new("test", PerfCategory::Cpu, MetricUnit::Milliseconds);
    assert!((m.average() - 0.0).abs() < f64::EPSILON);
}

#[test]
fn perf_metric_min_max() {
    let mut m = PerfMetric::new("test", PerfCategory::Cpu, MetricUnit::Milliseconds);
    m.push(5.0);
    m.push(25.0);
    m.push(10.0);
    assert!((m.min() - 5.0).abs() < f64::EPSILON);
    assert!((m.max() - 25.0).abs() < f64::EPSILON);
}

#[test]
fn perf_metric_is_over_budget() {
    let mut m = PerfMetric::new("test", PerfCategory::Cpu, MetricUnit::Milliseconds).with_budget(16.0);
    m.push(10.0);
    assert!(!m.is_over_budget());
    m.push(20.0);
    assert!(m.is_over_budget());
}

#[test]
fn perf_metric_no_budget_never_over() {
    let mut m = PerfMetric::new("test", PerfCategory::Cpu, MetricUnit::Milliseconds);
    m.push(999.0);
    assert!(!m.is_over_budget());
}

#[test]
fn perf_metric_budget_percent() {
    let mut m = PerfMetric::new("test", PerfCategory::Cpu, MetricUnit::Milliseconds).with_budget(10.0);
    m.push(5.0);
    assert!((m.budget_percent().unwrap() - 50.0).abs() < f64::EPSILON);
}

#[test]
fn perf_metric_budget_percent_none() {
    let m = PerfMetric::new("test", PerfCategory::Cpu, MetricUnit::Milliseconds);
    assert!(m.budget_percent().is_none());
}

#[test]
fn perf_metric_history_max_120() {
    let mut m = PerfMetric::new("test", PerfCategory::Cpu, MetricUnit::Milliseconds);
    for i in 0..200 {
        m.push(i as f64);
    }
    assert_eq!(m.history.len(), 120);
}

// ═══════════════════════════════════════════════════════════════════════════════════
// SUBSYSTEM TIMING
// ═══════════════════════════════════════════════════════════════════════════════════

#[test]
fn subsystem_timing_new() {
    let t = SubsystemTiming::new("Render", 8.0);
    assert_eq!(t.name, "Render");
    assert!((t.budget_ms - 8.0).abs() < f64::EPSILON);
    assert!((t.time_ms - 0.0).abs() < f64::EPSILON);
    assert_eq!(t.call_count, 0);
}

#[test]
fn subsystem_timing_push() {
    let mut t = SubsystemTiming::new("Physics", 4.0);
    t.push(3.0, 10);
    assert!((t.time_ms - 3.0).abs() < f64::EPSILON);
    assert_eq!(t.call_count, 10);
    assert_eq!(t.history.len(), 1);
}

#[test]
fn subsystem_timing_average() {
    let mut t = SubsystemTiming::new("AI", 5.0);
    t.push(2.0, 1);
    t.push(4.0, 1);
    t.push(6.0, 1);
    assert!((t.average() - 4.0).abs() < f64::EPSILON);
}

#[test]
fn subsystem_timing_over_budget() {
    let mut t = SubsystemTiming::new("Audio", 2.0);
    t.push(1.0, 1);
    assert!(!t.is_over_budget());
    t.push(3.0, 1);
    assert!(t.is_over_budget());
}

#[test]
fn subsystem_timing_budget_percent() {
    let mut t = SubsystemTiming::new("Render", 10.0);
    t.push(5.0, 1);
    assert!((t.budget_percent() - 50.0).abs() < f64::EPSILON);
}

#[test]
fn subsystem_timing_budget_percent_zero_budget() {
    let mut t = SubsystemTiming::new("Empty", 0.0);
    t.push(5.0, 1);
    assert!((t.budget_percent() - 0.0).abs() < f64::EPSILON);
}

#[test]
fn subsystem_timing_history_max_60() {
    let mut t = SubsystemTiming::new("Test", 5.0);
    for i in 0..100 {
        t.push(i as f64, 1);
    }
    assert_eq!(t.history.len(), 60);
}

// ═══════════════════════════════════════════════════════════════════════════════════
// MEMORY STATS
// ═══════════════════════════════════════════════════════════════════════════════════

#[test]
fn memory_stats_defaults() {
    let m = MemoryStats::default();
    assert!((m.total_allocated_mb - 0.0).abs() < f64::EPSILON);
    assert!((m.heap_used_mb - 0.0).abs() < f64::EPSILON);
    assert!((m.gpu_used_mb - 0.0).abs() < f64::EPSILON);
    assert_eq!(m.allocations_per_frame, 0);
}

#[test]
fn memory_stats_heap_usage_percent() {
    let m = MemoryStats {
        heap_used_mb: 50.0,
        heap_committed_mb: 100.0,
        ..Default::default()
    };
    assert!((m.heap_usage_percent() - 50.0).abs() < f64::EPSILON);
}

#[test]
fn memory_stats_heap_usage_percent_zero() {
    let m = MemoryStats::default();
    assert!((m.heap_usage_percent() - 0.0).abs() < f64::EPSILON);
}

#[test]
fn memory_stats_gpu_usage_percent() {
    let m = MemoryStats {
        gpu_used_mb: 2048.0,
        gpu_available_mb: 8192.0,
        ..Default::default()
    };
    assert!((m.gpu_usage_percent() - 25.0).abs() < f64::EPSILON);
}

#[test]
fn memory_stats_gpu_usage_percent_zero() {
    let m = MemoryStats::default();
    assert!((m.gpu_usage_percent() - 0.0).abs() < f64::EPSILON);
}

// ═══════════════════════════════════════════════════════════════════════════════════
// GPU STATS
// ═══════════════════════════════════════════════════════════════════════════════════

#[test]
fn gpu_stats_defaults() {
    let g = GpuStats::default();
    assert!((g.frame_time_ms - 0.0).abs() < f64::EPSILON);
    assert_eq!(g.vertex_count, 0);
    assert_eq!(g.triangle_count, 0);
    assert_eq!(g.draw_calls, 0);
    assert_eq!(g.compute_dispatches, 0);
}

// ═══════════════════════════════════════════════════════════════════════════════════
// ALERT SEVERITY
// ═══════════════════════════════════════════════════════════════════════════════════

#[test]
fn alert_severity_all_count() {
    assert_eq!(AlertSeverity::all().len(), 3);
}

#[test]
fn alert_severity_names() {
    assert_eq!(AlertSeverity::Info.name(), "Info");
    assert_eq!(AlertSeverity::Warning.name(), "Warning");
    assert_eq!(AlertSeverity::Critical.name(), "Critical");
}

#[test]
fn alert_severity_is_serious() {
    assert!(!AlertSeverity::Info.is_serious());
    assert!(AlertSeverity::Warning.is_serious());
    assert!(AlertSeverity::Critical.is_serious());
}

#[test]
fn alert_severity_display() {
    for sev in AlertSeverity::all() {
        let s = format!("{}", sev);
        assert!(s.contains(sev.name()));
    }
}
