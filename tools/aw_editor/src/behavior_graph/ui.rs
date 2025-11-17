use astraweave_behavior::BehaviorNode;
use egui::{self, ComboBox, Slider};

use super::document::{
    BehaviorGraphDocument, BehaviorGraphDocumentError, BehaviorGraphNodeKind, DecoratorKind,
    DecoratorNode, NodeId,
};

pub struct BehaviorGraphEditorUi {
    selected_node: Option<NodeId>,
    new_node_kind: NodeTemplate,
    new_node_label: String,
    path_input: String,
    status_line: Option<String>,
}

impl Default for BehaviorGraphEditorUi {
    fn default() -> Self {
        Self {
            selected_node: None,
            new_node_kind: NodeTemplate::Action,
            new_node_label: "new_action".into(),
            path_input: "content/sample.behavior.ron".into(),
            status_line: None,
        }
    }
}

impl BehaviorGraphEditorUi {
    pub fn show<F>(&mut self, ui: &mut egui::Ui, doc: &mut BehaviorGraphDocument, mut push_log: F)
    where
        F: FnMut(String),
    {
        self.ensure_selection(doc);

        self.draw_file_toolbar(ui, doc, &mut push_log);
        if let Some(status) = &self.status_line {
            ui.label(status);
        }

        ui.separator();
        self.draw_palette(ui, doc);

        ui.separator();
        ui.columns(2, |columns| {
            self.draw_node_list(&mut columns[0], doc);
            self.draw_node_details(&mut columns[1], doc);
        });
    }

    fn ensure_selection(&mut self, doc: &BehaviorGraphDocument) {
        if self.selected_node.and_then(|id| doc.node(id)).is_none() {
            self.selected_node = Some(doc.root_id());
        }
    }

    fn draw_file_toolbar<F>(
        &mut self,
        ui: &mut egui::Ui,
        doc: &mut BehaviorGraphDocument,
        push_log: &mut F,
    ) where
        F: FnMut(String),
    {
        ui.horizontal(|ui| {
            ui.label("File");
            ui.text_edit_singleline(&mut self.path_input);
            if ui.button("Save").clicked() {
                if self.path_input.trim().is_empty() {
                    self.status_line = Some("⚠️ Provide a file path before saving".into());
                } else {
                    match doc.save_to_path(&self.path_input) {
                        Ok(()) => {
                            push_log(format!("💾 Saved behavior graph to {}", self.path_input));
                            self.status_line = Some("Saved graph".into());
                        }
                        Err(err) => self.status_line = Some(format!("❌ Save failed: {err}")),
                    }
                }
            }
            if ui.button("Load").clicked() {
                match BehaviorGraphDocument::load_from_path(&self.path_input) {
                    Ok(loaded) => {
                        *doc = loaded;
                        self.selected_node = Some(doc.root_id());
                        push_log(format!("📂 Loaded behavior graph from {}", self.path_input));
                        self.status_line = Some("Loaded graph".into());
                    }
                    Err(err) => self.status_line = Some(format!("❌ Load failed: {err}")),
                }
            }
            if ui.button("Validate").clicked() {
                match doc.to_runtime() {
                    Ok(graph) => {
                        let node_count = count_runtime_nodes(&graph.root);
                        push_log(format!("✅ Behavior graph valid ({node_count} nodes)"));
                        self.status_line = Some(format!("Validated graph ({node_count} nodes)"));
                    }
                    Err(err) => self.status_line = Some(format!("❌ Validation failed: {err}")),
                }
            }
        });
    }

    fn draw_palette(&mut self, ui: &mut egui::Ui, doc: &mut BehaviorGraphDocument) {
        ui.horizontal(|ui| {
            ui.label("Palette");
            ComboBox::new("behavior_node_kind", "Node Type")
                .selected_text(self.new_node_kind.label())
                .show_ui(ui, |ui| {
                    for kind in NodeTemplate::ALL {
                        ui.selectable_value(&mut self.new_node_kind, *kind, kind.label());
                    }
                });
            ui.text_edit_singleline(&mut self.new_node_label);
            let can_add_child = self
                .selected_node
                .and_then(|id| doc.node(id))
                .map(|node| node.kind.supports_children())
                .unwrap_or(false);
            let add_child = ui.add_enabled(can_add_child, egui::Button::new("Add Child"));
            if add_child.clicked() {
                if let Err(err) = self.add_child_to_selection(doc) {
                    self.status_line = Some(format!("❌ {err}"));
                }
            }
        });
    }

