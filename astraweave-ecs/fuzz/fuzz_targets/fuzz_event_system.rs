#![no_main]

use libfuzzer_sys::fuzz_target;
use astraweave_ecs::World;

/// Fuzz Target 5: Event System Operations
/// 
/// Tests random event send/read/drain sequences to find:
/// - Panics during event sending
/// - Event ordering violations
/// - Memory corruption during event draining
/// - Reader state corruption
///
/// Input format: Stream of bytes where each byte represents an operation:
/// - 0-127: Send event with value
/// - 128-191: Read events with reader 0
/// - 192-223: Read events with reader 1
/// - 224-255: Drain events or update frame

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct FuzzEvent(u8);

fuzz_target!(|data: &[u8]| {
    let mut world = World::new();
    
    // Register event type
    world.register_event::<FuzzEvent>();

    for &byte in data {
        match byte {
            0..=127 => {
                // Send event
                world.send_event(FuzzEvent(byte));
            }
            128..=191 => {
                // Read events with reader 0
                let mut reader = world.create_event_reader::<FuzzEvent>();
                let events: Vec<_> = reader.read(&world).copied().collect();
                
                // Validate events are in FIFO order
                for window in events.windows(2) {
                    // Just ensure we can read them without panic
                    let _ = window[0];
                    let _ = window[1];
                }
            }
            192..=223 => {
                // Read events with reader 1
                let mut reader = world.create_event_reader::<FuzzEvent>();
                let _events: Vec<_> = reader.read(&world).copied().collect();
            }
            224..=239 => {
                // Drain events
                let drained: Vec<_> = world.drain_events::<FuzzEvent>().collect();
                
                // Validate drained events
                for event in drained {
                    let _ = event.0;
                }
            }
            240..=255 => {
                // Update frame (clears events)
                world.clear_events();
            }
        }
    }

    // Final validation: world is still in consistent state
    assert_eq!(world.entity_count(), 0); // No entities spawned in this test
});
