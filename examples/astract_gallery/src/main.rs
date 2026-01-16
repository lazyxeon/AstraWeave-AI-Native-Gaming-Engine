//! Astract Widget Gallery
//!
//! Comprehensive showcase of all Astract UI widgets with interactive demos

use astract::prelude::egui::*;

mod advanced_tab;
mod animation_tab;
mod charts_tab;
mod graphs_tab;

use advanced_tab::AdvancedTab;
use animation_tab::AnimationTab;
use charts_tab::ChartsTab;
use graphs_tab::GraphsTab;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_title("Astract Widget Gallery"),
        ..Default::default()
    };

    eframe::run_native(
        "Astract Widget Gallery",
        options,
        Box::new(|_cc| Ok(Box::new(GalleryApp::default()))),
    )
}

#[derive(Default)]
struct GalleryApp {
    selected_tab: GalleryTab,
    charts_tab: ChartsTab,
    advanced_tab: AdvancedTab,
    graphs_tab: GraphsTab,
    animation_tab: AnimationTab,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GalleryTab {
    Charts,
    Advanced,
    Graphs,
    Animation,
}

impl Default for GalleryTab {
    fn default() -> Self {
        Self::Charts
    }
}

impl eframe::App for GalleryApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        // Request continuous repaint for animations
        ctx.request_repaint();

        // Top bar with tabs
        TopBottomPanel::top("top_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("🎨 Astract Widget Gallery");
                ui.separator();

                ui.selectable_value(&mut self.selected_tab, GalleryTab::Charts, "📊 Charts");
                ui.selectable_value(&mut self.selected_tab, GalleryTab::Advanced, "🎨 Advanced");
                ui.selectable_value(&mut self.selected_tab, GalleryTab::Graphs, "🕸️ Graphs");
                ui.selectable_value(
                    &mut self.selected_tab,
                    GalleryTab::Animation,
                    "🎬 Animation",
                );
            });
        });

        // Main content area
        CentralPanel::default().show(ctx, |ui| {
            ScrollArea::vertical().show(ui, |ui| match self.selected_tab {
                GalleryTab::Charts => self.charts_tab.show(ui),
                GalleryTab::Advanced => self.advanced_tab.show(ui),
                GalleryTab::Graphs => self.graphs_tab.show(ui),
                GalleryTab::Animation => self.animation_tab.show(ctx),
            });
        });

        // Bottom status bar
        TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Astract v0.1.0");
                ui.separator();
                ui.label(format!("FPS: {:.1}", ctx.input(|i| 1.0 / i.stable_dt)));
                ui.separator();
                ui.hyperlink_to("📚 Documentation", "https://github.com/lazyxeon/AstraWeave");
            });
        });
    }
}
