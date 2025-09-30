// Minimal headless smoke test for the render graph scaffolding
use astraweave_render::graph::{GraphContext, RenderGraph, RenderNode};
use astraweave_render::graph::{ClearNode, RendererMainNode};

struct CountNode<'a> { name: &'static str, counter: &'a mut usize }
impl<'a> RenderNode for CountNode<'a> {
    fn name(&self) -> &str { self.name }
    fn run(&mut self, _ctx: &mut GraphContext) -> anyhow::Result<()> {
        *self.counter += 1;
        Ok(())
    }
}

#[test]
fn graph_executes_all_nodes() {
    let mut count = 0usize;
    let mut g = RenderGraph::new();
    g.add_node(CountNode { name: "shadow", counter: &mut count });
    g.add_node(CountNode { name: "main", counter: &mut count });
    g.add_node(CountNode { name: "post", counter: &mut count });

    let mut dummy = 0u32;
    let mut ctx = GraphContext::new(&mut dummy);
    g.execute(&mut ctx).expect("graph execution");
    assert_eq!(count, 3);
}

#[test]
fn adapter_nodes_compile() {
    let mut g = RenderGraph::new();
    g.add_node(ClearNode::new("clear", "surface", wgpu::Color::TRANSPARENT));
    g.add_node(RendererMainNode::new("main", "surface"));
    // do not execute (requires GPU). Just ensure GraphContext and node boxing type-checks.
    let mut dummy = 0usize;
    let mut ctx = GraphContext::new(&mut dummy);
    let _ = g.execute(&mut ctx).err();
}
