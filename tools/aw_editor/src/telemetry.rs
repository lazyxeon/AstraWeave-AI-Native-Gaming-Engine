use crate::interaction::{GizmoCommitMetadata, GizmoMeasurement, GizmoOperationKind};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, MutexGuard, OnceLock};
use tracing::info;

/// Structured telemetry event stream for aw_editor interactions.
#[derive(Debug, Clone, PartialEq)]
pub enum EditorTelemetryEvent {
    SelectionChanged {
        primary: Option<u32>,
        count: usize,
    },
    GridSettingsChanged {
        grid_enabled: bool,
        snap_size: f32,
        angle_enabled: bool,
        angle_increment: f32,
    },
    GizmoStarted {
        entity: u32,
        operation: GizmoOperationKind,
    },
    GizmoCommitted {
        entity: u32,
        operation: GizmoOperationKind,
        measurement: GizmoMeasurement,
        constraint: Option<String>,
    },
    GizmoCancelled {
        entity: u32,
        operation: GizmoOperationKind,
    },
}

impl From<GizmoCommitMetadata> for EditorTelemetryEvent {
    fn from(meta: GizmoCommitMetadata) -> Self {
        Self::GizmoCommitted {
            entity: meta.entity,
            operation: meta.operation,
            measurement: meta.measurement,
            constraint: meta.constraint,
        }
    }
}

static CAPTURE_ENABLED: AtomicBool = AtomicBool::new(false);
static CAPTURED_EVENTS: OnceLock<Mutex<Vec<EditorTelemetryEvent>>> = OnceLock::new();
static CAPTURE_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

fn storage() -> &'static Mutex<Vec<EditorTelemetryEvent>> {
    CAPTURED_EVENTS.get_or_init(|| Mutex::new(Vec::new()))
}

fn log_event(event: &EditorTelemetryEvent) {
    match event {
        EditorTelemetryEvent::SelectionChanged { primary, count } => {
            info!(target: "aw_editor::telemetry", primary = primary.unwrap_or_default(), count, "selection_changed");
        }
        EditorTelemetryEvent::GridSettingsChanged {
            grid_enabled,
            snap_size,
            angle_enabled,
            angle_increment,
        } => {
            info!(
                target: "aw_editor::telemetry",
                grid_enabled,
                snap_size,
                angle_enabled,
                angle_increment,
                "grid_settings_changed"
            );
        }
        EditorTelemetryEvent::GizmoStarted { entity, operation } => {
            info!(target: "aw_editor::telemetry", entity, operation = ?operation, "gizmo_started");
        }
        EditorTelemetryEvent::GizmoCommitted {
            entity,
            operation,
            measurement,
            constraint,
        } => {
            info!(
                target: "aw_editor::telemetry",
                entity,
                operation = ?operation,
                measurement = ?measurement,
                constraint = constraint.as_deref().unwrap_or(""),
                "gizmo_committed"
            );
        }
        EditorTelemetryEvent::GizmoCancelled { entity, operation } => {
            info!(target: "aw_editor::telemetry", entity, operation = ?operation, "gizmo_cancelled");
        }
    }
}

/// Record a telemetry event (always logged, optionally captured in tests).
pub fn record(event: EditorTelemetryEvent) {
    log_event(&event);

    if CAPTURE_ENABLED.load(Ordering::Relaxed) {
        if let Ok(mut guard) = storage().lock() {
            guard.push(event);
        }
    }
}

/// Guard that enables capture of telemetry events for deterministic testing.
pub struct TelemetryCaptureGuard {
    _lock: MutexGuard<'static, ()>,
}

impl Drop for TelemetryCaptureGuard {
    fn drop(&mut self) {
        CAPTURE_ENABLED.store(false, Ordering::SeqCst);
    }
}

/// Enable telemetry capture, clearing any previous buffer.
pub fn enable_capture() -> TelemetryCaptureGuard {
    let lock = CAPTURE_LOCK
        .get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    CAPTURE_ENABLED.store(true, Ordering::SeqCst);
    if let Ok(mut guard) = storage().lock() {
        guard.clear();
    }
    TelemetryCaptureGuard { _lock: lock }
}

/// Drain captured telemetry events (used by headless harness tests).
pub fn drain_captured_events() -> Vec<EditorTelemetryEvent> {
    if let Some(storage) = CAPTURED_EVENTS.get() {
        if let Ok(mut guard) = storage.lock() {
            return guard.drain(..).collect();
        }
    }
    Vec::new()
}

/// Get a snapshot of captured events without draining them.
pub fn peek_captured_events() -> Vec<EditorTelemetryEvent> {
    if let Some(storage) = CAPTURED_EVENTS.get() {
        if let Ok(guard) = storage.lock() {
            return guard.clone();
        }
    }
    Vec::new()
}

