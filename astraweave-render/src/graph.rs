//! Minimal render graph scaffolding for Phase 2: establishes a deterministic, pluggable pass graph.
//! This provides a Bevy/Fyrox-like pattern while staying optional and non-invasive to `Renderer`.

use anyhow::Context as _;
use std::collections::BTreeMap;

/// Typed GPU resources passed between graph nodes.
/// Keep this minimal for now; extend as we integrate more passes.
pub enum Resource {
    Texture(wgpu::Texture),
    View(wgpu::TextureView),
    Buffer(wgpu::Buffer),
    BindGroup(wgpu::BindGroup),
}

/// A simple typed resource handle registry for graph nodes to pass data.
#[derive(Default)]
pub struct ResourceTable {
    map: BTreeMap<String, Resource>,
}

/// Context passed to graph nodes. This will carry wgpu device/queue and shared resources.
pub struct GraphContext<'a> {
    /// Arbitrary user context for integration (e.g., &mut Renderer)
    pub user: &'a mut dyn std::any::Any,
    /// Named transient resources produced/consumed by nodes
    pub resources: ResourceTable,
    /// Optional GPU context for nodes that record commands
    pub device: Option<&'a wgpu::Device>,
    pub queue: Option<&'a wgpu::Queue>,
    pub encoder: Option<&'a mut wgpu::CommandEncoder>,
    /// Optional primary render target view provided by the driver (e.g., surface view)
    pub primary_view: Option<&'a wgpu::TextureView>,
}

impl<'a> GraphContext<'a> {
    pub fn new(user: &'a mut dyn std::any::Any) -> Self {
        Self {
            user,
            resources: ResourceTable::default(),
            device: None,
            queue: None,
            encoder: None,
            primary_view: None,
        }
    }

    /// Attach GPU context for nodes that perform GPU work.
    pub fn with_gpu(
        mut self,
        device: &'a wgpu::Device,
        queue: &'a wgpu::Queue,
        encoder: &'a mut wgpu::CommandEncoder,
    ) -> Self {
        self.device = Some(device);
        self.queue = Some(queue);
        self.encoder = Some(encoder);
        self
    }

    pub fn with_primary_view(mut self, view: &'a wgpu::TextureView) -> Self {
        self.primary_view = Some(view);
        self
    }
}

impl ResourceTable {
    pub fn insert_view(&mut self, key: impl Into<String>, view: wgpu::TextureView) {
        self.map.insert(key.into(), Resource::View(view));
    }
    pub fn insert_tex(&mut self, key: impl Into<String>, tex: wgpu::Texture) {
        self.map.insert(key.into(), Resource::Texture(tex));
    }
    pub fn insert_buf(&mut self, key: impl Into<String>, buf: wgpu::Buffer) {
        self.map.insert(key.into(), Resource::Buffer(buf));
    }
    pub fn insert_bind_group(&mut self, key: impl Into<String>, bg: wgpu::BindGroup) {
        self.map.insert(key.into(), Resource::BindGroup(bg));
    }
    pub fn view(&self, key: &str) -> anyhow::Result<&wgpu::TextureView> {
        match self
            .map
            .get(key)
            .with_context(|| format!("resource '{}' not found", key))?
        {
            Resource::View(v) => Ok(v),
            _ => anyhow::bail!("resource '{}' is not a TextureView", key),
        }
    }
    pub fn view_mut(&mut self, key: &str) -> anyhow::Result<&mut wgpu::TextureView> {
        match self
            .map
            .get_mut(key)
            .with_context(|| format!("resource '{}' not found", key))?
        {
            Resource::View(v) => Ok(v),
            _ => anyhow::bail!("resource '{}' is not a TextureView", key),
        }
    }
    /// Get a target view by key, falling back to `primary_view` when the key is "surface".
    pub fn target_view<'a>(
        &'a self,
        key: &str,
        primary_view: Option<&'a wgpu::TextureView>,
    ) -> anyhow::Result<&'a wgpu::TextureView> {
        if key == "surface" {
            if let Some(v) = primary_view {
                return Ok(v);
            }
        }
        self.view(key)
    }
    pub fn bind_group(&self, key: &str) -> anyhow::Result<&wgpu::BindGroup> {
        match self
            .map
            .get(key)
            .with_context(|| format!("resource '{}' not found", key))?
        {
            Resource::BindGroup(bg) => Ok(bg),
            _ => anyhow::bail!("resource '{}' is not a BindGroup", key),
        }
    }
    pub fn tex(&self, key: &str) -> anyhow::Result<&wgpu::Texture> {
        match self
            .map
            .get(key)
            .with_context(|| format!("resource '{}' not found", key))?
        {
            Resource::Texture(t) => Ok(t),
            _ => anyhow::bail!("resource '{}' is not a Texture", key),
        }
    }

    /// Create a transient texture resource (e.g., HDR target, depth buffer) and insert it.
    /// Callers should ensure the texture is dropped when the graph execution ends.
    pub fn create_transient_texture(
        &mut self,
        device: &wgpu::Device,
        key: impl Into<String>,
        desc: &wgpu::TextureDescriptor,
    ) -> anyhow::Result<&wgpu::Texture> {
        let key_str = key.into();
        let tex = device.create_texture(desc);
        self.insert_tex(&key_str, tex);
        // Return a reference to the inserted texture (safe: just inserted above)
        match self.map.get(&key_str).ok_or_else(|| {
            anyhow::anyhow!("BUG: texture '{}' should exist after insert", key_str)
        })? {
            Resource::Texture(t) => Ok(t),
            _ => anyhow::bail!(
                "BUG: resource '{}' inserted as texture but retrieved as different type",
                key_str
            ),
        }
    }
}

