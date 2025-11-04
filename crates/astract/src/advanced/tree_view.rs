use egui::{Color32, Sense, Ui};
use std::collections::HashMap;

/// Unique identifier for tree nodes
pub type TreeNodeId = usize;

/// A single node in the tree hierarchy
#[derive(Clone, Debug)]
pub struct TreeNode {
    pub id: TreeNodeId,
    pub label: String,
    pub icon: Option<String>, // Emoji or short text icon
    pub children: Vec<TreeNodeId>,
    pub expanded: bool,
    pub selected: bool,
    pub data: Option<String>, // Custom data payload
}

impl TreeNode {
    /// Create a new tree node
    pub fn new(id: TreeNodeId, label: impl Into<String>) -> Self {
        Self {
            id,
            label: label.into(),
            icon: None,
            children: Vec::new(),
            expanded: false,
            selected: false,
            data: None,
        }
    }

    /// Set the node icon
    pub fn with_icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    /// Set custom data
    pub fn with_data(mut self, data: impl Into<String>) -> Self {
        self.data = Some(data.into());
        self
    }

    /// Check if node has children
    pub fn has_children(&self) -> bool {
        !self.children.is_empty()
    }
}

/// A hierarchical tree view widget for displaying nested data structures
///
/// Supports:
/// - Expand/collapse nodes
/// - Node selection
/// - Custom icons (emoji)
/// - Indentation levels
/// - Click callbacks
///
/// # Example
/// ```rust,no_run
/// use astract::advanced::{TreeView, TreeNode};
///
/// let mut tree = TreeView::new();
///
/// // Add root node
/// let root = tree.add_node(TreeNode::new(1, "Root").with_icon("📁"));
///
/// // Add children
/// tree.add_child(root, TreeNode::new(2, "Child 1").with_icon("📄"));
/// tree.add_child(root, TreeNode::new(3, "Child 2").with_icon("📄"));
///
/// // Show returns selected node ID if any clicked
/// // if let Some(selected_id) = tree.show(ui) {
/// //     println!("Clicked node: {}", selected_id);
/// // }
/// ```
pub struct TreeView {
    nodes: HashMap<TreeNodeId, TreeNode>,
    root_nodes: Vec<TreeNodeId>,
    next_id: TreeNodeId,
    indent_size: f32,
    show_icons: bool,
}

impl Default for TreeView {
    fn default() -> Self {
        Self::new()
    }
}

