//! Settings application and integration

use crate::{AudioSettings, ControlSettings, GameSettings, GraphicsSettings};

impl GameSettings {
    /// Apply all settings to game systems
    pub fn apply(&self) {
        self.graphics.apply();
        self.audio.apply();
        self.controls.apply();
    }
}

impl GraphicsSettings {
    /// Apply graphics settings to renderer
    ///
    /// TODO: Integrate with Phase 8.2 renderer when available
    pub fn apply(&self) {
        println!("📊 Graphics Settings Applied:");
        println!("   Resolution: {}×{}", self.resolution.0, self.resolution.1);
        println!("   Quality: {:?}", self.quality);
        println!("   VSync: {}", self.vsync);
        println!("   Fullscreen: {}", self.fullscreen);
    }
}

impl AudioSettings {
    /// Apply audio settings to audio system
    ///
    /// TODO: Integrate with Phase 8.4 audio mixer when available
    pub fn apply(&self) {
        println!("🔊 Audio Settings Applied:");
        println!("   Master: {:.0}%", self.master_volume * 100.0);
        println!("   Music: {:.0}%", self.music_volume * 100.0);
        println!("   SFX: {:.0}%", self.sfx_volume * 100.0);
        println!("   Voice: {:.0}%", self.voice_volume * 100.0);
        println!("   Muted: {}", self.muted);
    }
}

impl ControlSettings {
    /// Apply control settings to input system
    ///
    /// TODO: Integrate with input system when available
    pub fn apply(&self) {
        println!("🎮 Control Settings Applied:");
        println!("   Mouse Sensitivity: {:.2}", self.mouse_sensitivity);
        println!("   Invert Y: {}", self.invert_y);
        println!("   Key Bindings: {} actions", self.key_bindings.len());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::QualityPreset;

    #[test]
    fn test_graphics_apply() {
        let settings = GraphicsSettings {
            resolution: (1920, 1080),
            quality: QualityPreset::High,
            vsync: true,
            fullscreen: false,
        };
        // Just verify no panic
        settings.apply();
    }

    #[test]
    fn test_audio_apply() {
        let settings = AudioSettings {
            master_volume: 0.8,
            music_volume: 0.6,
            sfx_volume: 0.7,
            voice_volume: 1.0,
            muted: false,
        };
        // Just verify no panic
        settings.apply();
    }

    #[test]
    fn test_audio_muted_apply() {
        let settings = AudioSettings {
            master_volume: 0.0,
            music_volume: 0.0,
            sfx_volume: 0.0,
            voice_volume: 0.0,
            muted: true,
        };
        settings.apply();
    }

    #[test]
    fn test_control_apply() {
        let settings = ControlSettings {
            mouse_sensitivity: 1.5,
            invert_y: true,
            key_bindings: std::collections::HashMap::new(),
        };
        settings.apply();
    }

    #[test]
    fn test_control_with_bindings() {
        let mut bindings = std::collections::HashMap::new();
        bindings.insert("Jump".to_string(), "Space".to_string());
        bindings.insert("Attack".to_string(), "MouseLeft".to_string());
        let settings = ControlSettings {
            mouse_sensitivity: 1.0,
            invert_y: false,
            key_bindings: bindings,
        };
        settings.apply();
    }

    #[test]
    fn test_game_settings_apply() {
        let settings = GameSettings {
            graphics: GraphicsSettings {
                resolution: (1920, 1080),
                quality: QualityPreset::Medium,
                vsync: true,
                fullscreen: false,
            },
            audio: AudioSettings {
                master_volume: 0.5,
                music_volume: 0.5,
                sfx_volume: 0.5,
                voice_volume: 0.5,
                muted: false,
            },
            controls: ControlSettings {
                mouse_sensitivity: 1.0,
                invert_y: false,
                key_bindings: std::collections::HashMap::new(),
            },
        };
        settings.apply();
    }
}
