/// Menu UI implementations (main menu, pause menu, settings menu)
///
/// Day 3 enhancements:
/// - Hover effects with color transitions
/// - Visual feedback on button states
/// - Smoother animations
/// - Better spacing and visual hierarchy
use super::menu::MenuAction;

/// Create a styled button with hover effects
fn styled_button(ui: &mut egui::Ui, text: &str, size: egui::Vec2, highlight: bool) -> egui::Response {
    let base_color = if highlight {
        egui::Color32::from_rgb(80, 180, 80)
    } else {
        egui::Color32::from_rgb(60, 60, 80)
    };

    let hover_color = if highlight {
        egui::Color32::from_rgb(100, 220, 100)
    } else {
        egui::Color32::from_rgb(80, 120, 180)
    };

    let text_color = egui::Color32::WHITE;

    ui.scope(|ui| {
        let style = ui.style_mut();
        style.visuals.widgets.inactive.weak_bg_fill = base_color;
        style.visuals.widgets.hovered.weak_bg_fill = hover_color;
        style.visuals.widgets.active.weak_bg_fill = hover_color;
        
        ui.add_sized(
            size,
            egui::Button::new(egui::RichText::new(text).size(20.0).color(text_color))
                .corner_radius(8.0),
        )
    })
    .inner
}

/// Show main menu UI
pub fn show_main_menu(ctx: &egui::Context) -> MenuAction {
    let mut action = MenuAction::None;

    // Full-screen dark background
    egui::Area::new(egui::Id::new("main_menu_bg"))
        .fixed_pos(egui::pos2(0.0, 0.0))
        .show(ctx, |ui| {
            let screen_rect = ctx.screen_rect();
            ui.painter().rect_filled(
                screen_rect,
                0.0,
                egui::Color32::from_rgba_unmultiplied(0, 0, 0, 220),
            );
        });

    // Centered menu window
    egui::Window::new("main_menu")
        .title_bar(false)
        .resizable(false)
        .fixed_size(egui::vec2(400.0, 500.0))
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(40.0);

                // Title
                ui.label(
                    egui::RichText::new("ASTRAWEAVE")
                        .size(56.0)
                        .color(egui::Color32::from_rgb(100, 200, 255)),
                );

                ui.add_space(10.0);

                ui.label(
                    egui::RichText::new("AI-Native Game Engine")
                        .size(18.0)
                        .color(egui::Color32::GRAY),
                );

                ui.add_space(60.0);

                // Buttons with hover effects
                if styled_button(ui, "New Game", egui::vec2(300.0, 50.0), false)
                    .clicked()
                {
                    action = MenuAction::NewGame;
                }

                ui.add_space(15.0);

                if styled_button(ui, "Load Game", egui::vec2(300.0, 50.0), false)
                    .clicked()
                {
                    action = MenuAction::LoadGame;
                }

                ui.add_space(15.0);

                if styled_button(ui, "Settings", egui::vec2(300.0, 50.0), false)
                    .clicked()
                {
                    action = MenuAction::Settings;
                }

                ui.add_space(15.0);

                if styled_button(ui, "Quit", egui::vec2(300.0, 50.0), false)
                    .clicked()
                {
                    action = MenuAction::Quit;
                }

                ui.add_space(40.0);

                // Version info
                ui.label(
                    egui::RichText::new(format!("Version {}", env!("CARGO_PKG_VERSION")))
                        .size(12.0)
                        .color(egui::Color32::DARK_GRAY),
                );
            });
        });

    action
}

