use super::Panel;
use astraweave_core::{Entity, World};
use egui::Ui;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone)]
pub struct HierarchyNode {
    pub entity: Entity,
    pub children: Vec<Entity>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum HierarchyAction {
    CreatePrefab(Entity),
    DeleteEntity(Entity),
    DuplicateEntity(Entity),
    FocusEntity(Entity),
}

pub struct HierarchyPanel {
    hierarchy: HashMap<Entity, HierarchyNode>,
    root_entities: Vec<Entity>,

    selected_entities: HashSet<Entity>,
    last_clicked: Option<Entity>,

    drag_source: Option<Entity>,
    rename_entity: Option<Entity>,
    rename_buffer: String,

    context_menu_entity: Option<Entity>,
    empty_node_counter: u32,

    pending_actions: Vec<HierarchyAction>,
    search_filter: String,
}

impl HierarchyPanel {
    pub fn new() -> Self {
        Self {
            hierarchy: HashMap::new(),
            root_entities: Vec::new(),
            selected_entities: HashSet::new(),
            last_clicked: None,
            drag_source: None,
            rename_entity: None,
            rename_buffer: String::new(),
            context_menu_entity: None,
            empty_node_counter: 0,
            pending_actions: Vec::new(),
            search_filter: String::new(),
        }
    }

    pub fn take_pending_actions(&mut self) -> Vec<HierarchyAction> {
        std::mem::take(&mut self.pending_actions)
    }

    pub fn sync_with_world(&mut self, world: &World) {
        let world_entities: HashSet<Entity> = world.entities().into_iter().collect();

        self.hierarchy.retain(|e, _| world_entities.contains(e));
        self.root_entities.retain(|e| world_entities.contains(e));
        self.selected_entities
            .retain(|e| world_entities.contains(e));

        for entity in world.entities() {
            if let std::collections::hash_map::Entry::Vacant(e) = self.hierarchy.entry(entity) {
                e.insert(HierarchyNode {
                    entity,
                    children: Vec::new(),
                });
                self.root_entities.push(entity);
            }
        }
    }

    pub fn get_selected(&self) -> Option<Entity> {
        self.selected_entities.iter().next().copied()
    }

    pub fn set_selected(&mut self, entity: Option<Entity>) {
        self.selected_entities.clear();
        if let Some(e) = entity {
            self.selected_entities.insert(e);
            self.last_clicked = Some(e);
        }
    }

    pub fn get_all_selected(&self) -> Vec<Entity> {
        self.selected_entities.iter().copied().collect()
    }

    pub fn set_selected_multiple(&mut self, entities: &[Entity]) {
        self.selected_entities.clear();
        for &e in entities {
            self.selected_entities.insert(e);
        }
        self.last_clicked = entities.first().copied();
    }

    fn add_child_to_parent(&mut self, child: Entity, parent: Entity) {
        if child == parent {
            return;
        }

        if self.is_ancestor_of(child, parent) {
            return;
        }

        self.remove_from_parent(child);

        if let Some(parent_node) = self.hierarchy.get_mut(&parent) {
            if !parent_node.children.contains(&child) {
                parent_node.children.push(child);
            }
        }

        self.root_entities.retain(|&e| e != child);
    }

    fn remove_from_parent(&mut self, child: Entity) {
        for node in self.hierarchy.values_mut() {
            node.children.retain(|&e| e != child);
        }

        if !self.root_entities.contains(&child) {
            self.root_entities.push(child);
        }
    }

    fn is_ancestor_of(&self, potential_ancestor: Entity, descendant: Entity) -> bool {
        if let Some(node) = self.hierarchy.get(&potential_ancestor) {
            for &child in &node.children {
                if child == descendant || self.is_ancestor_of(child, descendant) {
                    return true;
                }
            }
        }
        false
    }

    fn get_parent(&self, entity: Entity) -> Option<Entity> {
        for (parent, node) in &self.hierarchy {
            if node.children.contains(&entity) {
                return Some(*parent);
            }
        }
        None
    }

    fn entity_matches_filter(&self, world: &World, entity: Entity, filter_lower: &str) -> bool {
        if let Some(name) = world.name(entity) {
            if name.to_lowercase().contains(filter_lower) {
                return true;
            }
        }
        if let Some(node) = self.hierarchy.get(&entity) {
            for &child in &node.children {
                if self.entity_matches_filter(world, child, filter_lower) {
                    return true;
                }
            }
        }
        false
    }