    fn draw_node_list(&mut self, ui: &mut egui::Ui, doc: &BehaviorGraphDocument) {
        ui.heading("Nodes");
        egui::ScrollArea::vertical().show(ui, |ui| {
            for node in doc.nodes() {
                let title = if node.id == doc.root_id() {
                    format!("{} (root)", node.label)
                } else {
                    node.label.clone()
                };
                if ui
                    .selectable_label(self.selected_node == Some(node.id), title)
                    .clicked()
                {
                    self.selected_node = Some(node.id);
                }
            }
        });
    }

    fn draw_node_details(&mut self, ui: &mut egui::Ui, doc: &mut BehaviorGraphDocument) {
        ui.heading("Details");
        let Some(node_id) = self.selected_node else {
            ui.label("Select a node to edit its properties.");
            return;
        };
        let Some(snapshot) = doc.node(node_id).cloned() else {
            ui.label("Node missing from document");
            return;
        };

        let mut label_text = snapshot.label.clone();
        if ui.text_edit_singleline(&mut label_text).changed() {
            if let Some(node) = doc.node_mut(node_id) {
                node.label = label_text.clone();
            }
        }
        ui.label(snapshot.kind.display_name());

        match snapshot.kind {
            BehaviorGraphNodeKind::Action { mut name } => {
                if ui.text_edit_singleline(&mut name).changed() {
                    if let Some(node) = doc.node_mut(node_id) {
                        if let BehaviorGraphNodeKind::Action { name: current } = &mut node.kind {
                            *current = name;
                        }
                    }
                }
            }
            BehaviorGraphNodeKind::Condition { mut name } => {
                if ui.text_edit_singleline(&mut name).changed() {
                    if let Some(node) = doc.node_mut(node_id) {
                        if let BehaviorGraphNodeKind::Condition { name: current } = &mut node.kind {
                            *current = name;
                        }
                    }
                }
            }
            BehaviorGraphNodeKind::Sequence { ref children }
            | BehaviorGraphNodeKind::Selector { ref children } => {
                self.draw_children_list(ui, doc, children);
            }
            BehaviorGraphNodeKind::Parallel {
                ref children,
                success_threshold,
            } => {
                let child_count = children.len().max(1);
                let mut threshold = success_threshold.min(child_count);
                if ui
                    .add(Slider::new(&mut threshold, 0..=child_count).text("Success Threshold"))
                    .changed()
                {
                    if let Some(node) = doc.node_mut(node_id) {
                        if let BehaviorGraphNodeKind::Parallel {
                            success_threshold: current,
                            ..
                        } = &mut node.kind
                        {
                            *current = threshold;
                        }
                    }
                }
                self.draw_children_list(ui, doc, children);
            }
            BehaviorGraphNodeKind::Decorator(ref decorator) => {
                self.draw_decorator(ui, doc, node_id, decorator);
            }
        }

        ui.separator();
        ui.horizontal(|ui| {
            if ui.button("Set As Root").clicked() {
                if let Err(err) = doc.set_root(node_id) {
                    self.status_line = Some(format!("❌ {err}"));
                }
            }
            if ui.button("Delete").clicked() {
                if let Err(err) = doc.remove_node(node_id) {
                    self.status_line = Some(format!("❌ {err}"));
                } else {
                    self.selected_node = Some(doc.root_id());
                }
            }
        });
    }

    fn draw_children_list(
        &mut self,
        ui: &mut egui::Ui,
        doc: &mut BehaviorGraphDocument,
        children: &[NodeId],
    ) {
        ui.label("Children");
        for child_id in children {
            let label = doc
                .node(*child_id)
                .map(|node| node.label.clone())
                .unwrap_or_else(|| format!("Missing node #{}", child_id));
            ui.horizontal(|ui| {
                if ui
                    .selectable_label(self.selected_node == Some(*child_id), label)
                    .clicked()
                {
                    self.selected_node = Some(*child_id);
                }
                if ui.button("Remove").clicked() {
                    if let Err(err) = doc.remove_node(*child_id) {
                        self.status_line = Some(format!("❌ {err}"));
                    }
                }
            });
        }
    }

