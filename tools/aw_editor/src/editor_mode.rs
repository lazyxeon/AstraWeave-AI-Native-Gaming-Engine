use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum EditorMode {
    #[default]
    Edit,
    Play,
    Paused,
}

impl EditorMode {
    pub fn is_playing(&self) -> bool {
        matches!(self, EditorMode::Play)
    }

    pub fn is_paused(&self) -> bool {
        matches!(self, EditorMode::Paused)
    }

    pub fn is_editing(&self) -> bool {
        matches!(self, EditorMode::Edit)
    }

    pub fn can_edit(&self) -> bool {
        matches!(self, EditorMode::Edit)
    }

    pub fn status_text(&self) -> &'static str {
        match self {
            EditorMode::Edit => "Edit Mode",
            EditorMode::Play => "▶️ Playing",
            EditorMode::Paused => "⏸️ Paused",
        }
    }

    pub fn status_color(&self) -> egui::Color32 {
        match self {
            EditorMode::Edit => egui::Color32::from_rgb(100, 100, 100),
            EditorMode::Play => egui::Color32::from_rgb(100, 200, 100),
            EditorMode::Paused => egui::Color32::from_rgb(255, 180, 50),
        }
    }

    /// Get the keyboard shortcut hint for this mode
    pub fn shortcut_hint(&self) -> &'static str {
        match self {
            EditorMode::Edit => "Esc",
            EditorMode::Play => "F5",
            EditorMode::Paused => "F6",
        }
    }

    /// Check if transition to target mode is valid
    pub fn can_transition_to(&self, target: EditorMode) -> bool {
        match (self, target) {
            // Can always transition to same mode (no-op)
            (EditorMode::Edit, EditorMode::Edit) => true,
            (EditorMode::Play, EditorMode::Play) => true,
            (EditorMode::Paused, EditorMode::Paused) => true,
            // From Edit: can go to Play
            (EditorMode::Edit, EditorMode::Play) => true,
            // From Play: can go to Edit or Paused
            (EditorMode::Play, EditorMode::Edit) => true,
            (EditorMode::Play, EditorMode::Paused) => true,
            // From Paused: can go to Edit or Play
            (EditorMode::Paused, EditorMode::Edit) => true,
            (EditorMode::Paused, EditorMode::Play) => true,
            // Edit cannot go directly to Paused
            (EditorMode::Edit, EditorMode::Paused) => false,
        }
    }

    /// Get all valid transition targets from current mode
    pub fn valid_transitions(&self) -> Vec<EditorMode> {
        match self {
            EditorMode::Edit => vec![EditorMode::Play],
            EditorMode::Play => vec![EditorMode::Edit, EditorMode::Paused],
            EditorMode::Paused => vec![EditorMode::Edit, EditorMode::Play],
        }
    }

    /// Get the icon for this mode
    pub fn icon(&self) -> &'static str {
        match self {
            EditorMode::Edit => "🔧",
            EditorMode::Play => "▶️",
            EditorMode::Paused => "⏸️",
        }
    }

    /// Get all modes
    pub fn all() -> [EditorMode; 3] {
        [EditorMode::Edit, EditorMode::Play, EditorMode::Paused]
    }

    /// Get a description of this mode
    pub fn description(&self) -> &'static str {
        match self {
            EditorMode::Edit => "Modify scene objects, properties, and layout",
            EditorMode::Play => "Run the game simulation in real-time",
            EditorMode::Paused => "Simulation paused - can step frame by frame",
        }
    }

    /// Check if simulation is running (not paused)
    pub fn is_simulating(&self) -> bool {
        matches!(self, EditorMode::Play)
    }

    /// Check if this mode allows scene modifications
    pub fn allows_scene_changes(&self) -> bool {
        matches!(self, EditorMode::Edit)
    }

    /// Get the next mode in typical workflow
    pub fn next_mode(&self) -> EditorMode {
        match self {
            EditorMode::Edit => EditorMode::Play,
            EditorMode::Play => EditorMode::Paused,
            EditorMode::Paused => EditorMode::Play,
        }
    }

    /// Get the action verb for transitioning to this mode
    pub fn action_verb(&self) -> &'static str {
        match self {
            EditorMode::Edit => "Stop",
            EditorMode::Play => "Play",
            EditorMode::Paused => "Pause",
        }
    }
}

impl std::fmt::Display for EditorMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.status_text())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_editor_mode_states() {
        assert!(EditorMode::Edit.is_editing());
        assert!(!EditorMode::Edit.is_playing());
        assert!(!EditorMode::Edit.is_paused());

        assert!(!EditorMode::Play.is_editing());
        assert!(EditorMode::Play.is_playing());
        assert!(!EditorMode::Play.is_paused());

