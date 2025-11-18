//! Editor command system - Undo/Redo infrastructure
//!
//! Implements the Command pattern for all editor operations.
//! Every action (transform, create, delete, edit) is wrapped in a command
//! that can be undone and redone.
//!
//! # Architecture
//!
//! ```text
//! EditorCommand (trait)
//!   ├─ Execute: Apply the change
//!   ├─ Undo: Revert the change
//!   └─ Describe: Human-readable description
//!
//! UndoStack
//!   ├─ commands: Vec<Box<dyn EditorCommand>>
//!   ├─ cursor: Current position in history
//!   └─ max_size: Memory limit (default: 100)
//! ```
//!
//! # Example
//!
//! ```rust
//! use aw_editor::command::{EditorCommand, UndoStack};
//!
//! let mut undo_stack = UndoStack::new(100);
//!
//! // Execute a command
//! let cmd = MoveEntityCommand::new(entity_id, old_pos, new_pos);
//! undo_stack.execute(cmd, &mut world)?;
//!
//! // Undo
//! undo_stack.undo(&mut world)?;
//!
//! // Redo
//! undo_stack.redo(&mut world)?;
//! ```

use crate::clipboard::ClipboardData;
use anyhow::Result;
use astraweave_core::{Entity, IVec2, Team, World};
use std::fmt;

// ============================================================================
// Command Trait
// ============================================================================

/// Editor command that can be executed, undone, and redone.
///
/// All editor operations should implement this trait to support undo/redo.
/// Commands are stored in an `UndoStack` and executed in LIFO order.
///
/// # Thread Safety
///
/// Commands must be `Send` to allow async execution in the future.
///
/// # Memory
///
/// Commands should store minimal data (just enough to undo).
/// Large data (e.g., full mesh) should be stored as refs or handles.
pub trait EditorCommand: Send + fmt::Debug + std::any::Any {
    /// Execute the command (apply the change).
    ///
    /// # Errors
    ///
    /// Returns `Err` if the command fails (e.g., entity doesn't exist).
    /// Failed commands are NOT added to the undo stack.
    fn execute(&mut self, world: &mut World) -> Result<()>;

    /// Undo the command (revert the change).
    ///
    /// # Errors
    ///
    /// Returns `Err` if undo fails (should be rare - log and continue).
    fn undo(&mut self, world: &mut World) -> Result<()>;

    /// Human-readable description for UI (e.g., "Move Entity", "Rotate Entity").
    ///
    /// Shown in undo menu and status bar.
    fn describe(&self) -> String;

    /// Optional: Merge this command with another if they're similar.
    ///
    /// Used for continuous operations (e.g., dragging gizmo creates 100 move commands,
    /// merge them into one for cleaner undo history).
    ///
    /// Returns `true` if merged successfully.
    fn try_merge(&mut self, _other: &dyn EditorCommand) -> bool {
        false // Default: no merging
    }
}

// ============================================================================
// Undo Stack
// ============================================================================

/// Undo/redo stack for editor commands.
///
/// Maintains a history of executed commands with a cursor pointing to the
/// current position. Undo moves cursor backward, redo moves forward.
///
/// # Memory Management
///
/// - Limited to `max_size` commands (default: 100)
/// - Old commands auto-pruned when limit reached
/// - Branching: Executing new command after undo discards redo history
///
/// # Thread Safety
///
/// Not thread-safe by design (editor operations are sequential).
#[derive(Debug)]
pub struct UndoStack {
    /// Command history (oldest to newest)
    commands: Vec<Box<dyn EditorCommand>>,

    /// Current position in history (points to next command to redo)
    ///
    /// - `cursor == 0`: Nothing to undo, can redo commands[0]
    /// - `cursor == len`: Nothing to redo, can undo commands[len-1]
    cursor: usize,

    /// Maximum number of commands to store
    max_size: usize,

    /// Whether to merge consecutive commands (for continuous ops like drag)
    auto_merge: bool,
}

impl UndoStack {
    /// Create a new undo stack.
    ///
    /// # Arguments
    ///
    /// * `max_size` - Maximum commands to store (default: 100, recommended: 50-200)
    pub fn new(max_size: usize) -> Self {
        Self {
            commands: Vec::with_capacity(max_size),
            cursor: 0,
            max_size,
            auto_merge: true,
        }
    }

