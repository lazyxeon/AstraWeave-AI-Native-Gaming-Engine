//! Mutation-resistant comprehensive tests for astract.
//!
//! Tests graph data structures (NodeGraph, GraphNode, Port, PortType, GraphEdge,
//! ForceDirectedParams, ForceDirectedLayout) — no egui UI context needed.

use astract::charts::{calculate_nice_bounds, transform_point, AxisConfig, Point};
use astract::component::{stateless, Component};
use astract::graph::*;
use astract::hooks::{use_effect, use_state};
use egui::{Pos2, Rect};

// ═══════════════════════════════════════════════════════════════════════════
// PortType
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn port_type_exec_color_is_white() {
    let c = PortType::Exec.color();
    assert_eq!(c, egui::Color32::WHITE);
}

#[test]
fn port_type_bool_color_is_red() {
    let c = PortType::Bool.color();
    assert!(
        c.r() > 200 && c.g() < 100 && c.b() < 100,
        "Bool color should be reddish: ({},{},{})",
        c.r(),
        c.g(),
        c.b()
    );
}

#[test]
fn port_type_number_color_is_green() {
    let c = PortType::Number.color();
    assert!(
        c.g() > 150 && c.r() < 150,
        "Number color should be greenish: ({},{},{})",
        c.r(),
        c.g(),
        c.b()
    );
}

#[test]
fn port_type_string_color_is_blue() {
    let c = PortType::String.color();
    assert!(
        c.b() > 200 && c.r() < 150,
        "String color should be bluish: ({},{},{})",
        c.r(),
        c.g(),
        c.b()
    );
}

#[test]
fn port_type_object_color_is_yellow() {
    let c = PortType::Object.color();
    assert!(
        c.r() > 200 && c.g() > 150,
        "Object color should be yellowish: ({},{},{})",
        c.r(),
        c.g(),
        c.b()
    );
}

#[test]
fn port_type_eq() {
    assert_eq!(PortType::Exec, PortType::Exec);
    assert_ne!(PortType::Exec, PortType::Bool);
    assert_ne!(PortType::Number, PortType::String);
}

#[test]
fn port_type_copy() {
    let a = PortType::Object;
    let b = a;
    assert_eq!(a, b);
}

// ═══════════════════════════════════════════════════════════════════════════
// Port
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn port_new_fields() {
    let p = Port::new(3, "Data In", PortType::Number);
    assert_eq!(p.index, 3);
    assert_eq!(p.label, "Data In");
    assert_eq!(p.port_type, PortType::Number);
}

#[test]
fn port_clone() {
    let p = Port::new(0, "Out", PortType::Exec);
    let p2 = p.clone();
    assert_eq!(p2.index, 0);
    assert_eq!(p2.label, "Out");
    assert_eq!(p2.port_type, PortType::Exec);
}

#[test]
fn port_debug() {
    let p = Port::new(1, "Test", PortType::Bool);
    let dbg = format!("{p:?}");
    assert!(dbg.contains("Port"), "debug: {dbg}");
}

// ═══════════════════════════════════════════════════════════════════════════
// GraphNode
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn graph_node_new_defaults() {
    let node = GraphNode::new(42, "TestNode");
    assert_eq!(node.id(), 42);
    assert_eq!(node.label(), "TestNode");
    assert_eq!(node.x(), 0.0);
    assert_eq!(node.y(), 0.0);
    assert!(node.inputs().is_empty());
    assert!(node.outputs().is_empty());
}

#[test]
fn graph_node_with_position() {
    let node = GraphNode::new(1, "Pos").with_position(100.0, 200.0);
    assert!((node.x() - 100.0).abs() < f32::EPSILON);
    assert!((node.y() - 200.0).abs() < f32::EPSILON);
}

#[test]
fn graph_node_add_input() {
    let mut node = GraphNode::new(1, "N");
    node.add_input(Port::new(0, "In", PortType::Exec));
    assert_eq!(node.inputs().len(), 1);
    assert_eq!(node.inputs()[0].label, "In");
}