        assert!(!EditorMode::Paused.is_editing());
        assert!(!EditorMode::Paused.is_playing());
        assert!(EditorMode::Paused.is_paused());
    }

    #[test]
    fn test_can_edit() {
        assert!(EditorMode::Edit.can_edit());
        assert!(!EditorMode::Play.can_edit());
        assert!(!EditorMode::Paused.can_edit());
    }

    #[test]
    fn test_default() {
        assert_eq!(EditorMode::default(), EditorMode::Edit);
    }

    #[test]
    fn test_shortcut_hint() {
        assert_eq!(EditorMode::Edit.shortcut_hint(), "Esc");
        assert_eq!(EditorMode::Play.shortcut_hint(), "F5");
        assert_eq!(EditorMode::Paused.shortcut_hint(), "F6");
    }

    #[test]
    fn test_can_transition_to_same_mode() {
        // All modes can transition to themselves (no-op)
        assert!(EditorMode::Edit.can_transition_to(EditorMode::Edit));
        assert!(EditorMode::Play.can_transition_to(EditorMode::Play));
        assert!(EditorMode::Paused.can_transition_to(EditorMode::Paused));
    }

    #[test]
    fn test_can_transition_from_edit() {
        // From Edit: can go to Play, cannot go to Paused
        assert!(EditorMode::Edit.can_transition_to(EditorMode::Play));
        assert!(!EditorMode::Edit.can_transition_to(EditorMode::Paused));
    }

    #[test]
    fn test_can_transition_from_play() {
        // From Play: can go to Edit or Paused
        assert!(EditorMode::Play.can_transition_to(EditorMode::Edit));
        assert!(EditorMode::Play.can_transition_to(EditorMode::Paused));
    }

    #[test]
    fn test_can_transition_from_paused() {
        // From Paused: can go to Edit or Play
        assert!(EditorMode::Paused.can_transition_to(EditorMode::Edit));
        assert!(EditorMode::Paused.can_transition_to(EditorMode::Play));
    }

    #[test]
    fn test_valid_transitions_edit() {
        let transitions = EditorMode::Edit.valid_transitions();
        assert_eq!(transitions.len(), 1);
        assert!(transitions.contains(&EditorMode::Play));
    }

    #[test]
    fn test_valid_transitions_play() {
        let transitions = EditorMode::Play.valid_transitions();
        assert_eq!(transitions.len(), 2);
        assert!(transitions.contains(&EditorMode::Edit));
        assert!(transitions.contains(&EditorMode::Paused));
    }

    #[test]
    fn test_valid_transitions_paused() {
        let transitions = EditorMode::Paused.valid_transitions();
        assert_eq!(transitions.len(), 2);
        assert!(transitions.contains(&EditorMode::Edit));
        assert!(transitions.contains(&EditorMode::Play));
    }

    #[test]
    fn test_icon() {
        assert!(!EditorMode::Edit.icon().is_empty());
        assert!(!EditorMode::Play.icon().is_empty());
        assert!(!EditorMode::Paused.icon().is_empty());
    }

    #[test]
    fn test_all_modes() {
        let all = EditorMode::all();
        assert_eq!(all.len(), 3);
        assert!(all.contains(&EditorMode::Edit));
        assert!(all.contains(&EditorMode::Play));
        assert!(all.contains(&EditorMode::Paused));
    }

    #[test]
    fn test_status_text_not_empty() {
        for mode in EditorMode::all() {
            assert!(!mode.status_text().is_empty());
        }
    }

    #[test]
    fn test_all_modes_have_unique_icons() {
        let all = EditorMode::all();
        let icons: Vec<_> = all.iter().map(|m| m.icon()).collect();
        for (i, icon) in icons.iter().enumerate() {
            for (j, other) in icons.iter().enumerate() {
                if i != j {
                    assert_ne!(icon, other, "Icons should be unique");
                }
            }
        }
    }

    // ====================================================================
    // EditorMode New Methods Tests
    // ====================================================================

    #[test]
    fn test_editor_mode_description_not_empty() {
        for mode in EditorMode::all() {
            assert!(!mode.description().is_empty());
        }
    }

    #[test]
    fn test_editor_mode_is_simulating() {
        assert!(!EditorMode::Edit.is_simulating());
        assert!(EditorMode::Play.is_simulating());
        assert!(!EditorMode::Paused.is_simulating());
    }

    #[test]
    fn test_editor_mode_allows_scene_changes() {
        assert!(EditorMode::Edit.allows_scene_changes());
        assert!(!EditorMode::Play.allows_scene_changes());
        assert!(!EditorMode::Paused.allows_scene_changes());
    }

    #[test]
    fn test_editor_mode_next_mode() {
        assert_eq!(EditorMode::Edit.next_mode(), EditorMode::Play);
        assert_eq!(EditorMode::Play.next_mode(), EditorMode::Paused);
        assert_eq!(EditorMode::Paused.next_mode(), EditorMode::Play);
    }

    #[test]
    fn test_editor_mode_action_verb() {
        assert_eq!(EditorMode::Edit.action_verb(), "Stop");
        assert_eq!(EditorMode::Play.action_verb(), "Play");
        assert_eq!(EditorMode::Paused.action_verb(), "Pause");
    }

    #[test]
    fn test_editor_mode_display() {
        let display = format!("{}", EditorMode::Edit);
        assert!(display.contains("Edit"));
    }
}
