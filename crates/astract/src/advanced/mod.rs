// Advanced widgets module for game development use cases

pub mod color_picker;
pub mod range_slider;
pub mod tree_view;

// Re-exports
pub use color_picker::ColorPicker;
pub use range_slider::RangeSlider;
pub use tree_view::{TreeNode, TreeNodeId, TreeView};

#[cfg(test)]
mod tests {
    use super::*;
    use egui::Color32;

    #[test]
    fn test_color_picker_creation() {
        let picker = ColorPicker::new();
        assert_eq!(picker.color(), Color32::WHITE);
        assert_eq!(picker.alpha(), 255);
    }

    #[test]
    fn test_tree_view_creation() {
        let tree = TreeView::new();
        assert_eq!(tree.node_count(), 0);
    }

    #[test]
    fn test_range_slider_creation() {
        let slider = RangeSlider::new(0.0, 100.0);
        assert_eq!(slider.min_value(), 0.0);
        assert_eq!(slider.max_value(), 100.0);
    }
}