impl TreeView {
    /// Create a new empty tree view
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            root_nodes: Vec::new(),
            next_id: 1,
            indent_size: 20.0,
            show_icons: true,
        }
    }

    /// Set the indentation size per level
    pub fn with_indent(mut self, indent: f32) -> Self {
        self.indent_size = indent;
        self
    }

    /// Show/hide node icons
    pub fn with_icons(mut self, show: bool) -> Self {
        self.show_icons = show;
        self
    }

    /// Add a root-level node
    pub fn add_node(&mut self, node: TreeNode) -> TreeNodeId {
        let id = node.id;
        self.root_nodes.push(id);
        self.nodes.insert(id, node);
        self.next_id = self.next_id.max(id + 1);
        id
    }

    /// Add a child node to a parent
    pub fn add_child(&mut self, parent_id: TreeNodeId, child: TreeNode) -> Option<TreeNodeId> {
        if let Some(parent) = self.nodes.get_mut(&parent_id) {
            let child_id = child.id;
            parent.children.push(child_id);
            self.nodes.insert(child_id, child);
            self.next_id = self.next_id.max(child_id + 1);
            Some(child_id)
        } else {
            None
        }
    }

    /// Get a node by ID
    pub fn get_node(&self, id: TreeNodeId) -> Option<&TreeNode> {
        self.nodes.get(&id)
    }

    /// Get a mutable node by ID
    pub fn get_node_mut(&mut self, id: TreeNodeId) -> Option<&mut TreeNode> {
        self.nodes.get_mut(&id)
    }

    /// Expand a node
    pub fn expand(&mut self, id: TreeNodeId) {
        if let Some(node) = self.nodes.get_mut(&id) {
            node.expanded = true;
        }
    }

    /// Collapse a node
    pub fn collapse(&mut self, id: TreeNodeId) {
        if let Some(node) = self.nodes.get_mut(&id) {
            node.expanded = false;
        }
    }

    /// Select a node (deselects all others)
    pub fn select(&mut self, id: TreeNodeId) {
        // Deselect all
        for node in self.nodes.values_mut() {
            node.selected = false;
        }
        // Select target
        if let Some(node) = self.nodes.get_mut(&id) {
            node.selected = true;
        }
    }

    /// Get the number of nodes
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Clear all nodes
    pub fn clear(&mut self) {
        self.nodes.clear();
        self.root_nodes.clear();
        self.next_id = 1;
    }

    /// Show the tree view UI
    /// Returns the ID of the clicked node, if any
    pub fn show(&mut self, ui: &mut Ui) -> Option<TreeNodeId> {
        let mut clicked_id = None;

        // Render root nodes
        for &root_id in &self.root_nodes.clone() {
            if let Some(id) = self.render_node(ui, root_id, 0) {
                clicked_id = Some(id);
            }
        }

        clicked_id
    }

    /// Recursively render a node and its children
    fn render_node(
        &mut self,
        ui: &mut Ui,
        node_id: TreeNodeId,
        level: usize,
    ) -> Option<TreeNodeId> {
        let mut clicked_id = None;

        // Get node (must clone to avoid borrow checker issues)
        let node = if let Some(n) = self.nodes.get(&node_id) {
            n.clone()
        } else {
            return None;
        };

        // Indentation
        ui.horizontal(|ui| {
            ui.add_space(level as f32 * self.indent_size);

            // Expand/collapse arrow (if has children)
            if node.has_children() {
                let arrow = if node.expanded { "▼" } else { "▶" };
                if ui.small_button(arrow).clicked() {
                    if node.expanded {
                        self.collapse(node_id);
                    } else {
                        self.expand(node_id);
                    }
                }
            } else {
                // Spacer for alignment
                ui.add_space(20.0);
            }

            // Icon
            if self.show_icons {
                if let Some(icon) = &node.icon {
                    ui.label(icon);
                }
            }

            // Label (selectable)
            let text_color = if node.selected {
                Color32::from_rgb(100, 180, 255)
            } else {
                ui.style().visuals.text_color()
            };

            let label_response = ui.add(
                egui::Label::new(egui::RichText::new(&node.label).color(text_color))
                    .sense(Sense::click()),
            );

            if label_response.clicked() {
                self.select(node_id);
                clicked_id = Some(node_id);
            }
        });

        // Render children (if expanded)
        if node.expanded {
            for &child_id in &node.children {
                if let Some(id) = self.render_node(ui, child_id, level + 1) {
                    clicked_id = Some(id);
                }
            }
        }

        clicked_id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tree_view_creation() {
        let tree = TreeView::new();
        assert_eq!(tree.node_count(), 0);
    }

    #[test]
    fn test_add_root_node() {
        let mut tree = TreeView::new();
        let node = TreeNode::new(1, "Root");
        let id = tree.add_node(node);
        assert_eq!(id, 1);
        assert_eq!(tree.node_count(), 1);
    }

    #[test]
    fn test_add_child_node() {
        let mut tree = TreeView::new();
        let root = tree.add_node(TreeNode::new(1, "Root"));
        let child_id = tree.add_child(root, TreeNode::new(2, "Child"));

        assert_eq!(child_id, Some(2));
        assert_eq!(tree.node_count(), 2);

        let root_node = tree.get_node(root).unwrap();
        assert_eq!(root_node.children.len(), 1);
        assert_eq!(root_node.children[0], 2);
    }

    #[test]
    fn test_expand_collapse() {
        let mut tree = TreeView::new();
        let root = tree.add_node(TreeNode::new(1, "Root"));

        assert!(!tree.get_node(root).unwrap().expanded);

        tree.expand(root);
        assert!(tree.get_node(root).unwrap().expanded);

        tree.collapse(root);
        assert!(!tree.get_node(root).unwrap().expanded);
    }

    #[test]
    fn test_selection() {
        let mut tree = TreeView::new();
        let node1 = tree.add_node(TreeNode::new(1, "Node 1"));
        let node2 = tree.add_node(TreeNode::new(2, "Node 2"));

        tree.select(node1);
        assert!(tree.get_node(node1).unwrap().selected);
        assert!(!tree.get_node(node2).unwrap().selected);

        tree.select(node2);
        assert!(!tree.get_node(node1).unwrap().selected);
        assert!(tree.get_node(node2).unwrap().selected);
    }

    #[test]
    fn test_tree_node_with_icon() {
        let node = TreeNode::new(1, "Test").with_icon("📁");
        assert_eq!(node.icon, Some("📁".to_string()));
    }

    #[test]
    fn test_tree_node_with_data() {
        let node = TreeNode::new(1, "Test").with_data("custom_data");
        assert_eq!(node.data, Some("custom_data".to_string()));
    }

    #[test]
    fn test_tree_node_has_children() {
        let mut node = TreeNode::new(1, "Parent");
        assert!(!node.has_children());

        node.children.push(2);
        assert!(node.has_children());
    }

    #[test]
    fn test_clear() {
        let mut tree = TreeView::new();
        tree.add_node(TreeNode::new(1, "Node 1"));
        tree.add_node(TreeNode::new(2, "Node 2"));
        assert_eq!(tree.node_count(), 2);

        tree.clear();
        assert_eq!(tree.node_count(), 0);
    }
}