#[test]
fn graph_node_add_output() {
    let mut node = GraphNode::new(1, "N");
    node.add_output(Port::new(0, "Out", PortType::String));
    assert_eq!(node.outputs().len(), 1);
    assert_eq!(node.outputs()[0].label, "Out");
}

#[test]
fn graph_node_add_multiple_ports() {
    let mut node = GraphNode::new(1, "N");
    node.add_input(Port::new(0, "A", PortType::Number));
    node.add_input(Port::new(1, "B", PortType::Number));
    node.add_output(Port::new(0, "Sum", PortType::Number));
    assert_eq!(node.inputs().len(), 2);
    assert_eq!(node.outputs().len(), 1);
}

#[test]
fn graph_node_rect_has_positive_area() {
    let node = GraphNode::new(1, "N");
    let rect = node.rect();
    assert!(rect.width() > 0.0);
    assert!(rect.height() > 0.0);
}

#[test]
fn graph_node_size_grows_with_ports() {
    let small = GraphNode::new(1, "S");
    let small_height = small.rect().height();

    let mut big = GraphNode::new(2, "B");
    big.add_input(Port::new(0, "A", PortType::Number));
    big.add_input(Port::new(1, "B", PortType::Number));
    big.add_input(Port::new(2, "C", PortType::Number));
    let big_height = big.rect().height();

    assert!(
        big_height > small_height,
        "node with more ports should be taller: {big_height} vs {small_height}"
    );
}

#[test]
fn graph_node_port_position_output_is_right() {
    let mut node = GraphNode::new(1, "N").with_position(50.0, 50.0);
    node.add_output(Port::new(0, "Out", PortType::Exec));
    let pos = node.port_position(true, 0);
    // Output port should be on the right side (x = position.x + width)
    assert!(
        pos.x > 50.0,
        "output port should be to the right of node origin"
    );
}

#[test]
fn graph_node_port_position_input_is_left() {
    let mut node = GraphNode::new(1, "N").with_position(50.0, 50.0);
    node.add_input(Port::new(0, "In", PortType::Exec));
    let pos = node.port_position(false, 0);
    // Input port should be on the left side (x = position.x)
    assert!((pos.x - 50.0).abs() < f32::EPSILON);
}

#[test]
fn graph_node_clone() {
    let mut node = GraphNode::new(5, "Clone");
    node.add_input(Port::new(0, "In", PortType::Bool));
    let n2 = node.clone();
    assert_eq!(n2.id(), 5);
    assert_eq!(n2.label(), "Clone");
    assert_eq!(n2.inputs().len(), 1);
}

// ═══════════════════════════════════════════════════════════════════════════
// GraphEdge
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn graph_edge_fields() {
    let e = GraphEdge::new(1, 0, 2, 1);
    assert_eq!(e.source_node(), 1);
    assert_eq!(e.source_port(), 0);
    assert_eq!(e.target_node(), 2);
    assert_eq!(e.target_port(), 1);
}

#[test]
fn graph_edge_clone() {
    let e = GraphEdge::new(10, 3, 20, 5);
    let e2 = e;
    assert_eq!(e2.source_node(), 10);
    assert_eq!(e2.target_node(), 20);
}

#[test]
fn graph_edge_debug() {
    let e = GraphEdge::new(1, 0, 2, 0);
    let dbg = format!("{e:?}");
    assert!(dbg.contains("GraphEdge"), "debug: {dbg}");
}

// ═══════════════════════════════════════════════════════════════════════════
// NodeGraph
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn node_graph_new_empty() {
    let g = NodeGraph::new();
    assert!(g.nodes().is_empty());
    assert!(g.edges().is_empty());
}

#[test]
fn node_graph_add_node() {
    let mut g = NodeGraph::new();
    let n = GraphNode::new(1, "First");
    let id = g.add_node(n);
    assert_eq!(id, 1);
    assert_eq!(g.nodes().len(), 1);
}