    /// Execute a command and add to undo stack.
    ///
    /// # Behavior
    ///
    /// 1. Execute the command
    /// 2. If successful, add to stack (discarding redo history if any)
    /// 3. If auto_merge enabled, try merging with last command
    /// 4. Prune old commands if over max_size
    ///
    /// # Errors
    ///
    /// Returns `Err` if command execution fails. Failed commands are NOT added to stack.
    #[allow(dead_code)]
    pub fn execute(
        &mut self,
        mut command: Box<dyn EditorCommand>,
        world: &mut World,
    ) -> Result<()> {
        // Execute the command first
        command.execute(world)?;

        // Discard redo history (branching)
        self.commands.truncate(self.cursor);

        // Try merging with last command (if enabled and last command exists)
        if self.auto_merge && self.cursor > 0 {
            if let Some(last_cmd) = self.commands.last_mut() {
                if last_cmd.try_merge(command.as_ref()) {
                    // Merge successful, don't add new command
                    println!("🔄 Merged command: {}", command.describe());
                    return Ok(());
                }
            }
        }

        // Add command to stack
        self.commands.push(command);
        self.cursor += 1;

        // Prune old commands if over limit
        if self.commands.len() > self.max_size {
            let remove_count = self.commands.len() - self.max_size;
            self.commands.drain(0..remove_count);
            self.cursor = self.cursor.saturating_sub(remove_count);
        }

        Ok(())
    }

    /// Undo the last command.
    ///
    /// # Errors
    ///
    /// Returns `Err` if undo fails (logs error and continues).
    pub fn undo(&mut self, world: &mut World) -> Result<()> {
        if self.cursor == 0 {
            return Ok(()); // Nothing to undo
        }

        self.cursor -= 1;
        let cmd = &mut self.commands[self.cursor];

        println!("⏮️  Undo: {}", cmd.describe());
        cmd.undo(world)?;

        Ok(())
    }

    /// Redo the next command.
    ///
    /// # Errors
    ///
    /// Returns `Err` if redo fails (logs error and continues).
    pub fn redo(&mut self, world: &mut World) -> Result<()> {
        if self.cursor >= self.commands.len() {
            return Ok(()); // Nothing to redo
        }

        let cmd = &mut self.commands[self.cursor];

        println!("⏭️  Redo: {}", cmd.describe());
        cmd.execute(world)?;

        self.cursor += 1;

        Ok(())
    }

    /// Check if undo is available.
    pub fn can_undo(&self) -> bool {
        self.cursor > 0
    }

    /// Check if redo is available.
    pub fn can_redo(&self) -> bool {
        self.cursor < self.commands.len()
    }

    /// Get description of last undo command (for UI tooltip).
    pub fn undo_description(&self) -> Option<String> {
        if self.can_undo() {
            Some(self.commands[self.cursor - 1].describe())
        } else {
            None
        }
    }

    /// Get description of next redo command (for UI tooltip).
    pub fn redo_description(&self) -> Option<String> {
        if self.can_redo() {
            Some(self.commands[self.cursor].describe())
        } else {
            None
        }
    }

    /// Clear all commands (use when loading new scene).
    #[allow(dead_code)]
    pub fn clear(&mut self) {
        self.commands.clear();
        self.cursor = 0;
    }

    /// Enable/disable auto-merging of consecutive commands.
    #[allow(dead_code)]
    pub fn set_auto_merge(&mut self, enabled: bool) {
        self.auto_merge = enabled;
    }

    /// Get current command count (for debugging/UI).
    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.commands.len()
    }

    /// Check if stack is empty.
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }

    /// Add an already-executed command to the undo stack.
    ///
    /// Use this when you've already applied a transform (e.g., during gizmo drag)
    /// and just need to record it for undo/redo without executing again.
    pub fn push_executed(&mut self, command: Box<dyn EditorCommand>) {
        self.commands.truncate(self.cursor);

        if self.auto_merge && self.cursor > 0 {
            if let Some(last_cmd) = self.commands.last_mut() {
                if last_cmd.try_merge(command.as_ref()) {
                    println!("🔄 Merged command: {}", command.describe());
                    return;
                }
            }
        }

        self.commands.push(command);
        self.cursor += 1;

        if self.commands.len() > self.max_size {
            let remove_count = self.commands.len() - self.max_size;
            self.commands.drain(0..remove_count);
            self.cursor = self.cursor.saturating_sub(remove_count);
        }
    }
}

