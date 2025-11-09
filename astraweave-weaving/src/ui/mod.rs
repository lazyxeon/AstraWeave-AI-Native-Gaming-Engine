// UI Module - Veilweaver UI Components
//
// This module provides UI components for the Veilweaver demo, including:
// - Anchor inspection modal (inspect and repair anchors)
// - Echo HUD (currency display with animated feedback)
// - Ability unlock notification (slide-in animation)
// - Repair progress bar (world-space UI)

pub mod anchor_inspection_modal;
pub mod echo_hud;
pub mod ability_notification;
pub mod repair_progress_bar;

pub use anchor_inspection_modal::AnchorInspectionModal;
pub use echo_hud::{EchoHud, EchoFeedbackFloat};
pub use ability_notification::{AbilityUnlockNotification, NotificationState};
pub use repair_progress_bar::RepairProgressBar;