    pub fn show_with_world(&mut self, ui: &mut Ui, world: &mut World) -> Option<Entity> {
        let mut selected_changed = None;

        ui.heading("🌲 Hierarchy");
        ui.separator();

        ui.horizontal(|ui| {
            if ui.button("➕ Empty").clicked() {
                self.empty_node_counter += 1;
                let empty_name = format!("Empty_{}", self.empty_node_counter);
                let entity = world.spawn(
                    &empty_name,
                    astraweave_core::IVec2 { x: 0, y: 0 },
                    astraweave_core::Team { id: 0 },
                    0,
                    0,
                );
                self.hierarchy.insert(
                    entity,
                    HierarchyNode {
                        entity,
                        children: Vec::new(),
                    },
                );
                self.root_entities.push(entity);
            }

            if ui.button("🔄 Refresh").clicked() {
                self.sync_with_world(world);
            }
        });

        ui.add_space(5.0);

        if !self.selected_entities.is_empty() {
            ui.label(format!("Selected: {}", self.selected_entities.len()));
        }

        ui.separator();

        ui.horizontal(|ui| {
            ui.label("Search:");
            ui.add(
                egui::TextEdit::singleline(&mut self.search_filter)
                    .hint_text("Filter entities...")
                    .desired_width(ui.available_width() - 30.0),
            );
            if ui.button("X").clicked() {
                self.search_filter.clear();
            }
        });

        ui.add_space(5.0);

        let search_lower = self.search_filter.to_lowercase();
        let is_filtering = !self.search_filter.is_empty();

        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                let root_entities = self.root_entities.clone();
                for entity in root_entities {
                    if is_filtering && !self.entity_matches_filter(world, entity, &search_lower) {
                        continue;
                    }
                    if let Some(new_selection) = self.show_entity_tree(ui, world, entity, 0) {
                        selected_changed = Some(new_selection);
                    }
                }
            });