// ============================================================================
// Concrete Commands - Transform Operations
// ============================================================================

/// Move entity command (translate operation).
#[derive(Debug)]
pub struct MoveEntityCommand {
    entity_id: Entity,
    old_pos: IVec2,
    new_pos: IVec2,
}

impl MoveEntityCommand {
    pub fn new(entity_id: Entity, old_pos: IVec2, new_pos: IVec2) -> Box<Self> {
        Box::new(Self {
            entity_id,
            old_pos,
            new_pos,
        })
    }
}

impl EditorCommand for MoveEntityCommand {
    fn execute(&mut self, world: &mut World) -> Result<()> {
        if let Some(pose) = world.pose_mut(self.entity_id) {
            pose.pos = self.new_pos;
            Ok(())
        } else {
            anyhow::bail!("Entity {:?} not found", self.entity_id)
        }
    }

    fn undo(&mut self, world: &mut World) -> Result<()> {
        if let Some(pose) = world.pose_mut(self.entity_id) {
            pose.pos = self.old_pos;
            Ok(())
        } else {
            anyhow::bail!("Entity {:?} not found", self.entity_id)
        }
    }

    fn describe(&self) -> String {
        format!("Move Entity {:?}", self.entity_id)
    }

    fn try_merge(&mut self, other: &dyn EditorCommand) -> bool {
        // Try to downcast to MoveEntityCommand
        if let Some(other_move) = (other as &dyn std::any::Any).downcast_ref::<MoveEntityCommand>()
        {
            // Only merge if same entity
            if self.entity_id == other_move.entity_id {
                // Update new position but keep old position (chain multiple moves)
                self.new_pos = other_move.new_pos;
                return true;
            }
        }
        false
    }
}

/// Rotate entity command.
#[derive(Debug)]
pub struct RotateEntityCommand {
    entity_id: Entity,
    old_rotation_x: f32,
    old_rotation_y: f32,
    old_rotation_z: f32,
    new_rotation_x: f32,
    new_rotation_y: f32,
    new_rotation_z: f32,
}

impl RotateEntityCommand {
    pub fn new(
        entity_id: Entity,
        old_rotation: (f32, f32, f32), // (x, y, z)
        new_rotation: (f32, f32, f32),
    ) -> Box<Self> {
        Box::new(Self {
            entity_id,
            old_rotation_x: old_rotation.0,
            old_rotation_y: old_rotation.1,
            old_rotation_z: old_rotation.2,
            new_rotation_x: new_rotation.0,
            new_rotation_y: new_rotation.1,
            new_rotation_z: new_rotation.2,
        })
    }
}

impl EditorCommand for RotateEntityCommand {
    fn execute(&mut self, world: &mut World) -> Result<()> {
        if let Some(pose) = world.pose_mut(self.entity_id) {
            pose.rotation_x = self.new_rotation_x;
            pose.rotation = self.new_rotation_y; // Y-axis
            pose.rotation_z = self.new_rotation_z;
            Ok(())
        } else {
            anyhow::bail!("Entity {:?} not found", self.entity_id)
        }
    }

    fn undo(&mut self, world: &mut World) -> Result<()> {
        if let Some(pose) = world.pose_mut(self.entity_id) {
            pose.rotation_x = self.old_rotation_x;
            pose.rotation = self.old_rotation_y; // Y-axis
            pose.rotation_z = self.old_rotation_z;
            Ok(())
        } else {
            anyhow::bail!("Entity {:?} not found", self.entity_id)
        }
    }

    fn describe(&self) -> String {
        format!("Rotate Entity {:?}", self.entity_id)
    }

    fn try_merge(&mut self, other: &dyn EditorCommand) -> bool {
        if let Some(other_rot) = (other as &dyn std::any::Any).downcast_ref::<RotateEntityCommand>()
        {
            if self.entity_id == other_rot.entity_id {
                // Update new rotation but keep old rotation
                self.new_rotation_x = other_rot.new_rotation_x;
                self.new_rotation_y = other_rot.new_rotation_y;
                self.new_rotation_z = other_rot.new_rotation_z;
                return true;
            }
        }
        false
    }
}

