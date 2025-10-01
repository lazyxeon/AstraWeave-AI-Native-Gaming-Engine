use glam::{Mat4, Quat, Vec3};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Transform {
    pub translation: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            translation: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        }
    }
}

impl Transform {
    pub fn matrix(&self) -> Mat4 {
        Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.translation)
    }
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Node {
    pub name: String,
    pub transform: Transform,
    pub children: Vec<Node>,
}

impl Node {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            transform: Transform::default(),
            children: Vec::new(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Scene {
    pub root: Node,
}

impl Scene {
    pub fn new() -> Self {
        Self {
            root: Node::new("root"),
        }
    }

    pub fn traverse<'a>(&'a self, f: &mut impl FnMut(&'a Node, Mat4)) {
        fn walk<'a>(n: &'a Node, parent: Mat4, f: &mut impl FnMut(&'a Node, Mat4)) {
            let world = parent * n.transform.matrix();
            f(n, world);
            for c in &n.children {
                walk(c, world, f);
            }
        }
        walk(&self.root, Mat4::IDENTITY, f);
    }
}

// ECS Components for scene graph integration
#[cfg(feature = "ecs")]
pub mod ecs {
    use super::*;
    use astraweave_ecs::{Entity as EntityId, World as EcsWorld};
    use std::collections::{BTreeMap, BTreeSet};

    /// Component for local transform (relative to parent)
    #[derive(Clone, Copy, Debug)]
    pub struct CTransformLocal(pub Transform);

    /// Component for world-space transform (computed from hierarchy)
    #[derive(Clone, Copy, Debug)]
    pub struct CTransformWorld(pub Mat4);

    /// Component for parent-child relationships
    #[derive(Clone, Debug)]
    pub struct CParent(pub EntityId);

    /// Component for children list (maintained automatically)
    #[derive(Clone, Debug, Default)]
    pub struct CChildren(pub Vec<EntityId>);

    /// Tag component indicating transform needs update
    #[derive(Clone, Copy, Debug)]
    pub struct CDirtyTransform;

    /// Component for visibility culling
    #[derive(Clone, Copy, Debug)]
    pub struct CVisible(pub bool);

    /// Component for mesh handle (for rendering integration)
    #[derive(Clone, Copy, Debug)]
    pub struct CMesh(pub u32); // MeshHandle or index

    /// Component for material layer index (for rendering integration)
    #[derive(Clone, Copy, Debug)]
    pub struct CMaterial(pub u32); // Material layer index from MaterialManager

    /// Component for joint indices (skinning preparation)
    #[derive(Clone, Debug)]
    pub struct CJointIndices(pub Vec<u32>);

    // ========================================================================
    // Phase 2 Task 5: Skeletal Animation Components
    // ========================================================================

    /// Skeleton component (shared across instances via Arc)
    #[derive(Clone, Debug)]
    pub struct CSkeleton {
        pub joint_count: u32,
        pub root_indices: Vec<usize>,
        pub parent_indices: Vec<Option<usize>>,
        pub inverse_bind_matrices: Vec<Mat4>,
        pub local_transforms: Vec<Transform>, // Bind pose
    }

    /// Skinned mesh component
    #[derive(Clone, Debug)]
    pub struct CSkinnedMesh {
        pub mesh_handle: u32,
        pub max_influences: u8, // Typically 4
    }

    /// Animation playback state
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum PlaybackState {
        Playing,
        Paused,
        Stopped,
    }

    /// Animator component (per-entity animation state)
    #[derive(Clone, Debug)]
    pub struct CAnimator {
        pub clip_index: usize,
        pub time: f32,
        pub speed: f32,
        pub state: PlaybackState,
        pub looping: bool,
    }

    impl Default for CAnimator {
        fn default() -> Self {
            Self {
                clip_index: 0,
                time: 0.0,
                speed: 1.0,
                state: PlaybackState::Stopped,
                looping: true,
            }
        }
    }

    /// Computed joint matrices (cached per-frame)
    #[derive(Clone, Debug)]
    pub struct CJointMatrices {
        pub matrices: Vec<Mat4>, // Skinning matrices (world * inverse_bind)
        pub dirty: bool,
    }

    impl Default for CJointMatrices {
        fn default() -> Self {
            Self {
                matrices: Vec::new(),
                dirty: true,
            }
        }
    }

    /// Tag component for entities with dirty animation state
    #[derive(Clone, Copy, Debug)]
    pub struct CDirtyAnimation;

    /// Component for attaching entity to a specific joint (bone attachment)
    #[derive(Clone, Copy, Debug)]
    pub struct CParentBone {
        pub skeleton_entity: EntityId,
        pub joint_index: usize,
    }

    /// Helper structure for managing scene graph operations
    pub struct SceneGraph;

    impl SceneGraph {
        /// Attach a child entity to a parent, updating parent's CChildren
        pub fn attach(world: &mut EcsWorld, child: EntityId, parent: EntityId) {
            // Set parent
            world.insert(child, CParent(parent));

            // Mark child as dirty
            world.insert(child, CDirtyTransform);

            // Update parent's children list
            if let Some(children) = world.get_mut::<CChildren>(parent) {
                if !children.0.contains(&child) {
                    children.0.push(child);
                }
            } else {
                world.insert(parent, CChildren(vec![child]));
            }
        }

        /// Detach a child from its parent
        pub fn detach(world: &mut EcsWorld, child: EntityId) {
            if let Some(parent_comp) = world.get::<CParent>(child) {
                let parent = parent_comp.0;

                // Remove from parent's children list
                if let Some(children) = world.get_mut::<CChildren>(parent) {
                    children.0.retain(|&c| c != child);
                }

                // Remove parent component
                world.remove::<CParent>(child);

                // Mark as dirty (now a root)
                world.insert(child, CDirtyTransform);
            }
        }

        /// Reparent a child to a new parent
        pub fn reparent(world: &mut EcsWorld, child: EntityId, new_parent: EntityId) {
            Self::detach(world, child);
            Self::attach(world, child, new_parent);
        }

        /// Mark an entity and all descendants as dirty
        fn mark_dirty_recursive(
            world: &mut EcsWorld,
            entity: EntityId,
            visited: &mut BTreeSet<EntityId>,
        ) {
            if visited.contains(&entity) {
                return; // Avoid cycles
            }
            visited.insert(entity);

            world.insert(entity, CDirtyTransform);

            // Clone children list to avoid borrow checker issues
            let children = world.get::<CChildren>(entity).map(|c| c.0.clone());

            if let Some(children_list) = children {
                for child in children_list {
                    Self::mark_dirty_recursive(world, child, visited);
                }
            }
        }
    }

    /// System to mark dirty transforms when local transforms change
    pub fn mark_dirty_transforms(world: &mut EcsWorld) {
        // In a real system, this would check for changes to CTransformLocal
        // For now, we assume callers manually insert CDirtyTransform when needed
        // Future: integrate with ECS change detection
    }

    /// System to update world transforms from hierarchy (deterministic topological order)
    pub fn update_world_transforms(world: &mut EcsWorld) {
        // Collect all entities with transforms
        let mut entities: Vec<EntityId> = Vec::new();
        world.each_mut::<CTransformLocal>(|e, _| entities.push(e));

        // Sort by entity ID for determinism (BTreeMap iteration is already sorted, but explicit is better)
        entities.sort_by_key(|e| e.id());

        // Build parent-child map for topological sort
        let mut parents: BTreeMap<EntityId, EntityId> = BTreeMap::new();
        let mut children_map: BTreeMap<EntityId, Vec<EntityId>> = BTreeMap::new();

        for &entity in &entities {
            if let Some(parent_comp) = world.get::<CParent>(entity) {
                parents.insert(entity, parent_comp.0);
                children_map.entry(parent_comp.0).or_default().push(entity);
            }
        }

        // Find roots (entities with no parent)
        let mut roots: Vec<EntityId> = entities
            .iter()
            .filter(|e| !parents.contains_key(e))
            .copied()
            .collect();
        roots.sort_by_key(|e| e.id()); // Deterministic order

        // Traverse depth-first from roots, updating world transforms
        fn update_recursive(
            world: &mut EcsWorld,
            entity: EntityId,
            parent_world: Mat4,
            children_map: &BTreeMap<EntityId, Vec<EntityId>>,
        ) {
            // Get local transform
            let local_mat = if let Some(local) = world.get::<CTransformLocal>(entity) {
                local.0.matrix()
            } else {
                Mat4::IDENTITY
            };

            // Compute world transform
            let world_mat = parent_world * local_mat;

            // Store world transform
            world.insert(entity, CTransformWorld(world_mat));

            // Remove dirty flag
            world.remove::<CDirtyTransform>(entity);

            // Recurse to children (sorted for determinism)
            if let Some(children) = children_map.get(&entity) {
                let mut sorted_children = children.clone();
                sorted_children.sort_by_key(|e| e.id());
                for &child in &sorted_children {
                    update_recursive(world, child, world_mat, children_map);
                }
            }
        }

        // Update all roots and their descendants
        for root in roots {
            update_recursive(world, root, Mat4::IDENTITY, &children_map);
        }
    }

    /// Render instance data (minimal, for renderer integration)
    #[derive(Clone, Debug)]
    pub struct RenderInstance {
        pub entity: EntityId,
        pub world_transform: Mat4,
        pub mesh_handle: u32,
        pub material_index: u32,
    }

    /// System to sync scene to renderer (collect visible instances)
    pub fn sync_scene_to_renderer(world: &mut EcsWorld) -> Vec<RenderInstance> {
        let mut instances = Vec::new();

        // First, collect all entities with world transforms
        let mut entities_with_transforms = Vec::new();
        world.each_mut::<CTransformWorld>(|e, _| {
            entities_with_transforms.push(e);
        });

        // Then query each entity for its components
        for entity in entities_with_transforms {
            // Check visibility
            let visible = world.get::<CVisible>(entity).map(|v| v.0).unwrap_or(true); // Default to visible

            if !visible {
                continue;
            }

            // Must have mesh and material
            let mesh_handle = match world.get::<CMesh>(entity) {
                Some(m) => m.0,
                None => continue,
            };

            let material_index = match world.get::<CMaterial>(entity) {
                Some(m) => m.0,
                None => continue,
            };

            let world_transform = world
                .get::<CTransformWorld>(entity)
                .map(|t| t.0)
                .unwrap_or(Mat4::IDENTITY);

            instances.push(RenderInstance {
                entity,
                world_transform,
                mesh_handle,
                material_index,
            });
        }

        // Sort for deterministic rendering order
        instances.sort_by_key(|inst| inst.entity.id());

        instances
    }

    // ========================================================================
    // Phase 2 Task 5: Skeletal Animation Systems
    // ========================================================================

    /// System to update animation time (advance playback)
    /// Call this each frame with delta time
    pub fn update_animations(world: &mut EcsWorld, dt: f32, clip_durations: &[f32]) {
        // Collect entities with animators
        let mut entities = Vec::new();
        world.each_mut::<CAnimator>(|e, _| entities.push(e));

        for entity in entities {
            if let Some(animator) = world.get_mut::<CAnimator>(entity) {
                if animator.state != PlaybackState::Playing {
                    continue;
                }

                let clip_duration = clip_durations
                    .get(animator.clip_index)
                    .copied()
                    .unwrap_or(1.0);

                // Advance time
                animator.time += dt * animator.speed;

                // Handle looping/clamping
                if animator.looping {
                    // Wrap around
                    if animator.time > clip_duration {
                        animator.time = animator.time % clip_duration;
                    }
                    if animator.time < 0.0 {
                        animator.time = clip_duration + (animator.time % clip_duration);
                    }
                } else {
                    // Clamp and stop at end
                    animator.time = animator.time.clamp(0.0, clip_duration);
                    if animator.time >= clip_duration {
                        animator.state = PlaybackState::Stopped;
                    }
                }

                // Mark for recomputation
                world.insert(entity, CDirtyAnimation);
            }
        }
    }

    /// System to compute joint matrices from animation state
    /// This uses the animation sampling and joint matrix computation from astraweave-render
    /// Note: This is a stub - full implementation requires AnimationClip from render crate
    pub fn compute_poses_stub(world: &mut EcsWorld) {
        // Collect entities needing pose update
        let mut entities = Vec::new();
        world.each_mut::<CDirtyAnimation>(|e, _| entities.push(e));

        for entity in entities {
            // Check if entity has all required components
            let has_skeleton = world.get::<CSkeleton>(entity).is_some();
            let has_matrices = world.get::<CJointMatrices>(entity).is_some();

            if !has_skeleton || !has_matrices {
                continue;
            }

            // Get skeleton to know joint count
            let joint_count = world
                .get::<CSkeleton>(entity)
                .map(|s| s.joint_count)
                .unwrap_or(0);

            // Mark matrices as dirty (actual computation happens in renderer integration)
            if let Some(matrices) = world.get_mut::<CJointMatrices>(entity) {
                matrices.dirty = true;
                // Ensure matrices vector has correct size
                if matrices.matrices.len() != joint_count as usize {
                    matrices
                        .matrices
                        .resize(joint_count as usize, Mat4::IDENTITY);
                }
            }

            // Remove dirty animation flag
            world.remove::<CDirtyAnimation>(entity);
        }
    }

    /// System to sync bone transforms to scene graph nodes
    /// This propagates joint world transforms to attached entities (e.g., weapon to hand)
    pub fn sync_bone_attachments(world: &mut EcsWorld) {
        // Collect all entities with bone attachments
        let mut attachments = Vec::new();
        world.each_mut::<CParentBone>(|e, _| attachments.push(e));

        for entity in attachments {
            if let Some(attachment) = world.get::<CParentBone>(entity) {
                let skeleton_entity = attachment.skeleton_entity;
                let joint_index = attachment.joint_index;

                // Get joint matrices from skeleton entity
                if let Some(matrices) = world.get::<CJointMatrices>(skeleton_entity) {
                    if joint_index < matrices.matrices.len() {
                        let joint_world_matrix = matrices.matrices[joint_index];

                        // Set attached entity's world transform to joint transform
                        world.insert(entity, CTransformWorld(joint_world_matrix));

                        // Also update local transform if entity has parent
                        // (for proper scene graph integration)
                        if let Some(parent_comp) = world.get::<CParent>(entity) {
                            let parent_world = world
                                .get::<CTransformWorld>(parent_comp.0)
                                .map(|t| t.0)
                                .unwrap_or(Mat4::IDENTITY);

                            // Compute local = parent_inv * world
                            // Note: glam 0.30 inverse() returns Mat4, not Option
                            let parent_inv = parent_world.inverse();
                            let local_mat = parent_inv * joint_world_matrix;
                            // Decompose to TRS (approximate for now)
                            let (scale, rotation, translation) =
                                local_mat.to_scale_rotation_translation();
                            world.insert(
                                entity,
                                CTransformLocal(Transform {
                                    translation,
                                    rotation,
                                    scale,
                                }),
                            );
                        }
                    }
                }
            }
        }
    }
}

#[cfg(not(feature = "ecs"))]
pub mod ecs {
    // Stub when ECS not enabled
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transform_matrix() {
        let t = Transform {
            translation: Vec3::new(1.0, 2.0, 3.0),
            rotation: Quat::from_rotation_z(std::f32::consts::PI / 2.0),
            scale: Vec3::new(2.0, 2.0, 2.0),
        };
        let mat = t.matrix();
        // Check translation
        assert_eq!(mat.w_axis, glam::Vec4::new(1.0, 2.0, 3.0, 1.0));
    }

    #[test]
    fn test_scene_traverse() {
        let mut scene = Scene::new();
        scene.root.children.push(Node::new("child"));
        let mut count = 0;
        scene.traverse(&mut |node, _world| {
            count += 1;
            if node.name == "root" {
                assert!(node.children.len() == 1);
            }
        });
        assert_eq!(count, 2);
    }

    #[cfg(feature = "ecs")]
    #[test]
    fn test_ecs_components() {
        use astraweave_ecs::World as EcsWorld;
        use ecs::*;
        let mut world = EcsWorld::new();
        let id = world.spawn();
        world.insert(id, CTransformLocal(Transform::default()));
        let transform = world.get::<CTransformLocal>(id).unwrap();
        assert_eq!(transform.0.translation, Vec3::ZERO);
    }

    #[cfg(feature = "ecs")]
    #[test]
    fn test_transform_hierarchy_three_levels() {
        use astraweave_ecs::World as EcsWorld;
        use ecs::*;
        let mut world = EcsWorld::new();

        // Create hierarchy: root -> child -> grandchild
        let root = world.spawn();
        let child = world.spawn();
        let grandchild = world.spawn();

        // Set up transforms (root at origin, child offset +1 X, grandchild offset +1 X from child)
        world.insert(
            root,
            CTransformLocal(Transform {
                translation: Vec3::ZERO,
                rotation: Quat::IDENTITY,
                scale: Vec3::ONE,
            }),
        );
        world.insert(
            child,
            CTransformLocal(Transform {
                translation: Vec3::new(1.0, 0.0, 0.0),
                rotation: Quat::IDENTITY,
                scale: Vec3::ONE,
            }),
        );
        world.insert(
            grandchild,
            CTransformLocal(Transform {
                translation: Vec3::new(1.0, 0.0, 0.0),
                rotation: Quat::IDENTITY,
                scale: Vec3::ONE,
            }),
        );

        // Attach hierarchy
        SceneGraph::attach(&mut world, child, root);
        SceneGraph::attach(&mut world, grandchild, child);

        // Update transforms
        update_world_transforms(&mut world);

        // Verify world transforms
        let root_world = world.get::<CTransformWorld>(root).unwrap().0;
        assert_eq!(root_world.w_axis.truncate(), Vec3::ZERO);

        let child_world = world.get::<CTransformWorld>(child).unwrap().0;
        assert_eq!(child_world.w_axis.truncate(), Vec3::new(1.0, 0.0, 0.0));

        let grandchild_world = world.get::<CTransformWorld>(grandchild).unwrap().0;
        assert_eq!(grandchild_world.w_axis.truncate(), Vec3::new(2.0, 0.0, 0.0));
    }

    #[cfg(feature = "ecs")]
    #[test]
    fn test_reparenting_invalidates_world_transforms() {
        use astraweave_ecs::World as EcsWorld;
        use ecs::*;
        let mut world = EcsWorld::new();

        // Create entities
        let parent1 = world.spawn();
        let parent2 = world.spawn();
        let child = world.spawn();

        world.insert(
            parent1,
            CTransformLocal(Transform {
                translation: Vec3::new(10.0, 0.0, 0.0),
                ..Default::default()
            }),
        );
        world.insert(
            parent2,
            CTransformLocal(Transform {
                translation: Vec3::new(20.0, 0.0, 0.0),
                ..Default::default()
            }),
        );
        world.insert(child, CTransformLocal(Transform::default()));

        // Attach to parent1
        SceneGraph::attach(&mut world, child, parent1);
        update_world_transforms(&mut world);

        let child_world_1 = world.get::<CTransformWorld>(child).unwrap().0;
        assert_eq!(child_world_1.w_axis.truncate(), Vec3::new(10.0, 0.0, 0.0));

        // Reparent to parent2
        SceneGraph::reparent(&mut world, child, parent2);

        // Verify dirty flag is set
        assert!(world.get::<CDirtyTransform>(child).is_some());

        // Update transforms
        update_world_transforms(&mut world);

        // Verify world transform reflects new parent
        let child_world_2 = world.get::<CTransformWorld>(child).unwrap().0;
        assert_eq!(child_world_2.w_axis.truncate(), Vec3::new(20.0, 0.0, 0.0));

        // Dirty flag should be removed
        assert!(world.get::<CDirtyTransform>(child).is_none());
    }

    #[cfg(feature = "ecs")]
    #[test]
    fn test_deterministic_traversal_order() {
        use astraweave_ecs::World as EcsWorld;
        use ecs::*;
        let mut world = EcsWorld::new();

        // Create root and multiple children in non-sorted order
        let root = world.spawn();
        world.insert(root, CTransformLocal(Transform::default()));

        let child_ids: Vec<_> = (0..10).map(|_| world.spawn()).collect();

        for &child in &child_ids {
            world.insert(child, CTransformLocal(Transform::default()));
            SceneGraph::attach(&mut world, child, root);
        }

        // Update transforms multiple times
        for _ in 0..3 {
            update_world_transforms(&mut world);

            // Collect entities that got world transforms
            let mut updated: Vec<_> = Vec::new();
            world.each_mut::<CTransformWorld>(|e: astraweave_ecs::Entity, _| updated.push(e));
            updated.sort_by_key(|e| e.id());

            // Verify all entities updated
            assert!(updated.len() >= child_ids.len() + 1); // children + root

            // Verify root is in the list
            assert!(updated.contains(&root));
        }
    }

    #[cfg(feature = "ecs")]
    #[test]
    fn test_visibility_culling() {
        use astraweave_ecs::World as EcsWorld;
        use ecs::*;
        let mut world = EcsWorld::new();

        // Create entities with meshes and materials
        let visible = world.spawn();
        world.insert(visible, CTransformLocal(Transform::default()));
        world.insert(visible, CMesh(1));
        world.insert(visible, CMaterial(0));
        world.insert(visible, CVisible(true));

        let invisible = world.spawn();
        world.insert(invisible, CTransformLocal(Transform::default()));
        world.insert(invisible, CMesh(2));
        world.insert(invisible, CMaterial(1));
        world.insert(invisible, CVisible(false));

        let no_visibility_component = world.spawn();
        world.insert(
            no_visibility_component,
            CTransformLocal(Transform::default()),
        );
        world.insert(no_visibility_component, CMesh(3));
        world.insert(no_visibility_component, CMaterial(2));
        // No CVisible component (defaults to visible)

        // Update transforms
        update_world_transforms(&mut world);

        // Sync to renderer
        let instances = sync_scene_to_renderer(&mut world);

        // Should have 2 instances (visible + no_visibility_component, but NOT invisible)
        assert_eq!(instances.len(), 2);
        assert!(instances.iter().any(|i| i.entity == visible));
        assert!(instances
            .iter()
            .any(|i| i.entity == no_visibility_component));
        assert!(!instances.iter().any(|i| i.entity == invisible));
    }

    #[cfg(feature = "ecs")]
    #[test]
    fn test_scene_graph_detach() {
        use astraweave_ecs::World as EcsWorld;
        use ecs::*;
        let mut world = EcsWorld::new();

        let parent = world.spawn();
        let child = world.spawn();

        world.insert(parent, CTransformLocal(Transform::default()));
        world.insert(child, CTransformLocal(Transform::default()));

        // Attach
        SceneGraph::attach(&mut world, child, parent);
        assert!(world.get::<CParent>(child).is_some());
        assert!(world.get::<CChildren>(parent).unwrap().0.contains(&child));

        // Detach
        SceneGraph::detach(&mut world, child);
        assert!(world.get::<CParent>(child).is_none());
        assert!(!world.get::<CChildren>(parent).unwrap().0.contains(&child));

        // Child should be marked dirty
        assert!(world.get::<CDirtyTransform>(child).is_some());
    }

    // ========================================================================
    // Phase 2 Task 5: Skeletal Animation Tests
    // ========================================================================

    #[cfg(feature = "ecs")]
    #[test]
    fn test_animator_component() {
        use astraweave_ecs::World as EcsWorld;
        use ecs::*;
        let mut world = EcsWorld::new();

        let entity = world.spawn();
        let animator = CAnimator {
            clip_index: 0,
            time: 0.5,
            speed: 1.0,
            state: PlaybackState::Playing,
            looping: true,
        };
        world.insert(entity, animator);

        let retrieved = world.get::<CAnimator>(entity).unwrap();
        assert_eq!(retrieved.time, 0.5);
        assert_eq!(retrieved.state, PlaybackState::Playing);
    }

    #[cfg(feature = "ecs")]
    #[test]
    fn test_update_animations_looping() {
        use astraweave_ecs::World as EcsWorld;
        use ecs::*;
        let mut world = EcsWorld::new();

        let entity = world.spawn();
        world.insert(
            entity,
            CAnimator {
                clip_index: 0,
                time: 0.8,
                speed: 1.0,
                state: PlaybackState::Playing,
                looping: true,
            },
        );

        let clip_durations = vec![1.0]; // 1 second clip
        update_animations(&mut world, 0.5, &clip_durations);

        let animator = world.get::<CAnimator>(entity).unwrap();
        // time = 0.8 + 0.5 = 1.3 -> wraps to 0.3
        assert!((animator.time - 0.3).abs() < 0.001);
        assert_eq!(animator.state, PlaybackState::Playing);
        assert!(world.get::<CDirtyAnimation>(entity).is_some());
    }

    #[cfg(feature = "ecs")]
    #[test]
    fn test_update_animations_clamping() {
        use astraweave_ecs::World as EcsWorld;
        use ecs::*;
        let mut world = EcsWorld::new();

        let entity = world.spawn();
        world.insert(
            entity,
            CAnimator {
                clip_index: 0,
                time: 0.8,
                speed: 1.0,
                state: PlaybackState::Playing,
                looping: false,
            },
        );

        let clip_durations = vec![1.0];
        update_animations(&mut world, 0.5, &clip_durations);

        let animator = world.get::<CAnimator>(entity).unwrap();
        // time = 0.8 + 0.5 = 1.3 -> clamps to 1.0
        assert_eq!(animator.time, 1.0);
        assert_eq!(animator.state, PlaybackState::Stopped); // Should stop at end
    }

    #[cfg(feature = "ecs")]
    #[test]
    fn test_skeleton_component() {
        use astraweave_ecs::World as EcsWorld;
        use ecs::*;
        let mut world = EcsWorld::new();

        let entity = world.spawn();
        let skeleton = CSkeleton {
            joint_count: 3,
            root_indices: vec![0],
            parent_indices: vec![None, Some(0), Some(0)],
            inverse_bind_matrices: vec![Mat4::IDENTITY; 3],
            local_transforms: vec![Transform::default(); 3],
        };
        world.insert(entity, skeleton);

        let retrieved = world.get::<CSkeleton>(entity).unwrap();
        assert_eq!(retrieved.joint_count, 3);
        assert_eq!(retrieved.root_indices, vec![0]);
    }

    #[cfg(feature = "ecs")]
    #[test]
    fn test_joint_matrices_initialization() {
        use astraweave_ecs::World as EcsWorld;
        use ecs::*;
        let mut world = EcsWorld::new();

        let entity = world.spawn();
        let skeleton = CSkeleton {
            joint_count: 2,
            root_indices: vec![0],
            parent_indices: vec![None, Some(0)],
            inverse_bind_matrices: vec![Mat4::IDENTITY; 2],
            local_transforms: vec![Transform::default(); 2],
        };
        world.insert(entity, skeleton);
        world.insert(entity, CJointMatrices::default());
        world.insert(entity, CDirtyAnimation);

        // Run compute poses stub
        compute_poses_stub(&mut world);

        let matrices = world.get::<CJointMatrices>(entity).unwrap();
        assert_eq!(matrices.matrices.len(), 2);
        assert!(matrices.dirty); // Marked for renderer upload
        assert!(world.get::<CDirtyAnimation>(entity).is_none()); // Flag removed
    }

    #[cfg(feature = "ecs")]
    #[test]
    fn test_bone_attachment() {
        use astraweave_ecs::World as EcsWorld;
        use ecs::*;
        let mut world = EcsWorld::new();

        // Create skeleton entity
        let skeleton_entity = world.spawn();
        let mut matrices = CJointMatrices::default();
        matrices.matrices = vec![
            Mat4::from_translation(Vec3::new(1.0, 0.0, 0.0)), // Joint 0
            Mat4::from_translation(Vec3::new(2.0, 0.0, 0.0)), // Joint 1 (hand)
        ];
        world.insert(skeleton_entity, matrices);

        // Create attached entity (e.g., sword)
        let sword_entity = world.spawn();
        world.insert(
            sword_entity,
            CParentBone {
                skeleton_entity,
                joint_index: 1, // Attach to hand joint
            },
        );

        // Sync bone attachments
        sync_bone_attachments(&mut world);

        // Verify sword is at hand position
        let sword_world = world.get::<CTransformWorld>(sword_entity).unwrap().0;
        assert_eq!(sword_world.w_axis.truncate(), Vec3::new(2.0, 0.0, 0.0));
    }
}
