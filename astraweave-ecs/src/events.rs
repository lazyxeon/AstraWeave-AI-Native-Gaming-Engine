//! Event system for AI-native game engine.
//!
//! Events are crucial for AI perception and reactive behaviors.
//! This system provides deterministic event ordering and efficient queries.

#[cfg(feature = "profiling")]
use astraweave_profiling::span;

use std::any::{Any, TypeId};
use std::collections::{HashMap, VecDeque};
use std::marker::PhantomData;

/// Event trait marker
pub trait Event: 'static + Send + Sync {}

/// Event storage for a single event type
struct EventQueue<E: Event> {
    events: VecDeque<E>,
    /// Frame when events were added (for cleanup)
    frame_added: VecDeque<u64>,
}

impl<E: Event> EventQueue<E> {
    fn new() -> Self {
        Self {
            events: VecDeque::new(),
            frame_added: VecDeque::new(),
        }
    }

    fn send(&mut self, event: E, frame: u64) {
        self.events.push_back(event);
        self.frame_added.push_back(frame);
    }

    fn drain(&mut self) -> impl Iterator<Item = E> + '_ {
        self.frame_added.clear();
        self.events.drain(..)
    }

    fn iter(&self) -> impl Iterator<Item = &E> {
        self.events.iter()
    }

    /// Remove events older than N frames
    #[allow(dead_code)]
    fn cleanup(&mut self, current_frame: u64, keep_frames: u64) {
        while let Some(&frame) = self.frame_added.front() {
            if current_frame.saturating_sub(frame) > keep_frames {
                self.events.pop_front();
                self.frame_added.pop_front();
            } else {
                break;
            }
        }
    }

    fn len(&self) -> usize {
        self.events.len()
    }

    fn clear(&mut self) {
        self.events.clear();
        self.frame_added.clear();
    }
}

/// Central event registry for all event types
pub struct Events {
    /// Map from TypeId to type-erased event queue
    queues: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
    /// Current simulation frame
    current_frame: u64,
    /// How many frames to keep events before cleanup
    keep_frames: u64,
}

impl Events {
    pub fn new() -> Self {
        Self {
            queues: HashMap::new(),
            current_frame: 0,
            keep_frames: 2, // Keep events for 2 frames by default
        }
    }

    pub fn with_keep_frames(mut self, frames: u64) -> Self {
        self.keep_frames = frames;
        self
    }

    /// Send an event
    pub fn send<E: Event>(&mut self, event: E) {
        let queue = self
            .queues
            .entry(TypeId::of::<E>())
            .or_insert_with(|| Box::new(EventQueue::<E>::new()));

        let queue = queue
            .downcast_mut::<EventQueue<E>>()
            .expect("EventQueue type mismatch: just inserted correct type, downcast should never fail");
        queue.send(event, self.current_frame);
    }

    /// Get event reader for type E
    pub fn get_reader<E: Event>(&self) -> EventReader<E> {
        EventReader {
            type_id: TypeId::of::<E>(),
            _marker: PhantomData,
        }
    }

    /// Read events of type E
    pub fn read<E: Event>(&self) -> impl Iterator<Item = &E> {
        self.queues
            .get(&TypeId::of::<E>())
            .and_then(|q| q.downcast_ref::<EventQueue<E>>())
            .map(|q| q.iter())
            .into_iter()
            .flatten()
    }

    /// Drain all events of type E (consumes them)
    pub fn drain<E: Event>(&mut self) -> impl Iterator<Item = E> + '_ {
        self.queues
            .get_mut(&TypeId::of::<E>())
            .and_then(|q| q.downcast_mut::<EventQueue<E>>())
            .map(|q| q.drain())
            .into_iter()
            .flatten()
    }

    /// Clear all events of type E
    pub fn clear<E: Event>(&mut self) {
        if let Some(queue) = self.queues.get_mut(&TypeId::of::<E>()) {
            if let Some(q) = queue.downcast_mut::<EventQueue<E>>() {
                q.clear();
            }
        }
    }

    /// Get event count for type E
    pub fn len<E: Event>(&self) -> usize {
        self.queues
            .get(&TypeId::of::<E>())
            .and_then(|q| q.downcast_ref::<EventQueue<E>>())
            .map(|q| q.len())
            .unwrap_or(0)
    }

    /// Check if events queue is empty for type E
    pub fn is_empty<E: Event>(&self) -> bool {
        self.len::<E>() == 0
    }

    /// Advance frame and cleanup old events
    pub fn update(&mut self) {
        #[cfg(feature = "profiling")]
        span!("ECS::Events::update");
        
        self.current_frame += 1;

        // Cleanup old events from all queues
        for _queue in self.queues.values_mut() {
            // Type erasure: we need to cast to EventQueue<T> but don't know T
            // For now, we'll skip automatic cleanup and rely on explicit clear
            // TODO: Store cleanup function pointer or use trait object
        }
    }

    /// Clear all events
    pub fn clear_all(&mut self) {
        self.queues.clear();
    }

    pub fn current_frame(&self) -> u64 {
        self.current_frame
    }
}