/// Scale entity command.
#[derive(Debug)]
pub struct ScaleEntityCommand {
    entity_id: Entity,
    old_scale: f32,
    new_scale: f32,
}

impl ScaleEntityCommand {
    pub fn new(entity_id: Entity, old_scale: f32, new_scale: f32) -> Box<Self> {
        Box::new(Self {
            entity_id,
            old_scale,
            new_scale,
        })
    }
}

impl EditorCommand for ScaleEntityCommand {
    fn execute(&mut self, world: &mut World) -> Result<()> {
        if let Some(pose) = world.pose_mut(self.entity_id) {
            pose.scale = self.new_scale;
            Ok(())
        } else {
            anyhow::bail!("Entity {:?} not found", self.entity_id)
        }
    }

    fn undo(&mut self, world: &mut World) -> Result<()> {
        if let Some(pose) = world.pose_mut(self.entity_id) {
            pose.scale = self.old_scale;
            Ok(())
        } else {
            anyhow::bail!("Entity {:?} not found", self.entity_id)
        }
    }

    fn describe(&self) -> String {
        format!("Scale Entity {:?}", self.entity_id)
    }

    fn try_merge(&mut self, other: &dyn EditorCommand) -> bool {
        if let Some(other_scale) =
            (other as &dyn std::any::Any).downcast_ref::<ScaleEntityCommand>()
        {
            if self.entity_id == other_scale.entity_id {
                self.new_scale = other_scale.new_scale;
                return true;
            }
        }
        false
    }
}

#[derive(Debug)]
pub struct EditHealthCommand {
    entity: Entity,
    old_hp: i32,
    new_hp: i32,
}

impl EditHealthCommand {
    pub fn new(entity: Entity, old_hp: i32, new_hp: i32) -> Box<Self> {
        Box::new(Self {
            entity,
            old_hp,
            new_hp,
        })
    }
}

impl EditorCommand for EditHealthCommand {
    #[allow(dead_code)]
    fn execute(&mut self, world: &mut World) -> Result<()> {
        if let Some(health) = world.health_mut(self.entity) {
            health.hp = self.new_hp;
            Ok(())
        } else {
            Err(anyhow::anyhow!(
                "Entity {} has no Health component",
                self.entity
            ))
        }
    }

    fn undo(&mut self, world: &mut World) -> Result<()> {
        if let Some(health) = world.health_mut(self.entity) {
            health.hp = self.old_hp;
            Ok(())
        } else {
            Err(anyhow::anyhow!(
                "Entity {} has no Health component",
                self.entity
            ))
        }
    }

    fn describe(&self) -> String {
        format!("Edit Health (Entity #{})", self.entity)
    }
}

#[derive(Debug)]
pub struct EditTeamCommand {
    entity: Entity,
    old_team: Team,
    new_team: Team,
}

impl EditTeamCommand {
    pub fn new(entity: Entity, old_team: Team, new_team: Team) -> Box<Self> {
        Box::new(Self {
            entity,
            old_team,
            new_team,
        })
    }
}

impl EditorCommand for EditTeamCommand {
    #[allow(dead_code)]
    fn execute(&mut self, world: &mut World) -> Result<()> {
        if let Some(team) = world.team_mut(self.entity) {
            *team = self.new_team;
            Ok(())
        } else {
            Err(anyhow::anyhow!(
                "Entity {} has no Team component",
                self.entity
            ))
        }
    }

    fn undo(&mut self, world: &mut World) -> Result<()> {
        if let Some(team) = world.team_mut(self.entity) {
            *team = self.old_team;
            Ok(())
        } else {
            Err(anyhow::anyhow!(
                "Entity {} has no Team component",
                self.entity
            ))
        }
    }

    fn describe(&self) -> String {
        format!("Edit Team (Entity #{})", self.entity)
    }
}

#[derive(Debug)]
pub struct EditAmmoCommand {
    entity: Entity,
    old_rounds: i32,
    new_rounds: i32,
}

impl EditAmmoCommand {
    pub fn new(entity: Entity, old_rounds: i32, new_rounds: i32) -> Box<Self> {
        Box::new(Self {
            entity,
            old_rounds,
            new_rounds,
        })
    }
}