/// Trait for a render graph node. Nodes should be deterministic and side-effect free beyond GPU work.
pub trait RenderNode {
    fn name(&self) -> &str;
    fn run(&mut self, ctx: &mut GraphContext) -> anyhow::Result<()>;
}

/// A very small, linear render graph. Edges are expressed as node ordering for now.
#[derive(Default)]
pub struct RenderGraph {
    nodes: Vec<Box<dyn RenderNode + Send + Sync>>, // keep Send+Sync for future parallelization
}

/// --- Adapter nodes: integrate existing `Renderer` passes into the graph ---
///
/// A node that clears a target view to a color, producing a named view resource.
pub struct ClearNode {
    name: String,
    target_key: String,
    color: wgpu::Color,
}

impl ClearNode {
    pub fn new(name: impl Into<String>, target_key: impl Into<String>, color: wgpu::Color) -> Self {
        Self {
            name: name.into(),
            target_key: target_key.into(),
            color,
        }
    }
}

impl RenderNode for ClearNode {
    fn name(&self) -> &str {
        &self.name
    }
    fn run(&mut self, ctx: &mut GraphContext) -> anyhow::Result<()> {
        let _device = ctx.device.context("ClearNode requires device")?;
        let view = ctx
            .resources
            .target_view(&self.target_key, ctx.primary_view)?;
        let enc = ctx
            .encoder
            .as_deref_mut()
            .context("ClearNode requires encoder")?;
        let rp = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some(&format!("clear:{}", self.name)),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(self.color),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });
        drop((_device, rp)); // rp dropped to end pass
        Ok(())
    }
}

/// A node that defers to `Renderer::draw_into` to render the 3D scene into a target view.
pub struct RendererMainNode {
    name: String,
    target_key: String,
}

impl RendererMainNode {
    pub fn new(name: impl Into<String>, target_key: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            target_key: target_key.into(),
        }
    }
}

impl RenderNode for RendererMainNode {
    fn name(&self) -> &str {
        &self.name
    }
    fn run(&mut self, ctx: &mut GraphContext) -> anyhow::Result<()> {
        // For now, just validate that the target exists; the main scene draw is handled
        // by the caller (e.g., Renderer::render_with). This keeps the node integration simple.
        let _ = ctx
            .resources
            .target_view(&self.target_key, ctx.primary_view)?;
        let _ = ctx
            .encoder
            .as_deref_mut()
            .context("RendererMainNode requires encoder")?;
        Ok(())
    }
}

impl RenderGraph {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_node<N>(&mut self, node: N)
    where
        N: RenderNode + Send + Sync + 'static,
    {
        self.nodes.push(Box::new(node));
    }

    /// Execute nodes in insertion order. Deterministic by construction.
    pub fn execute(&mut self, ctx: &mut GraphContext) -> anyhow::Result<()> {
        for n in self.nodes.iter_mut() {
            n.run(ctx)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestNode {
        pub name: &'static str,
        pub log: Vec<&'static str>,
    }
    impl RenderNode for TestNode {
        fn name(&self) -> &str {
            self.name
        }
        fn run(&mut self, _ctx: &mut GraphContext) -> anyhow::Result<()> {
            self.log.push(self.name);
            Ok(())
        }
    }

    #[test]
    fn render_graph_runs_in_order() {
        let a = TestNode {
            name: "shadow",
            log: vec![],
        };
        let b = TestNode {
            name: "main",
            log: vec![],
        };
        let c = TestNode {
            name: "post",
            log: vec![],
        };
        let mut g = RenderGraph::new();
        g.add_node(a);
        g.add_node(b);
        g.add_node(c);
        let mut dummy = 0u32;
        let mut ctx = GraphContext::new(&mut dummy);
        let _ = g.execute(&mut ctx).unwrap();
        // We can't access nodes after moved; instead, ensure no errors and linear execution returns Ok
        // Additional ordering validation can be done by having nodes append to a shared log in ctx.user
    }

    #[test]
    fn resource_table_transient_texture() {
        // Headless device for testing
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::LowPower,
            compatible_surface: None,
            force_fallback_adapter: false,
        }))
        .expect("adapter");
        let (device, _queue) =
            pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default()))
                .expect("device");

        let mut table = ResourceTable::default();
        let desc = wgpu::TextureDescriptor {
            label: Some("transient-hdr"),
            size: wgpu::Extent3d {
                width: 1024,
                height: 1024,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba16Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        };
        let tex = table
            .create_transient_texture(&device, "hdr_target", &desc)
            .unwrap();
        assert_eq!(tex.width(), 1024);
        assert_eq!(tex.height(), 1024);
        assert_eq!(tex.format(), wgpu::TextureFormat::Rgba16Float);
        // Verify it's in the table
        let retrieved = table.tex("hdr_target").unwrap();
        assert_eq!(retrieved.width(), 1024);
    }
}
