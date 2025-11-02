//! Player profile demo
//! 
//! Run with: cargo run -p astraweave-persistence-player --example profile_demo

use astraweave_persistence_player::PlayerProfile;

fn main() {
    println!("=== AstraWeave Player Profile Demo ===\n");
    
    // Load or create profile
    println!("📂 Loading profile...");
    let mut profile = PlayerProfile::quick_load().unwrap();
    
    // Display current state
    println!("\n📊 Current Profile:");
    println!("   Player: {}", profile.name);
    println!("   Playtime: {} seconds ({:.1} hours)", 
        profile.stats.playtime_seconds, 
        profile.stats.playtime_seconds as f64 / 3600.0
    );
    println!("   Kills: {}", profile.stats.enemies_defeated);
    println!("   Deaths: {}", profile.stats.deaths);
    println!("   Achievements: {}", profile.stats.achievements.len());
    println!("   Abilities: {}", profile.unlocks.abilities.len());
    println!("   Items: {}", profile.unlocks.items.len());
    println!("   Levels: {}", profile.unlocks.levels.len());
    
    // Make some changes
    println!("\n🎮 Making changes...");
    profile.unlock_ability("Dash");
    profile.unlock_ability("Double Jump");
    profile.unlock_item("Health Potion");
    profile.unlock_level("Level 2");
    profile.grant_achievement("First Blood");
    profile.record_kill();
    profile.record_kill();
    profile.add_playtime(120); // 2 minutes
    
    // Display settings
    println!("\n⚙️  Settings:");
    println!("   Resolution: {}×{}", 
        profile.settings.graphics.resolution.0,
        profile.settings.graphics.resolution.1
    );
    println!("   Quality: {:?}", profile.settings.graphics.quality);
    println!("   Master Volume: {:.0}%", profile.settings.audio.master_volume * 100.0);
    println!("   Mouse Sensitivity: {:.2}", profile.settings.controls.mouse_sensitivity);
    
    // Apply settings (will just log for now)
    println!("\n📤 Applying settings...");
    profile.settings.apply();
    
    // Save
    println!("\n💾 Saving profile...");
    profile.quick_save().unwrap();
    
    println!("\n✅ Profile updated and saved to: saves/player_profile.toml");
    println!("✅ You can inspect the file to see human-readable TOML format");
}
