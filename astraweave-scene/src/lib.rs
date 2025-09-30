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
    use astraweave_ecs::{Component, EntityId};

    /// Component for hierarchical transforms
    #[derive(Clone, Copy, Debug)]
    pub struct CTransform(pub Transform);

    impl Component for CTransform {}

    /// Component for parent-child relationships
    #[derive(Clone, Debug)]
    pub struct CParent(pub EntityId);

    impl Component for CParent {}

    /// Component for children list
    #[derive(Clone, Debug)]
    pub struct CChildren(pub Vec<EntityId>);

    impl Component for CChildren {}

    /// System to compute world transforms from hierarchy
    pub fn update_world_transforms(world: &mut astraweave_ecs::World) {
        // Simple topological sort and update
        // For now, assume no cycles and update in entity order
        let mut world_transforms = std::collections::HashMap::new();

        for (id, transform) in world.query::<CTransform>() {
            let world_mat = if let Some(parent_id) = world.get::<CParent>(id).map(|p| p.0) {
                if let Some(parent_mat) = world_transforms.get(&parent_id) {
                    *parent_mat * transform.0.matrix()
                } else {
                    transform.0.matrix()
                }
            } else {
                transform.0.matrix()
            };
            world_transforms.insert(id, world_mat);
        }

        // Store back or use for rendering
        // For now, just compute; renderer can query
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
        use astraweave_ecs::{World, EntityId};
        let mut world = World::new();
        let id = world.spawn();
        world.insert(id, ecs::CTransform(Transform::default()));
        let transform = world.get::<ecs::CTransform>(id).unwrap();
        assert_eq!(transform.0.translation, Vec3::ZERO);
    }
}
