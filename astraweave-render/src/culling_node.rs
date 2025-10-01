//! Culling node for render graph integration (Phase 2 Task 3)

use super::culling::{CullingPipeline, CullingResources, FrustumPlanes, InstanceAABB};
use super::graph::{GraphContext, RenderNode};
use anyhow::Context;

/// Render graph node for GPU-driven frustum culling
pub struct CullingNode {
    name: String,
    pipeline: CullingPipeline,
    resources: Option<CullingResources>,
    instance_count: u32,
}

impl CullingNode {
    pub fn new(device: &wgpu::Device, name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            pipeline: CullingPipeline::new(device),
            resources: None,
            instance_count: 0,
        }
    }

    /// Prepare culling data before graph execution
    /// This must be called before run() with the instances to cull
    pub fn prepare(
        &mut self,
        device: &wgpu::Device,
        instances: &[InstanceAABB],
        frustum: &FrustumPlanes,
    ) {
        self.instance_count = instances.len() as u32;
        self.resources = Some(
            self.pipeline
                .create_culling_resources(device, instances, frustum),
        );
    }

    /// Get reference to culling resources (for accessing buffers)
    pub fn resources(&self) -> Option<&CullingResources> {
        self.resources.as_ref()
    }
}

impl RenderNode for CullingNode {
    fn name(&self) -> &str {
        &self.name
    }

    fn run(&mut self, ctx: &mut GraphContext) -> anyhow::Result<()> {
        let encoder = ctx
            .encoder
            .as_deref_mut()
            .context("CullingNode requires encoder")?;

        let resources = self
            .resources
            .as_ref()
            .context("CullingNode::prepare() must be called before run()")?;

        // Reset count buffer to 0
        encoder.clear_buffer(&resources.count_buffer, 0, None);

        // Execute compute culling
        self.pipeline
            .execute(encoder, &resources.bind_group, self.instance_count);

        // Note: Resources remain owned by CullingNode for lifetime management
        // Downstream nodes can access buffers via resources() method or
        // by implementing a resource sharing mechanism in GraphContext

        Ok(())
    }
}
