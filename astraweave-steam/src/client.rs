//! Steam client lifecycle management
//!
//! Handles Steam initialization and the callback loop (heartbeat).
//! **CRITICAL**: `update()` must be called every frame!

use anyhow::{anyhow, Result};
use steamworks::Client;

use crate::platform::Platform;

/// Steam integration client
///
/// Wraps the Steamworks SDK client and manages the callback loop.
pub struct SteamIntegration {
    client: Client,
}

impl SteamIntegration {
    /// Initialize Steam integration
    ///
    /// # Notes
    /// - Requires Steam client to be running
    /// - Requires `steam_appid.txt` with App ID in working directory
    ///   (use 480 for testing with Spacewar)
    ///
    /// # Errors
    /// Returns an error if:
    /// - Steam client is not running
    /// - `steam_appid.txt` is missing or invalid
    /// - Initialization fails
    pub fn init() -> Result<Self> {
        let client = Client::init().map_err(|e| anyhow!("Steam initialization failed: {:?}", e))?;

        tracing::info!("Steam initialized successfully");

        Ok(Self { client })
    }

    /// Update the Steam callbacks
    ///
    /// **MUST be called every frame** for Steam features to work properly.
    /// This processes achievement popups, cloud save sync, overlay input, etc.
    pub fn update(&self) {
        self.client.run_callbacks();
    }

    /// Get the underlying Steam client (for advanced usage)
    pub fn client(&self) -> &Client {
        &self.client
    }
}

impl Platform for SteamIntegration {
    fn unlock_achievement(&self, name: &str) -> Result<()> {
        let user_stats = self.client.user_stats();
        user_stats
            .achievement(name)
            .set()
            .map_err(|e| anyhow!("Failed to set achievement: {:?}", e))?;
        user_stats
            .store_stats()
            .map_err(|e| anyhow!("Failed to store stats: {:?}", e))?;
        tracing::info!("Achievement unlocked: {}", name);
        Ok(())
    }

    fn set_stat_i32(&self, name: &str, value: i32) -> Result<()> {
        let user_stats = self.client.user_stats();
        user_stats
            .set_stat_i32(name, value)
            .map_err(|e| anyhow!("Failed to set int stat: {:?}", e))?;
        Ok(())
    }

    fn set_stat_f32(&self, name: &str, value: f32) -> Result<()> {
        let user_stats = self.client.user_stats();
        user_stats
            .set_stat_f32(name, value)
            .map_err(|e| anyhow!("Failed to set float stat: {:?}", e))?;
        Ok(())
    }

    fn get_stat_i32(&self, name: &str) -> Result<i32> {
        let user_stats = self.client.user_stats();
        let value = user_stats
            .get_stat_i32(name)
            .map_err(|e| anyhow!("Failed to get stat: {:?}", e))?;
        Ok(value)
    }

    fn cloud_save(&self, filename: &str, data: &[u8]) -> Result<()> {
        // TODO: Implement using remote_storage callback-based API
        // For now, log and succeed (can use file-based fallback)
        tracing::warn!(
            "Steam Cloud save not yet implemented: {} ({} bytes)",
            filename,
            data.len()
        );
        Ok(())
    }

    fn cloud_load(&self, filename: &str) -> Result<Vec<u8>> {
        // TODO: Implement using remote_storage callback-based API
        tracing::warn!("Steam Cloud load not yet implemented: {}", filename);
        Err(anyhow!("Cloud save not yet implemented"))
    }

    fn cloud_enabled(&self) -> bool {
        self.client.remote_storage().is_cloud_enabled_for_app()
    }

    fn store_stats(&self) -> Result<()> {
        let user_stats = self.client.user_stats();
        user_stats
            .store_stats()
            .map_err(|e| anyhow!("Failed to store stats: {:?}", e))?;
        Ok(())
    }

    fn player_name(&self) -> String {
        self.client.friends().name()
    }

    fn is_available(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    // Note: These tests require Steam to be running and logged in,
    // so they are marked as ignored by default.
    // Run with: cargo test -- --ignored

    #[test]
    #[ignore = "requires Steam client running"]
    fn test_steam_init() {
        use super::*;
        let result = SteamIntegration::init();
        assert!(
            result.is_ok(),
            "Steam should initialize with steam_appid.txt present"
        );
    }

    #[test]
    #[ignore = "requires Steam client running"]
    fn test_steam_player_name() {
        use super::*;
        let steam = SteamIntegration::init().unwrap();
        let name = steam.player_name();
        assert!(!name.is_empty(), "Player name should not be empty");
    }
}
