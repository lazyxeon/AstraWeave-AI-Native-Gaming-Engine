//! Platform trait for testable Steam integration
//!
//! This trait defines the interface between the game and the platform (Steam).
//! During testing, a MockPlatform can be used instead of the real Steam client.

use anyhow::Result;

/// Platform abstraction for Steam features
///
/// Implement this trait to provide platform-specific functionality.
/// Use `MockPlatform` for testing without Steam.
pub trait Platform: Send + Sync {
    /// Unlock an achievement by name
    fn unlock_achievement(&self, name: &str) -> Result<()>;

    /// Set a stat value (integer)
    fn set_stat_i32(&self, name: &str, value: i32) -> Result<()>;

    /// Set a stat value (float)
    fn set_stat_f32(&self, name: &str, value: f32) -> Result<()>;

    /// Get a stat value (integer)
    fn get_stat_i32(&self, name: &str) -> Result<i32>;

    /// Save data to cloud storage
    fn cloud_save(&self, filename: &str, data: &[u8]) -> Result<()>;

    /// Load data from cloud storage
    fn cloud_load(&self, filename: &str) -> Result<Vec<u8>>;

    /// Check if cloud storage is enabled
    fn cloud_enabled(&self) -> bool;

    /// Store stats to Steam servers
    fn store_stats(&self) -> Result<()>;

    /// Get the player's display name
    fn player_name(&self) -> String;

    /// Check if the platform is available
    fn is_available(&self) -> bool;
}

/// Mock platform for testing without Steam
#[cfg(any(test, feature = "mock"))]
pub struct MockPlatform {
    player_name: String,
}

#[cfg(any(test, feature = "mock"))]
impl MockPlatform {
    pub fn new(player_name: impl Into<String>) -> Self {
        Self {
            player_name: player_name.into(),
        }
    }
}

#[cfg(any(test, feature = "mock"))]
impl Platform for MockPlatform {
    fn unlock_achievement(&self, name: &str) -> Result<()> {
        tracing::info!("[MockPlatform] Achievement unlocked: {}", name);
        Ok(())
    }

    fn set_stat_i32(&self, name: &str, value: i32) -> Result<()> {
        tracing::info!("[MockPlatform] Stat {} = {}", name, value);
        Ok(())
    }

    fn set_stat_f32(&self, name: &str, value: f32) -> Result<()> {
        tracing::info!("[MockPlatform] Stat {} = {}", name, value);
        Ok(())
    }

    fn get_stat_i32(&self, _name: &str) -> Result<i32> {
        Ok(0)
    }

    fn cloud_save(&self, filename: &str, data: &[u8]) -> Result<()> {
        tracing::info!(
            "[MockPlatform] Cloud save: {} ({} bytes)",
            filename,
            data.len()
        );
        Ok(())
    }

    fn cloud_load(&self, filename: &str) -> Result<Vec<u8>> {
        tracing::info!("[MockPlatform] Cloud load: {}", filename);
        Ok(vec![])
    }

    fn cloud_enabled(&self) -> bool {
        true
    }

    fn store_stats(&self) -> Result<()> {
        tracing::info!("[MockPlatform] Stats stored");
        Ok(())
    }

    fn player_name(&self) -> String {
        self.player_name.clone()
    }

    fn is_available(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_platform_achievement() {
        let platform = MockPlatform::new("TestPlayer");
        assert!(platform.unlock_achievement("first_blood").is_ok());
    }

    #[test]
    fn test_mock_platform_stats() {
        let platform = MockPlatform::new("TestPlayer");
        assert!(platform.set_stat_i32("kills", 42).is_ok());
        assert!(platform.set_stat_f32("playtime", 10.5).is_ok());
    }

    #[test]
    fn test_mock_platform_cloud() {
        let platform = MockPlatform::new("TestPlayer");
        assert!(platform.cloud_save("save.dat", b"test data").is_ok());
        assert!(platform.cloud_load("save.dat").is_ok());
    }

    #[test]
    fn test_mock_platform_player_name() {
        let platform = MockPlatform::new("TestPlayer");
        assert_eq!(platform.player_name(), "TestPlayer");
    }

    #[test]
    fn test_mock_platform_available() {
        let platform = MockPlatform::new("TestPlayer");
        assert!(platform.is_available());
    }
}
