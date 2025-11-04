use astract::graph::{ForceDirectedParams, GraphNode, NodeGraph, Port, PortType};
use egui::Ui;

/// Panel demonstrating graph visualization widgets for visual scripting and behavior trees
pub struct GraphPanel {
    behavior_tree_graph: NodeGraph,
    shader_graph: NodeGraph,
    dialogue_graph: NodeGraph,

    initialized: bool,
}

impl Default for GraphPanel {
    fn default() -> Self {
        Self::new()
    }
}

impl GraphPanel {
    pub fn new() -> Self {
        Self {
            behavior_tree_graph: NodeGraph::new(),
            shader_graph: NodeGraph::new(),
            dialogue_graph: NodeGraph::new(),
            initialized: false,
        }
    }

    /// Initialize example graphs with nodes and edges
    fn init(&mut self) {
        if self.initialized {
            return;
        }

        // --- Behavior Tree Graph ---
        // AI behavior: Patrol → Detect Enemy → Attack
        let mut root = GraphNode::new(1, "Root").with_position(50.0, 50.0);
        root.add_output(Port::new(0, "Out", PortType::Exec));

        let mut selector = GraphNode::new(2, "Selector").with_position(200.0, 50.0);
        selector.add_input(Port::new(0, "In", PortType::Exec));
        selector.add_output(Port::new(1, "Option A", PortType::Exec));
        selector.add_output(Port::new(2, "Option B", PortType::Exec));

        let mut patrol = GraphNode::new(3, "Patrol").with_position(350.0, 20.0);
        patrol.add_input(Port::new(0, "In", PortType::Exec));
        patrol.add_output(Port::new(1, "Complete", PortType::Bool));

        let mut attack_seq = GraphNode::new(4, "Attack Sequence").with_position(350.0, 100.0);
        attack_seq.add_input(Port::new(0, "In", PortType::Exec));
        attack_seq.add_output(Port::new(1, "Target", PortType::Object));

        let mut detect = GraphNode::new(5, "Detect Enemy").with_position(500.0, 70.0);
        detect.add_input(Port::new(0, "Target", PortType::Object));
        detect.add_output(Port::new(1, "Found", PortType::Bool));

        self.behavior_tree_graph.add_node(root);
        self.behavior_tree_graph.add_node(selector);
        self.behavior_tree_graph.add_node(patrol);
        self.behavior_tree_graph.add_node(attack_seq);
        self.behavior_tree_graph.add_node(detect);

        // Connect behavior tree
        self.behavior_tree_graph.add_edge(1, 0, 2, 0); // Root → Selector
        self.behavior_tree_graph.add_edge(2, 1, 3, 0); // Selector → Patrol
        self.behavior_tree_graph.add_edge(2, 2, 4, 0); // Selector → Attack
        self.behavior_tree_graph.add_edge(4, 1, 5, 0); // Attack → Detect

        // --- Shader Graph ---
        // Texture → Color Adjust → Output
        let mut texture_in = GraphNode::new(11, "Texture Input").with_position(50.0, 50.0);
        texture_in.add_output(Port::new(0, "Color", PortType::Object));
        texture_in.add_output(Port::new(1, "UV", PortType::Number));

        let mut multiply = GraphNode::new(12, "Multiply").with_position(200.0, 50.0);
        multiply.add_input(Port::new(0, "A", PortType::Number));
        multiply.add_input(Port::new(1, "B", PortType::Number));
        multiply.add_output(Port::new(2, "Result", PortType::Number));

        let mut color_adjust = GraphNode::new(13, "Color Adjust").with_position(350.0, 50.0);
        color_adjust.add_input(Port::new(0, "Color", PortType::Object));
        color_adjust.add_input(Port::new(1, "Brightness", PortType::Number));
        color_adjust.add_output(Port::new(2, "Out", PortType::Object));

        let mut output = GraphNode::new(14, "Material Output").with_position(500.0, 50.0);
        output.add_input(Port::new(0, "Base Color", PortType::Object));

        self.shader_graph.add_node(texture_in);
        self.shader_graph.add_node(multiply);
        self.shader_graph.add_node(color_adjust);
        self.shader_graph.add_node(output);

        // Connect shader graph
        self.shader_graph.add_edge(11, 0, 13, 0); // Texture → Color Adjust (Color)
        self.shader_graph.add_edge(11, 1, 12, 0); // Texture → Multiply (UV)
        self.shader_graph.add_edge(12, 2, 13, 1); // Multiply → Color Adjust (Brightness)
        self.shader_graph.add_edge(13, 2, 14, 0); // Color Adjust → Output

        // --- Dialogue Graph ---
        // Branching conversation with choices
        let mut start_dlg = GraphNode::new(21, "Start").with_position(50.0, 50.0);
        start_dlg.add_output(Port::new(0, "Next", PortType::Exec));

        let mut greeting = GraphNode::new(22, "Greeting").with_position(200.0, 50.0);
        greeting.add_input(Port::new(0, "In", PortType::Exec));
        greeting.add_output(Port::new(1, "Option A", PortType::String));
        greeting.add_output(Port::new(2, "Option B", PortType::String));

        let mut friendly = GraphNode::new(23, "Friendly Response").with_position(350.0, 20.0);
        friendly.add_input(Port::new(0, "Choice", PortType::String));
        friendly.add_output(Port::new(1, "Next", PortType::Exec));

        let mut hostile = GraphNode::new(24, "Hostile Response").with_position(350.0, 100.0);
        hostile.add_input(Port::new(0, "Choice", PortType::String));
        hostile.add_output(Port::new(1, "Next", PortType::Exec));

        let mut end_dlg = GraphNode::new(25, "End").with_position(500.0, 60.0);
        end_dlg.add_input(Port::new(0, "In", PortType::Exec));

        self.dialogue_graph.add_node(start_dlg);
        self.dialogue_graph.add_node(greeting);
        self.dialogue_graph.add_node(friendly);
        self.dialogue_graph.add_node(hostile);
        self.dialogue_graph.add_node(end_dlg);

        // Connect dialogue graph
        self.dialogue_graph.add_edge(21, 0, 22, 0); // Start → Greeting
        self.dialogue_graph.add_edge(22, 1, 23, 0); // Greeting → Friendly (Option A)
        self.dialogue_graph.add_edge(22, 2, 24, 0); // Greeting → Hostile (Option B)
        self.dialogue_graph.add_edge(23, 1, 25, 0); // Friendly → End
        self.dialogue_graph.add_edge(24, 1, 25, 0); // Hostile → End

        self.initialized = true;
    }