impl Default for Events {
    fn default() -> Self {
        Self::new()
    }
}

// Note: Events implements Resource via the blanket impl in lib.rs
// impl Resource for Events {} // Removed - conflicts with blanket impl

/// Event reader - provides a handle to read events of a specific type
#[allow(dead_code)]
pub struct EventReader<E: Event> {
    type_id: TypeId,
    _marker: PhantomData<E>,
}

impl<E: Event> EventReader<E> {
    /// Read events from the Events resource
    pub fn read<'a>(&self, events: &'a Events) -> impl Iterator<Item = &'a E> {
        events.read::<E>()
    }
}

// Common game events for AI systems

/// Entity spawned event
#[derive(Clone, Debug)]
pub struct EntitySpawnedEvent {
    pub entity: crate::Entity,
    pub entity_type: String,
}
impl Event for EntitySpawnedEvent {}

/// Entity despawned event
#[derive(Clone, Debug)]
pub struct EntityDespawnedEvent {
    pub entity: crate::Entity,
}
impl Event for EntityDespawnedEvent {}

/// Health changed event (for AI perception)
#[derive(Clone, Debug)]
pub struct HealthChangedEvent {
    pub entity: crate::Entity,
    pub old_health: i32,
    pub new_health: i32,
    pub source: Option<crate::Entity>,
}
impl Event for HealthChangedEvent {}

/// AI planning failed event
#[derive(Clone, Debug)]
pub struct AiPlanningFailedEvent {
    pub entity: crate::Entity,
    pub reason: String,
}
impl Event for AiPlanningFailedEvent {}

