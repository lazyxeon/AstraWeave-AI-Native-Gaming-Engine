use glam::{Mat4, Quat, Vec3};
use serde::{Deserialize, Serialize};

pub mod gpu_resource_manager;
pub mod partitioned_scene;
pub mod streaming;
pub mod world_partition;

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
        #[allow(dead_code)]
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
    #[allow(unused_variables)]
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



