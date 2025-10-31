//! Autosave system

use crate::PlayerProfile;
use std::time::{Duration, Instant};

/// Autosaver for periodic profile saving
pub struct AutoSaver {
    last_save: Instant,
    interval: Duration,
    dirty: bool,
}

impl AutoSaver {
    /// Create new autosaver (default: save every 30 seconds)
    pub fn new() -> Self {
        Self {
            last_save: Instant::now(),
            interval: Duration::from_secs(30),
            dirty: false,
        }
    }
    
    /// Create autosaver with custom interval
    pub fn with_interval(interval: Duration) -> Self {
        Self {
            last_save: Instant::now(),
            interval,
            dirty: false,
        }
    }
    
    /// Mark profile as dirty (needs save)
    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }
    
    /// Update autosaver (call every frame or tick)
    /// 
    /// This will save the profile if:
    /// - Profile is dirty (has changes)
    /// - Interval has elapsed since last save
    pub fn update(&mut self, profile: &PlayerProfile) {
        if !self.dirty {
            return;
        }
        
        if self.last_save.elapsed() >= self.interval {
            if let Err(e) = profile.quick_save() {
                eprintln!("⚠️  Autosave failed: {}", e);
            } else {
                println!("💾 Autosaved profile");
            }
            
            self.last_save = Instant::now();
            self.dirty = false;
        }
    }
}

impl Default for AutoSaver {
    fn default() -> Self {
        Self::new()
    }
}
