use aw_editor_lib::ui::menu_bar::{MenuBar, MenuActionHandler, AlignDirection};
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use std::sync::{Arc, Mutex};

struct MockHandler {
    new_called: bool,
    undo_called: bool,
    selection_count: usize,
    group_called: bool,
}

impl MenuActionHandler for MockHandler {
    fn on_new(&mut self) { self.new_called = true; }
    fn on_open(&mut self) {}
    fn on_save(&mut self) {}
    fn on_save_json(&mut self) {}
    fn on_save_scene(&mut self) {}
    fn on_load_scene(&mut self) {}
    
    fn on_undo(&mut self) { self.undo_called = true; }
    fn on_redo(&mut self) {}
    
    fn selection_count(&self) -> usize { self.selection_count }
    fn on_apply_material(&mut self) {}
    fn on_group_selection(&mut self) { self.group_called = true; }
    fn on_ungroup_selection(&mut self) {}
    fn on_align_selection(&mut self, _dir: AlignDirection) {}
}

#[test]
fn test_menu_new_button() {
    let handler = Arc::new(Mutex::new(MockHandler { 
        new_called: false, undo_called: false, selection_count: 0, group_called: false 
    }));
    let handler_clone = handler.clone();

    let mut harness = Harness::new_ui(move |ui| {
        let mut h = handler_clone.lock().unwrap();
        MenuBar::show(ui, &mut *h);
    });

    harness.get_by_label("New").click();
    harness.run();

    assert!(handler.lock().unwrap().new_called, "New button should trigger handler");
}

#[test]
fn test_menu_edit_undo() {
    let handler = Arc::new(Mutex::new(MockHandler { 
        new_called: false, undo_called: false, selection_count: 0, group_called: false 
    }));
    let handler_clone = handler.clone();

    let mut harness = Harness::new_ui(move |ui| {
        let mut h = handler_clone.lock().unwrap();
        MenuBar::show(ui, &mut *h);
    });

    // Click "Edit" menu button to open popup
    harness.get_by_label("✏️ Edit").click();
    harness.run();
    
    // Now look for Undo in the popup
    harness.get_by_label("↩️ Undo (Ctrl+Z)").click();
    harness.run();

    assert!(handler.lock().unwrap().undo_called, "Undo button in menu should trigger handler");
}

#[test]
fn test_menu_selection_state() {
    // Test with selection > 1 (should enable group button)
    let handler = Arc::new(Mutex::new(MockHandler { 
        new_called: false, undo_called: false, selection_count: 2, group_called: false 
    }));
    let handler_clone = handler.clone();

    let mut harness = Harness::new_ui(move |ui| {
        let mut h = handler_clone.lock().unwrap();
        MenuBar::show(ui, &mut *h);
    });

    harness.get_by_label("✏️ Edit").click();
    harness.run();
    
    // Check if Group Selection is clickable
    harness.get_by_label("📁 Group Selection (Ctrl+G)").click();
    harness.run();
    
    assert!(handler.lock().unwrap().group_called, "Group selection should be called when items selected");
}
