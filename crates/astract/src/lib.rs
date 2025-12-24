pub use astract_macro::rsx;

pub mod advanced;
pub mod animation;
pub mod charts;
pub mod component;
pub mod graph;
pub mod hooks;
pub mod widgets;

// Re-export egui for convenience
pub mod prelude {
    pub use crate::advanced::{ColorPicker, RangeSlider, TreeNode, TreeView};
    pub use crate::animation::{
        Animatable, AnimationController, AnimationState, EasingFunction, Spring, SpringParams,
        Tween,
    };
    pub use crate::charts::{BarChart, LineChart, ScatterPlot};
    pub use crate::component::{stateless, Component};
    pub use crate::graph::{
        ForceDirectedLayout, ForceDirectedParams, GraphEdge, GraphNode, NodeGraph, Port, PortType,
    };
    pub use crate::hooks::{use_effect, use_memo, use_state};
    pub use crate::rsx;
    pub use crate::widgets::*;
    pub use egui;
}

#[cfg(test)]
#[allow(unused_must_use)]
mod tests {
    use crate::prelude::*;

    #[test]
    fn test_prelude_exports() {
        let ctx = egui::Context::default();
        let _ = ctx.run(Default::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                rsx!("Test");
            });
        });
    }

    #[test]
    fn test_rsx_label_tag() {
        let ctx = egui::Context::default();
        let _ = ctx.run(Default::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                rsx!(<Label text="Hello" />);
            });
        });
    }

    #[test]
    fn test_rsx_button_tag() {
        let ctx = egui::Context::default();
        let _ = ctx.run(Default::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                rsx!(<Button text="Click Me" />);
            });
        });
    }

    #[test]
    fn test_rsx_vstack() {
        let ctx = egui::Context::default();
        let _ = ctx.run(Default::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                rsx!(<VStack></VStack>);
            });
        });
    }

    #[test]
    fn test_rsx_hstack() {
        let ctx = egui::Context::default();
        let _ = ctx.run(Default::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                rsx!(<HStack></HStack>);
            });
        });
    }

    #[test]
    fn test_rsx_button_with_callback() {
        let ctx = egui::Context::default();
        let _ = ctx.run(Default::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let mut count = 0;
                rsx!(<Button text="Increment" on_click={|| count += 1} />);
                assert_eq!(count, 0); // Callback not called unless button clicked
            });
        });
    }

    #[test]
    fn test_rsx_nested_children() {
        let ctx = egui::Context::default();
        let _ = ctx.run(Default::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                rsx!(<VStack>
                    <Label text="Child 1" />
                    <Label text="Child 2" />
                </VStack>);
            });
        });
    }

    #[test]
    fn test_rsx_complex_tree() {
        let ctx = egui::Context::default();
        let _ = ctx.run(Default::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                rsx!(<VStack>
                    <Label text="Title" />
                    <HStack>
                        <Button text="Cancel" />
                        <Button text="OK" />
                    </HStack>
                </VStack>);
            });
        });
    }
}
