// Quest Panel UI component for displaying active quest, objectives, and progress
// Integrates with QuestManager to show real-time quest status

use crate::{Quest, QuestReward, QuestState};

/// Quest panel UI state
#[derive(Debug, Clone)]
pub struct QuestPanel {
    /// Whether panel is visible
    pub visible: bool,
    /// Position (x, y) on screen (pixels)
    pub position: (f32, f32),
    /// Size (width, height) in pixels
    pub size: (f32, f32),
    /// Animation timer for completion notification
    pub completion_animation_timer: f32,
    /// Last completed quest for notification
    pub last_completed: Option<String>,
    /// Notification message
    pub notification_message: Option<String>,
}

impl QuestPanel {
    /// Create new quest panel
    pub fn new() -> Self {
        Self {
            visible: true,
            position: (20.0, 100.0), // Top-left area
            size: (350.0, 200.0),
            completion_animation_timer: 0.0,
            last_completed: None,
            notification_message: None,
        }
    }

    /// Update animations
    pub fn update(&mut self, delta_time: f32) {
        if self.completion_animation_timer > 0.0 {
            self.completion_animation_timer -= delta_time;
            if self.completion_animation_timer <= 0.0 {
                self.completion_animation_timer = 0.0; // Clamp to zero
                self.last_completed = None;
                self.notification_message = None;
            }
        }
    }

    /// Show completion notification
    pub fn show_completion(&mut self, quest_title: &str, rewards: &[QuestReward]) {
        self.last_completed = Some(quest_title.to_string());

        // Build rewards message
        let mut reward_msgs = Vec::new();
        for reward in rewards {
            match reward {
                QuestReward::EchoCurrency(amount) => {
                    reward_msgs.push(format!("+{} Echo", amount));
                }
                QuestReward::AbilityUnlock(ability) => {
                    reward_msgs.push(format!("Unlocked: {}", ability));
                }
                QuestReward::StatBoost { stat, amount } => {
                    reward_msgs.push(format!("+{} {}", amount, stat));
                }
                QuestReward::Multiple(multi_rewards) => {
                    for sub_reward in multi_rewards {
                        match sub_reward {
                            QuestReward::EchoCurrency(amt) => {
                                reward_msgs.push(format!("+{} Echo", amt));
                            }
                            QuestReward::AbilityUnlock(ab) => {
                                reward_msgs.push(format!("Unlocked: {}", ab));
                            }
                            QuestReward::StatBoost { stat, amount } => {
                                reward_msgs.push(format!("+{} {}", amount, stat));
                            }
                            _ => {}
                        }
                    }
                }
            }
        }

        let rewards_text = if reward_msgs.is_empty() {
            String::new()
        } else {
            format!(" ({})", reward_msgs.join(", "))
        };

        self.notification_message =
            Some(format!("Quest Complete: {}{}", quest_title, rewards_text));
        self.completion_animation_timer = 5.0; // 5 seconds
    }

