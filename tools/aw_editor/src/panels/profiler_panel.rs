use super::Panel;
use egui::Ui;
use std::collections::VecDeque;

pub struct ProfilerPanel {
    frame_times: VecDeque<f32>,
    fps_samples: VecDeque<f32>,
    memory_samples: VecDeque<usize>,
    max_samples: usize,
    last_update: std::time::Instant,
    update_interval_ms: u64,
    show_frame_graph: bool,
    show_fps_graph: bool,
    show_memory_graph: bool,
    peak_frame_time: f32,
    peak_fps: f32,
    peak_memory_kb: usize,
}

impl ProfilerPanel {
    pub fn new() -> Self {
        Self {
            frame_times: VecDeque::with_capacity(120),
            fps_samples: VecDeque::with_capacity(120),
            memory_samples: VecDeque::with_capacity(120),
            max_samples: 120,
            last_update: std::time::Instant::now(),
            update_interval_ms: 16,
            show_frame_graph: true,
            show_fps_graph: true,
            show_memory_graph: false,
            peak_frame_time: 0.0,
            peak_fps: 0.0,
            peak_memory_kb: 0,
        }
    }

    pub fn push_frame_time(&mut self, frame_time_ms: f32) {
        if self.frame_times.len() >= self.max_samples {
            self.frame_times.pop_front();
        }
        self.frame_times.push_back(frame_time_ms);

        if frame_time_ms > self.peak_frame_time {
            self.peak_frame_time = frame_time_ms;
        }

        let fps = if frame_time_ms > 0.0 {
            1000.0 / frame_time_ms
        } else {
            0.0
        };

        if self.fps_samples.len() >= self.max_samples {
            self.fps_samples.pop_front();
        }
        self.fps_samples.push_back(fps);

        if fps > self.peak_fps {
            self.peak_fps = fps;
        }
    }

    pub fn push_memory_sample(&mut self, memory_kb: usize) {
        if self.memory_samples.len() >= self.max_samples {
            self.memory_samples.pop_front();
        }
        self.memory_samples.push_back(memory_kb);

        if memory_kb > self.peak_memory_kb {
            self.peak_memory_kb = memory_kb;
        }
    }

    pub fn reset_peaks(&mut self) {
        self.peak_frame_time = 0.0;
        self.peak_fps = 0.0;
        self.peak_memory_kb = 0;
    }

    fn avg_frame_time(&self) -> f32 {
        if self.frame_times.is_empty() {
            return 0.0;
        }
        self.frame_times.iter().sum::<f32>() / self.frame_times.len() as f32
    }

    fn avg_fps(&self) -> f32 {
        if self.fps_samples.is_empty() {
            return 0.0;
        }
        self.fps_samples.iter().sum::<f32>() / self.fps_samples.len() as f32
    }

    fn min_fps(&self) -> f32 {
        self.fps_samples
            .iter()
            .cloned()
            .fold(f32::INFINITY, f32::min)
    }

    fn max_fps(&self) -> f32 {
        self.fps_samples.iter().cloned().fold(0.0, f32::max)
    }

    fn draw_graph(
        &self,
        ui: &mut Ui,
        data: &VecDeque<f32>,
        label: &str,
        max_val: f32,
        color: egui::Color32,
    ) {
        let height = 60.0;
        let width = ui.available_width().min(300.0);

        ui.label(label);
        let (response, painter) =
            ui.allocate_painter(egui::vec2(width, height), egui::Sense::hover());
        let rect = response.rect;

        painter.rect_filled(rect, 2.0, egui::Color32::from_gray(30));

        if data.len() < 2 {
            return;
        }

        let max_display = if max_val > 0.0 { max_val } else { 1.0 };
        let step = width / (self.max_samples as f32 - 1.0);

        let points: Vec<egui::Pos2> = data
            .iter()
            .enumerate()
            .map(|(i, &val)| {
                let x = rect.left() + i as f32 * step;
                let normalized = (val / max_display).clamp(0.0, 1.0);
                let y = rect.bottom() - normalized * height;
                egui::pos2(x, y)
            })
            .collect();

        if points.len() >= 2 {
            painter.add(egui::Shape::line(points, egui::Stroke::new(1.5, color)));
        }

        let budget_line_y = rect.bottom() - (16.67 / max_display).clamp(0.0, 1.0) * height;
        if label.contains("Frame") && budget_line_y > rect.top() {
            painter.line_segment(
                [
                    egui::pos2(rect.left(), budget_line_y),
                    egui::pos2(rect.right(), budget_line_y),
                ],
                egui::Stroke::new(1.0, egui::Color32::from_rgb(255, 100, 100)),
            );
        }
    }
}

impl Default for ProfilerPanel {
    fn default() -> Self {
        Self::new()
    }
}

impl Panel for ProfilerPanel {
    fn name(&self) -> &str {
        "Profiler"
    }

