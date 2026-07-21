//! AD.4.A §5a **Option C** (director-ratified, AD.5.A Fix 3): name-based
//! palette→layer remap at paint time.
//!
//! The 21-entry `MaterialLibrary` palette and a loaded canonical biome pack
//! (≤32 layers; 8 for `assets/materials/biomes`) are independent namespaces.
//! Historically the raw palette ID was written into the splat, so IDs ≥
//! `active_layer_count` rendered the shader's layer-0 fallback (stretched
//! grass) and IDs 0–7 carried a name↔slot mismatch ("Wood Planks" painted
//! the beach-sand layer). The remap resolves each palette entry against the
//! loaded pack; entries with no honest match are unpaintable and the UI
//! greys them out — that is the design working, not a missing feature.
//!
//! **Join key** (evidence-based, derived from the loaded pack's data): the
//! lowercase file stem of each layer's albedo source versus the library
//! entry's canonical `name`. The library `name` is itself defined as an
//! asset-path stem (`assets/materials/{name}.png` —
//! `astraweave-render/src/material_library.rs`), and the pack layer's albedo
//! stem is the pack's material identity (`materials.toml` `albedo` key); for
//! most matches the two literally denote the same file. The pack layer `key`
//! field is *not* usable — it names the biome slot ("grassland", "desert"),
//! not the material. No hardcoded index table exists to rot when the pack
//! changes: swapping the pack re-resolves the whole palette.

use crate::viewport::types::MATERIAL_NAMES;

/// Number of palette entries (the canonical `MaterialLibrary` size).
pub const PALETTE_LEN: usize = MATERIAL_NAMES.len();

/// Resolved palette→layer mapping for the currently-loaded terrain layer set.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PaletteRemap {
    /// `resolved[palette_id]` = pack layer index to paint, `None` = unpaintable.
    resolved: [Option<u32>; PALETTE_LEN],
    /// True when derived from the synthetic placeholder set (no pack loaded).
    placeholder: bool,
}

impl PaletteRemap {
    /// Resolve the palette against the loaded pack's per-layer albedo stems
    /// (`stems[layer_index]`; `None` for layers with no loaded albedo).
    ///
    /// Duplicate stems resolve to the lowest layer index — deterministic, and
    /// benign for the biomes pack whose `desert`/`beach` layers share one
    /// `sand` file at identical tiling.
    pub fn resolve(stems: &[Option<String>]) -> Self {
        let mut resolved = [None; PALETTE_LEN];
        for (palette_id, name) in MATERIAL_NAMES.iter().enumerate() {
            resolved[palette_id] = stems.iter().enumerate().find_map(|(idx, stem)| {
                stem.as_deref()
                    .is_some_and(|s| s.eq_ignore_ascii_case(name))
                    .then_some(idx as u32)
            });
        }
        Self {
            resolved,
            placeholder: false,
        }
    }

    /// Identity mapping for the synthetic placeholder layer set (pack absent
    /// or failed to load): palette IDs `0..active_layers` paint their own
    /// slot (legacy behaviour, distinct synthetic checkers), the rest are
    /// unpaintable — which retires the ID-≥-count layer-0 collapse even in
    /// placeholder mode.
    pub fn placeholder_identity(active_layers: u32) -> Self {
        let mut resolved = [None; PALETTE_LEN];
        let n = (active_layers as usize).min(PALETTE_LEN);
        for (id, slot) in resolved.iter_mut().enumerate().take(n) {
            *slot = Some(id as u32);
        }
        Self {
            resolved,
            placeholder: true,
        }
    }

    /// The pack layer index this palette entry paints, `None` = unpaintable.
    pub fn layer_for(&self, palette_id: usize) -> Option<u32> {
        self.resolved.get(palette_id).copied().flatten()
    }

    /// Whether this mapping came from the synthetic placeholder set.
    pub fn is_placeholder(&self) -> bool {
        self.placeholder
    }

    /// Palette IDs that resolve to a paintable layer, ascending.
    pub fn paintable_ids(&self) -> impl Iterator<Item = usize> + '_ {
        self.resolved
            .iter()
            .enumerate()
            .filter_map(|(i, r)| r.map(|_| i))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stems(v: &[Option<&str>]) -> Vec<Option<String>> {
        v.iter().map(|s| s.map(str::to_string)).collect()
    }