/// Show pause menu UI
pub fn show_pause_menu(ctx: &egui::Context) -> MenuAction {
    let mut action = MenuAction::None;

    // Full-screen very dark background (game is paused)
    egui::Area::new(egui::Id::new("pause_menu_bg"))
        .fixed_pos(egui::pos2(0.0, 0.0))
        .show(ctx, |ui| {
            let screen_rect = ctx.screen_rect();
            ui.painter().rect_filled(
                screen_rect,
                0.0,
                egui::Color32::from_rgba_unmultiplied(0, 0, 0, 180),
            );
        });

    // Centered menu window
    egui::Window::new("pause_menu")
        .title_bar(false)
        .resizable(false)
        .fixed_size(egui::vec2(400.0, 450.0))
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(40.0);

                // Title
                ui.label(
                    egui::RichText::new("PAUSED")
                        .size(48.0)
                        .color(egui::Color32::from_rgb(255, 200, 100)),
                );

                ui.add_space(50.0);

                // Resume button (highlighted with hover effect)
                if styled_button(ui, "Resume", egui::vec2(300.0, 50.0), true)
                    .clicked()
                {
                    action = MenuAction::Resume;
                }

                ui.add_space(15.0);

                if styled_button(ui, "Save Game", egui::vec2(300.0, 50.0), false)
                    .clicked()
                {
                    action = MenuAction::SaveGame;
                }

                ui.add_space(15.0);

                if styled_button(ui, "Settings", egui::vec2(300.0, 50.0), false)
                    .clicked()
                {
                    action = MenuAction::Settings;
                }

                ui.add_space(15.0);

                if styled_button(ui, "Quit to Main Menu", egui::vec2(300.0, 50.0), false)
                    .clicked()
                {
                    action = MenuAction::Quit;
                }

                ui.add_space(40.0);

                // Hint text
                ui.label(
                    egui::RichText::new("Press ESC to resume")
                        .size(14.0)
                        .italics()
                        .color(egui::Color32::GRAY),
                );
            });
        });

    action
}