    fn draw_decorator(
        &mut self,
        ui: &mut egui::Ui,
        doc: &mut BehaviorGraphDocument,
        node_id: NodeId,
        decorator: &DecoratorNode,
    ) {
        let mut decorator_kind = decorator.decorator.clone();
        ComboBox::new("decorator_kind", "Decorator")
            .selected_text(format_decorator_label(&decorator_kind))
            .show_ui(ui, |ui| {
                for option in DecoratorKindOption::ALL {
                    ui.selectable_value(&mut decorator_kind, option.kind.clone(), option.label);
                }
            });

        match decorator_kind {
            DecoratorKind::Repeat(ref mut max) | DecoratorKind::Retry(ref mut max) => {
                let mut value = (*max).max(1);
                if ui
                    .add(Slider::new(&mut value, 1..=10).text("Iterations"))
                    .changed()
                {
                    *max = value;
                }
            }
            _ => {}
        }

        if let Some(node) = doc.node_mut(node_id) {
            if let BehaviorGraphNodeKind::Decorator(data) = &mut node.kind {
                data.decorator = decorator_kind.clone();
            }
        }

        if let Some(child_id) = decorator.child {
            ui.label("Child");
            if let Some(child) = doc.node(child_id) {
                if ui
                    .selectable_label(self.selected_node == Some(child_id), &child.label)
                    .clicked()
                {
                    self.selected_node = Some(child_id);
                }
            }
            if ui.button("Remove Child").clicked() {
                if let Err(err) = doc.remove_node(child_id) {
                    self.status_line = Some(format!("❌ {err}"));
                }
            }
        } else {
            ui.label("No child assigned. Use 'Add Child' to create one.");
        }
    }

    fn add_child_to_selection(
        &mut self,
        doc: &mut BehaviorGraphDocument,
    ) -> Result<(), BehaviorGraphDocumentError> {
        let Some(parent_id) = self.selected_node else {
            return Ok(());
        };
        let label = self
            .new_node_kind
            .default_label(&self.new_node_label, doc.nodes().len());
        let kind = self.new_node_kind.to_kind(&self.new_node_label);
        match doc.add_child_node(parent_id, label, kind) {
            Ok(id) => {
                self.selected_node = Some(id);
                Ok(())
            }
            Err(err) => Err(err),
        }
    }
}