    /// The live 8-layer biomes pack post-T.1 (albedo stems from
    /// assets/materials/biomes/materials.toml, in arrays.toml slot order).
    /// T.1 (2026-07-21): slot 6 is `beach` (coast_sand_01 cook) — no longer
    /// the desert `sand` duplicate. `beach` has no MaterialLibrary name, so
    /// slot 6 is biome-paint-only (unmatched by the palette) BY DESIGN; the
    /// duplicate-stem rule stays covered by the synthetic fixture below.
    fn biomes_pack_stems() -> Vec<Option<String>> {
        stems(&[
            Some("grass"),         // 0 grassland
            Some("sand"),          // 1 desert
            Some("tree_leaves"),   // 2 forest (derived_1k, Fix 1)
            Some("mountain_rock"), // 3 mountain
            Some("snow"),          // 4 tundra
            Some("mud"),           // 5 swamp
            Some("beach"),         // 6 beach (derived_1k, T.1)
            Some("gravel"),        // 7 river (derived_1k)
        ])
    }

    #[test]
    fn biomes_pack_resolves_exactly_seven_entries() {
        let remap = PaletteRemap::resolve(&biomes_pack_stems());
        let paintable: Vec<usize> = remap.paintable_ids().collect();
        // grass=0, sand=1, mountain_rock=3, snow=4, mud=5, gravel=12, tree_leaves=20
        assert_eq!(paintable, vec![0, 1, 3, 4, 5, 12, 20]);
        assert_eq!(remap.layer_for(0), Some(0)); // Grass → grassland
        assert_eq!(remap.layer_for(1), Some(1)); // Sand → desert (unique match post-T.1)
        assert_eq!(remap.layer_for(3), Some(3)); // Mountain Rock → mountain
        assert_eq!(remap.layer_for(4), Some(4)); // Snow → tundra
        assert_eq!(remap.layer_for(5), Some(5)); // Mud → swamp
        assert_eq!(remap.layer_for(12), Some(7)); // Gravel → river
        assert_eq!(remap.layer_for(20), Some(2)); // Tree Leaves → forest
        assert!(!remap.is_placeholder());
    }

    #[test]
    fn mismatched_names_do_not_cross_paint() {
        let remap = PaletteRemap::resolve(&biomes_pack_stems());
        // The S1 mislabel: "Wood Planks" (id 6) must NOT paint beach sand
        // (layer 6) — it has no stem match, so it is unpaintable.
        assert_eq!(remap.layer_for(6), None);
        // "Forest Floor" (id 2) is not the forest layer's material name.
        assert_eq!(remap.layer_for(2), None);
        // Former ID-≥-8 fallback victims are unpaintable, not layer-0.
        for id in [8usize, 10, 17, 18] {
            assert_eq!(remap.layer_for(id), None, "palette id {id}");
        }
    }

    #[test]
    fn no_match_when_pack_shares_no_stems() {
        let remap = PaletteRemap::resolve(&stems(&[Some("basalt"), Some("obsidian"), None]));
        assert_eq!(remap.paintable_ids().count(), 0);
    }

    #[test]
    fn duplicate_stem_resolves_to_lowest_layer_index() {
        let remap = PaletteRemap::resolve(&stems(&[Some("mud"), Some("sand"), Some("sand")]));
        assert_eq!(remap.layer_for(1), Some(1)); // Sand → layer 1, not 2
    }

    #[test]
    fn unloaded_albedo_slots_are_skipped() {
        let remap = PaletteRemap::resolve(&stems(&[None, Some("grass")]));
        assert_eq!(remap.layer_for(0), Some(1)); // Grass matched at slot 1
    }

    #[test]
    fn pack_swap_re_resolution_moves_the_mapping() {
        let a = PaletteRemap::resolve(&stems(&[Some("grass"), Some("ice")]));
        let b = PaletteRemap::resolve(&stems(&[Some("ice"), Some("grass")]));
        assert_ne!(a, b);
        assert_eq!(a.layer_for(0), Some(0));
        assert_eq!(b.layer_for(0), Some(1)); // grass followed its layer
        assert_eq!(a.layer_for(13), Some(1)); // ice
        assert_eq!(b.layer_for(13), Some(0));
    }

    #[test]
    fn placeholder_identity_maps_first_n_only() {
        let remap = PaletteRemap::placeholder_identity(8);
        for id in 0..8usize {
            assert_eq!(remap.layer_for(id), Some(id as u32));
        }
        for id in 8..PALETTE_LEN {
            assert_eq!(remap.layer_for(id), None, "palette id {id}");
        }
        assert!(remap.is_placeholder());
    }

    #[test]
    fn out_of_range_palette_id_is_none() {
        let remap = PaletteRemap::resolve(&biomes_pack_stems());
        assert_eq!(remap.layer_for(PALETTE_LEN), None);
        assert_eq!(remap.layer_for(usize::MAX), None);
    }
}
