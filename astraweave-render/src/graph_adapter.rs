use crate::graph::{GraphContext, RenderGraph};

/// Drive the provided `RenderGraph` within `Renderer::render_with`, avoiding private field access.
pub fn run_graph_on_renderer(
    renderer: &mut crate::renderer::Renderer,
    graph: &mut RenderGraph,
) -> anyhow::Result<()> {
    // We'll pass a temporary dummy Any as user context to avoid borrowing renderer in the closure.
    struct Dummy;
    let mut dummy: Dummy = Dummy;
    let mut exec_res: Option<anyhow::Result<()>> = None;
    renderer.render_with(|surface_view, enc, device, queue, _size| {
        let mut ctx = GraphContext::new(&mut dummy as &mut dyn std::any::Any)
            .with_gpu(device, queue, enc)
            .with_primary_view(surface_view);
        exec_res = Some(graph.execute(&mut ctx));
    })?;
    exec_res.unwrap_or_else(|| Ok(()))
}