impl EditorCommand for EditAmmoCommand {
    #[allow(dead_code)]
    fn execute(&mut self, world: &mut World) -> Result<()> {
        if let Some(ammo) = world.ammo_mut(self.entity) {
            ammo.rounds = self.new_rounds;
            Ok(())
        } else {
            Err(anyhow::anyhow!(
                "Entity {} has no Ammo component",
                self.entity
            ))
        }
    }

    fn undo(&mut self, world: &mut World) -> Result<()> {
        if let Some(ammo) = world.ammo_mut(self.entity) {
            ammo.rounds = self.old_rounds;
            Ok(())
        } else {
            Err(anyhow::anyhow!(
                "Entity {} has no Ammo component",
                self.entity
            ))
        }
    }

    fn describe(&self) -> String {
        format!("Edit Ammo (Entity #{})", self.entity)
    }
}

#[derive(Debug)]
pub struct SpawnEntitiesCommand {
    spawned_entities: Vec<Entity>,
    clipboard_data: ClipboardData,
    offset: IVec2,
}

impl SpawnEntitiesCommand {
    pub fn new(clipboard_data: ClipboardData, offset: IVec2) -> Box<dyn EditorCommand> {
        Box::new(Self {
            spawned_entities: Vec::new(),
            clipboard_data,
            offset,
        })
    }
}

impl EditorCommand for SpawnEntitiesCommand {
    fn execute(&mut self, world: &mut World) -> Result<()> {
        self.spawned_entities = self.clipboard_data.spawn_entities(world, self.offset)?;
        Ok(())
    }

    fn undo(&mut self, world: &mut World) -> Result<()> {
        for &entity in &self.spawned_entities {
            if let Some(pose) = world.pose_mut(entity) {
                *pose = astraweave_core::Pose {
                    pos: IVec2 {
                        x: -10000,
                        y: -10000,
                    },
                    rotation: 0.0,
                    rotation_x: 0.0,
                    rotation_z: 0.0,
                    scale: 0.0,
                };
            }
        }
        Ok(())
    }

    fn describe(&self) -> String {
        format!("Paste {} entities", self.clipboard_data.entities.len())
    }
}

#[derive(Debug)]
pub struct DuplicateEntitiesCommand {
    source_entities: Vec<Entity>,
    spawned_entities: Vec<Entity>,
    offset: IVec2,
}

impl DuplicateEntitiesCommand {
    pub fn new(source_entities: Vec<Entity>, offset: IVec2) -> Box<dyn EditorCommand> {
        Box::new(Self {
            source_entities,
            spawned_entities: Vec::new(),
            offset,
        })
    }
}

impl EditorCommand for DuplicateEntitiesCommand {
    fn execute(&mut self, world: &mut World) -> Result<()> {
        let clipboard = ClipboardData::from_entities(world, &self.source_entities);
        self.spawned_entities = clipboard.spawn_entities(world, self.offset)?;
        Ok(())
    }

    fn undo(&mut self, world: &mut World) -> Result<()> {
        for &entity in &self.spawned_entities {
            if let Some(pose) = world.pose_mut(entity) {
                *pose = astraweave_core::Pose {
                    pos: IVec2 {
                        x: -10000,
                        y: -10000,
                    },
                    rotation: 0.0,
                    rotation_x: 0.0,
                    rotation_z: 0.0,
                    scale: 0.0,
                };
            }
        }
        Ok(())
    }

    fn describe(&self) -> String {
        format!("Duplicate {} entities", self.source_entities.len())
    }
}

#[derive(Debug)]
pub struct DeleteEntitiesCommand {
    entities: Vec<Entity>,
    clipboard_data: Option<ClipboardData>,
}

impl DeleteEntitiesCommand {
    pub fn new(entities: Vec<Entity>) -> Box<dyn EditorCommand> {
        Box::new(Self {
            entities,
            clipboard_data: None,
        })
    }
}

impl EditorCommand for DeleteEntitiesCommand {
    fn execute(&mut self, world: &mut World) -> Result<()> {
        self.clipboard_data = Some(ClipboardData::from_entities(world, &self.entities));

        for &entity in &self.entities {
            world.destroy_entity(entity);
        }
        Ok(())
    }

