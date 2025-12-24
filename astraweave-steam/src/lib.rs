//! # AstraWeave Steam Integration
//!
//! Steamworks SDK integration for AstraWeave game engine.
//!
//! ## Features
//!
//! - **Achievements**: Track and unlock player achievements
//! - **Cloud Saves**: Sync saves across devices via Steam Cloud
//! - **Stats**: Track player statistics
//! - **Platform Trait**: Testable abstraction for mocking
//!
//! ## Critical Usage Notes
//!
//! 1. **Callback Loop**: You MUST call `SteamIntegration::update()` every frame
//! 2. **App ID**: `steam_appid.txt` with your App ID must exist at runtime
//! 3. **Thread Safety**: Initialize/access on main thread only
//!
//! ## Example
//!
//! ```ignore
//! use astraweave_steam::{SteamIntegration, Platform};
//!
//! // Initialize (use 480 for testing)
//! let steam = SteamIntegration::init(480)?;
//!
//! // Game loop
//! loop {
//!     // CRITICAL: Call every frame!
//!     steam.update();
//!     
//!     // Use platform features
//!     steam.unlock_achievement("first_blood")?;
//! }
//! ```
//!
//! ## Testing
//!
//! Use `MockPlatform` for unit tests (requires `mock` feature):
//!
//! ```ignore
//! use astraweave_steam::MockPlatform;
//! use astraweave_steam::Platform;
//!
//! let platform = MockPlatform::new("TestPlayer");
//! platform.unlock_achievement("test").unwrap();
//! ```

pub mod client;
pub mod platform;

// Re-exports
pub use client::SteamIntegration;
pub use platform::Platform;

#[cfg(any(test, feature = "mock"))]
pub use platform::MockPlatform;

/// Default Steam App ID for testing (Spacewar)
pub const TEST_APP_ID: u32 = 480;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exports() {
        // Verify exports compile - init takes no parameters
        let _: fn() -> anyhow::Result<SteamIntegration> = SteamIntegration::init;
    }

    #[test]
    fn test_mock_platform_integration() {
        let platform = MockPlatform::new("Tester");

        // Test all Platform trait methods
        assert!(platform.unlock_achievement("test_achievement").is_ok());
        assert!(platform.set_stat_i32("kills", 10).is_ok());
        assert!(platform.set_stat_f32("playtime", 5.5).is_ok());
        assert!(platform.cloud_save("test.sav", b"data").is_ok());
        assert!(platform.cloud_load("test.sav").is_ok());
        assert!(platform.cloud_enabled());
        assert!(platform.store_stats().is_ok());
        assert_eq!(platform.player_name(), "Tester");
        assert!(platform.is_available());
    }

    #[test]
    fn test_test_app_id() {
        assert_eq!(TEST_APP_ID, 480);
    }
}
