//! Deterministic events resource for ECS systems
use std::collections::VecDeque;
use astraweave_ecs as ecs;
use crate::IVec2;

pub struct Events<T> {
    q: VecDeque<T>,
}

impl<T> Events<T> {
    pub fn writer(&mut self) -> EventWriter<'_, T> { EventWriter { q: &mut self.q } }
    pub fn reader(&mut self) -> EventReader<'_, T> { EventReader { q: &mut self.q } }
    pub fn clear(&mut self) { self.q.clear(); }
    pub fn len(&self) -> usize { self.q.len() }
    pub fn is_empty(&self) -> bool { self.q.is_empty() }
}

impl<T> Default for Events<T> {
    fn default() -> Self { Self { q: VecDeque::new() } }
}

pub struct EventWriter<'a, T> { q: &'a mut VecDeque<T> }
impl<'a, T> EventWriter<'a, T> { pub fn send(&mut self, ev: T) { self.q.push_back(ev); } }

pub struct EventReader<'a, T> { q: &'a mut VecDeque<T> }
impl<'a, T> EventReader<'a, T> {
    pub fn drain(&mut self) -> std::collections::vec_deque::Drain<'_, T> { self.q.drain(..) }
}

// Concrete event types used by core + AI plugin
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MovedEvent {
    pub entity: ecs::Entity,
    pub from: IVec2,
    pub to: IVec2,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AiPlannedEvent {
    pub entity: ecs::Entity,
    pub target: IVec2,
}
