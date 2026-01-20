//! Fuzz target for packet parsing in networking module.
//!
//! Tests that arbitrary byte sequences don't cause crashes when parsed.

#![no_main]

use libfuzzer_sys::fuzz_target;

use astraweave_core::IVec2;
use astraweave_net::{Delta, EntityDelta, EntityState, Snapshot};

fuzz_target!(|data: &[u8]| {
    // Attempt to deserialize as Snapshot
    let _ = bincode::deserialize::<Snapshot>(data);

    // Attempt to deserialize as Delta
    let _ = bincode::deserialize::<Delta>(data);

    // Attempt to deserialize as EntityState
    let _ = bincode::deserialize::<EntityState>(data);

    // Attempt to deserialize as EntityDelta
    let _ = bincode::deserialize::<EntityDelta>(data);

    // Attempt to deserialize as JSON (common network format)
    let _ = serde_json::from_slice::<Snapshot>(data);
    let _ = serde_json::from_slice::<Delta>(data);

    // Test MessagePack if available (common for game networking)
    #[cfg(feature = "rmp")]
    {
        let _ = rmp_serde::from_slice::<Snapshot>(data);
        let _ = rmp_serde::from_slice::<Delta>(data);
    }
});
