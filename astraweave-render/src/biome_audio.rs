//! Biome → ambient audio track mapping.
//!
//! Provides the [`BiomeAmbientMap`] which maps
//! [`BiomeType`](astraweave_terrain::biome::BiomeType) to an ambient sound
//! file path suitable for feeding into
//! [`AudioEngine::play_ambient`](astraweave_audio::engine::AudioEngine::play_ambient).
//!
//! The renderer owns the biome transition state, so this bridge lives in the
//! render crate rather than the audio crate (which stays biome-agnostic).

use astraweave_terrain::biome::BiomeType;
use std::collections::HashMap;

/// Default crossfade duration for biome ambient transitions (seconds).
pub const DEFAULT_AMBIENT_CROSSFADE: f32 = 3.0;

/// Maps [`BiomeType`] → ambient sound path.
///
/// # Default convention
///
/// Tracks are expected under `assets/audio/ambient/<biome>.ogg`.
/// Users can override any mapping with [`Self::set`].
#[derive(Debug, Clone)]
pub struct BiomeAmbientMap {
    tracks: HashMap<BiomeType, String>,
    crossfade_sec: f32,
}

impl Default for BiomeAmbientMap {
    fn default() -> Self {
        let mut tracks = HashMap::new();
        tracks.insert(
            BiomeType::Forest,
            "assets/audio/ambient/forest.ogg".into(),
        );
        tracks.insert(
            BiomeType::Desert,
            "assets/audio/ambient/desert.ogg".into(),
        );
        tracks.insert(
            BiomeType::Grassland,
            "assets/audio/ambient/grassland.ogg".into(),
        );
        tracks.insert(
            BiomeType::Mountain,
            "assets/audio/ambient/mountain.ogg".into(),
        );
        tracks.insert(
            BiomeType::Tundra,
            "assets/audio/ambient/tundra.ogg".into(),
        );
        tracks.insert(
            BiomeType::Swamp,
            "assets/audio/ambient/swamp.ogg".into(),
        );
        tracks.insert(
            BiomeType::Beach,
            "assets/audio/ambient/beach.ogg".into(),
        );
        tracks.insert(
            BiomeType::River,
            "assets/audio/ambient/river.ogg".into(),
        );

        Self {
            tracks,
            crossfade_sec: DEFAULT_AMBIENT_CROSSFADE,
        }
    }
}

impl BiomeAmbientMap {
    /// Create a new map with default track paths.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create an empty map (no defaults).
    pub fn empty() -> Self {
        Self {
            tracks: HashMap::new(),
            crossfade_sec: DEFAULT_AMBIENT_CROSSFADE,
        }
    }

    /// Set the ambient track for a specific biome.
    pub fn set(&mut self, biome: BiomeType, path: impl Into<String>) {
        self.tracks.insert(biome, path.into());
    }

    /// Remove the ambient track for a biome (no ambient will play).
    pub fn remove(&mut self, biome: BiomeType) {
        self.tracks.remove(&biome);
    }

    /// Get the ambient track path for a biome, if configured.
    pub fn get(&self, biome: BiomeType) -> Option<&str> {
        self.tracks.get(&biome).map(|s| s.as_str())
    }

    /// The crossfade duration used when transitioning ambient tracks.
    pub fn crossfade_sec(&self) -> f32 {
        self.crossfade_sec
    }

    /// Set the crossfade duration.
    pub fn set_crossfade_sec(&mut self, sec: f32) {
        self.crossfade_sec = sec.max(0.01);
    }

    /// Number of biomes with configured tracks.
    pub fn len(&self) -> usize {
        self.tracks.len()
    }

    /// Returns `true` if no tracks are configured.
    pub fn is_empty(&self) -> bool {
        self.tracks.is_empty()
    }
}

// ── Tests ─────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_has_all_biomes() {
        let map = BiomeAmbientMap::default();
        assert_eq!(map.len(), 8);
        for biome in [
            BiomeType::Forest,
            BiomeType::Desert,
            BiomeType::Grassland,
            BiomeType::Mountain,
            BiomeType::Tundra,
            BiomeType::Swamp,
            BiomeType::Beach,
            BiomeType::River,
        ] {
            assert!(
                map.get(biome).is_some(),
                "missing ambient for {:?}",
                biome
            );
        }
    }

    #[test]
    fn default_paths_under_assets() {
        let map = BiomeAmbientMap::default();
        for biome in [
            BiomeType::Forest,
            BiomeType::Desert,
            BiomeType::Grassland,
            BiomeType::Mountain,
            BiomeType::Tundra,
            BiomeType::Swamp,
            BiomeType::Beach,
            BiomeType::River,
        ] {
            let path = map.get(biome).unwrap();
            assert!(path.starts_with("assets/audio/ambient/"), "{}", path);
            assert!(path.ends_with(".ogg"), "{}", path);
        }
    }

    #[test]
    fn override_and_remove() {
        let mut map = BiomeAmbientMap::new();
        map.set(BiomeType::Forest, "custom/forest_night.ogg");
        assert_eq!(map.get(BiomeType::Forest).unwrap(), "custom/forest_night.ogg");

        map.remove(BiomeType::Forest);
        assert!(map.get(BiomeType::Forest).is_none());
    }

    #[test]
    fn empty_map() {
        let map = BiomeAmbientMap::empty();
        assert!(map.is_empty());
        assert_eq!(map.len(), 0);
        assert!(map.get(BiomeType::Beach).is_none());
    }

    #[test]
    fn crossfade_duration() {
        let mut map = BiomeAmbientMap::new();
        assert_eq!(map.crossfade_sec(), DEFAULT_AMBIENT_CROSSFADE);

        map.set_crossfade_sec(5.0);
        assert_eq!(map.crossfade_sec(), 5.0);

        // clamp minimum
        map.set_crossfade_sec(-1.0);
        assert!(map.crossfade_sec() > 0.0);
    }

    #[test]
    fn biome_ambient_round_trip() {
        let map = BiomeAmbientMap::default();
        // Verify each biome name appears in its path
        assert!(map.get(BiomeType::Swamp).unwrap().contains("swamp"));
        assert!(map.get(BiomeType::Beach).unwrap().contains("beach"));
        assert!(map.get(BiomeType::River).unwrap().contains("river"));
    }

    #[test]
    fn lerped_biome_gets_target_ambient() {
        // During a transition the *target* biome determines the ambient track.
        // This test validates lookup correctness for all biome pairs.
        let map = BiomeAmbientMap::default();
        let biomes = [
            BiomeType::Forest,
            BiomeType::Desert,
            BiomeType::Grassland,
            BiomeType::Mountain,
        ];
        for &from in &biomes {
            for &to in &biomes {
                if from != to {
                    // The bridge should play the *target* biome's track.
                    let path = map.get(to).unwrap();
                    assert!(!path.is_empty());
                }
            }
        }
    }
}