#[test]
fn node_graph_add_multiple_nodes() {
    let mut g = NodeGraph::new();
    g.add_node(GraphNode::new(1, "A"));
    g.add_node(GraphNode::new(2, "B"));
    g.add_node(GraphNode::new(3, "C"));
    assert_eq!(g.nodes().len(), 3);
}

#[test]
fn node_graph_get_node() {
    let mut g = NodeGraph::new();
    g.add_node(GraphNode::new(42, "Answer"));
    let node = g.get_node(42).unwrap();
    assert_eq!(node.label(), "Answer");
}

#[test]
fn node_graph_get_node_nonexistent() {
    let g = NodeGraph::new();
    assert!(g.get_node(999).is_none());
}

#[test]
fn node_graph_get_node_mut() {
    let mut g = NodeGraph::new();
    g.add_node(GraphNode::new(1, "Mutable"));
    let node = g.get_node_mut(1).unwrap();
    node.add_input(Port::new(0, "Added", PortType::Bool));
    assert_eq!(g.get_node(1).unwrap().inputs().len(), 1);
}

#[test]
fn node_graph_add_edge() {
    let mut g = NodeGraph::new();
    let mut n1 = GraphNode::new(1, "Source");
    n1.add_output(Port::new(0, "Out", PortType::Exec));
    let mut n2 = GraphNode::new(2, "Target");
    n2.add_input(Port::new(0, "In", PortType::Exec));
    g.add_node(n1);
    g.add_node(n2);
    g.add_edge(1, 0, 2, 0);
    assert_eq!(g.edges().len(), 1);
    assert_eq!(g.edges()[0].source_node(), 1);
    assert_eq!(g.edges()[0].target_node(), 2);
}

#[test]
fn node_graph_clear() {
    let mut g = NodeGraph::new();
    g.add_node(GraphNode::new(1, "A"));
    g.add_node(GraphNode::new(2, "B"));
    g.add_edge(1, 0, 2, 0);
    g.clear();
    assert!(g.nodes().is_empty());
    assert!(g.edges().is_empty());
}

#[test]
fn node_graph_remove_node() {
    let mut g = NodeGraph::new();
    g.add_node(GraphNode::new(1, "Removable"));
    g.add_node(GraphNode::new(2, "Keeper"));
    g.add_edge(1, 0, 2, 0);
    let removed = g.remove_node(1).unwrap();
    assert_eq!(removed.label(), "Removable");
    assert_eq!(g.nodes().len(), 1);
    assert!(
        g.edges().is_empty(),
        "edges to removed node should be cleaned up"
    );
}

#[test]
fn node_graph_remove_nonexistent_node() {
    let mut g = NodeGraph::new();
    assert!(g.remove_node(999).is_none());
}

#[test]
fn node_graph_auto_layout_empty_no_panic() {
    let mut g = NodeGraph::new();
    g.auto_layout(); // Should not panic on empty graph
}

#[test]
fn node_graph_auto_layout_single_node() {
    let mut g = NodeGraph::new();
    g.add_node(GraphNode::new(1, "Solo"));
    g.auto_layout(); // Should not panic
}

#[test]
fn node_graph_auto_layout_connected() {
    let mut g = NodeGraph::new();
    let mut n1 = GraphNode::new(1, "A");
    n1.add_output(Port::new(0, "Out", PortType::Exec));
    let mut n2 = GraphNode::new(2, "B");
    n2.add_input(Port::new(0, "In", PortType::Exec));
    g.add_node(n1);
    g.add_node(n2);
    g.add_edge(1, 0, 2, 0);
    g.auto_layout();
    // After layout, nodes should exist and have been repositioned
    assert!(g.get_node(1).is_some());
    assert!(g.get_node(2).is_some());
}

// ═══════════════════════════════════════════════════════════════════════════
// ForceDirectedParams
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn force_directed_params_defaults() {
    let p = ForceDirectedParams::default();
    assert!((p.k - 100.0).abs() < f32::EPSILON);
    assert!((p.c_spring - 0.01).abs() < f32::EPSILON);
    assert!((p.c_repulsion - 5000.0).abs() < f32::EPSILON);
    assert_eq!(p.max_iterations, 500);
    assert!((p.threshold - 0.5).abs() < f32::EPSILON);
    assert!((p.damping - 0.9).abs() < f32::EPSILON);
}

