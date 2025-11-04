//! Entity allocator with generational indices for safe entity lifecycle management.
//!
//! # Problem
//!
//! Without generational indices, entity IDs are recycled after despawn, leading to
//! use-after-free bugs:
//!
//! ```rust,ignore
//! let e1 = world.spawn();  // ID = 1
//! world.despawn(e1);
//! let e2 = world.spawn();  // ID = 1 (reused!)
//! world.get::<Position>(e1);  // ❌ Accesses e2's data! (use-after-free)
//! ```
//!
//! # Solution
//!
//! Generational indices add a generation counter that increments on reuse:
//!
//! ```rust,ignore
//! let e1 = world.spawn();  // Entity { id: 1, generation: 0 }
//! world.despawn(e1);       // Generation[1] = 1
//! let e2 = world.spawn();  // Entity { id: 1, generation: 1 }
//! world.get::<Position>(e1);  // ✅ Returns None (generation mismatch)
//! ```
//!
//! # Performance
//!
//! - Entity struct: 64-bit (no size increase from bare u64)
//! - Validation: O(1) array lookup
//! - Spawn/despawn: O(1) amortized (free list)

use std::fmt;

/// Entity identifier with generational index for safe lifecycle management.
///
/// # Memory Layout
///
/// ```text
/// Entity (8 bytes):
/// ┌──────────────────────────────────┬──────────────────────────────────┐
/// │         ID (32 bits)             │      Generation (32 bits)        │
/// └──────────────────────────────────┴──────────────────────────────────┘
/// ```
///
/// - **ID**: Entity index (recycled after despawn)
/// - **Generation**: Counter incremented on reuse (detects stale handles)
///
/// # Guarantees
///
/// - Deterministic: Same operations → same entities
/// - Safe: Stale entity handles rejected (no use-after-free)
/// - Ordered: Implements `Ord` for stable iteration
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Entity {
    id: u32,
    generation: u32,
}

impl Entity {
    /// Create a new entity with given ID and generation.
    ///
    /// # Internal Use Only
    ///
    /// This is used by `EntityAllocator`. User code should use `World::spawn()`.
    #[inline]
    pub(crate) fn new(id: u32, generation: u32) -> Self {
        Self { id, generation }
    }

    /// Get the entity index (recycled).
    ///
    /// # Note
    ///
    /// This is the slot index in the allocator, not a unique identifier.
    /// Use the full `Entity` (id + generation) for identity checks.
    #[inline]
    pub fn id(&self) -> u32 {
        self.id
    }

    /// Get the generation counter.
    ///
    /// # Generation Semantics
    ///
    /// - Starts at 0 for first spawn
    /// - Increments on each despawn
    /// - Used to detect stale entity handles
    #[inline]
    pub fn generation(&self) -> u32 {
        self.generation
    }

    /// Convert to raw u64 for serialization or external APIs.
    ///
    /// # Format
    ///
    /// ```text
    /// u64 = (id as u64) | ((generation as u64) << 32)
    /// ```
    #[inline]
    pub fn to_raw(&self) -> u64 {
        (self.id as u64) | ((self.generation as u64) << 32)
    }

    /// Reconstruct entity from raw u64.
    ///
    /// # Safety
    ///
    /// The caller must ensure this entity is valid in the target `World`.
    /// Use `World::is_alive(entity)` to validate.
    #[inline]
    pub unsafe fn from_raw(raw: u64) -> Self {
        Self {
            id: raw as u32,
            generation: (raw >> 32) as u32,
        }
    }

    /// Create a null entity (invalid, for initialization).
    ///
    /// # Note
    ///
    /// Null entities fail all `is_alive()` checks.
    #[inline]
    pub const fn null() -> Self {
        Self {
            id: u32::MAX,
            generation: u32::MAX,
        }
    }

    /// Check if this is a null entity.
    #[inline]
    pub const fn is_null(&self) -> bool {
        self.id == u32::MAX && self.generation == u32::MAX
    }
}

impl fmt::Debug for Entity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Entity({}v{})", self.id, self.generation)
    }
}

impl fmt::Display for Entity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}v{}", self.id, self.generation)
    }
}

/// Entity allocator with generational index tracking.
///
/// # Algorithm
///
/// - **Free List**: Recycled IDs stored in `Vec<u32>` (LIFO)
/// - **Generations**: Per-slot generation counter in `Vec<u32>`
/// - **Spawn**: Pop from free list or allocate new ID
/// - **Despawn**: Increment generation, push to free list
///
/// # Complexity
///
/// - Spawn: O(1) amortized
/// - Despawn: O(1)
/// - Is Alive: O(1) array lookup
///
/// # Capacity
///
/// - Max entities: 2^32 - 1 (4.2 billion)
/// - Max generations per slot: 2^32 - 1 (4.2 billion reuses)
#[derive(Debug, Clone)]
pub struct EntityAllocator {
    /// Free list of recycled entity IDs (LIFO)
    free_list: Vec<u32>,

    /// Generation counter per entity slot
    generations: Vec<u32>,

    /// Next entity ID if free list is empty
    next_id: u32,

