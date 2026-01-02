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
