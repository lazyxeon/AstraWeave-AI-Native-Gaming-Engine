//! Graphs showcase tab - demonstrates NodeGraph widget with different graph types

use astract::graph::{GraphNode, NodeGraph, Port, PortType};
use astract::prelude::egui::*;

pub struct GraphsTab {
    // Different graph examples
    behavior_tree: NodeGraph,
    shader_graph: NodeGraph,
    dialogue_graph: NodeGraph,

    // UI state
    selected_graph: GraphType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GraphType {
    BehaviorTree,
    ShaderGraph,
    Dialogue,
}

impl Default for GraphsTab {
    fn default() -> Self {
        let mut tab = Self {
            behavior_tree: NodeGraph::new(),
            shader_graph: NodeGraph::new(),
            dialogue_graph: NodeGraph::new(),
            selected_graph: GraphType::BehaviorTree,
        };

        // Initialize graphs
        tab.setup_behavior_tree();
        tab.setup_shader_graph();
        tab.setup_dialogue_graph();

        tab
    }
}

impl GraphsTab {
    pub fn show(&mut self, ui: &mut Ui) {
        ui.heading("🕸️ Node Graphs Showcase");
        ui.add_space(10.0);

        // Graph type selector
        ui.horizontal(|ui| {
            ui.label("Graph Type:");
            ui.selectable_value(
                &mut self.selected_graph,
                GraphType::BehaviorTree,
                "🌳 Behavior Tree",
            );
            ui.selectable_value(
                &mut self.selected_graph,
                GraphType::ShaderGraph,
                "🎨 Shader Graph",
            );
            ui.selectable_value(
                &mut self.selected_graph,
                GraphType::Dialogue,
                "💬 Dialogue System",
            );
        });

        ui.add_space(10.0);

        // Description based on selected graph
        match self.selected_graph {
            GraphType::BehaviorTree => {
                ui.label("AI decision tree with execution flow");
            }
            GraphType::ShaderGraph => {
                ui.label("Visual shader programming with data flow");
            }
            GraphType::Dialogue => {
                ui.label("Branching conversation system");
            }
        }

        ui.add_space(10.0);

        // Show selected graph
        ui.group(|ui| {
            let graph = match self.selected_graph {
                GraphType::BehaviorTree => &mut self.behavior_tree,
                GraphType::ShaderGraph => &mut self.shader_graph,
                GraphType::Dialogue => &mut self.dialogue_graph,
            };

            graph.show(ui);
        });

        ui.add_space(20.0);

        // Code example
        ui.collapsing("📝 Code Example", |ui| {
            ui.label("Creating a simple node graph:");
            ui.code(
                r#"use astract::graph::{NodeGraph, GraphNode, Port, PortType};

let mut graph = NodeGraph::new();

// Create nodes
let mut start_node = GraphNode::new(1, "Start");
start_node.add_output(Port::new(0, "Out", PortType::Exec));
let start_id = graph.add_node(start_node);

let mut action_node = GraphNode::new(2, "Action");
action_node.add_input(Port::new(0, "In", PortType::Exec));
action_node.add_output(Port::new(1, "Out", PortType::Exec));
let action_id = graph.add_node(action_node);

// Connect nodes
graph.add_edge(start_id, 0, action_id, 0);

// Show in UI
graph.show(ui);"#,
            );
        });
    }

    fn setup_behavior_tree(&mut self) {
        // Root selector node
        let mut root = GraphNode::new(1, "Root Selector");
        root.add_input(Port::new(0, "In", PortType::Exec));
        root.add_output(Port::new(1, "Out1", PortType::Exec));
        root.add_output(Port::new(2, "Out2", PortType::Exec));
        let root = root.with_position(100.0, 50.0);
        let root_id = self.behavior_tree.add_node(root);

        // Combat sequence
        let mut combat = GraphNode::new(2, "Combat Sequence");
        combat.add_input(Port::new(0, "In", PortType::Exec));
        combat.add_output(Port::new(1, "Out", PortType::Exec));
        let combat = combat.with_position(50.0, 150.0);
        let combat_id = self.behavior_tree.add_node(combat);

        // Attack action
        let mut attack = GraphNode::new(3, "Attack");
        attack.add_input(Port::new(0, "In", PortType::Exec));
        let attack = attack.with_position(50.0, 250.0);
        let attack_id = self.behavior_tree.add_node(attack);

        // Patrol sequence
        let mut patrol = GraphNode::new(4, "Patrol Sequence");
        patrol.add_input(Port::new(0, "In", PortType::Exec));
        patrol.add_output(Port::new(1, "Out", PortType::Exec));
        let patrol = patrol.with_position(250.0, 150.0);
        let patrol_id = self.behavior_tree.add_node(patrol);

        // Move action
        let mut move_node = GraphNode::new(5, "Move");
        move_node.add_input(Port::new(0, "In", PortType::Exec));
        let move_node = move_node.with_position(250.0, 250.0);
        let move_id = self.behavior_tree.add_node(move_node);

        // Connect edges
        self.behavior_tree.add_edge(root_id, 1, combat_id, 0);
        self.behavior_tree.add_edge(root_id, 2, patrol_id, 0);
        self.behavior_tree.add_edge(combat_id, 1, attack_id, 0);
        self.behavior_tree.add_edge(patrol_id, 1, move_id, 0);
    }

