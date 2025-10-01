//! AstraWeave ECS — deterministic, minimal ECS core tailored for AI-first simulation.
//! Phase 1 goal: provide archetype-like storage, a fixed schedule, and a plugin boundary.

use std::{any::TypeId, collections::{BTreeMap, HashMap}, hash::Hash};

pub trait Component: 'static + Send + Sync {}
impl<T: 'static + Send + Sync> Component for T {}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct Entity(u64);

impl Entity {
    /// Get the raw entity ID. Use with caution.
    pub fn id(&self) -> u64 {
        self.0
    }

    /// # Safety
    /// The caller must ensure the entity ID is valid in the target World.
    pub unsafe fn from_raw(id: u64) -> Self {
        Entity(id)
    }
}
#[derive(Default)]
pub struct World {
    next: u64,
    // Component storage: TypeId -> Entity -> Box<dyn Any>
    // Phase 1: use BTreeMap to keep deterministic iteration order.
    comps: HashMap<TypeId, BTreeMap<Entity, Box<dyn std::any::Any + Send + Sync>>>,
    resources: HashMap<TypeId, Box<dyn std::any::Any + Send + Sync>>, // singletons
}

impl World {
    pub fn new() -> Self { Self::default() }
    pub fn spawn(&mut self) -> Entity { let e = Entity(self.next); self.next+=1; e }
    pub fn insert<T: Component>(&mut self, e: Entity, c: T) {
        self.comps.entry(TypeId::of::<T>())
            .or_default()
            .insert(e, Box::new(c));
    }
    pub fn get<T: Component>(&self, e: Entity) -> Option<&T> {
        self.comps.get(&TypeId::of::<T>())
            .and_then(|m| m.get(&e))
            .and_then(|b| b.downcast_ref::<T>())
    }
    pub fn get_mut<T: Component>(&mut self, e: Entity) -> Option<&mut T> {
        self.comps.get_mut(&TypeId::of::<T>())
            .and_then(|m| m.get_mut(&e))
            .and_then(|b| b.downcast_mut::<T>())
    }
    pub fn insert_resource<T: 'static + Send + Sync>(&mut self, r: T) {
        self.resources.insert(TypeId::of::<T>(), Box::new(r));
    }
    pub fn resource<T: 'static + Send + Sync>(&self) -> Option<&T> {
        self.resources.get(&TypeId::of::<T>())?.downcast_ref()
    }
    pub fn resource_mut<T: 'static + Send + Sync>(&mut self) -> Option<&mut T> {
        self.resources.get_mut(&TypeId::of::<T>())?.downcast_mut()
    }

    // Iterate mutably over all components of type T
    pub fn each_mut<T: Component>(&mut self, mut f: impl FnMut(Entity, &mut T)) {
        if let Some(map) = self.comps.get_mut(&TypeId::of::<T>()) {
            for (e, b) in map.iter_mut() {
                if let Some(r) = b.downcast_mut::<T>() {
                    f(*e, r);
                }
            }
        }
    }
}

// Simple query over one or two component types (Phase 1 minimal)
pub struct Query<'w, T: Component> {
    world: &'w World,
    ty: TypeId,
    it: Option<std::collections::btree_map::Iter<'w, Entity, Box<dyn std::any::Any + Send + Sync>>>,
    _m: std::marker::PhantomData<T>,
}

impl<'w, T: Component> Query<'w, T> {
    pub fn new(world: &'w World) -> Self {
        let ty = TypeId::of::<T>();
        let it = world.comps.get(&ty).map(|m| m.iter());
        Self { world, ty, it, _m: Default::default() }
    }
}

// Read-only two-component query: yields entities that have both A and B
pub struct Query2<'w, A: Component, B: Component> {
    wb: &'w World,
    ita: Option<std::collections::btree_map::Iter<'w, Entity, Box<dyn std::any::Any + Send + Sync>>>,
    _m: std::marker::PhantomData<(A, B)>,
}