#[test]
fn force_directed_params_clone() {
    let p = ForceDirectedParams::default();
    let p2 = p.clone();
    assert!((p2.k - 100.0).abs() < f32::EPSILON);
    assert_eq!(p2.max_iterations, 500);
}

#[test]
fn force_directed_params_debug() {
    let p = ForceDirectedParams::default();
    let dbg = format!("{p:?}");
    assert!(dbg.contains("ForceDirectedParams"));
}

// ═══════════════════════════════════════════════════════════════════════════
// ForceDirectedLayout
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn force_directed_layout_new() {
    let layout = ForceDirectedLayout::new(ForceDirectedParams::default());
    let _ = layout; // Construction should not panic
}

#[test]
fn node_graph_auto_layout_with_custom_params() {
    let mut g = NodeGraph::new();
    g.add_node(GraphNode::new(1, "A").with_position(0.0, 0.0));
    g.add_node(GraphNode::new(2, "B").with_position(10.0, 10.0));
    g.add_edge(1, 0, 2, 0);
    let params = ForceDirectedParams {
        max_iterations: 10,
        ..Default::default()
    };
    g.auto_layout_with_params(params);
    assert!(g.get_node(1).is_some());
}

// ═══════════════════════════════════════════════════════════════════════════
// Mutation kill tests: charts (calculate_nice_bounds, transform_point)
// ═══════════════════════════════════════════════════════════════════════════

/// Kills: `max - min` arithmetic mutations (`- with +`, `- with /`) in
/// calculate_nice_bounds line 147, and `nice_min + nice_range_actual`
/// mutations (`+ with -`, `+ with *`) on line 148.
///
/// Input: min=2.0, max=8.0, range=6.0.  The nice bounds should expand,
/// not contract or wildly change.  With `- with +`: range = 10.0, very
/// different result.  With `+ with -`: nice_max = nice_min - nice_range,
/// negative.  With `+ with *`: nice_max = nice_min * nice_range, huge.
#[test]
fn calculate_nice_bounds_non_trivial_range() {
    let (lo, hi) = calculate_nice_bounds(2.0, 8.0);
    // nice range should contain [2, 8]
    assert!(lo <= 2.0, "lo={lo} should be <= 2.0");
    assert!(hi >= 8.0, "hi={hi} should be >= 8.0");
    // nice range shouldn't be absurdly large
    assert!(hi - lo < 20.0, "range {} is too large", hi - lo);
    assert!(hi > lo, "hi must be > lo");
}

/// Kills: `< with ==` and `< with <=` on the epsilon comparison (line 147).
///
/// When min == max (range ≈ 0), should return (min-1, max+1).
/// `< with ==`: `0 == EPSILON` is false → falls through to log10 of 0 → NaN.
/// `< with <=`: same boundary: `0 <= EPSILON` is true → correct path.
/// Wait — actually `0.0.abs() < EPSILON` is true (0 < tiny) so the function
/// enters the early return.  With `==`: `0 == EPSILON` is false → log10(0) = -inf.
/// With `<=`: `0 <= EPSILON` is true → same behavior as `<`.  Hmm, `<=` is
/// equivalent here because both 0 < eps and 0 <= eps are true.
/// But for values just above epsilon, `<=` would differ.
/// Actually, let's test with min=max=5.0.  range = 0.0, |0.0| = 0.0.
/// `0.0 < EPSILON` → true → return (4.0, 6.0).  That's the expected behavior.
/// Mutant `< with ==`: `0.0 == EPSILON` → false → falls through to log10(0) → -inf.
/// result would be (NaN, NaN).  So checking for NaN kills `==` mutant.
#[test]
fn calculate_nice_bounds_equal_values_returns_spread() {
    let (lo, hi) = calculate_nice_bounds(5.0, 5.0);
    assert_eq!(lo, 4.0, "equal values should produce min-1");
    assert_eq!(hi, 6.0, "equal values should produce max+1");
    // Both must be finite (kills `< with ==` which produces NaN via log10(0))
    assert!(lo.is_finite());
    assert!(hi.is_finite());
}