        selected_changed
    }

    fn show_entity_tree(
        &mut self,
        ui: &mut Ui,
        world: &World,
        entity: Entity,
        depth: usize,
    ) -> Option<Entity> {
        let mut selected_changed = None;

        let name = world.name(entity).unwrap_or("Unknown");
        let is_selected = self.selected_entities.contains(&entity);

        let children = if let Some(node) = self.hierarchy.get(&entity) {
            node.children.clone()
        } else {
            Vec::new()
        };
        let has_children = !children.is_empty();

        let indent = depth as f32 * 16.0;

        ui.horizontal(|ui| {
            ui.add_space(indent);

            let (id, rect) = ui.allocate_space(egui::vec2(ui.available_width(), 20.0));
            let response = ui.interact(rect, id, egui::Sense::click_and_drag());

            if response.drag_started() {
                self.drag_source = Some(entity);
            }

            if response.drag_stopped() {
                if let Some(source) = self.drag_source.take() {
                    if source != entity {
                        self.add_child_to_parent(source, entity);
                    }
                }
            }

            if response.clicked() {
                let modifiers = ui.input(|i| i.modifiers);

                if modifiers.ctrl {
                    if self.selected_entities.contains(&entity) {
                        self.selected_entities.remove(&entity);
                    } else {
                        self.selected_entities.insert(entity);
                    }
                    self.last_clicked = Some(entity);
                } else if modifiers.shift {
                    if let Some(last) = self.last_clicked {
                        let all_entities = self.get_all_entities_in_tree();
                        if let (Some(start_idx), Some(end_idx)) = (
                            all_entities.iter().position(|&e| e == last),
                            all_entities.iter().position(|&e| e == entity),
                        ) {
                            let (min_idx, max_idx) = if start_idx < end_idx {
                                (start_idx, end_idx)
                            } else {
                                (end_idx, start_idx)
                            };
                            for &e in &all_entities[min_idx..=max_idx] {
                                self.selected_entities.insert(e);
                            }
                        }
                    }
                    self.last_clicked = Some(entity);
                } else {
                    self.selected_entities.clear();
                    self.selected_entities.insert(entity);
                    self.last_clicked = Some(entity);
                    selected_changed = Some(entity);
                }
            }

            if response.double_clicked() {
                self.pending_actions
                    .push(HierarchyAction::FocusEntity(entity));
            }

            response.context_menu(|ui| {
                self.context_menu_entity = Some(entity);

                if ui.button("📝 Rename").clicked() {
                    self.rename_entity = Some(entity);
                    self.rename_buffer = name.to_string();
                    ui.close();
                }

                if ui.button("📋 Duplicate").clicked() {
                    self.pending_actions
                        .push(HierarchyAction::DuplicateEntity(entity));
                    ui.close();
                }

                if ui.button("🗑️ Delete").clicked() {
                    self.pending_actions
                        .push(HierarchyAction::DeleteEntity(entity));
                    ui.close();
                }

                ui.separator();

                if ui.button("💾 Create Prefab").clicked() {
                    self.pending_actions
                        .push(HierarchyAction::CreatePrefab(entity));
                    ui.close();
                }

                ui.separator();

                if ui.button("📤 Unparent").clicked() {
                    self.remove_from_parent(entity);
                    ui.close();
                }
            });

            let painter = ui.painter_at(rect);

            if is_selected {
                painter.rect_filled(
                    rect,
                    2.0,
                    egui::Color32::from_rgb(50, 100, 200).linear_multiply(0.3),
                );
            } else if response.hovered() {
                painter.rect_filled(
                    rect,
                    2.0,
                    egui::Color32::from_rgb(255, 255, 255).linear_multiply(0.1),
                );
            }

            if self.drag_source.is_some() && response.hovered() {
                painter.rect_stroke(
                    rect,
                    2.0,
                    egui::Stroke::new(2.0, egui::Color32::YELLOW),
                    egui::StrokeKind::Outside,
                );
            }

            let text_pos = rect.min
                + egui::vec2(
                    if has_children { 20.0 } else { 10.0 },
                    rect.height() * 0.5 - 7.0,
                );

            if has_children {
                let arrow = if ui.memory(|mem| {
                    mem.data
                        .get_temp::<bool>(egui::Id::new(format!("collapse_{}", entity)))
                        .unwrap_or(false)
                }) {
                    "▼"
                } else {
                    "▶"
                };
                painter.text(
                    rect.min + egui::vec2(5.0, rect.height() * 0.5 - 7.0),
                    egui::Align2::LEFT_TOP,
                    arrow,
                    egui::FontId::default(),
                    egui::Color32::WHITE,
                );

                let arrow_rect =
                    egui::Rect::from_min_size(rect.min, egui::vec2(15.0, rect.height()));
                if ui
                    .interact(
                        arrow_rect,
                        egui::Id::new(format!("arrow_{}", entity)),
                        egui::Sense::click(),
                    )
                    .clicked()
                {
                    let current = ui.memory(|mem| {
                        mem.data
                            .get_temp::<bool>(egui::Id::new(format!("collapse_{}", entity)))
                            .unwrap_or(false)
                    });
                    ui.memory_mut(|mem| {
                        mem.data
                            .insert_temp(egui::Id::new(format!("collapse_{}", entity)), !current)
                    });
                }
            }

            if Some(entity) == self.rename_entity {
                let text_edit =
                    egui::TextEdit::singleline(&mut self.rename_buffer).desired_width(150.0);
                let text_response = ui.put(
                    egui::Rect::from_min_size(text_pos, egui::vec2(150.0, 16.0)),
                    text_edit,
                );

                if text_response.lost_focus() || ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    self.rename_entity = None;
                }
            } else {
                painter.text(
                    text_pos,
                    egui::Align2::LEFT_TOP,
                    name,
                    egui::FontId::default(),
                    if is_selected {
                        egui::Color32::WHITE
                    } else {
                        egui::Color32::LIGHT_GRAY
                    },
                );
            }
        });

        if has_children {
            let is_expanded = ui.memory(|mem| {
                mem.data
                    .get_temp::<bool>(egui::Id::new(format!("collapse_{}", entity)))
                    .unwrap_or(false)
            });
            if is_expanded {
                for child in children {
                    if let Some(new_selection) = self.show_entity_tree(ui, world, child, depth + 1)
                    {
                        selected_changed = Some(new_selection);
                    }
                }
            }
        }

        selected_changed
    }

    fn get_all_entities_in_tree(&self) -> Vec<Entity> {
        let mut result = Vec::new();
        for &entity in &self.root_entities {
            self.collect_entities_recursive(entity, &mut result);
        }
        result
    }

    fn collect_entities_recursive(&self, entity: Entity, result: &mut Vec<Entity>) {
        result.push(entity);
        if let Some(node) = self.hierarchy.get(&entity) {
            for &child in &node.children {
                self.collect_entities_recursive(child, result);
            }
        }
    }
}