impl<'w, A: Component, B: Component> Query2<'w, A, B> {
    pub fn new(world: &'w World) -> Self {
        let ita = world.comps.get(&TypeId::of::<A>()).map(|m| m.iter());
    Self { wb: world, ita, _m: Default::default() }
    }
}

impl<'w, A: Component, B: Component> Iterator for Query2<'w, A, B> {
    type Item = (Entity, &'w A, &'w B);
    fn next(&mut self) -> Option<Self::Item> {
        let it = self.ita.as_mut()?;
        for (e, ba) in it {
            if let Some(ra) = ba.downcast_ref::<A>() {
                if let Some(bmap) = self.wb.comps.get(&TypeId::of::<B>()) {
                    if let Some(bb) = bmap.get(e) {
                        if let Some(rb) = bb.downcast_ref::<B>() {
                            return Some((*e, ra, rb));
                        }
                    }
                }
            }
        }
        None
    }
}

impl<'w, T: Component> Iterator for Query<'w, T> {
    type Item = (Entity, &'w T);
    fn next(&mut self) -> Option<Self::Item> {
        let it = self.it.as_mut()?;
        for (e, b) in it {
            if let Some(r) = b.downcast_ref::<T>() {
                return Some((*e, r));
            }
        }
        None
    }
}

// Schedule and systems
pub type SystemFn = fn(&mut World);

#[derive(Default)]
pub struct Schedule {
    pub stages: Vec<Stage>,
}

pub struct Stage {
    pub name: &'static str,
    pub systems: Vec<SystemFn>,
}

impl Schedule {
    pub fn with_stage(mut self, name: &'static str) -> Self {
        self.stages.push(Stage { name, systems: vec![] });
        self
    }
    pub fn add_system(&mut self, stage: &'static str, sys: SystemFn) {
        if let Some(s) = self.stages.iter_mut().find(|s| s.name==stage) {
            s.systems.push(sys);
        }
    }
    pub fn run(&self, world: &mut World) {
        for s in &self.stages {
            for f in &s.systems {
                (f)(world);
            }
        }
    }
}

// App-like builder with deterministic fixed-timestep driver
pub struct App {
    pub world: World,
    pub schedule: Schedule,
}

impl App {
    pub fn new() -> Self {
        let mut schedule = Schedule::default();
        schedule = schedule
            .with_stage("perception")
            .with_stage("simulation")
            .with_stage("ai_planning")
            .with_stage("physics")
            .with_stage("presentation");
        Self { world: World::new(), schedule }
    }
    pub fn add_system(&mut self, stage: &'static str, sys: SystemFn) {
        self.schedule.add_system(stage, sys);
    }
    pub fn insert_resource<T: 'static + Send + Sync>(mut self, r: T) -> Self { self.world.insert_resource(r); self }
    pub fn run_fixed(mut self, steps: u32) -> Self {
        for _ in 0..steps { self.schedule.run(&mut self.world); }
        self
    }
}

// Plugin pattern similar to Bevy
pub trait Plugin {
    fn build(&self, app: &mut App);
}
impl App {
    pub fn add_plugin(mut self, p: impl Plugin) -> Self { p.build(&mut self); self }
}

// Filtered query: yields entities that have T and pass a filter function
pub struct FilteredQuery<'w, T: Component, F: Fn(&T) -> bool> {
    query: Query<'w, T>,
    filter: F,
}

impl<'w, T: Component, F: Fn(&T) -> bool> FilteredQuery<'w, T, F> {
    pub fn new(world: &'w World, filter: F) -> Self {
        Self { query: Query::new(world), filter }
    }
}

