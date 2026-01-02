//! Chat UI module for hello_companion visual demo
//!
//! Provides an egui-based chat interface for user interaction with the NPC.

#![allow(dead_code)] // Some fields/methods reserved for future expansion

use crate::scene::DemoMode;

/// Chat message with sender and timestamp
#[derive(Debug, Clone)]
pub struct ChatMessage {
    pub sender: MessageSender,
    pub text: String,
    pub timestamp: std::time::Instant,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageSender {
    User,
    Npc,
    System,
}

/// Chat UI state
pub struct ChatUi {
    /// Chat history
    pub messages: Vec<ChatMessage>,
    /// Current input text
    pub input: String,
    /// Current demo mode
    pub mode: DemoMode,
    /// Whether chat panel is visible
    pub visible: bool,
    /// Whether NPC is "typing" (thinking)
    pub npc_typing: bool,
    /// Scroll to bottom flag
    pub scroll_to_bottom: bool,
    /// Typing preview (partial streaming output)
    pub typing_preview: Option<String>,
}

impl ChatUi {
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
            input: String::new(),
            mode: DemoMode::Arbiter,
            visible: true,
            npc_typing: false,
            scroll_to_bottom: true,
            typing_preview: None,
        }
    }

    /// Add a user message
    pub fn add_user_message(&mut self, text: impl Into<String>) {
        self.messages.push(ChatMessage {
            sender: MessageSender::User,
            text: text.into(),
            timestamp: std::time::Instant::now(),
        });
        self.scroll_to_bottom = true;
    }

    /// Add an NPC message
    pub fn add_npc_message(&mut self, text: impl Into<String>) {
        self.messages.push(ChatMessage {
            sender: MessageSender::Npc,
            text: text.into(),
            timestamp: std::time::Instant::now(),
        });
        self.scroll_to_bottom = true;
        self.npc_typing = false;
        self.typing_preview = None;
    }

    /// Add a system message
    pub fn add_system_message(&mut self, text: impl Into<String>) {
        self.messages.push(ChatMessage {
            sender: MessageSender::System,
            text: text.into(),
            timestamp: std::time::Instant::now(),
        });
        self.scroll_to_bottom = true;
    }

    /// Set NPC typing indicator
    pub fn set_npc_typing(&mut self, typing: bool) {
        self.npc_typing = typing;
        if !typing {
            self.typing_preview = None;
        }
    }

    /// Set typing preview (partial streaming output)
    pub fn set_typing_preview(&mut self, preview: &str) {
        self.typing_preview = Some(preview.to_string());
    }

    /// Switch demo mode
    pub fn switch_mode(&mut self, mode: DemoMode) {
        if self.mode != mode {
            self.mode = mode;
            self.add_system_message(format!("Switched to {} mode", mode.display_name()));
        }
    }

    /// Draw the chat UI using egui
    pub fn draw(&mut self, ctx: &egui::Context) -> Option<String> {
        let mut submitted_text = None;

        // Mode indicator at top
        egui::TopBottomPanel::top("mode_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Hello Companion Demo");
                ui.separator();

                // Mode buttons
                for mode in [DemoMode::PureLlm, DemoMode::PureGoap, DemoMode::Arbiter] {
                    let selected = self.mode == mode;
                    if ui
                        .selectable_label(
                            selected,
                            format!("[{}] {}", mode.hotkey(), mode.display_name()),
                        )
                        .clicked()
                    {
                        self.switch_mode(mode);
                    }
                }
            });
        });

        // Chat panel on the right side
        egui::SidePanel::right("chat_panel")
            .min_width(350.0)
            .default_width(400.0)
            .show(ctx, |ui| {
                ui.heading("Chat");
                ui.separator();

                // Chat history
                let available_height = ui.available_height() - 50.0;
                egui::ScrollArea::vertical()
                    .max_height(available_height)
                    .auto_shrink([false, false])
                    .stick_to_bottom(self.scroll_to_bottom)
                    .show(ui, |ui| {
                        for msg in &self.messages {
                            self.draw_message(ui, msg);
                        }

                        // Typing indicator with streaming preview
                        if self.npc_typing {
                            ui.horizontal_wrapped(|ui| {
                                ui.label(
                                    egui::RichText::new("NPC")
                                        .color(egui::Color32::from_rgb(100, 200, 100))
                                        .strong(),
                                );
                                if let Some(ref preview) = self.typing_preview {
                                    // Show partial streaming output.
                                    ui.label(
                                        egui::RichText::new(format!(": {} ▌", preview))
                                            .italics()
                                            .color(egui::Color32::from_rgb(180, 200, 180)),
                                    );
                                } else {
                                    ui.label(egui::RichText::new(": typing...").italics());
                                }
                            });
                        }
                    });

                self.scroll_to_bottom = false;

                ui.separator();

                // Input area
                ui.horizontal(|ui| {
                    let input_response = ui.add_sized(
                        [ui.available_width() - 60.0, 30.0],
                        egui::TextEdit::singleline(&mut self.input).hint_text("Type a message..."),
                    );

                    let send_clicked = ui.button("Send").clicked();
                    let enter_pressed = input_response.lost_focus()
                        && ui.input(|i| i.key_pressed(egui::Key::Enter));

                    if (send_clicked || enter_pressed) && !self.input.trim().is_empty() {
                        let text = std::mem::take(&mut self.input);
                        self.add_user_message(&text);
                        submitted_text = Some(text);
                    }

                    // Keep focus on input
                    if send_clicked || enter_pressed {
                        input_response.request_focus();
                    }
                });
            });

        submitted_text
    }

    /// Draw a single chat message
    fn draw_message(&self, ui: &mut egui::Ui, msg: &ChatMessage) {
        ui.horizontal_wrapped(|ui| {
            let (label, color) = match msg.sender {
                MessageSender::User => ("You", egui::Color32::from_rgb(100, 150, 255)),
                MessageSender::Npc => ("NPC", egui::Color32::from_rgb(100, 200, 100)),
                MessageSender::System => ("System", egui::Color32::from_rgb(200, 200, 100)),
            };

            ui.label(
                egui::RichText::new(format!("{}: ", label))
                    .color(color)
                    .strong(),
            );

            // Check if it's an action (starts and ends with *)
            if msg.text.starts_with('*') && msg.text.ends_with('*') {
                ui.label(
                    egui::RichText::new(&msg.text)
                        .italics()
                        .color(egui::Color32::from_rgb(180, 180, 180)),
                );
            } else {
                ui.label(&msg.text);
            }
        });
        ui.add_space(4.0);
    }
}

impl Default for ChatUi {
    fn default() -> Self {
        Self::new()
    }
}

/// Help overlay for controls
pub fn draw_help_overlay(ctx: &egui::Context) {
    egui::Window::new("Controls")
        .anchor(egui::Align2::LEFT_BOTTOM, [10.0, -10.0])
        .resizable(false)
        .collapsible(true)
        .default_open(false)
        .show(ctx, |ui| {
            ui.label("WASD - Move camera");
            ui.label("Mouse - Look around");
            ui.label("1/2/3 - Switch AI mode");
            ui.label("Enter - Send message");
            ui.label("ESC - Toggle mouse capture");
        });
}