    /// Total entities spawned (for statistics)
    spawned_count: u64,

    /// Total entities despawned (for statistics)
    despawned_count: u64,
}

impl EntityAllocator {
    /// Create a new entity allocator.
    pub fn new() -> Self {
        Self {
            free_list: Vec::new(),
            generations: Vec::new(),
            next_id: 0,
            spawned_count: 0,
            despawned_count: 0,
        }
    }

    /// Create allocator with pre-allocated capacity.
    ///
    /// # Example
    ///
    /// ```
    /// use astraweave_ecs::entity_allocator::EntityAllocator;
    ///
    /// let allocator = EntityAllocator::with_capacity(10_000);
    /// // No allocations until entity count exceeds 10,000
    /// ```
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            free_list: Vec::new(),
            generations: Vec::with_capacity(capacity),
            next_id: 0,
            spawned_count: 0,
            despawned_count: 0,
        }
    }

    /// Spawn a new entity.
    ///
    /// # Algorithm
    ///
    /// 1. Pop from free list if available
    /// 2. Otherwise allocate new ID
    /// 3. Return `Entity { id, generation }`
    ///
    /// # Example
    ///
    /// ```
    /// use astraweave_ecs::entity_allocator::EntityAllocator;
    ///
    /// let mut allocator = EntityAllocator::new();
    /// let e1 = allocator.spawn();  // Entity(0v0)
    /// let e2 = allocator.spawn();  // Entity(1v0)
    /// ```
    pub fn spawn(&mut self) -> Entity {
        let id = if let Some(id) = self.free_list.pop() {
            // Reuse recycled ID with current generation
            id
        } else {
            // Allocate new ID
            let id = self.next_id;
            self.next_id = self.next_id.checked_add(1).expect(
                "Entity ID overflow: spawned 2^32 entities. \
                 Consider increasing entity slot size or implementing entity pooling.",
            );
            self.generations.push(0);
            id
        };

        let generation = self.generations[id as usize];
        self.spawned_count += 1;

        Entity::new(id, generation)
    }

    /// Despawn an entity.
    ///
    /// # Returns
    ///
    /// - `true` if entity was alive and despawned
    /// - `false` if entity was already dead (stale handle)
    ///
    /// # Algorithm
    ///
    /// 1. Check generation matches (is alive)
    /// 2. Increment generation
    /// 3. Add to free list
    ///
    /// # Example
    ///
    /// ```
    /// use astraweave_ecs::entity_allocator::EntityAllocator;
    ///
    /// let mut allocator = EntityAllocator::new();
    /// let e1 = allocator.spawn();
    ///
    /// assert!(allocator.despawn(e1));  // First despawn succeeds
    /// assert!(!allocator.despawn(e1)); // Second despawn fails (stale)
    /// ```
    pub fn despawn(&mut self, entity: Entity) -> bool {
        let id = entity.id as usize;

        // Validate entity exists
        if id >= self.generations.len() {
            return false;
        }

        // Check generation (is alive)
        if self.generations[id] != entity.generation {
            return false; // Stale entity
        }

        // Increment generation
        self.generations[id] = self.generations[id].wrapping_add(1);

        // Add to free list
        self.free_list.push(entity.id);

        self.despawned_count += 1;

        true
    }

    /// Check if an entity is alive.
    ///
    /// # Returns
    ///
    /// - `true` if entity ID and generation match
    /// - `false` if entity is dead or never existed
    ///
    /// # Complexity
    ///
    /// O(1) array lookup
    ///
    /// # Example
    ///
    /// ```
    /// use astraweave_ecs::entity_allocator::EntityAllocator;
    ///
    /// let mut allocator = EntityAllocator::new();
    /// let e1 = allocator.spawn();
    ///
    /// assert!(allocator.is_alive(e1));
    ///
    /// allocator.despawn(e1);
    /// assert!(!allocator.is_alive(e1));
    /// ```
    #[inline]
    pub fn is_alive(&self, entity: Entity) -> bool {
        let id = entity.id as usize;
        self.generations
            .get(id)
            .map(|&gen| gen == entity.generation)
            .unwrap_or(false)
    }

    /// Get the current generation for an entity slot.
    ///
    /// Returns `None` if the ID has never been allocated.
    #[inline]
    pub fn generation(&self, id: u32) -> Option<u32> {
        self.generations.get(id as usize).copied()
    }

    /// Get total number of entities currently alive.
    #[inline]
    pub fn alive_count(&self) -> usize {
        (self.spawned_count - self.despawned_count) as usize
    }

    /// Get total number of entity slots allocated.
    #[inline]
    pub fn capacity(&self) -> usize {
        self.generations.len()
    }

    /// Get total number of entities spawned (including despawned).
    #[inline]
    pub fn spawned_count(&self) -> u64 {
        self.spawned_count
    }

    /// Get total number of entities despawned.
    #[inline]
    pub fn despawned_count(&self) -> u64 {
        self.despawned_count
    }

    /// Clear all entities and reset allocator.
    ///
    /// # Warning
    ///
    /// All existing `Entity` handles become invalid.
    pub fn clear(&mut self) {
        self.free_list.clear();
        self.generations.clear();
        self.next_id = 0;
        self.spawned_count = 0;
        self.despawned_count = 0;
    }

    /// Reserve capacity for additional entities.
    ///
    /// This pre-allocates space to avoid reallocation during spawn.
    pub fn reserve(&mut self, additional: usize) {
        self.generations.reserve(additional);
    }
}