    /// Render quest panel (ASCII visualization for testing)
    pub fn render(&self, quest: Option<&Quest>) -> String {
        if !self.visible {
            return String::new();
        }

        let mut output = String::new();
        output.push_str("╔═══════════════════════════════════════╗\n");
        output.push_str("║           ACTIVE QUEST                ║\n");
        output.push_str("╠═══════════════════════════════════════╣\n");

        if let Some(q) = quest {
            // Quest title and description
            output.push_str(&format!(
                "║ {}{}║\n",
                q.title,
                " ".repeat(39 - q.title.len())
            ));

            // Wrap description (max 39 chars per line)
            let desc_words: Vec<&str> = q.description.split_whitespace().collect();
            let mut current_line = String::from("║ ");
            for word in desc_words {
                if current_line.len() + word.len() + 1 > 40 {
                    output.push_str(&format!(
                        "{}{}║\n",
                        current_line,
                        " ".repeat(41 - current_line.len())
                    ));
                    current_line = String::from("║ ");
                }
                current_line.push_str(word);
                current_line.push(' ');
            }
            if current_line.len() > 2 {
                output.push_str(&format!(
                    "{}{}║\n",
                    current_line.trim_end(),
                    " ".repeat(40 - current_line.trim_end().len())
                ));
            }

            output.push_str("╠═══════════════════════════════════════╣\n");

            // Objectives
            for obj in q.objectives.iter() {
                let status = if obj.is_complete() { "✓" } else { " " };
                let desc = obj.description();
                let desc_display = if desc.len() > 33 {
                    format!("{}...", &desc[0..30])
                } else {
                    desc
                };

                output.push_str(&format!(
                    "║ [{}] {}{}║\n",
                    status,
                    desc_display,
                    " ".repeat(36 - desc_display.len())
                ));

                // Progress bar
                let progress = obj.progress();
                let bar_width = 30;
                let filled = (progress * bar_width as f32) as usize;
                let bar = format!(
                    "[{}{}] {:.0}%",
                    "█".repeat(filled),
                    "░".repeat(bar_width - filled),
                    progress * 100.0
                );
                let padding = if bar.len() < 36 { 36 - bar.len() } else { 0 };
                output.push_str(&format!("║   {}{}║\n", bar, " ".repeat(padding)));
            }

            output.push_str("╠═══════════════════════════════════════╣\n");

            // Overall progress
            let total_progress = q.progress();
            let status_text = match q.state {
                QuestState::Active => format!("In Progress: {:.0}%", total_progress * 100.0),
                QuestState::Completed => "COMPLETED ✓".to_string(),
                QuestState::Failed => "FAILED ✗".to_string(),
                QuestState::Inactive => "Not Started".to_string(),
            };
            output.push_str(&format!(
                "║ {}{}║\n",
                status_text,
                " ".repeat(39 - status_text.len())
            ));
        } else {
            output.push_str("║ No active quest                       ║\n");
            output.push_str("║                                       ║\n");
            output.push_str("║ Visit a quest giver to start a quest ║\n");
        }

        output.push_str("╚═══════════════════════════════════════╝\n");

        // Completion notification
        if let Some(msg) = &self.notification_message {
            output.push('\n');
            output.push_str("╔═══════════════════════════════════════╗\n");
            output.push_str(&format!("║ {}{}║\n", msg, " ".repeat(39 - msg.len())));
            output.push_str("╚═══════════════════════════════════════╝\n");
        }

        output
    }

    /// Get panel bounds for UI interaction
    pub fn bounds(&self) -> (f32, f32, f32, f32) {
        (self.position.0, self.position.1, self.size.0, self.size.1)
    }

    /// Toggle panel visibility
    pub fn toggle(&mut self) {
        self.visible = !self.visible;
    }
}

impl Default for QuestPanel {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ObjectiveType;

    #[test]
    fn test_quest_panel_creation() {
        let panel = QuestPanel::new();
        assert!(panel.visible);
        assert_eq!(panel.position, (20.0, 100.0));
        assert_eq!(panel.completion_animation_timer, 0.0);
        assert!(panel.last_completed.is_none());
    }

    #[test]
    fn test_quest_panel_toggle() {
        let mut panel = QuestPanel::new();
        assert!(panel.visible);

        panel.toggle();
        assert!(!panel.visible);

        panel.toggle();
        assert!(panel.visible);
    }

    #[test]
    fn test_quest_panel_update_animation() {
        let mut panel = QuestPanel::new();
        panel.completion_animation_timer = 5.0;
        panel.last_completed = Some("Test Quest".to_string());

        panel.update(2.0);
        assert_eq!(panel.completion_animation_timer, 3.0);
        assert!(panel.last_completed.is_some());

        panel.update(3.5);
        assert_eq!(panel.completion_animation_timer, 0.0);
        assert!(panel.last_completed.is_none());
    }

    #[test]
    fn test_quest_panel_show_completion() {
        let mut panel = QuestPanel::new();
        let rewards = vec![
            QuestReward::EchoCurrency(100),
            QuestReward::AbilityUnlock("Echo Dash".to_string()),
        ];

        panel.show_completion("Test Quest", &rewards);

        assert!(panel.last_completed.is_some());
        assert_eq!(panel.last_completed.as_ref().unwrap(), "Test Quest");
        assert!(panel.notification_message.is_some());
        assert!(panel
            .notification_message
            .as_ref()
            .unwrap()
            .contains("Test Quest"));
        assert!(panel
            .notification_message
            .as_ref()
            .unwrap()
            .contains("+100 Echo"));
        assert!(panel
            .notification_message
            .as_ref()
            .unwrap()
            .contains("Echo Dash"));
        assert_eq!(panel.completion_animation_timer, 5.0);
    }

