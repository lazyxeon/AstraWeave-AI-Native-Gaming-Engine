use anyhow::Result;
use egui::{self, Color32};
use egui_plot::{Line, Plot, PlotPoints};
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use std::{
    path::PathBuf,
    time::Instant,
};
use tracing_subscriber::prelude::*;

/// Simple perf HUD state
pub struct PerfHud {
    frame_times: Vec<f32>,
    max_samples: usize,
    last_frame: Instant,
    pub fps: f32,
    pub systems_snapshot: Vec<(String, f32)>, // (system name, ms)
    pub entity_count: u32,
    pub event_log: EventLog,
}

impl PerfHud {
    pub fn new() -> Self {
        Self {
            frame_times: vec![],
            max_samples: 240,
            last_frame: Instant::now(),
            fps: 0.0,
            systems_snapshot: vec![],
            entity_count: 0,
            event_log: EventLog::new(100),
        }
    }

    pub fn frame(&mut self) {
        let dt = self.last_frame.elapsed().as_secs_f32();
        self.last_frame = Instant::now();
        self.frame_times.push(dt);
        if self.frame_times.len() > self.max_samples {
            self.frame_times.remove(0);
        }
        let avg =
            self.frame_times.iter().cloned().sum::<f32>() / self.frame_times.len().max(1) as f32;
        self.fps = if avg > 0.0 { 1.0 / avg } else { 0.0 };
    }

    pub fn ui(&mut self, ui: &mut egui::Ui) {
        ui.heading("AstraWeave Debug HUD");
        ui.label(format!("FPS: {:.1}", self.fps));
        ui.label(format!("Entities: {}", self.entity_count));

        Plot::new("ft_plot").view_aspect(2.5).show(ui, |plot_ui| {
            let ys: Vec<[f64; 2]> = self
                .frame_times
                .iter()
                .enumerate()
                .map(|(i, dt)| [i as f64, (*dt * 1000.0) as f64])
                .collect();
            let line = Line::new(PlotPoints::new(ys)).color(Color32::from_rgb(100, 200, 100));
            plot_ui.line(line);
        });

        ui.separator();
        ui.collapsing("Systems (ms)", |ui| {
            for (name, ms) in &self.systems_snapshot {
                ui.horizontal(|ui| {
                    ui.label(format!("{name:24}"));
                    ui.label(format!("{ms:6.2} ms"));
                });
            }
        });

        ui.separator();
        ui.collapsing("Event Log", |ui| {
            self.event_log.ui(ui);
        });
    }

    pub fn log_event(&mut self, category: &str, message: &str) {
        self.event_log.add(category, message);
    }
}

/// Event log for tracking system events
pub struct EventLog {
    events: Vec<LogEvent>,
    max_events: usize,
}

struct LogEvent {
    timestamp: Instant,
    category: String,
    message: String,
}

impl EventLog {
    pub fn new(max_events: usize) -> Self {
        Self {
            events: Vec::with_capacity(max_events),
            max_events,
        }
    }

    pub fn add(&mut self, category: &str, message: &str) {
        if self.events.len() >= self.max_events {
            self.events.remove(0);
        }

        self.events.push(LogEvent {
            timestamp: Instant::now(),
            category: category.to_string(),
            message: message.to_string(),
        });
    }

    pub fn ui(&self, ui: &mut egui::Ui) {
        for event in self.events.iter().rev() {
            let elapsed = event.timestamp.elapsed();
            let time_str = format!("{:.2}s ago", elapsed.as_secs_f32());

            ui.horizontal(|ui| {
                ui.label(time_str);
                ui.colored_label(
                    match event.category.as_str() {
                        "error" => Color32::RED,
                        "warning" => Color32::YELLOW,
                        "ai" => Color32::LIGHT_BLUE,
                        "physics" => Color32::LIGHT_GREEN,
                        "script" => Color32::GOLD,
                        _ => Color32::WHITE,
                    },
                    &event.category,
                );
                ui.label(&event.message);
            });
        }

        if self.events.is_empty() {
            ui.label("No events recorded");
        }
    }
}

/// Chrome trace producer
pub struct ChromeTraceGuard {
    _guard: tracing_chrome::FlushGuard,
}

impl ChromeTraceGuard {
    pub fn init(path: &str) -> Self {
        let (layer, guard) = tracing_chrome::ChromeLayerBuilder::new().file(path).build();
        tracing_subscriber::registry().with(layer).init();
        Self { _guard: guard }
    }
}

/// Watches a directory for Rhai/script changes and calls your callback.
pub fn watch_scripts(
    dir: PathBuf,
    on_change: impl Fn() + Send + 'static,
) -> Result<RecommendedWatcher> {
    let mut watcher = notify::recommended_watcher(move |res| {
        if let Ok(_ev) = res {
            on_change();
        }
    })?;
    watcher.watch(&dir, RecursiveMode::Recursive)?;
    Ok(watcher)
}

/// Watches for the reload.signal file and calls your callback.
pub fn watch_reload_signal(
    content_dir: PathBuf,
    on_reload: impl Fn() + Send + 'static,
) -> Result<RecommendedWatcher> {
    let mut watcher = notify::recommended_watcher(move |res| {
        if let Ok(event) = res {
            if let notify::Event {
                kind: notify::EventKind::Create(_) | notify::EventKind::Modify(_),
                ..
            } = event
            {
                on_reload();
            }
        }
    })?;

    // Watch the content directory for the reload.signal file
    watcher.watch(&content_dir, RecursiveMode::NonRecursive)?;

    Ok(watcher)
}