impl Default for EntityAllocator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spawn_despawn_cycle() {
        let mut allocator = EntityAllocator::new();

        let e1 = allocator.spawn();
        assert_eq!(e1.id(), 0);
        assert_eq!(e1.generation(), 0);
        assert!(allocator.is_alive(e1));

        allocator.despawn(e1);
        assert!(!allocator.is_alive(e1));

        let e2 = allocator.spawn();
        assert_eq!(e2.id(), 0); // Reused ID
        assert_eq!(e2.generation(), 1); // Incremented generation
        assert!(!allocator.is_alive(e1)); // Old handle invalid
        assert!(allocator.is_alive(e2)); // New handle valid
    }

    #[test]
    fn test_stale_entity_rejection() {
        let mut allocator = EntityAllocator::new();

        let e1 = allocator.spawn();
        allocator.despawn(e1);

        // Trying to despawn again should fail
        assert!(!allocator.despawn(e1));
        assert!(!allocator.is_alive(e1));
    }

    #[test]
    fn test_multiple_entities() {
        let mut allocator = EntityAllocator::new();

        let e1 = allocator.spawn();
        let e2 = allocator.spawn();
        let e3 = allocator.spawn();

        assert_eq!(e1.id(), 0);
        assert_eq!(e2.id(), 1);
        assert_eq!(e3.id(), 2);

        assert!(allocator.is_alive(e1));
        assert!(allocator.is_alive(e2));
        assert!(allocator.is_alive(e3));

        allocator.despawn(e2);
        assert!(allocator.is_alive(e1));
        assert!(!allocator.is_alive(e2));
        assert!(allocator.is_alive(e3));
    }

    #[test]
    fn test_generation_overflow() {
        let mut allocator = EntityAllocator::new();

        let e1 = allocator.spawn();
        let id = e1.id();

        // Simulate many despawn/spawn cycles
        for i in 0..10 {
            allocator.despawn(Entity::new(id, i));
            let e = allocator.spawn();
            assert_eq!(e.id(), id);
            assert_eq!(e.generation(), i + 1);
        }
    }

    #[test]
    fn test_entity_ordering() {
        let e1 = Entity::new(0, 0);
        let e2 = Entity::new(1, 0);
        let e3 = Entity::new(0, 1);

        assert!(e1 < e2); // Different IDs
        assert!(e1 < e3); // Same ID, different generation
        assert!(e3 < e2); // Generation comparison
    }

    #[test]
    fn test_entity_display() {
        let e = Entity::new(42, 7);
        assert_eq!(format!("{}", e), "42v7");
        assert_eq!(format!("{:?}", e), "Entity(42v7)");
    }

    #[test]
    fn test_null_entity() {
        let null = Entity::null();
        assert!(null.is_null());

        let allocator = EntityAllocator::new();
        assert!(!allocator.is_alive(null));
    }

    #[test]
    fn test_raw_conversion() {
        let e = Entity::new(0x12345678, 0xABCDEF01);
        let raw = e.to_raw();
        let restored = unsafe { Entity::from_raw(raw) };

        assert_eq!(e, restored);
        assert_eq!(e.id(), restored.id());
        assert_eq!(e.generation(), restored.generation());
    }

    #[test]
    fn test_capacity_tracking() {
        let mut allocator = EntityAllocator::new();

        assert_eq!(allocator.alive_count(), 0);
        assert_eq!(allocator.capacity(), 0);

        let e1 = allocator.spawn();
        assert_eq!(allocator.alive_count(), 1);
        assert_eq!(allocator.capacity(), 1);

        let _e2 = allocator.spawn();
        assert_eq!(allocator.alive_count(), 2);
        assert_eq!(allocator.capacity(), 2);

        allocator.despawn(e1);
        assert_eq!(allocator.alive_count(), 1);
        assert_eq!(allocator.capacity(), 2); // Capacity doesn't shrink
    }

    #[test]
    fn test_with_capacity() {
        let allocator = EntityAllocator::with_capacity(100);
        assert_eq!(allocator.capacity(), 0); // No entities spawned yet
        assert!(allocator.generations.capacity() >= 100);
    }

    #[test]
    fn test_clear() {
        let mut allocator = EntityAllocator::new();

        let e1 = allocator.spawn();
        let e2 = allocator.spawn();

        allocator.clear();

        assert_eq!(allocator.alive_count(), 0);
        assert_eq!(allocator.capacity(), 0);
        assert!(!allocator.is_alive(e1));
        assert!(!allocator.is_alive(e2));

        let e3 = allocator.spawn();
        assert_eq!(e3.id(), 0); // Reset to ID 0
        assert_eq!(e3.generation(), 0); // Reset generation
    }
}