/// Get count of captured events by type.
pub fn count_by_type() -> TelemetryStats {
    let events = peek_captured_events();
    let mut stats = TelemetryStats::default();

    for event in &events {
        match event {
            EditorTelemetryEvent::SelectionChanged { .. } => stats.selection_changes += 1,
            EditorTelemetryEvent::GridSettingsChanged { .. } => stats.grid_changes += 1,
            EditorTelemetryEvent::GizmoStarted { .. } => stats.gizmo_starts += 1,
            EditorTelemetryEvent::GizmoCommitted { .. } => stats.gizmo_commits += 1,
            EditorTelemetryEvent::GizmoCancelled { .. } => stats.gizmo_cancels += 1,
        }
    }

    stats
}

/// Filter captured events by operation kind.
pub fn filter_gizmo_events(kind: GizmoOperationKind) -> Vec<EditorTelemetryEvent> {
    peek_captured_events()
        .into_iter()
        .filter(|e| match e {
            EditorTelemetryEvent::GizmoStarted { operation, .. } => *operation == kind,
            EditorTelemetryEvent::GizmoCommitted { operation, .. } => *operation == kind,
            EditorTelemetryEvent::GizmoCancelled { operation, .. } => *operation == kind,
            _ => false,
        })
        .collect()
}

/// Telemetry statistics.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct TelemetryStats {
    pub selection_changes: usize,
    pub grid_changes: usize,
    pub gizmo_starts: usize,
    pub gizmo_commits: usize,
    pub gizmo_cancels: usize,
}

impl TelemetryStats {
    /// Total number of events
    pub fn total(&self) -> usize {
        self.selection_changes
            + self.grid_changes
            + self.gizmo_starts
            + self.gizmo_commits
            + self.gizmo_cancels
    }

    /// Gizmo commit success rate (commits / starts)
    pub fn gizmo_success_rate(&self) -> f32 {
        if self.gizmo_starts == 0 {
            0.0
        } else {
            self.gizmo_commits as f32 / self.gizmo_starts as f32
        }
    }

