pub mod progress;
pub mod status_bar;
pub mod toast;
pub mod menu_bar;

pub use progress::{ProgressManager, TaskCategory, TaskId};
pub use status_bar::{ResourceUsage, StatusBar};
pub use toast::{ToastAction, ToastLevel, ToastManager};
pub use menu_bar::{MenuBar, MenuActionHandler, AlignDirection, DistributeDirection};

#[cfg(test)]
mod tests_progress;
#[cfg(test)]
mod tests_toast;
// Toast struct is internal to ToastManager, not exported directly
// BackgroundTaskSummary available via status_bar module if needed