    #[test]
    fn test_quest_panel_render_no_quest() {
        let panel = QuestPanel::new();
        let output = panel.render(None);

        assert!(output.contains("ACTIVE QUEST"));
        assert!(output.contains("No active quest"));
    }

    #[test]
    fn test_quest_panel_render_with_quest() {
        let panel = QuestPanel::new();
        let mut quest = Quest::new("quest1", "Kill Enemies", "Defeat the corruption")
            .with_objective(ObjectiveType::Kill {
                target_type: "enemy".to_string(),
                required: 10,
                current: 5,
            });

        quest.activate(); // Must activate quest

        let output = panel.render(Some(&quest));

        assert!(output.contains("ACTIVE QUEST"));
        assert!(output.contains("Kill Enemies"));
        assert!(output.contains("Defeat the corruption"));
        assert!(output.contains("Kill 5/10 enemy"));
        assert!(output.contains("50%")); // Progress
        assert!(output.contains("In Progress"));
    }

    #[test]
    fn test_quest_panel_render_completed_quest() {
        let panel = QuestPanel::new();
        let mut quest = Quest::new("quest1", "Completed Quest", "You did it!").with_objective(
            ObjectiveType::Kill {
                target_type: "enemy".to_string(),
                required: 5,
                current: 5,
            },
        );

        quest.activate();
        quest.check_completion();

        let output = panel.render(Some(&quest));

        assert!(output.contains("COMPLETED ✓"));
        assert!(output.contains("✓")); // Objective check
    }

    #[test]
    fn test_quest_panel_render_multiple_objectives() {
        let panel = QuestPanel::new();
        let quest = Quest::new("quest1", "Complex Quest", "Multi-objective quest")
            .with_objective(ObjectiveType::Kill {
                target_type: "enemy".to_string(),
                required: 10,
                current: 7,
            })
            .with_objective(ObjectiveType::Repair {
                required: 3,
                current: 1,
                min_stability: 0.8,
            });

        let output = panel.render(Some(&quest));

        assert!(output.contains("Kill 7/10 enemy"));
        assert!(output.contains("Repair 1/3 anchors"));
        assert!(output.contains("70%")); // First objective
        assert!(output.contains("33%")); // Second objective (1/3)
    }

    #[test]
    fn test_quest_panel_bounds() {
        let panel = QuestPanel::new();
        let (x, y, w, h) = panel.bounds();

        assert_eq!(x, 20.0);
        assert_eq!(y, 100.0);
        assert_eq!(w, 350.0);
        assert_eq!(h, 200.0);
    }

    #[test]
    fn test_quest_panel_render_when_hidden() {
        let mut panel = QuestPanel::new();
        panel.visible = false;

        let quest = Quest::new("quest1", "Test", "Test");
        let output = panel.render(Some(&quest));

        assert!(output.is_empty());
    }

    #[test]
    fn test_quest_panel_notification_display() {
        let mut panel = QuestPanel::new();
        let rewards = vec![QuestReward::EchoCurrency(50)];

        panel.show_completion("Simple Quest", &rewards);

        let quest = Quest::new("quest1", "Test", "Test");
        let output = panel.render(Some(&quest));

        assert!(output.contains("Quest Complete: Simple Quest"));
        assert!(output.contains("+50 Echo"));
    }

    #[test]
    fn test_quest_panel_multiple_rewards() {
        let mut panel = QuestPanel::new();
        let rewards = vec![QuestReward::Multiple(vec![
            QuestReward::EchoCurrency(100),
            QuestReward::AbilityUnlock("Teleport".to_string()),
            QuestReward::StatBoost {
                stat: "Health".to_string(),
                amount: 25.0,
            },
        ])];

        panel.show_completion("Epic Quest", &rewards);

        assert!(panel
            .notification_message
            .as_ref()
            .unwrap()
            .contains("+100 Echo"));
        assert!(panel
            .notification_message
            .as_ref()
            .unwrap()
            .contains("Teleport"));
        assert!(panel
            .notification_message
            .as_ref()
            .unwrap()
            .contains("+25 Health"));
    }
}