    /// Gizmo cancel rate (cancels / starts)
    pub fn gizmo_cancel_rate(&self) -> f32 {
        if self.gizmo_starts == 0 {
            0.0
        } else {
            self.gizmo_cancels as f32 / self.gizmo_starts as f32
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interaction::GizmoOperationKind;
    use astraweave_core::IVec2;

    #[test]
    fn test_telemetry_stats_default() {
        let stats = TelemetryStats::default();
        assert_eq!(stats.selection_changes, 0);
        assert_eq!(stats.grid_changes, 0);
        assert_eq!(stats.gizmo_starts, 0);
        assert_eq!(stats.gizmo_commits, 0);
        assert_eq!(stats.gizmo_cancels, 0);
    }

    #[test]
    fn test_telemetry_stats_total() {
        let stats = TelemetryStats {
            selection_changes: 2,
            grid_changes: 3,
            gizmo_starts: 5,
            gizmo_commits: 4,
            gizmo_cancels: 1,
        };
        assert_eq!(stats.total(), 15);
    }

    #[test]
    fn test_telemetry_stats_gizmo_success_rate_zero_starts() {
        let stats = TelemetryStats::default();
        assert_eq!(stats.gizmo_success_rate(), 0.0);
    }

    #[test]
    fn test_telemetry_stats_gizmo_success_rate() {
        let stats = TelemetryStats {
            gizmo_starts: 10,
            gizmo_commits: 8,
            gizmo_cancels: 2,
            ..Default::default()
        };
        assert!((stats.gizmo_success_rate() - 0.8).abs() < 0.001);
    }

    #[test]
    fn test_telemetry_stats_gizmo_cancel_rate() {
        let stats = TelemetryStats {
            gizmo_starts: 10,
            gizmo_commits: 8,
            gizmo_cancels: 2,
            ..Default::default()
        };
        assert!((stats.gizmo_cancel_rate() - 0.2).abs() < 0.001);
    }

    #[test]
    fn test_telemetry_stats_gizmo_cancel_rate_zero_starts() {
        let stats = TelemetryStats::default();
        assert_eq!(stats.gizmo_cancel_rate(), 0.0);
    }

    #[test]
    fn test_editor_telemetry_event_selection_changed() {
        let event = EditorTelemetryEvent::SelectionChanged {
            primary: Some(42),
            count: 3,
        };
        match event {
            EditorTelemetryEvent::SelectionChanged { primary, count } => {
                assert_eq!(primary, Some(42));
                assert_eq!(count, 3);
            }
            _ => panic!("Wrong event type"),
        }
    }

    #[test]
    fn test_editor_telemetry_event_grid_settings_changed() {
        let event = EditorTelemetryEvent::GridSettingsChanged {
            grid_enabled: true,
            snap_size: 1.0,
            angle_enabled: true,
            angle_increment: 15.0,
        };
        match event {
            EditorTelemetryEvent::GridSettingsChanged {
                grid_enabled,
                snap_size,
                angle_enabled,
                angle_increment,
            } => {
                assert!(grid_enabled);
                assert_eq!(snap_size, 1.0);
                assert!(angle_enabled);
                assert_eq!(angle_increment, 15.0);
            }
            _ => panic!("Wrong event type"),
        }
    }

    #[test]
    fn test_editor_telemetry_event_gizmo_started() {
        let event = EditorTelemetryEvent::GizmoStarted {
            entity: 1,
            operation: GizmoOperationKind::Translate,
        };
        match event {
            EditorTelemetryEvent::GizmoStarted { entity, operation } => {
                assert_eq!(entity, 1);
                assert_eq!(operation, GizmoOperationKind::Translate);
            }
            _ => panic!("Wrong event type"),
        }
    }

    #[test]
    fn test_editor_telemetry_event_from_gizmo_commit_metadata() {
        let meta = GizmoCommitMetadata {
            entity: 5,
            operation: GizmoOperationKind::Scale,
            measurement: GizmoMeasurement::Scale { from: 1.0, to: 2.0 },
            constraint: Some("uniform".to_string()),
        };
        let event: EditorTelemetryEvent = meta.into();
        match event {
            EditorTelemetryEvent::GizmoCommitted {
                entity,
                operation,
                measurement,
                constraint,
            } => {
                assert_eq!(entity, 5);
                assert_eq!(operation, GizmoOperationKind::Scale);
                assert_eq!(measurement, GizmoMeasurement::Scale { from: 1.0, to: 2.0 });
                assert_eq!(constraint, Some("uniform".to_string()));
            }
            _ => panic!("Wrong event type"),
        }
    }

    #[test]
    fn test_capture_and_drain() {
        let _guard = enable_capture();
        
        record(EditorTelemetryEvent::SelectionChanged {
            primary: Some(1),
            count: 1,
        });
        record(EditorTelemetryEvent::SelectionChanged {
            primary: Some(2),
            count: 2,
        });

        let events = drain_captured_events();
        assert_eq!(events.len(), 2);
        
        // After drain, should be empty
        let empty = drain_captured_events();
        assert!(empty.is_empty());
    }

    #[test]
    fn test_peek_captured_events() {
        let _guard = enable_capture();
        
        record(EditorTelemetryEvent::GridSettingsChanged {
            grid_enabled: true,
            snap_size: 1.0,
            angle_enabled: false,
            angle_increment: 0.0,
        });

        let peeked1 = peek_captured_events();
        let peeked2 = peek_captured_events();
        
        // Peek should not drain
        assert_eq!(peeked1.len(), 1);
        assert_eq!(peeked2.len(), 1);
    }

    #[test]
    fn test_count_by_type() {
        let _guard = enable_capture();
        
        record(EditorTelemetryEvent::SelectionChanged { primary: None, count: 0 });
        record(EditorTelemetryEvent::SelectionChanged { primary: None, count: 0 });
        record(EditorTelemetryEvent::GizmoStarted { entity: 1, operation: GizmoOperationKind::Translate });
        record(EditorTelemetryEvent::GizmoCommitted { 
            entity: 1, 
            operation: GizmoOperationKind::Translate,
            measurement: GizmoMeasurement::Translate { from: IVec2::new(0, 0), to: IVec2::new(1, 1) },
            constraint: None,
        });

        let stats = count_by_type();
        assert_eq!(stats.selection_changes, 2);
        assert_eq!(stats.gizmo_starts, 1);
        assert_eq!(stats.gizmo_commits, 1);
        assert_eq!(stats.gizmo_cancels, 0);
    }

    #[test]
    fn test_filter_gizmo_events() {
        let _guard = enable_capture();
        
        record(EditorTelemetryEvent::GizmoStarted { entity: 1, operation: GizmoOperationKind::Translate });
        record(EditorTelemetryEvent::GizmoStarted { entity: 2, operation: GizmoOperationKind::Rotate });
        record(EditorTelemetryEvent::GizmoStarted { entity: 3, operation: GizmoOperationKind::Translate });
        record(EditorTelemetryEvent::SelectionChanged { primary: None, count: 0 });

        let translate_events = filter_gizmo_events(GizmoOperationKind::Translate);
        assert_eq!(translate_events.len(), 2);
        
        let rotate_events = filter_gizmo_events(GizmoOperationKind::Rotate);
        assert_eq!(rotate_events.len(), 1);
        
        let scale_events = filter_gizmo_events(GizmoOperationKind::Scale);
        assert!(scale_events.is_empty());
    }
}