    fn undo(&mut self, world: &mut World) -> Result<()> {
        if let Some(clipboard) = &self.clipboard_data {
            clipboard.spawn_entities(world, IVec2 { x: 0, y: 0 })?;
        }
        Ok(())
    }

    fn describe(&self) -> String {
        format!("Delete {} entities", self.entities.len())
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn spawn_basic_entity(world: &mut World) -> Entity {
        world.spawn("test", IVec2::new(0, 0), Team { id: 0 }, 100, 30)
    }

    #[test]
    fn test_undo_stack_basic() {
        let mut world = World::new();
        let entity = spawn_basic_entity(&mut world);

        let mut stack = UndoStack::new(10);

        // Execute move command
        let cmd = MoveEntityCommand::new(entity, IVec2::new(0, 0), IVec2::new(5, 5));
        stack.execute(cmd, &mut world).unwrap();

        assert_eq!(world.pose(entity).unwrap().pos, IVec2::new(5, 5));
        assert!(stack.can_undo());
        assert!(!stack.can_redo());

        // Undo
        stack.undo(&mut world).unwrap();
        assert_eq!(world.pose(entity).unwrap().pos, IVec2::new(0, 0));
        assert!(!stack.can_undo());
        assert!(stack.can_redo());

        // Redo
        stack.redo(&mut world).unwrap();
        assert_eq!(world.pose(entity).unwrap().pos, IVec2::new(5, 5));
        assert!(stack.can_undo());
        assert!(!stack.can_redo());
    }

    #[test]
    fn test_undo_stack_branching() {
        let mut world = World::new();
        let entity = spawn_basic_entity(&mut world);

        let mut stack = UndoStack::new(10);
        stack.set_auto_merge(false); // Disable merging to test branching

        // Execute two commands
        stack
            .execute(
                MoveEntityCommand::new(entity, IVec2::new(0, 0), IVec2::new(5, 5)),
                &mut world,
            )
            .unwrap();
        stack
            .execute(
                MoveEntityCommand::new(entity, IVec2::new(5, 5), IVec2::new(10, 10)),
                &mut world,
            )
            .unwrap();

        // Undo once
        stack.undo(&mut world).unwrap();
        assert_eq!(world.pose(entity).unwrap().pos, IVec2::new(5, 5));

        // Execute new command (should discard redo history)
        stack
            .execute(
                MoveEntityCommand::new(entity, IVec2::new(5, 5), IVec2::new(20, 20)),
                &mut world,
            )
            .unwrap();
        assert_eq!(world.pose(entity).unwrap().pos, IVec2::new(20, 20));
        assert!(!stack.can_redo()); // Redo history discarded
    }

    #[test]
    fn test_command_merging() {
        let mut world = World::new();
        let entity = spawn_basic_entity(&mut world);

        let mut stack = UndoStack::new(10);
        stack.set_auto_merge(true);

        // Execute 5 consecutive moves (should merge into 1)
        for i in 1..=5 {
            stack
                .execute(
                    MoveEntityCommand::new(entity, IVec2::new(i - 1, i - 1), IVec2::new(i, i)),
                    &mut world,
                )
                .unwrap();
        }

        assert_eq!(world.pose(entity).unwrap().pos, IVec2::new(5, 5));
        assert_eq!(stack.len(), 1); // All 5 commands merged into 1

        // Undo once (should revert all 5 moves)
        stack.undo(&mut world).unwrap();
        assert_eq!(world.pose(entity).unwrap().pos, IVec2::new(0, 0));
    }

    #[test]
    fn test_rotate_command() {
        let mut world = World::new();
        let entity = spawn_basic_entity(&mut world);

        let mut stack = UndoStack::new(10);

        let old_rot = (0.0, 0.0, 0.0);
        let new_rot = (0.5, 1.0, 1.5);

        stack
            .execute(
                RotateEntityCommand::new(entity, old_rot, new_rot),
                &mut world,
            )
            .unwrap();

        let pose = world.pose(entity).unwrap();
        assert!((pose.rotation_x - 0.5).abs() < 0.01);
        assert!((pose.rotation - 1.0).abs() < 0.01);
        assert!((pose.rotation_z - 1.5).abs() < 0.01);

        stack.undo(&mut world).unwrap();

        let pose = world.pose(entity).unwrap();
        assert!((pose.rotation_x - 0.0).abs() < 0.01);
        assert!((pose.rotation - 0.0).abs() < 0.01);
        assert!((pose.rotation_z - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_scale_command() {
        let mut world = World::new();
        let entity = spawn_basic_entity(&mut world);

        let mut stack = UndoStack::new(10);

        stack
            .execute(ScaleEntityCommand::new(entity, 1.0, 2.5), &mut world)
            .unwrap();

        assert!((world.pose(entity).unwrap().scale - 2.5).abs() < 0.01);

        stack.undo(&mut world).unwrap();
        assert!((world.pose(entity).unwrap().scale - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_edit_health_command() {
        let mut world = World::new();
        let entity = world.spawn("Player", IVec2::new(0, 0), Team { id: 0 }, 100, 30);

        let mut stack = UndoStack::new(10);

        stack.push_executed(EditHealthCommand::new(entity, 100, 50));

        stack.undo(&mut world).unwrap();
        assert_eq!(world.health(entity).unwrap().hp, 100);

        stack.redo(&mut world).unwrap();
        assert_eq!(world.health(entity).unwrap().hp, 50);
    }

    #[test]
    fn test_edit_team_command() {
        let mut world = World::new();
        let entity = world.spawn("Enemy", IVec2::new(0, 0), Team { id: 2 }, 100, 30);

        let mut stack = UndoStack::new(10);
        let old_team = Team { id: 2 };
        let new_team = Team { id: 0 };

        stack.push_executed(EditTeamCommand::new(entity, old_team, new_team));

        stack.undo(&mut world).unwrap();
        assert_eq!(world.team(entity).unwrap().id, 2);

        stack.redo(&mut world).unwrap();
        assert_eq!(world.team(entity).unwrap().id, 0);
    }

    #[test]
    fn test_edit_ammo_command() {
        let mut world = World::new();
        let entity = world.spawn("Shooter", IVec2::new(0, 0), Team { id: 0 }, 100, 30);

        let mut stack = UndoStack::new(10);

        stack.push_executed(EditAmmoCommand::new(entity, 30, 15));

        stack.undo(&mut world).unwrap();
        assert_eq!(world.ammo(entity).unwrap().rounds, 30);

        stack.redo(&mut world).unwrap();
        assert_eq!(world.ammo(entity).unwrap().rounds, 15);
    }

    #[test]
    fn test_undo_stack_max_size() {
        let mut world = World::new();
        let entity = spawn_basic_entity(&mut world);

        let mut stack = UndoStack::new(5);
        stack.set_auto_merge(false); // Disable merging to test max size

        for i in 0..10 {
            stack
                .execute(
                    MoveEntityCommand::new(entity, IVec2::new(i, i), IVec2::new(i + 1, i + 1)),
                    &mut world,
                )
                .unwrap();
        }

        assert_eq!(stack.len(), 5);
        assert_eq!(stack.cursor, 5);
    }

    #[test]
    fn test_undo_stack_descriptions() {
        let mut world = World::new();
        let entity = spawn_basic_entity(&mut world);

        let mut stack = UndoStack::new(10);

        stack
            .execute(
                MoveEntityCommand::new(entity, IVec2::new(0, 0), IVec2::new(5, 5)),
                &mut world,
            )
            .unwrap();

        assert!(stack.undo_description().unwrap().contains("Move Entity"));
        assert!(stack.redo_description().is_none());

        stack.undo(&mut world).unwrap();
        assert!(stack.redo_description().unwrap().contains("Move Entity"));
        assert!(stack.undo_description().is_none());
    }

    #[test]
    fn test_push_executed() {
        let mut world = World::new();
        let entity = spawn_basic_entity(&mut world);

        world.pose_mut(entity).unwrap().pos = IVec2::new(5, 5);

        let mut stack = UndoStack::new(10);
        stack.push_executed(MoveEntityCommand::new(
            entity,
            IVec2::new(0, 0),
            IVec2::new(5, 5),
        ));

        assert_eq!(world.pose(entity).unwrap().pos, IVec2::new(5, 5));

        stack.undo(&mut world).unwrap();
        assert_eq!(world.pose(entity).unwrap().pos, IVec2::new(0, 0));
    }
}