fn count_runtime_nodes(node: &BehaviorNode) -> usize {
    match node {
        BehaviorNode::Action(_) | BehaviorNode::Condition(_) => 1,
        BehaviorNode::Sequence(children) | BehaviorNode::Selector(children) => {
            1 + children.iter().map(count_runtime_nodes).sum::<usize>()
        }
        BehaviorNode::Parallel(children, _) => {
            1 + children.iter().map(count_runtime_nodes).sum::<usize>()
        }
        BehaviorNode::Decorator(_, child) => 1 + count_runtime_nodes(child),
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum NodeTemplate {
    Action,
    Condition,
    Sequence,
    Selector,
    Parallel,
    DecoratorInverter,
    DecoratorSucceeder,
    DecoratorFailer,
    DecoratorRepeat,
    DecoratorRetry,
}

impl NodeTemplate {
    const ALL: &'static [NodeTemplate] = &[
        NodeTemplate::Action,
        NodeTemplate::Condition,
        NodeTemplate::Sequence,
        NodeTemplate::Selector,
        NodeTemplate::Parallel,
        NodeTemplate::DecoratorInverter,
        NodeTemplate::DecoratorSucceeder,
        NodeTemplate::DecoratorFailer,
        NodeTemplate::DecoratorRepeat,
        NodeTemplate::DecoratorRetry,
    ];

    fn label(&self) -> &'static str {
        match self {
            NodeTemplate::Action => "Action",
            NodeTemplate::Condition => "Condition",
            NodeTemplate::Sequence => "Sequence",
            NodeTemplate::Selector => "Selector",
            NodeTemplate::Parallel => "Parallel",
            NodeTemplate::DecoratorInverter => "Decorator • Inverter",
            NodeTemplate::DecoratorSucceeder => "Decorator • Succeeder",
            NodeTemplate::DecoratorFailer => "Decorator • Failer",
            NodeTemplate::DecoratorRepeat => "Decorator • Repeat",
            NodeTemplate::DecoratorRetry => "Decorator • Retry",
        }
    }

    fn default_label(&self, name: &str, count: usize) -> String {
        match self {
            NodeTemplate::Action => format!("Action: {}", fallback_name(name, "action")),
            NodeTemplate::Condition => format!("Condition: {}", fallback_name(name, "condition")),
            NodeTemplate::Sequence => format!("Sequence {}", count),
            NodeTemplate::Selector => format!("Selector {}", count),
            NodeTemplate::Parallel => format!("Parallel {}", count),
            NodeTemplate::DecoratorInverter
            | NodeTemplate::DecoratorSucceeder
            | NodeTemplate::DecoratorFailer
            | NodeTemplate::DecoratorRepeat
            | NodeTemplate::DecoratorRetry => format!("Decorator {}", count),
        }
    }

    fn to_kind(&self, input: &str) -> BehaviorGraphNodeKind {
        match self {
            NodeTemplate::Action => BehaviorGraphNodeKind::Action {
                name: fallback_name(input, "action"),
            },
            NodeTemplate::Condition => BehaviorGraphNodeKind::Condition {
                name: fallback_name(input, "condition"),
            },
            NodeTemplate::Sequence => BehaviorGraphNodeKind::Sequence {
                children: Vec::new(),
            },
            NodeTemplate::Selector => BehaviorGraphNodeKind::Selector {
                children: Vec::new(),
            },
            NodeTemplate::Parallel => BehaviorGraphNodeKind::Parallel {
                children: Vec::new(),
                success_threshold: 1,
            },
            NodeTemplate::DecoratorInverter => {
                BehaviorGraphNodeKind::Decorator(DecoratorNode::new(DecoratorKind::Inverter))
            }
            NodeTemplate::DecoratorSucceeder => {
                BehaviorGraphNodeKind::Decorator(DecoratorNode::new(DecoratorKind::Succeeder))
            }
            NodeTemplate::DecoratorFailer => {
                BehaviorGraphNodeKind::Decorator(DecoratorNode::new(DecoratorKind::Failer))
            }
            NodeTemplate::DecoratorRepeat => {
                BehaviorGraphNodeKind::Decorator(DecoratorNode::new(DecoratorKind::Repeat(2)))
            }
            NodeTemplate::DecoratorRetry => {
                BehaviorGraphNodeKind::Decorator(DecoratorNode::new(DecoratorKind::Retry(2)))
            }
        }
    }
}

fn fallback_name(input: &str, default_value: &str) -> String {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        default_value.to_string()
    } else {
        trimmed.to_string()
    }
}

struct DecoratorKindOption {
    label: &'static str,
    kind: DecoratorKind,
}

impl DecoratorKindOption {
    const ALL: &'static [DecoratorKindOption] = &[
        DecoratorKindOption {
            label: "Inverter",
            kind: DecoratorKind::Inverter,
        },
        DecoratorKindOption {
            label: "Succeeder",
            kind: DecoratorKind::Succeeder,
        },
        DecoratorKindOption {
            label: "Failer",
            kind: DecoratorKind::Failer,
        },
        DecoratorKindOption {
            label: "Repeat",
            kind: DecoratorKind::Repeat(2),
        },
        DecoratorKindOption {
            label: "Retry",
            kind: DecoratorKind::Retry(2),
        },
    ];
}

fn format_decorator_label(kind: &DecoratorKind) -> String {
    match kind {
        DecoratorKind::Inverter => "Inverter".into(),
        DecoratorKind::Succeeder => "Succeeder".into(),
        DecoratorKind::Failer => "Failer".into(),
        DecoratorKind::Repeat(max) => format!("Repeat ({max})"),
        DecoratorKind::Retry(max) => format!("Retry ({max})"),
    }
}