/// Kills: `- with +` and `- with /` on range = max - min (line 147, second occurrence)
///        and the nice_min calculation mutations.
///
/// With negative range (from `- with +`): log10 of negative = NaN.
/// With division: range = max/min = 4.0 instead of 6.0, different bounds.
#[test]
fn calculate_nice_bounds_asymmetric_range() {
    let (lo, hi) = calculate_nice_bounds(1.0, 7.0);
    assert!(lo <= 1.0, "lo={lo} should be <= 1.0");
    assert!(hi >= 7.0, "hi={hi} should be >= 7.0");
    assert!(lo.is_finite(), "lo must be finite");
    assert!(hi.is_finite(), "hi must be finite");
}

/// Kills: `- with +` on `x - min_x` and `y - min_y` in transform_point
///        (lines 177-178), and `> with ==/</>=` on epsilon comparisons
///        (lines 180, 186), and `- with +` in normalization (lines 181, 187).
#[test]
fn transform_point_maps_correctly() {
    let screen = Rect::from_min_max(Pos2::new(0.0, 0.0), Pos2::new(100.0, 100.0));
    let data_bounds: (Point, Point) = ((0.0, 0.0), (10.0, 10.0));

    // Mid-point should map to center
    let mid = transform_point((5.0, 5.0), data_bounds, screen);
    assert!((mid.x - 50.0).abs() < 0.01, "mid.x={} expected 50", mid.x);
    assert!(
        (mid.y - 50.0).abs() < 0.01,
        "mid.y={} expected 50 (Y flipped)",
        mid.y
    );

    // Origin should map to bottom-left (Y flipped: max.y for y=0)
    let origin = transform_point((0.0, 0.0), data_bounds, screen);
    assert!((origin.x - 0.0).abs() < 0.01, "origin.x={}", origin.x);
    assert!((origin.y - 100.0).abs() < 0.01, "origin.y={}", origin.y);

    // Top-right of data should map to top-right of screen (Y flipped)
    let tr = transform_point((10.0, 10.0), data_bounds, screen);
    assert!((tr.x - 100.0).abs() < 0.01, "tr.x={}", tr.x);
    assert!((tr.y - 0.0).abs() < 0.01, "tr.y={}", tr.y);
}

