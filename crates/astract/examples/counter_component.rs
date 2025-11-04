// Example: Counter component using Astract hooks
//
// Demonstrates:
// - use_state hook for state management
// - Component trait implementation
// - RSX macro with callbacks
// - Stateful UI in egui

use astract::prelude::*;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([320.0, 240.0])
            .with_title("Astract Counter Example"),
        ..Default::default()
    };

    eframe::run_simple_native("counter", options, move |ctx, _frame| {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Counter Component Example");
            ui.separator();

            // Use the use_state hook for counter
            let (count, set_count) = use_state(ui, "counter", 0);

            ui.horizontal(|ui| {
                if ui.button("➖ Decrement").clicked() {
                    set_count.set(ui, count - 1);
                }

                ui.label(format!("Count: {}", count));

                if ui.button("Increment ➕").clicked() {
                    set_count.set(ui, count + 1);
                }
            });

            ui.separator();

            if ui.button("Reset").clicked() {
                set_count.set(ui, 0);
            }

            // Memoized expensive calculation
            let squared = use_memo(ui, "squared", count, |c| c * c);
            ui.label(format!("Count² = {}", squared));

            // Effect example (would log on change in real app)
            use_effect(ui, "count_effect", count, |c| {
                // In a real app, this might trigger side effects
                // like logging or API calls
                let _ = c; // Use value to avoid warning
            });
        });
    })
}
