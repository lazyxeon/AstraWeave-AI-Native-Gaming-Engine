//! Graph visualization module
//!
//! Provides node graph editor widgets for visual scripting, behavior trees, and data flow visualization.

pub mod force_directed;
pub mod node_graph;

pub use force_directed::{ForceDirectedLayout, ForceDirectedParams};
pub use node_graph::{GraphEdge, GraphNode, NodeGraph, NodeId, Port, PortType};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_graph_creation() {
        let graph = NodeGraph::new();
        assert_eq!(graph.nodes().len(), 0);
        assert_eq!(graph.edges().len(), 0);
    }

    #[test]
    fn test_graph_node_creation() {
        let node = GraphNode::new(1, "Test Node");
        assert_eq!(node.id(), 1);
        assert_eq!(node.label(), "Test Node");
        assert_eq!(node.inputs().len(), 0);
        assert_eq!(node.outputs().len(), 0);
    }

    #[test]
    fn test_graph_edge_creation() {
        let edge = GraphEdge::new(1, 0, 2, 0);
        assert_eq!(edge.source_node(), 1);
        assert_eq!(edge.source_port(), 0);
        assert_eq!(edge.target_node(), 2);
        assert_eq!(edge.target_port(), 0);
    }
}
