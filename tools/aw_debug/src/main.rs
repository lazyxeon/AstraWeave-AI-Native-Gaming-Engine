use aw_debug::{watch_reload_signal, watch_scripts, ChromeTraceGuard, PerfHud};
use eframe::egui;
use std::path::PathBuf;

struct DebugApp {
    hud: PerfHud,
    system_times: Vec<(String, f32)>,
    entity_count: u32,
}

impl DebugApp {
    fn new() -> Self {
        let mut hud = PerfHud::new();

        // Example system times
        let system_times = vec![
            ("physics_update".into(), 1.2),
            ("ai_planning".into(), 2.5),
            ("rendering".into(), 4.1),
            ("animation".into(), 0.8),
            ("audio".into(), 0.3),
        ];

        hud.systems_snapshot = system_times.clone();
        hud.entity_count = 120;

        // Add some example events
        hud.log_event("system", "Debug HUD initialized");
        hud.log_event("ai", "AI Director loaded");
        hud.log_event("physics", "Physics world initialized");

        Self {
            hud,
            system_times,
            entity_count: 120,
        }
    }

    fn simulate_frame(&mut self) {
        // Simulate some changing system times
        for (_, time) in &mut self.system_times {
            *time += (rand::random::<f32>() - 0.5) * 0.2;
            *time = time.max(0.1);
        }

        self.hud.systems_snapshot = self.system_times.clone();

        // Occasionally log events
        if rand::random::<f32>() < 0.05 {
            let events = [
                ("ai", "Path recalculated"),
                ("physics", "Collision detected"),
                ("script", "Rhai script executed"),
                ("render", "Shader recompiled"),
                ("audio", "Sound effect played"),
            ];
            let (category, msg) = events[rand::random::<usize>() % events.len()];
            self.hud.log_event(category, msg);
        }

        // Update entity count occasionally
        if rand::random::<f32>() < 0.02 {
            self.entity_count =
                (self.entity_count as f32 * (1.0 + (rand::random::<f32>() - 0.5) * 0.1)) as u32;
            self.hud.entity_count = self.entity_count;
        }
    }
}

impl eframe::App for DebugApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Simulate a frame update
        self.simulate_frame();
        self.hud.frame();

        egui::Window::new("AstraWeave Debug Tools")
            .default_width(400.0)
            .show(ctx, |ui| {
                self.hud.ui(ui);
            });

        // Request continuous repaints
        ctx.request_repaint();
    }
}

fn main() -> Result<(), eframe::Error> {
    // Initialize Chrome tracing (optional)
    let _trace_guard = ChromeTraceGuard::init("astraweave_trace.json");

    // Watch for script changes (example)
    let content_dir = PathBuf::from("content");
    std::fs::create_dir_all(&content_dir).ok();

    let _script_watcher = watch_scripts(content_dir.join("encounters"), || {
        println!("Script changed, reloading...");
        // In a real app, you would reload your scripts here
    })
    .ok();

    let _reload_watcher = watch_reload_signal(content_dir.clone(), || {
        println!("Reload signal detected, reloading level...");
        // In a real app, you would reload your level here
    })
    .ok();

    // Start the debug UI
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0])
            .with_title("AstraWeave Debug Tools"),
        ..Default::default()
    };

    eframe::run_native(
        "AstraWeave Debug Tools",
        options,
        Box::new(|_cc| Box::new(DebugApp::new())),
    )
}
