//! Layout generation (rooms, paths)

use crate::SeedRng;
use glam::IVec2;
use serde::{Deserialize, Serialize};

/// A room in the layout
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Room {
    /// Minimum and maximum bounds of the room
    pub bounds: (IVec2, IVec2),
    /// Indices of connected rooms
    pub connections: Vec<usize>,
}

impl Room {
    /// Get the center point of the room
    pub fn center(&self) -> IVec2 {
        (self.bounds.0 + self.bounds.1) / 2
    }

    /// Get the size of the room
    pub fn size(&self) -> IVec2 {
        self.bounds.1 - self.bounds.0
    }

    /// Check if a point is inside the room
    pub fn contains(&self, point: IVec2) -> bool {
        point.x >= self.bounds.0.x
            && point.x <= self.bounds.1.x
            && point.y >= self.bounds.0.y
            && point.y <= self.bounds.1.y
    }

    /// Check if this room overlaps with another
    pub fn overlaps(&self, other: &Room) -> bool {
        !(self.bounds.1.x < other.bounds.0.x
            || self.bounds.0.x > other.bounds.1.x
            || self.bounds.1.y < other.bounds.0.y
            || self.bounds.0.y > other.bounds.1.y)
    }
}

/// Generator for layouts
pub struct LayoutGenerator {
    pub grid_size: IVec2,
    pub room_min_size: IVec2,
    pub room_max_size: IVec2,
    pub max_placement_attempts: usize,
}

impl LayoutGenerator {
    pub fn new(grid_size: IVec2) -> Self {
        Self {
            grid_size,
            room_min_size: IVec2::new(5, 5),
            room_max_size: IVec2::new(15, 15),
            max_placement_attempts: 100,
        }
    }

    /// Generate rooms with connections
    pub fn generate_rooms(&self, rng: &mut SeedRng, count: u32) -> Vec<Room> {
        let mut rooms = Vec::new();

        for _ in 0..count {
            if let Some(room) = self.try_place_room(rng, &rooms) {
                rooms.push(room);
            }
        }

        // Connect rooms (simple chain + some random connections)
        self.connect_rooms(rng, &mut rooms);

        rooms
    }

    fn try_place_room(&self, rng: &mut SeedRng, existing: &[Room]) -> Option<Room> {
        for _ in 0..self.max_placement_attempts {
            let width = rng.gen_range(self.room_min_size.x..=self.room_max_size.x);
            let height = rng.gen_range(self.room_min_size.y..=self.room_max_size.y);

            let max_x = (self.grid_size.x - width).max(0);
            let max_y = (self.grid_size.y - height).max(0);

            if max_x <= 0 || max_y <= 0 {
                continue;
            }

            let x = rng.gen_range(0..=max_x);
            let y = rng.gen_range(0..=max_y);

            let room = Room {
                bounds: (IVec2::new(x, y), IVec2::new(x + width, y + height)),
                connections: Vec::new(),
            };

            // Check for overlaps
            if !existing.iter().any(|r| r.overlaps(&room)) {
                return Some(room);
            }
        }

        None
    }

    fn connect_rooms(&self, rng: &mut SeedRng, rooms: &mut [Room]) {
        if rooms.is_empty() {
            return;
        }

        // Chain connection (ensures all rooms reachable)
        for i in 0..rooms.len() - 1 {
            rooms[i].connections.push(i + 1);
            rooms[i + 1].connections.push(i);
        }

        // Add some random connections for cycles
        let extra_connections = (rooms.len() / 3).max(1);
        for _ in 0..extra_connections {
            let i = rng.gen_range(0..rooms.len());
            let j = rng.gen_range(0..rooms.len());

            if i != j && !rooms[i].connections.contains(&j) {
                rooms[i].connections.push(j);
                rooms[j].connections.push(i);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deterministic_generation() {
        let gen = LayoutGenerator::new(IVec2::new(100, 100));

        let mut rng1 = SeedRng::new(42, "test");
        let mut rng2 = SeedRng::new(42, "test");

        let rooms1 = gen.generate_rooms(&mut rng1, 5);
        let rooms2 = gen.generate_rooms(&mut rng2, 5);

        assert_eq!(rooms1.len(), rooms2.len());

        for (r1, r2) in rooms1.iter().zip(rooms2.iter()) {
            assert_eq!(r1.bounds, r2.bounds);
            assert_eq!(r1.connections, r2.connections);
        }
    }

    #[test]
    fn test_no_overlaps() {
        let gen = LayoutGenerator::new(IVec2::new(100, 100));
        let mut rng = SeedRng::new(42, "test");

        let rooms = gen.generate_rooms(&mut rng, 10);

        for i in 0..rooms.len() {
            for j in (i + 1)..rooms.len() {
                assert!(
                    !rooms[i].overlaps(&rooms[j]),
                    "Rooms {} and {} overlap",
                    i,
                    j
                );
            }
        }
    }

    #[test]
    fn test_rooms_in_bounds() {
        let grid_size = IVec2::new(50, 50);
        let gen = LayoutGenerator::new(grid_size);
        let mut rng = SeedRng::new(42, "test");

        let rooms = gen.generate_rooms(&mut rng, 10);

        for room in &rooms {
            assert!(room.bounds.0.x >= 0);
            assert!(room.bounds.0.y >= 0);
            assert!(room.bounds.1.x <= grid_size.x);
            assert!(room.bounds.1.y <= grid_size.y);
        }
    }

    #[test]
    fn test_all_rooms_connected() {
        let gen = LayoutGenerator::new(IVec2::new(100, 100));
        let mut rng = SeedRng::new(42, "test");

        let rooms = gen.generate_rooms(&mut rng, 10);

        // Check that all rooms are reachable via BFS
        if rooms.is_empty() {
            return;
        }

        let mut visited = vec![false; rooms.len()];
        let mut queue = vec![0];
        visited[0] = true;

        while let Some(i) = queue.pop() {
            for &j in &rooms[i].connections {
                if !visited[j] {
                    visited[j] = true;
                    queue.push(j);
                }
            }
        }

        assert!(visited.iter().all(|&v| v), "Not all rooms are connected");
    }

    #[test]
    fn test_room_center() {
        let room = Room {
            bounds: (IVec2::new(0, 0), IVec2::new(10, 10)),
            connections: Vec::new(),
        };
        assert_eq!(room.center(), IVec2::new(5, 5));
    }

    #[test]
    fn test_room_size() {
        let room = Room {
            bounds: (IVec2::new(0, 0), IVec2::new(10, 20)),
            connections: Vec::new(),
        };
        assert_eq!(room.size(), IVec2::new(10, 20));
    }

    #[test]
    fn test_room_contains() {
        let room = Room {
            bounds: (IVec2::new(0, 0), IVec2::new(10, 10)),
            connections: Vec::new(),
        };

        assert!(room.contains(IVec2::new(5, 5)));
        assert!(room.contains(IVec2::new(0, 0)));
        assert!(room.contains(IVec2::new(10, 10)));
        assert!(!room.contains(IVec2::new(-1, 5)));
        assert!(!room.contains(IVec2::new(11, 5)));
    }
}