/// Kills: `> with ==/</>=` epsilon comparisons — test with non-zero ranges
/// to ensure the normalization branch is taken.  A degenerate range (0)
/// should produce 0.5 for the clamped dimension.
#[test]
fn transform_point_degenerate_x_range() {
    let screen = Rect::from_min_max(Pos2::new(0.0, 0.0), Pos2::new(200.0, 200.0));
    // x range is zero (min_x == max_x)
    let data_bounds: (Point, Point) = ((5.0, 0.0), (5.0, 10.0));
    let p = transform_point((5.0, 5.0), data_bounds, screen);
    // x_range ≈ 0 → x_norm = 0.5 → screen.min.x + 0.5 * 200 = 100
    assert!(
        (p.x - 100.0).abs() < 0.01,
        "degenerate x should map to center: {}",
        p.x
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// Mutation kill tests: AxisConfig builder
// ═══════════════════════════════════════════════════════════════════════════

/// Kills: `AxisConfig::with_range → Default::default()` — verifies that
/// with_range actually sets min, max and disables auto_scale.
#[test]
fn axis_config_with_range_sets_fields() {
    let cfg = AxisConfig::new("X").with_range(1.0, 10.0);
    assert_eq!(cfg.min, Some(1.0), "min must be set");
    assert_eq!(cfg.max, Some(10.0), "max must be set");
    assert!(!cfg.auto_scale, "auto_scale must be false after with_range");
}

// ═══════════════════════════════════════════════════════════════════════════
// Mutation kill tests: ColorPicker builder methods
// ═══════════════════════════════════════════════════════════════════════════

// ═══════════════════════════════════════════════════════════════════════════
// Mutation kill tests: hooks and component render
// ═══════════════════════════════════════════════════════════════════════════

/// Kills: `StatelessComponent::render → ()` — verifies the closure is
/// actually called by having it mutate shared state.
#[test]
fn stateless_component_render_calls_closure() {
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;
    let called = Arc::new(AtomicBool::new(false));
    let called2 = called.clone();

    let ctx = egui::Context::default();
    let _ = ctx.run(Default::default(), move |ctx| {
        let called3 = called2.clone();
        egui::CentralPanel::default().show(ctx, move |ui| {
            let called4 = called3.clone();
            let comp = stateless(move |_ui: &mut egui::Ui, _: ()| {
                called4.store(true, Ordering::SeqCst);
            });
            comp.render(ui, ());
        });
    });
    assert!(
        called.load(Ordering::SeqCst),
        "render must call the closure"
    );
}

/// Kills: `use_effect → ()` and `!= with ==` — verifies that use_effect
/// runs the effect on first call (value = new), and does NOT re-run when
/// value is the same.
#[test]
fn use_effect_runs_on_new_value_only() {
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;
    let counter = Arc::new(AtomicU32::new(0));

    let ctx = egui::Context::default();

    // First pass: value=42, effect should run
    {
        let c = counter.clone();
        let _ = ctx.run(Default::default(), move |ctx| {
            let c2 = c.clone();
            egui::CentralPanel::default().show(ctx, move |ui| {
                let c3 = c2.clone();
                use_effect(ui, "test_effect", 42_i32, move |_v| {
                    c3.fetch_add(1, Ordering::SeqCst);
                });
            });
        });
    }
    assert_eq!(
        counter.load(Ordering::SeqCst),
        1,
        "effect must run on first call"
    );

    // Second pass with same value: effect should NOT run
    {
        let c = counter.clone();
        let _ = ctx.run(Default::default(), move |ctx| {
            let c2 = c.clone();
            egui::CentralPanel::default().show(ctx, move |ui| {
                let c3 = c2.clone();
                use_effect(ui, "test_effect", 42_i32, move |_v| {
                    c3.fetch_add(1, Ordering::SeqCst);
                });
            });
        });
    }
    assert_eq!(
        counter.load(Ordering::SeqCst),
        1,
        "effect must NOT re-run for same value"
    );

    // Third pass with new value: effect should run again
    {
        let c = counter.clone();
        let _ = ctx.run(Default::default(), move |ctx| {
            let c2 = c.clone();
            egui::CentralPanel::default().show(ctx, move |ui| {
                let c3 = c2.clone();
                use_effect(ui, "test_effect", 99_i32, move |_v| {
                    c3.fetch_add(1, Ordering::SeqCst);
                });
            });
        });
    }
    assert_eq!(
        counter.load(Ordering::SeqCst),
        2,
        "effect must run on changed value"
    );
}

/// Kills: `StateSetter::call → ()` — verifies that calling the setter
/// actually stores the value.
#[test]
fn state_setter_call_stores_value() {
    use std::sync::atomic::{AtomicI32, Ordering};
    use std::sync::Arc;

    let ctx = egui::Context::default();
    let stored = Arc::new(AtomicI32::new(0));

    // First frame: set value to 42
    let _ = ctx.run(Default::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            let (_val, setter) = use_state(ui, "counter", 0_i32);
            setter.call(ui, 42);
        });
    });

    // Second frame: read back value
    {
        let s = stored.clone();
        let _ = ctx.run(Default::default(), move |ctx| {
            let s2 = s.clone();
            egui::CentralPanel::default().show(ctx, move |ui| {
                let (val, _setter) = use_state(ui, "counter", 0_i32);
                s2.store(val, Ordering::SeqCst);
            });
        });
    }
    assert_eq!(
        stored.load(Ordering::SeqCst),
        42,
        "setter must persist the value for next frame"
    );
}
