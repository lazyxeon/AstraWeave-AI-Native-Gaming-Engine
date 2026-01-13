use super::Panel;
use crate::terrain_integration::{all_biome_options, biome_display_name, TerrainState};
use crate::LevelDoc;
use egui::Ui;

pub struct WorldPanel {
    pub terrain_state: TerrainState,
    chunk_radius: i32,
    generation_status: Option<String>,
    auto_regenerate: bool,
}

impl WorldPanel {
    pub fn new() -> Self {
        Self {
            terrain_state: TerrainState::new(),
            chunk_radius: 2,
            generation_status: None,
            auto_regenerate: false,
        }
    }

    pub fn show_with_level(&mut self, ui: &mut Ui, level: &mut LevelDoc) {
        ui.heading("World State");
        ui.separator();

        let old_biome = level.biome.clone();
        let old_seed = level.seed;

        ui.group(|ui| {
            ui.label("Biome");
            
            egui::ComboBox::from_id_salt("biome_selector")
                .selected_text(biome_display_name(&level.biome))
                .show_ui(ui, |ui| {
                    for (value, display) in all_biome_options() {
                        if ui.selectable_label(level.biome == *value, *display).clicked() {
                            level.biome = value.to_string();
                        }
                    }
                });

            ui.label(format!("Current: {}", biome_display_name(&level.biome)));
        });

        ui.add_space(10.0);

        ui.group(|ui| {
            ui.label("Seed");
            ui.add(egui::Slider::new(&mut level.seed, 0..=99999).text("seed"));
            if ui.button("Randomize").clicked() {
                level.seed = rand::random::<u64>() % 100000;
            }
        });

        ui.add_space(10.0);

        ui.group(|ui| {
            ui.label("Terrain Generation");
            
            ui.horizontal(|ui| {
                ui.label("Chunk Radius:");
                ui.add(egui::Slider::new(&mut self.chunk_radius, 1..=5).text("chunks"));
            });

            let chunks_to_generate = (self.chunk_radius * 2 + 1).pow(2);
            ui.label(format!("Will generate {} chunks", chunks_to_generate));

            ui.horizontal(|ui| {
                ui.checkbox(&mut self.auto_regenerate, "Auto-regenerate on change");
            });

            ui.add_space(5.0);

            let generate_clicked = ui.button("Generate Terrain").clicked();
            
            self.terrain_state.configure(level.seed, &level.biome);

            let should_generate = generate_clicked || 
                (self.auto_regenerate && (old_biome != level.biome || old_seed != level.seed));

            if should_generate {
                match self.terrain_state.generate_terrain(self.chunk_radius) {
                    Ok(count) => {
                        self.generation_status = Some(format!(
                            "Generated {} chunks with seed {} and biome {}",
                            count,
                            level.seed,
                            biome_display_name(&level.biome)
                        ));
                    }
                    Err(e) => {
                        self.generation_status = Some(format!("Generation failed: {}", e));
                    }
                }
            }

            if let Some(status) = &self.generation_status {
                ui.add_space(5.0);
                ui.label(status);
            }

            if self.terrain_state.chunk_count() > 0 {
                ui.add_space(5.0);
                ui.label(format!(
                    "Active: {} chunks loaded",
                    self.terrain_state.chunk_count()
                ));
            }
        });

        ui.add_space(10.0);

        ui.group(|ui| {
            ui.label("Time of Day");
            ui.horizontal(|ui| {
                ui.label("Current:");
                ui.text_edit_singleline(&mut level.sky.time_of_day);
            });

            ui.horizontal(|ui| {
                if ui.button("Dawn").clicked() {
                    level.sky.time_of_day = "dawn".to_string();
                }
                if ui.button("Noon").clicked() {
                    level.sky.time_of_day = "noon".to_string();
                }
                if ui.button("Dusk").clicked() {
                    level.sky.time_of_day = "dusk".to_string();
                }
            });
        });

        ui.add_space(10.0);
        
        ui.group(|ui| {
            ui.label("Weather");
            ui.text_edit_singleline(&mut level.sky.weather);
        });
    }

    pub fn terrain_state(&self) -> &TerrainState {
        &self.terrain_state
    }

    pub fn terrain_state_mut(&mut self) -> &mut TerrainState {
        &mut self.terrain_state
    }
}

impl Default for WorldPanel {
    fn default() -> Self {
        Self::new()
    }
}

impl Panel for WorldPanel {
    fn name(&self) -> &str {
        "World"
    }

    fn show(&mut self, _ui: &mut Ui) {
    }
}
