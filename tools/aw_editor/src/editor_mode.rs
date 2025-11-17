use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EditorMode {
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
}

impl Default for EditorMode {
    fn default() -> Self {
        EditorMode::Edit
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
}
