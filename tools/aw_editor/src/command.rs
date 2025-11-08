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

use anyhow::Result;
use astraweave_core::{Entity, IVec2, World};
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
    pub fn execute(&mut self, mut command: Box<dyn EditorCommand>, world: &mut World) -> Result<()> {
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
    pub fn clear(&mut self) {
        self.commands.clear();
        self.cursor = 0;
    }

    /// Enable/disable auto-merging of consecutive commands.
    pub fn set_auto_merge(&mut self, enabled: bool) {
        self.auto_merge = enabled;
    }

    /// Get current command count (for debugging/UI).
    pub fn len(&self) -> usize {
        self.commands.len()
    }

    /// Check if stack is empty.
    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
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
        if let Some(other_move) = (other as &dyn std::any::Any).downcast_ref::<MoveEntityCommand>() {
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
        if let Some(other_rot) = (other as &dyn std::any::Any).downcast_ref::<RotateEntityCommand>() {
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
        if let Some(other_scale) = (other as &dyn std::any::Any).downcast_ref::<ScaleEntityCommand>() {
            if self.entity_id == other_scale.entity_id {
                self.new_scale = other_scale.new_scale;
                return true;
            }
        }
        false
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_undo_stack_basic() {
        let mut world = World::new();
        let entity = world.spawn_entity(IVec2::new(0, 0), "test");

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
        let entity = world.spawn_entity(IVec2::new(0, 0), "test");

        let mut stack = UndoStack::new(10);

        // Execute two commands
        stack.execute(MoveEntityCommand::new(entity, IVec2::new(0, 0), IVec2::new(5, 5)), &mut world).unwrap();
        stack.execute(MoveEntityCommand::new(entity, IVec2::new(5, 5), IVec2::new(10, 10)), &mut world).unwrap();

        // Undo once
        stack.undo(&mut world).unwrap();
        assert_eq!(world.pose(entity).unwrap().pos, IVec2::new(5, 5));

        // Execute new command (should discard redo history)
        stack.execute(MoveEntityCommand::new(entity, IVec2::new(5, 5), IVec2::new(20, 20)), &mut world).unwrap();
        assert_eq!(world.pose(entity).unwrap().pos, IVec2::new(20, 20));
        assert!(!stack.can_redo()); // Redo history discarded
    }

    #[test]
    fn test_command_merging() {
        let mut world = World::new();
        let entity = world.spawn_entity(IVec2::new(0, 0), "test");

        let mut stack = UndoStack::new(10);
        stack.set_auto_merge(true);

        // Execute 5 consecutive moves (should merge into 1)
        for i in 1..=5 {
            stack.execute(
                MoveEntityCommand::new(entity, IVec2::new(i - 1, i - 1), IVec2::new(i, i)),
                &mut world,
            ).unwrap();
        }

        assert_eq!(world.pose(entity).unwrap().pos, IVec2::new(5, 5));
        assert_eq!(stack.len(), 1); // All 5 commands merged into 1

        // Undo once (should revert all 5 moves)
        stack.undo(&mut world).unwrap();
        assert_eq!(world.pose(entity).unwrap().pos, IVec2::new(0, 0));
    }
}