    fn show(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.heading("Performance Profiler");
            if ui.small_button("Reset Peaks").clicked() {
                self.reset_peaks();
            }
        });
        ui.separator();

        egui::Grid::new("profiler_stats_grid")
            .num_columns(4)
            .spacing([15.0, 4.0])
            .striped(true)
            .show(ui, |ui| {
                ui.label("");
                ui.label(egui::RichText::new("Current").strong());
                ui.label(egui::RichText::new("Average").strong());
                ui.label(egui::RichText::new("Peak").strong());
                ui.end_row();

                let current_ft = self.frame_times.back().copied().unwrap_or(0.0);
                let avg_ft = self.avg_frame_time();
                ui.label("Frame Time:");
                ui.label(format!("{:.2} ms", current_ft));
                ui.label(format!("{:.2} ms", avg_ft));
                ui.label(format!("{:.2} ms", self.peak_frame_time));
                ui.end_row();

                let current_fps = self.fps_samples.back().copied().unwrap_or(0.0);
                let avg_fps = self.avg_fps();
                ui.label("FPS:");
                ui.label(format!("{:.1}", current_fps));
                ui.label(format!("{:.1}", avg_fps));
                ui.label(format!("{:.1}", self.peak_fps));
                ui.end_row();

                if !self.memory_samples.is_empty() {
                    let current_mem = self.memory_samples.back().copied().unwrap_or(0);
                    let avg_mem: usize =
                        self.memory_samples.iter().sum::<usize>() / self.memory_samples.len();
                    ui.label("Memory:");
                    ui.label(format!("{} KB", current_mem));
                    ui.label(format!("{} KB", avg_mem));
                    ui.label(format!("{} KB", self.peak_memory_kb));
                    ui.end_row();
                }
            });

        ui.add_space(8.0);
        ui.horizontal(|ui| {
            ui.checkbox(&mut self.show_frame_graph, "Frame Time");
            ui.checkbox(&mut self.show_fps_graph, "FPS");
            ui.checkbox(&mut self.show_memory_graph, "Memory");
        });

        if self.show_frame_graph && !self.frame_times.is_empty() {
            ui.add_space(4.0);
            let max_ft = self
                .frame_times
                .iter()
                .cloned()
                .fold(0.0f32, f32::max)
                .max(33.33);
            self.draw_graph(
                ui,
                &self.frame_times,
                "Frame Time (ms) - Red line = 60 FPS budget",
                max_ft,
                egui::Color32::from_rgb(100, 200, 255),
            );
        }

        if self.show_fps_graph && !self.fps_samples.is_empty() {
            ui.add_space(4.0);
            let max_fps = self.max_fps().max(120.0);
            self.draw_graph(
                ui,
                &self.fps_samples,
                "FPS",
                max_fps,
                egui::Color32::from_rgb(100, 255, 100),
            );
        }

        if self.show_memory_graph && !self.memory_samples.is_empty() {
            ui.add_space(4.0);
            let mem_floats: VecDeque<f32> = self.memory_samples.iter().map(|&m| m as f32).collect();
            let max_mem = self.peak_memory_kb as f32;
            self.draw_graph(
                ui,
                &mem_floats,
                "Memory (KB)",
                max_mem.max(1024.0),
                egui::Color32::from_rgb(255, 180, 100),
            );
        }

        ui.add_space(8.0);
        ui.separator();
        ui.horizontal(|ui| {
            ui.label("Min FPS:");
            ui.label(format!("{:.1}", self.min_fps()));
            ui.separator();
            ui.label("Max FPS:");
            ui.label(format!("{:.1}", self.max_fps()));
            ui.separator();
            ui.label("Samples:");
            ui.label(format!("{}", self.frame_times.len()));
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_profiler_panel_creation() {
        let panel = ProfilerPanel::new();
        assert!(panel.frame_times.is_empty());
        assert_eq!(panel.max_samples, 120);
    }

    #[test]
    fn test_push_frame_time() {
        let mut panel = ProfilerPanel::new();
        panel.push_frame_time(16.67);
        panel.push_frame_time(8.33);
        assert_eq!(panel.frame_times.len(), 2);
        assert_eq!(panel.fps_samples.len(), 2);
        assert!((panel.peak_frame_time - 16.67).abs() < 0.01);
    }

    #[test]
    fn test_avg_calculations() {
        let mut panel = ProfilerPanel::new();
        panel.push_frame_time(10.0);
        panel.push_frame_time(20.0);
        assert!((panel.avg_frame_time() - 15.0).abs() < 0.01);
    }

    #[test]
    fn test_reset_peaks() {
        let mut panel = ProfilerPanel::new();
        panel.push_frame_time(100.0);
        panel.push_memory_sample(50000);
        panel.reset_peaks();
        assert_eq!(panel.peak_frame_time, 0.0);
        assert_eq!(panel.peak_memory_kb, 0);
    }

    #[test]
    fn test_max_samples_limit() {
        let mut panel = ProfilerPanel::new();
        for i in 0..150 {
            panel.push_frame_time(i as f32);
        }
        assert_eq!(panel.frame_times.len(), 120);
    }
}
