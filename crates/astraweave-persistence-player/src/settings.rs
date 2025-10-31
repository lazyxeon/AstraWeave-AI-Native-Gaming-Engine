//! Settings application and integration

use crate::{GameSettings, GraphicsSettings, AudioSettings, ControlSettings};

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