impl<'w, T: Component, F: Fn(&T) -> bool> Iterator for FilteredQuery<'w, T, F> {
    type Item = (Entity, &'w T);
    fn next(&mut self) -> Option<Self::Item> {
        while let Some((e, t)) = self.query.next() {
            if (self.filter)(t) {
                return Some((e, t));
            }
        }
        None
    }
}

// Convenience macro for common queries
#[macro_export]
macro_rules! query {
    ($world:expr, $comp:ty) => {
        $crate::Query::<$comp>::new($world)
    };
    ($world:expr, $comp:ty, where $filter:expr) => {
        $crate::FilteredQuery::<$comp, _>::new($world, $filter)
    };
}

#[macro_export]
macro_rules! query2 {
    ($world:expr, $a:ty, $b:ty) => {
        $crate::Query2::<$a, $b>::new($world)
    };
}

// Convenience methods on World
impl World {
    /// Get all entities with a specific component
    pub fn entities_with<T: Component>(&self) -> Vec<Entity> {
        self.comps.get(&TypeId::of::<T>())
            .map(|m| m.keys().cloned().collect())
            .unwrap_or_default()
    }

    /// Check if entity has component
    pub fn has<T: Component>(&self, e: Entity) -> bool {
        self.get::<T>(e).is_some()
    }

    /// Remove component from entity (Phase 1: basic implementation)
    pub fn remove<T: Component>(&mut self, e: Entity) -> bool {
        self.comps.get_mut(&TypeId::of::<T>())
            .and_then(|m| m.remove(&e))
            .is_some()
    }

    /// Count entities with component
    pub fn count<T: Component>(&self) -> usize {
        self.comps.get(&TypeId::of::<T>())
            .map(|m| m.len())
            .unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[derive(Clone, Copy)]
    struct Pos { x: i32 }

    fn sim(world: &mut World) { // increments all positions
        let mut to_update: Vec<Entity> = vec![];
        {
            let q = Query::<Pos>::new(world);
            for (e, _p) in q { to_update.push(e); }
        }
        for e in to_update { if let Some(p)=world.get_mut::<Pos>(e) { p.x += 1; } }
    }

    #[test]
    fn schedule_runs_in_order() {
        let mut app = App::new();
        let e = app.world.spawn();
        app.world.insert(e, Pos { x: 0 });
        app.add_system("simulation", sim);
        app = app.run_fixed(3);
        assert_eq!(app.world.get::<Pos>(e).unwrap().x, 3);
    }

    #[test]
    fn query2_yields_only_entities_with_both() {
        #[derive(Clone, Copy)] struct A(u32);
        #[derive(Clone, Copy)] struct B(u32);
        let mut app = App::new();
        let e1 = app.world.spawn();
        let e2 = app.world.spawn();
        app.world.insert(e1, A(1));
        app.world.insert(e1, B(2));
        app.world.insert(e2, A(3)); // missing B
        let mut seen = Vec::new();
        let q = Query2::<A, B>::new(&app.world);
        for (e, a, b) in q { seen.push((e, a.0, b.0)); }
        assert_eq!(seen.len(), 1);
        assert_eq!(seen[0].1, 1);
        assert_eq!(seen[0].2, 2);
    }

    #[test]
    fn filtered_query_works() {
        #[derive(Clone, Copy)] struct Health(i32);
        let mut world = World::new();
        let e1 = world.spawn();
        let e2 = world.spawn();
        world.insert(e1, Health(100));
        world.insert(e2, Health(50));
        
        let mut healthy = Vec::new();
        let fq = FilteredQuery::new(&world, |h: &Health| h.0 > 75);
        for (e, h) in fq { healthy.push((e, h.0)); }
        assert_eq!(healthy.len(), 1);
        assert_eq!(healthy[0].1, 100);
    }

    #[test]
    fn world_convenience_methods() {
        #[derive(Clone, Copy)] struct TestComp(u32);
        let mut world = World::new();
        let e1 = world.spawn();
        let e2 = world.spawn();
        world.insert(e1, TestComp(42));
        world.insert(e2, TestComp(24));
        
        assert_eq!(world.count::<TestComp>(), 2);
        assert!(world.has::<TestComp>(e1));
        assert!(!world.has::<TestComp>(Entity(999)));
        
        let entities = world.entities_with::<TestComp>();
        assert_eq!(entities.len(), 2);
        assert!(entities.contains(&e1));
        assert!(entities.contains(&e2));
        
        assert!(world.remove::<TestComp>(e1));
        assert_eq!(world.count::<TestComp>(), 1);
        assert!(!world.has::<TestComp>(e1));
    }
}
