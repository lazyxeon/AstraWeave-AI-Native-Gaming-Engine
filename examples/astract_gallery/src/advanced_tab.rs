//! Advanced widgets showcase tab

use astract::advanced::{ColorPicker, RangeSlider, TreeNode, TreeView};
use astract::prelude::egui::*;

pub struct AdvancedTab {
    // ColorPicker state
    color_picker: ColorPicker,

    // TreeView state
    tree_view: TreeView,

    // RangeSlider state
    range_slider: RangeSlider,
}

impl Default for AdvancedTab {
    fn default() -> Self {
        Self {
            color_picker: ColorPicker::new()
                .with_color(Color32::from_rgb(100, 150, 200))
                .show_alpha(true)
                .show_presets(true),
            tree_view: create_demo_tree(),
            range_slider: RangeSlider::new(0.0, 100.0).with_min(25.0).with_max(75.0),
        }
    }
}

impl AdvancedTab {
    pub fn show(&mut self, ui: &mut Ui) {
        ui.heading("🎨 Advanced Widgets Showcase");
        ui.add_space(10.0);

        // ColorPicker
        ui.group(|ui| {
            ui.heading("Color Picker");
            ui.label("Interactive RGB(A) color selection with preview");

            ui.horizontal(|ui| {
                self.color_picker.show(ui);

                let color = self.color_picker.color();

                ui.vertical(|ui| {
                    ui.label("Selected Color:");
                    let (rect, _) = ui.allocate_exact_size(Vec2::new(100.0, 100.0), Sense::hover());
                    ui.painter().rect_filled(rect, 4.0, color);

                    ui.add_space(5.0);
                    ui.label(format!("RGB({}, {}, {})", color.r(), color.g(), color.b()));
                    ui.label(format!("Alpha: {}", color.a()));
                });
            });
        });

        ui.add_space(20.0);

        // TreeView
        ui.group(|ui| {
            ui.heading("Tree View");
            ui.label("Hierarchical data visualization with expand/collapse");

            ui.horizontal(|ui| {
                if ui.button("Reset Tree").clicked() {
                    self.tree_view = create_demo_tree();
                }
            });

            ui.add_space(5.0);

            // Show tree view
            if let Some(selected_id) = self.tree_view.show(ui) {
                if let Some(node) = self.tree_view.get_node(selected_id) {
                    ui.label(format!("Selected: {} (ID: {})", node.label, node.id));
                }
            }
        });

        ui.add_space(20.0);

        // RangeSlider
        ui.group(|ui| {
            ui.heading("Range Slider");
            ui.label("Select a range between two values");

            self.range_slider.show(ui);

            let min = self.range_slider.min_value();
            let max = self.range_slider.max_value();

            ui.label(format!("Selected range: {:.1} - {:.1}", min, max));
            ui.label(format!("Range width: {:.1}", max - min));

            // Visual representation
            ui.add_space(5.0);
            let min = self.range_slider.min_value();
            let max = self.range_slider.max_value();
            let available_width = ui.available_width();
            let bar_height = 30.0;
            let (rect, _) =
                ui.allocate_exact_size(Vec2::new(available_width, bar_height), Sense::hover());

            // Background
            ui.painter().rect_filled(rect, 2.0, Color32::from_gray(50));

            // Selected range
            let min_x = rect.min.x + (min / 100.0) as f32 * available_width;
            let max_x = rect.min.x + (max / 100.0) as f32 * available_width;
            let range_rect =
                Rect::from_min_max(Pos2::new(min_x, rect.min.y), Pos2::new(max_x, rect.max.y));
            ui.painter()
                .rect_filled(range_rect, 2.0, Color32::from_rgb(100, 150, 255));
        });

        ui.add_space(20.0);

        // Code examples
        ui.collapsing("📝 Code Examples", |ui| {
            ui.label("Color Picker:");
            ui.code(
                r#"use astract::advanced::ColorPicker;

let mut color = Color32::RED;
ui.add(ColorPicker::new(&mut color).show_alpha(true));"#,
            );

            ui.add_space(10.0);
            ui.label("Tree View:");
            ui.code(
                r#"use astract::advanced::{TreeView, TreeNode};

let mut tree = TreeView::new();

// Add root
let root = tree.add_node(
    TreeNode::new(1, "Root").with_icon("📁")
);

// Add child
tree.add_child(root, 
    TreeNode::new(2, "Child").with_icon("📄")
);

// Show and handle selection
if let Some(id) = tree.show(ui) {
    println!("Selected node: {}", id);
}"#,
            );

            ui.add_space(10.0);
            ui.label("Range Slider:");
            ui.code(
                r#"use astract::advanced::RangeSlider;

let mut min = 0.0;
let mut max = 100.0;
ui.add(RangeSlider::new(&mut min, &mut max, 0.0..=100.0));"#,
            );
        });
    }
}

/// Create a demo tree structure
fn create_demo_tree() -> TreeView {
    let mut tree = TreeView::new();

    // Root: Project
    let root = tree.add_node(TreeNode::new(1, "Project").with_icon("📁"));

    // Level 1: Source folders
    let src = tree
        .add_child(root, TreeNode::new(2, "src").with_icon("📁"))
        .unwrap();

    let assets = tree
        .add_child(root, TreeNode::new(3, "assets").with_icon("📁"))
        .unwrap();

    let docs = tree
        .add_child(root, TreeNode::new(4, "docs").with_icon("📁"))
        .unwrap();

    // Level 2: Source files
    tree.add_child(src, TreeNode::new(5, "main.rs").with_icon("🦀"));

    tree.add_child(src, TreeNode::new(6, "lib.rs").with_icon("🦀"));

    let components = tree
        .add_child(src, TreeNode::new(7, "components").with_icon("📁"))
        .unwrap();

    // Level 3: Component files
    tree.add_child(components, TreeNode::new(8, "button.rs").with_icon("🦀"));

    tree.add_child(components, TreeNode::new(9, "input.rs").with_icon("🦀"));

    tree.add_child(components, TreeNode::new(10, "slider.rs").with_icon("🦀"));

    // Level 2: Assets
    tree.add_child(assets, TreeNode::new(11, "logo.png").with_icon("🖼️"));

    tree.add_child(assets, TreeNode::new(12, "icon.svg").with_icon("🎨"));

    // Level 2: Docs
    tree.add_child(docs, TreeNode::new(13, "README.md").with_icon("📄"));

    tree.add_child(docs, TreeNode::new(14, "CHANGELOG.md").with_icon("📄"));

    tree
}