impl Default for HierarchyPanel {
    fn default() -> Self {
        Self::new()
    }
}

impl Panel for HierarchyPanel {
    fn name(&self) -> &str {
        "Hierarchy"
    }

    fn show(&mut self, ui: &mut Ui) {
        ui.heading("🌲 Hierarchy");
        ui.label("Hierarchy panel requires world integration");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hierarchy_panel_new() {
        let panel = HierarchyPanel::new();
        assert_eq!(panel.root_entities.len(), 0);
        assert_eq!(panel.selected_entities.len(), 0);
    }

    #[test]
    fn test_add_child_to_parent() {
        let mut panel = HierarchyPanel::new();

        panel.hierarchy.insert(
            1,
            HierarchyNode {
                entity: 1,
                children: Vec::new(),
            },
        );
        panel.hierarchy.insert(
            2,
            HierarchyNode {
                entity: 2,
                children: Vec::new(),
            },
        );
        panel.root_entities = vec![1, 2];

        panel.add_child_to_parent(2, 1);

        assert_eq!(panel.hierarchy.get(&1).unwrap().children, vec![2]);
        assert!(!panel.root_entities.contains(&2));
        assert_eq!(panel.root_entities, vec![1]);
    }

    #[test]
    fn test_prevent_circular_parenting() {
        let mut panel = HierarchyPanel::new();

        panel.hierarchy.insert(
            1,
            HierarchyNode {
                entity: 1,
                children: vec![2],
            },
        );
        panel.hierarchy.insert(
            2,
            HierarchyNode {
                entity: 2,
                children: vec![3],
            },
        );
        panel.hierarchy.insert(
            3,
            HierarchyNode {
                entity: 3,
                children: Vec::new(),
            },
        );

        panel.add_child_to_parent(1, 3);

        assert!(!panel.hierarchy.get(&3).unwrap().children.contains(&1));
    }

    #[test]
    fn test_remove_from_parent() {
        let mut panel = HierarchyPanel::new();

        panel.hierarchy.insert(
            1,
            HierarchyNode {
                entity: 1,
                children: vec![2],
            },
        );
        panel.hierarchy.insert(
            2,
            HierarchyNode {
                entity: 2,
                children: Vec::new(),
            },
        );
        panel.root_entities = vec![1];

        panel.remove_from_parent(2);

        assert_eq!(panel.hierarchy.get(&1).unwrap().children.len(), 0);
        assert!(panel.root_entities.contains(&2));
    }

    #[test]
    fn test_multi_selection() {
        let mut panel = HierarchyPanel::new();

        panel.selected_entities.insert(1);
        panel.selected_entities.insert(2);
        panel.selected_entities.insert(3);

        assert_eq!(panel.get_all_selected().len(), 3);
    }

    #[test]
    fn test_sync_with_world_removes_deleted_entities() {
        let mut panel = HierarchyPanel::new();
        let mut world = World::new();

        let e1 = world.spawn(
            "Entity1",
            astraweave_core::IVec2 { x: 0, y: 0 },
            astraweave_core::Team { id: 0 },
            100,
            10,
        );

        panel.hierarchy.insert(
            e1,
            HierarchyNode {
                entity: e1,
                children: Vec::new(),
            },
        );
        panel.hierarchy.insert(
            999,
            HierarchyNode {
                entity: 999,
                children: Vec::new(),
            },
        );
        panel.root_entities = vec![e1, 999];

        panel.sync_with_world(&world);

        assert!(panel.hierarchy.contains_key(&e1));
        assert!(!panel.hierarchy.contains_key(&999));
        assert!(!panel.root_entities.contains(&999));
    }
}