/// Tool validation failed event
#[derive(Clone, Debug)]
pub struct ToolValidationFailedEvent {
    pub entity: crate::Entity,
    pub tool_name: String,
    pub reason: String,
}
impl Event for ToolValidationFailedEvent {}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug, PartialEq)]
    struct TestEvent {
        value: i32,
    }
    impl Event for TestEvent {}

    #[test]
    fn test_send_and_read_events() {
        let mut events = Events::new();

        events.send(TestEvent { value: 42 });
        events.send(TestEvent { value: 100 });

        let collected: Vec<_> = events.read::<TestEvent>().collect();
        assert_eq!(collected.len(), 2);
        assert_eq!(collected[0].value, 42);
        assert_eq!(collected[1].value, 100);
    }

    #[test]
    fn test_drain_events() {
        let mut events = Events::new();

        events.send(TestEvent { value: 1 });
        events.send(TestEvent { value: 2 });

        let drained: Vec<_> = events.drain::<TestEvent>().collect();
        assert_eq!(drained.len(), 2);

        // Events should be gone after drain
        assert_eq!(events.len::<TestEvent>(), 0);
    }

    #[test]
    fn test_clear_events() {
        let mut events = Events::new();

        events.send(TestEvent { value: 1 });
        events.send(TestEvent { value: 2 });

        assert_eq!(events.len::<TestEvent>(), 2);

        events.clear::<TestEvent>();
        assert_eq!(events.len::<TestEvent>(), 0);
    }

    #[test]
    fn test_event_reader() {
        let mut events = Events::new();
        let reader = events.get_reader::<TestEvent>();

        events.send(TestEvent { value: 42 });

        let collected: Vec<_> = reader.read(&events).collect();
        assert_eq!(collected.len(), 1);
        assert_eq!(collected[0].value, 42);
    }

    #[test]
    fn test_frame_tracking() {
        let mut events = Events::new();
        assert_eq!(events.current_frame(), 0);

        events.update();
        assert_eq!(events.current_frame(), 1);

        events.update();
        assert_eq!(events.current_frame(), 2);
    }

    // ========================================================================
    // Phase 3.3: Event Ordering Tests (Determinism)
    // ========================================================================

    #[test]
    fn test_events_delivered_in_fifo_order() {
        let mut events = Events::new();

        // Send 100 events in sequence
        for i in 0..100 {
            events.send(TestEvent { value: i });
        }

        // Read events and verify FIFO order
        let collected: Vec<_> = events.read::<TestEvent>().collect();
        assert_eq!(collected.len(), 100, "Should have 100 events");

        for (i, event) in collected.iter().enumerate() {
            assert_eq!(
                event.value, i as i32,
                "Event {} should have value {}, got {}",
                i, i, event.value
            );
        }
    }

    #[test]
    fn test_drain_preserves_fifo_order() {
        let mut events = Events::new();

        // Send events
        for i in 0..50 {
            events.send(TestEvent { value: i });
        }

        // Drain events and verify FIFO order
        let drained: Vec<_> = events.drain::<TestEvent>().collect();
        assert_eq!(drained.len(), 50, "Should have drained 50 events");

        for (i, event) in drained.iter().enumerate() {
            assert_eq!(
                event.value, i as i32,
                "Drained event {} should have value {}, got {}",
                i, i, event.value
            );
        }

        // Verify events are gone
        assert_eq!(events.len::<TestEvent>(), 0, "Events should be consumed after drain");
    }

    #[test]
    fn test_frame_boundaries_respected() {
        let mut events = Events::new();
        assert_eq!(events.current_frame(), 0);

        // Frame 0: Send events
        events.send(TestEvent { value: 1 });
        events.send(TestEvent { value: 2 });

        // Advance to frame 1
        events.update();
        assert_eq!(events.current_frame(), 1);

        // Frame 1: Send more events
        events.send(TestEvent { value: 3 });
        events.send(TestEvent { value: 4 });

        // All events should still be present (no cross-frame interference)
        let collected: Vec<_> = events.read::<TestEvent>().collect();
        assert_eq!(collected.len(), 4, "Should have all 4 events");

        // Verify FIFO order across frames
        assert_eq!(collected[0].value, 1);
        assert_eq!(collected[1].value, 2);
        assert_eq!(collected[2].value, 3);
        assert_eq!(collected[3].value, 4);
    }

    #[test]
    fn test_multiple_readers_independent() {
        let mut events = Events::new();

        events.send(TestEvent { value: 42 });
        events.send(TestEvent { value: 100 });

        // Create two independent readers
        let reader1 = events.get_reader::<TestEvent>();
        let reader2 = events.get_reader::<TestEvent>();

        // Both readers should see same events
        let collected1: Vec<_> = reader1.read(&events).collect();
        let collected2: Vec<_> = reader2.read(&events).collect();

        assert_eq!(collected1.len(), 2);
        assert_eq!(collected2.len(), 2);
        assert_eq!(collected1[0].value, collected2[0].value);
        assert_eq!(collected1[1].value, collected2[1].value);
    }

    #[test]
    fn test_clear_removes_all_events() {
        let mut events = Events::new();

        // Send events
        for i in 0..20 {
            events.send(TestEvent { value: i });
        }

        assert_eq!(events.len::<TestEvent>(), 20);

        // Clear all events
        events.clear::<TestEvent>();

        // Verify all events removed
        assert_eq!(events.len::<TestEvent>(), 0);
        let collected: Vec<_> = events.read::<TestEvent>().collect();
        assert_eq!(collected.len(), 0);
    }

    #[test]
    fn test_multiple_event_types_independent() {
        #[derive(Clone, Debug)]
        struct EventA { id: u32 }
        impl Event for EventA {}

        #[derive(Clone, Debug)]
        struct EventB { name: String }
        impl Event for EventB {}

        let mut events = Events::new();

        // Send events of different types
        events.send(EventA { id: 1 });
        events.send(EventB { name: "first".to_string() });
        events.send(EventA { id: 2 });
        events.send(EventB { name: "second".to_string() });

        // Each type should maintain its own FIFO order
        let a_events: Vec<_> = events.read::<EventA>().collect();
        let b_events: Vec<_> = events.read::<EventB>().collect();

        assert_eq!(a_events.len(), 2);
        assert_eq!(b_events.len(), 2);

        assert_eq!(a_events[0].id, 1);
        assert_eq!(a_events[1].id, 2);
        assert_eq!(b_events[0].name, "first");
        assert_eq!(b_events[1].name, "second");
    }

    #[test]
    fn test_clear_one_type_preserves_others() {
        #[derive(Clone, Debug)]
        struct EventA { #[allow(dead_code)] value: i32 }
        impl Event for EventA {}

        #[derive(Clone, Debug)]
        struct EventB { #[allow(dead_code)] value: i32 }
        impl Event for EventB {}

        let mut events = Events::new();

        events.send(EventA { value: 1 });
        events.send(EventB { value: 2 });

        assert_eq!(events.len::<EventA>(), 1);
        assert_eq!(events.len::<EventB>(), 1);

        // Clear only EventA
        events.clear::<EventA>();

        // EventA should be gone, EventB should remain
        assert_eq!(events.len::<EventA>(), 0);
        assert_eq!(events.len::<EventB>(), 1);

        let b_events: Vec<_> = events.read::<EventB>().collect();
        assert_eq!(b_events[0].value, 2);
    }

    #[test]
    fn test_interleaved_send_and_read() {
        let mut events = Events::new();

        // Send first batch
        events.send(TestEvent { value: 1 });
        events.send(TestEvent { value: 2 });

        // Read (non-consuming)
        let first_read: Vec<_> = events.read::<TestEvent>().collect();
        assert_eq!(first_read.len(), 2);

        // Send more events
        events.send(TestEvent { value: 3 });
        events.send(TestEvent { value: 4 });

        // Read again (should see all events in FIFO order)
        let second_read: Vec<_> = events.read::<TestEvent>().collect();
        assert_eq!(second_read.len(), 4);
        assert_eq!(second_read[0].value, 1);
        assert_eq!(second_read[1].value, 2);
        assert_eq!(second_read[2].value, 3);
        assert_eq!(second_read[3].value, 4);
    }

    #[test]
    fn test_repeated_drain_produces_empty_results() {
        let mut events = Events::new();

        events.send(TestEvent { value: 42 });

        // First drain
        let first_drain: Vec<_> = events.drain::<TestEvent>().collect();
        assert_eq!(first_drain.len(), 1);

        // Second drain (should be empty)
        let second_drain: Vec<_> = events.drain::<TestEvent>().collect();
        assert_eq!(second_drain.len(), 0);

        // Third drain (should still be empty)
        let third_drain: Vec<_> = events.drain::<TestEvent>().collect();
        assert_eq!(third_drain.len(), 0);
    }

    #[test]
    fn test_large_event_batch_maintains_order() {
        let mut events = Events::new();
        const BATCH_SIZE: usize = 10_000;

        // Send large batch
        for i in 0..BATCH_SIZE {
            events.send(TestEvent { value: i as i32 });
        }

        assert_eq!(events.len::<TestEvent>(), BATCH_SIZE);

        // Verify all events in correct order
        let collected: Vec<_> = events.read::<TestEvent>().collect();
        assert_eq!(collected.len(), BATCH_SIZE);

        for (i, event) in collected.iter().enumerate() {
            assert_eq!(event.value, i as i32, "Event {} out of order", i);
        }
    }

    #[test]
    fn test_clear_all_removes_all_event_types() {
        #[derive(Clone, Debug)]
        struct EventA { #[allow(dead_code)] value: i32 }
        impl Event for EventA {}

        #[derive(Clone, Debug)]
        struct EventB { #[allow(dead_code)] value: i32 }
        impl Event for EventB {}

        let mut events = Events::new();

        events.send(EventA { value: 1 });
        events.send(EventB { value: 2 });

        assert_eq!(events.len::<EventA>(), 1);
        assert_eq!(events.len::<EventB>(), 1);

        // Clear all event types
        events.clear_all();

        // Both types should be gone
        assert_eq!(events.len::<EventA>(), 0);
        assert_eq!(events.len::<EventB>(), 0);
    }
}