/// Show settings menu UI with graphics/audio/controls settings (Week 2 Days 1-3)
pub fn show_settings_menu(
    ctx: &egui::Context, 
    settings: &mut super::menu::SettingsState,
    rebinding_key: &mut Option<String>,
) -> MenuAction {
    use super::menu::QualityPreset;
    
    let mut action = MenuAction::None;

    // Full-screen dark background
    egui::Area::new(egui::Id::new("settings_menu_bg"))
        .fixed_pos(egui::pos2(0.0, 0.0))
        .show(ctx, |ui| {
            let screen_rect = ctx.screen_rect();
            ui.painter().rect_filled(
                screen_rect,
                0.0,
                egui::Color32::from_rgba_unmultiplied(0, 0, 0, 200),
            );
        });

    // Centered settings window (taller to fit all controls + buttons)
    egui::Window::new("settings_menu")
        .title_bar(false)
        .resizable(false)
        .fixed_size(egui::vec2(700.0, 900.0)) // Fixed height with scrollable content
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(20.0);

                // Title
                ui.label(
                    egui::RichText::new("SETTINGS")
                        .size(42.0)
                        .color(egui::Color32::from_rgb(100, 200, 255)),
                );

                ui.add_space(30.0);
            });

            // Wrap ALL content in a ScrollArea so bottom buttons are always accessible
            egui::ScrollArea::vertical()
                .max_height(750.0) // Leave room for title (100px) + padding
                .show(ui, |ui| {
                    // Main content (left-aligned for controls)
                    ui.add_space(10.0);

            // === GRAPHICS SETTINGS ===
            ui.label(
                egui::RichText::new("Graphics")
                    .size(20.0)
                    .color(egui::Color32::from_rgb(100, 200, 255)),
            );
            ui.separator();
            ui.add_space(10.0);

            // Resolution dropdown
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new("Resolution:")
                        .size(14.0)
                        .color(egui::Color32::LIGHT_GRAY),
                );
                ui.add_space(10.0);

                let current_res_text = format!("{}x{}", settings.graphics.resolution.0, settings.graphics.resolution.1);
                
                egui::ComboBox::from_id_salt("resolution")
                    .selected_text(current_res_text)
                    .width(150.0)
                    .show_ui(ui, |ui| {
                        let resolutions = [
                            (1280, 720),
                            (1920, 1080),
                            (2560, 1440),
                            (3840, 2160),
                        ];
                        
                        for &(w, h) in &resolutions {
                            let text = format!("{}x{}", w, h);
                            if ui.selectable_value(&mut settings.graphics.resolution, (w, h), text).clicked() {
                                // Resolution changed
                            }
                        }
                    });
            });

            ui.add_space(10.0);

            // Quality preset dropdown
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new("Quality:")
                        .size(14.0)
                        .color(egui::Color32::LIGHT_GRAY),
                );
                ui.add_space(10.0);

                egui::ComboBox::from_id_salt("quality")
                    .selected_text(settings.graphics.quality.as_str())
                    .width(150.0)
                    .show_ui(ui, |ui| {
                        for &preset in QualityPreset::all() {
                            ui.selectable_value(&mut settings.graphics.quality, preset, preset.as_str());
                        }
                    });
            });

            ui.add_space(10.0);

            // Fullscreen checkbox
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new("Fullscreen:")
                        .size(14.0)
                        .color(egui::Color32::LIGHT_GRAY),
                );
                ui.add_space(10.0);
                ui.checkbox(&mut settings.graphics.fullscreen, "");
            });

            ui.add_space(10.0);

            // VSync checkbox
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new("VSync:")
                        .size(14.0)
                        .color(egui::Color32::LIGHT_GRAY),
                );
                ui.add_space(10.0);
                ui.checkbox(&mut settings.graphics.vsync, "");
            });

            ui.add_space(30.0);

            // === AUDIO SETTINGS (Week 2 Day 2) ===
            ui.label(
                egui::RichText::new("Audio")
                    .size(20.0)
                    .color(egui::Color32::from_rgb(100, 200, 255)),
            );
            ui.separator();
            ui.add_space(10.0);

            // Master volume
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new("Master Volume:")
                        .size(14.0)
                        .color(egui::Color32::LIGHT_GRAY),
                );
                ui.add(
                    egui::Slider::new(&mut settings.audio.master_volume, 0.0..=100.0)
                        .suffix("%")
                        .show_value(true),
                );
                ui.checkbox(&mut settings.audio.master_mute, "Mute");
            });

            ui.add_space(5.0);

            // Music volume
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new("Music Volume:")
                        .size(14.0)
                        .color(egui::Color32::LIGHT_GRAY),
                );
                ui.add(
                    egui::Slider::new(&mut settings.audio.music_volume, 0.0..=100.0)
                        .suffix("%")
                        .show_value(true),
                );
                ui.checkbox(&mut settings.audio.music_mute, "Mute");
            });

            ui.add_space(5.0);

            // SFX volume
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new("SFX Volume:")
                        .size(14.0)
                        .color(egui::Color32::LIGHT_GRAY),
                );
                ui.add(
                    egui::Slider::new(&mut settings.audio.sfx_volume, 0.0..=100.0)
                        .suffix("%")
                        .show_value(true),
                );
                ui.checkbox(&mut settings.audio.sfx_mute, "Mute");
            });

            ui.add_space(5.0);

            // Voice volume
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new("Voice Volume:")
                        .size(14.0)
                        .color(egui::Color32::LIGHT_GRAY),
                );
                ui.add(
                    egui::Slider::new(&mut settings.audio.voice_volume, 0.0..=100.0)
                        .suffix("%")
                        .show_value(true),
                );
                ui.checkbox(&mut settings.audio.voice_mute, "Mute");
            });

            ui.add_space(20.0);

            // === CONTROLS SETTINGS (Week 2 Day 3) ===
            ui.label(
                egui::RichText::new("Controls")
                    .size(20.0)
                    .color(egui::Color32::from_rgb(100, 200, 255)),
            );
            ui.separator();
            ui.add_space(10.0);

            // Key bindings section (no inner scroll - outer ScrollArea handles everything)
            {
                // Helper function for key binding row
                let mut show_key_binding = |ui: &mut egui::Ui, label: &str, key: &mut String, key_id: &str| {
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new(format!("{}:", label))
                                    .size(14.0)
                                    .color(egui::Color32::LIGHT_GRAY),
                            );
                            ui.add_space(10.0);

                            let is_rebinding = rebinding_key.as_ref() == Some(&key_id.to_string());
                            let button_text = if is_rebinding {
                                "Press any key...".to_string()
                            } else {
                                key.clone()
                            };

                            let button_color = if is_rebinding {
                                egui::Color32::from_rgb(255, 200, 100)
                            } else {
                                egui::Color32::from_rgb(100, 150, 200)
                            };

                            if ui.add(
                                egui::Button::new(button_text)
                                    .fill(button_color)
                                    .min_size(egui::vec2(120.0, 30.0))
                            ).clicked() {
                                *rebinding_key = Some(key_id.to_string());
                            }
                        });
                        ui.add_space(5.0);
                    };

                    // Movement keys
                    show_key_binding(ui, "Move Forward", &mut settings.controls.move_forward, "move_forward");
                    show_key_binding(ui, "Move Backward", &mut settings.controls.move_backward, "move_backward");
                    show_key_binding(ui, "Move Left", &mut settings.controls.move_left, "move_left");
                    show_key_binding(ui, "Move Right", &mut settings.controls.move_right, "move_right");
                    
                    ui.add_space(5.0);
                    
                // Action keys
                show_key_binding(ui, "Jump", &mut settings.controls.jump, "jump");
                show_key_binding(ui, "Crouch", &mut settings.controls.crouch, "crouch");
                show_key_binding(ui, "Sprint", &mut settings.controls.sprint, "sprint");
                show_key_binding(ui, "Attack", &mut settings.controls.attack, "attack");
                show_key_binding(ui, "Interact", &mut settings.controls.interact, "interact");
                show_key_binding(ui, "Inventory", &mut settings.controls.inventory, "inventory");
            }

            ui.add_space(15.0);            // Mouse sensitivity
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new("Mouse Sensitivity:")
                        .size(14.0)
                        .color(egui::Color32::LIGHT_GRAY),
                );
                ui.add(
                    egui::Slider::new(&mut settings.controls.mouse_sensitivity, 0.1..=5.0)
                        .suffix("x")
                        .show_value(true),
                );
            });

            ui.add_space(10.0);

            // Invert Y-axis
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new("Invert Y-Axis:")
                        .size(14.0)
                        .color(egui::Color32::LIGHT_GRAY),
                );
                ui.add_space(10.0);
                ui.checkbox(&mut settings.controls.invert_y, "");
            });

            ui.add_space(15.0);

            // Reset to defaults button
            ui.vertical_centered(|ui| {
                if styled_button(ui, "Reset to Defaults", egui::vec2(200.0, 35.0), false)
                    .clicked()
                {
                    settings.controls = super::menu::ControlsSettings::default();
                    *rebinding_key = None;
                }
            });

            ui.add_space(30.0);

            // Bottom buttons (centered)
            ui.vertical_centered(|ui| {
                // Apply and Cancel buttons (side by side)
                ui.horizontal(|ui| {
                    // Apply button (green with hover)
                    let apply_btn = ui.scope(|ui| {
                        let style = ui.style_mut();
                        style.visuals.widgets.inactive.weak_bg_fill = egui::Color32::from_rgb(80, 180, 80);
                        style.visuals.widgets.hovered.weak_bg_fill = egui::Color32::from_rgb(100, 220, 100);
                        style.visuals.widgets.active.weak_bg_fill = egui::Color32::from_rgb(100, 220, 100);
                        
                        ui.add_sized(
                            egui::vec2(140.0, 45.0),
                            egui::Button::new(egui::RichText::new("Apply").size(18.0).color(egui::Color32::WHITE))
                                .corner_radius(8.0),
                        )
                    }).inner;
                    
                    if apply_btn.clicked() {
                        action = MenuAction::ApplySettings;
                    }

                    ui.add_space(10.0);

                    // Cancel button (red with hover)
                    let cancel_btn = ui.scope(|ui| {
                        let style = ui.style_mut();
                        style.visuals.widgets.inactive.weak_bg_fill = egui::Color32::from_rgb(180, 80, 80);
                        style.visuals.widgets.hovered.weak_bg_fill = egui::Color32::from_rgb(220, 100, 100);
                        style.visuals.widgets.active.weak_bg_fill = egui::Color32::from_rgb(220, 100, 100);
                        
                        ui.add_sized(
                            egui::vec2(140.0, 45.0),
                            egui::Button::new(egui::RichText::new("Cancel").size(18.0).color(egui::Color32::WHITE))
                                .corner_radius(8.0),
                        )
                    }).inner;
                    
                    if cancel_btn.clicked() {
                        action = MenuAction::CancelSettings;
                    }
                });

                ui.add_space(15.0);

                // Back button
                if styled_button(ui, "Back", egui::vec2(250.0, 45.0), false)
                    .clicked()
                {
                    action = MenuAction::Quit; // "Back" acts as quit from settings
                    *rebinding_key = None; // Clear any active rebinding
                }

                ui.add_space(10.0);

                // Hint text
                ui.label(
                    egui::RichText::new("Apply saves settings to disk | Cancel reverts changes")
                        .size(12.0)
                        .italics()
                        .color(egui::Color32::GRAY),
                );
            });
        }); // Close the main ScrollArea

    action
}