    fn setup_shader_graph(&mut self) {
        // Texture input
        let mut texture = GraphNode::new(1, "Texture2D");
        texture.add_output(Port::new(0, "RGB", PortType::Object));
        let texture = texture.with_position(50.0, 100.0);
        let texture_id = self.shader_graph.add_node(texture);

        // Color multiply
        let mut multiply = GraphNode::new(2, "Multiply");
        multiply.add_input(Port::new(0, "A", PortType::Object));
        multiply.add_input(Port::new(1, "B", PortType::Number));
        multiply.add_output(Port::new(2, "Result", PortType::Object));
        let multiply = multiply.with_position(250.0, 100.0);
        let multiply_id = self.shader_graph.add_node(multiply);

        // Brightness value
        let mut brightness = GraphNode::new(3, "Brightness");
        brightness.add_output(Port::new(0, "Value", PortType::Number));
        let brightness = brightness.with_position(50.0, 200.0);
        let brightness_id = self.shader_graph.add_node(brightness);

        // Output
        let mut output = GraphNode::new(4, "Final Color");
        output.add_input(Port::new(0, "Color", PortType::Object));
        let output = output.with_position(450.0, 100.0);
        let output_id = self.shader_graph.add_node(output);

        // Connect
        self.shader_graph.add_edge(texture_id, 0, multiply_id, 0);
        self.shader_graph.add_edge(brightness_id, 0, multiply_id, 1);
        self.shader_graph.add_edge(multiply_id, 2, output_id, 0);
    }

    fn setup_dialogue_graph(&mut self) {
        // Start node
        let mut start = GraphNode::new(1, "Quest Start");
        start.add_output(Port::new(0, "Next", PortType::Exec));
        let start = start.with_position(100.0, 50.0);
        let start_id = self.dialogue_graph.add_node(start);

        // Choice node
        let mut choice = GraphNode::new(2, "Accept quest?");
        choice.add_input(Port::new(0, "In", PortType::Exec));
        choice.add_output(Port::new(1, "Yes", PortType::Exec));
        choice.add_output(Port::new(2, "No", PortType::Exec));
        let choice = choice.with_position(100.0, 150.0);
        let choice_id = self.dialogue_graph.add_node(choice);

        // Accept path
        let mut accept = GraphNode::new(3, "Thank you!");
        accept.add_input(Port::new(0, "In", PortType::Exec));
        accept.add_output(Port::new(1, "Next", PortType::Exec));
        let accept = accept.with_position(50.0, 250.0);
        let accept_id = self.dialogue_graph.add_node(accept);

        // Decline path
        let mut decline = GraphNode::new(4, "Maybe later");
        decline.add_input(Port::new(0, "In", PortType::Exec));
        let decline = decline.with_position(250.0, 250.0);
        let decline_id = self.dialogue_graph.add_node(decline);

        // Quest start
        let mut quest = GraphNode::new(5, "Start Quest");
        quest.add_input(Port::new(0, "In", PortType::Exec));
        let quest = quest.with_position(50.0, 350.0);
        let quest_id = self.dialogue_graph.add_node(quest);

        // Connect
        self.dialogue_graph.add_edge(start_id, 0, choice_id, 0);
        self.dialogue_graph.add_edge(choice_id, 1, accept_id, 0);
        self.dialogue_graph.add_edge(choice_id, 2, decline_id, 0);
        self.dialogue_graph.add_edge(accept_id, 1, quest_id, 0);
    }
}
