//! Veilweaver Slice Loader
//!
//! Utility demo that force-loads the Loomspire greybox cells and prints streaming metrics.

use anyhow::Result;
use astraweave_gameplay::VeilweaverSliceMetadata;
use astraweave_scene::partitioned_scene::PartitionedScene;
use astraweave_scene::streaming::{StreamingConfig, StreamingEvent};
use astraweave_scene::world_partition::{GridConfig, GridCoord};
use glam::Vec3;

const VEILWEAVER_CELLS: &[GridCoord] = &[
    GridCoord { x: 100, y: 0, z: 0 },
    GridCoord { x: 101, y: 0, z: 0 },
    GridCoord { x: 102, y: 0, z: 0 },
    GridCoord { x: 102, y: 1, z: 0 },
    GridCoord { x: 103, y: 0, z: 0 },
    GridCoord { x: 104, y: 0, z: 0 },
];

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== Veilweaver Slice Loader ===\n");

    let grid_config = GridConfig {
        cell_size: 100.0,
        world_bounds: (-20000.0, 20000.0, -20000.0, 20000.0),
    };

    let streaming_config = StreamingConfig {
        max_active_cells: 12,
        lru_cache_size: 4,
        streaming_radius: 400.0,
        max_concurrent_loads: 4,
    };

    let mut scene = PartitionedScene::new(grid_config, streaming_config);

    scene.manager.add_event_listener(|event| match event {
        StreamingEvent::CellLoaded(coord) => {
            println!("[LOADED] Cell ({}, {}, {})", coord.x, coord.y, coord.z);
        }
        StreamingEvent::CellLoadFailed(coord, err) => {
            eprintln!(
                "[FAILED] Cell ({}, {}, {}): {}",
                coord.x, coord.y, coord.z, err
            );
        }
        _ => {}
    });

    println!("Forcing load of Veilweaver Loomspire cells...\n");
    for coord in VEILWEAVER_CELLS {
        scene.manager.force_load_cell(*coord).await?;
    }

    // Simulate moving the camera along the slice once to trigger streaming updates.
    for coord in VEILWEAVER_CELLS {
        let center = coord.to_world_center(grid_config.cell_size);
        scene
            .manager
            .update_streaming(Vec3::new(center.x, 5.0, center.z))
            .await?;
    }

    let metrics = scene.manager.metrics().clone();
    println!("\nStreaming Metrics:");
    println!("  Active cells: {}", metrics.active_cells);
    println!("  Loaded cells: {}", metrics.loaded_cells);
    println!("  Total loads : {}", metrics.total_loads);
    println!("  Total unloads: {}", metrics.total_unloads);
    println!("  Failed loads: {}", metrics.failed_loads);

    let slice_metadata = {
        let partition = scene.partition.read().await;
        let cells: Vec<_> = VEILWEAVER_CELLS
            .iter()
            .filter_map(|coord| partition.get_cell(*coord))
            .collect();
        VeilweaverSliceMetadata::from_cells(&cells)?
    };

    println!("\nAnchors: {}", slice_metadata.anchors.len());
    for anchor in &slice_metadata.anchors {
        println!(
            "  [{}] id={} type={:?} echo_cost={:?}",
            format_coord(anchor.cell),
            anchor.anchor_id,
            anchor.anchor_type,
            anchor.echo_cost
        );
    }

    println!("\nTriggers: {}", slice_metadata.trigger_zones.len());
    for trig in &slice_metadata.trigger_zones {
        println!(
            "  [{}] id={} shape={:?} radius={:?} extents={:?}",
            format_coord(trig.cell),
            trig.trigger_id,
            trig.shape,
            trig.radius,
            trig.extents
        );
    }

    println!(
        "\nDecision Prompts: {}",
        slice_metadata.decision_prompts.len()
    );
    for prompt in &slice_metadata.decision_prompts {
        println!(
            "  [{}] id={} options={:?}",
            format_coord(prompt.cell),
            prompt.trigger_id,
            prompt.options
        );
    }

    println!("\nVeilweaver Loomspire greybox cells ready.\n");
    Ok(())
}

fn format_coord(coord: GridCoord) -> String {
    format!("({}, {}, {})", coord.x, coord.y, coord.z)
}