    pub fn show(&mut self, ui: &mut Ui) {
        self.init(); // Lazy init

        ui.heading("Graph Visualization");
        ui.label("Node graph editor widgets for visual scripting, behavior trees, and data flow visualization.");
        ui.separator();

        ui.collapsing("Behavior Tree (AI Logic)", |ui| {
            ui.label("AI behavior: Patrol → Detect Enemy → Attack");

            ui.horizontal(|ui| {
                if ui.button("🔄 Auto-Layout").clicked() {
                    self.behavior_tree_graph.auto_layout();
                }
                if ui.button("🎨 Custom Layout").clicked() {
                    let params = ForceDirectedParams {
                        k: 150.0, // Wider spacing
                        max_iterations: 300,
                        ..Default::default()
                    };
                    self.behavior_tree_graph.auto_layout_with_params(params);
                }
                if ui.button("🔙 Reset").clicked() {
                    self.behavior_tree_graph.clear();
                    self.initialized = false;
                    self.init();
                }
            });

            ui.add_space(8.0);
            self.behavior_tree_graph.show(ui);

            ui.label("Port Types:");
            ui.horizontal(|ui| {
                ui.label("⚪ Exec");
                ui.label("🔴 Bool");
                ui.label("🟢 Number");
                ui.label("🔵 String");
                ui.label("🟡 Object");
            });
        });

        ui.add_space(16.0);

        ui.collapsing("Shader Graph (Material Nodes)", |ui| {
            ui.label("Material nodes: Texture → Color Adjust → Output");

            ui.horizontal(|ui| {
                if ui.button("🔄 Auto-Layout").clicked() {
                    self.shader_graph.auto_layout();
                }
                if ui.button("🔙 Reset").clicked() {
                    self.shader_graph.clear();
                    self.initialized = false;
                    self.init();
                }
            });

            ui.add_space(8.0);
            self.shader_graph.show(ui);
        });

        ui.add_space(16.0);

        ui.collapsing("Dialogue Graph (Branching Conversations)", |ui| {
            ui.label("NPC dialogue: Start → Greeting → [Friendly/Hostile] → End");

            ui.horizontal(|ui| {
                if ui.button("🔄 Auto-Layout").clicked() {
                    self.dialogue_graph.auto_layout();
                }
                if ui.button("🔙 Reset").clicked() {
                    self.dialogue_graph.clear();
                    self.initialized = false;
                    self.init();
                }
            });

            ui.add_space(8.0);
            self.dialogue_graph.show(ui);
        });

        ui.add_space(16.0);

        ui.collapsing("About Graph Widgets", |ui| {
            ui.label("Features:");
            ui.label("• Node graph editor with drag-and-drop nodes");
            ui.label("• Bezier curve connections between ports");
            ui.label("• Type-colored ports (Exec, Bool, Number, String, Object)");
            ui.label("• Force-directed auto-layout (spring forces + repulsion)");
            ui.label("• Pan/zoom support");
            ui.label("• Click detection for node selection");
            ui.separator();
            ui.label("Use Cases:");
            ui.label("• Visual scripting (behavior trees, state machines)");
            ui.label("• Shader graph editors");
            ui.label("• Dialogue systems with branching paths");
            ui.label("• AI planning visualization");
            ui.label("• Data flow graphs (signal processing, animation blending)");
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_graph_panel_creation() {
        let panel = GraphPanel::new();
        assert!(!panel.initialized);
    }

    #[test]
    fn test_graph_panel_initialization() {
        let mut panel = GraphPanel::new();
        panel.init();
        assert!(panel.initialized);

        // Behavior tree should have 5 nodes, 4 edges
        assert_eq!(panel.behavior_tree_graph.nodes().len(), 5);
        assert_eq!(panel.behavior_tree_graph.edges().len(), 4);

        // Shader graph should have 4 nodes, 4 edges
        assert_eq!(panel.shader_graph.nodes().len(), 4);
        assert_eq!(panel.shader_graph.edges().len(), 4);

        // Dialogue graph should have 5 nodes, 5 edges
        assert_eq!(panel.dialogue_graph.nodes().len(), 5);
        assert_eq!(panel.dialogue_graph.edges().len(), 5);
    }

    #[test]
    fn test_graph_panel_double_init_safe() {
        let mut panel = GraphPanel::new();
        panel.init();
        let nodes_count = panel.behavior_tree_graph.nodes().len();

        // Calling init again should not duplicate nodes
        panel.init();
        assert_eq!(panel.behavior_tree_graph.nodes().len(), nodes_count);
    }

    #[test]
    fn test_graph_panel_reset() {
        let mut panel = GraphPanel::new();
        panel.init();

        // Clear and re-init
        panel.behavior_tree_graph.clear();
        panel.initialized = false;
        panel.init();

        // Should still have 5 nodes
        assert_eq!(panel.behavior_tree_graph.nodes().len(), 5);
    }
}
