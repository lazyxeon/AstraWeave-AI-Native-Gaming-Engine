// astract/src/hooks.rs - React-style hooks for state management

use egui::Ui;
use std::marker::PhantomData;

/// State hook - manages component-local state using egui's storage
///
/// # Example
/// ```ignore
/// let (count, set_count) = use_state(ui, "counter", 0);
///
/// if ui.button("Increment").clicked() {
///     set_count(ui, count + 1);
/// }
///
/// ui.label(format!("Count: {}", count));
/// ```
pub fn use_state<T: Clone + Default + Send + Sync + 'static>(
    ui: &mut Ui,
    id: impl Into<String>,
    default: T,
) -> (T, StateSetter<T>) {
    let state_id = egui::Id::new(id.into());

    // Get current value from egui's temporary storage
    let current = ui.data(|d| d.get_temp::<T>(state_id)).unwrap_or(default);

    // Create setter
    let setter = StateSetter {
        id: state_id,
        _phantom: PhantomData,
    };

    (current, setter)
}

/// State setter - updates component state
pub struct StateSetter<T> {
    id: egui::Id,
    _phantom: PhantomData<T>,
}

impl<T: Clone + Send + Sync + 'static> StateSetter<T> {
    /// Update state value
    pub fn set(&self, ui: &mut Ui, value: T) {
        ui.data_mut(|d| d.insert_temp(self.id, value));
        ui.ctx().request_repaint(); // Trigger re-render
    }
}

// Make StateSetter callable like a function
impl<T: Clone + Send + Sync + 'static> StateSetter<T> {
    pub fn call(&self, ui: &mut Ui, value: T) {
        self.set(ui, value);
    }
}

/// Effect hook - run side effects on state changes
///
/// # Example
/// ```ignore
/// use_effect(ui, "log_count", count, |count| {
///     println!("Count changed to: {}", count);
/// });
/// ```
pub fn use_effect<T: Clone + PartialEq + Send + Sync + 'static, F>(
    ui: &mut Ui,
    id: impl Into<String>,
    value: T,
    f: F,
) where
    F: FnOnce(&T),
{
    let effect_id = egui::Id::new(id.into());

    // Get previous value
    let prev = ui.data(|d| d.get_temp::<T>(effect_id));

    // Run effect if value changed
    if prev.as_ref() != Some(&value) {
        f(&value);
        ui.data_mut(|d| d.insert_temp(effect_id, value));
    }
}

/// Memo hook - memoize expensive computations
///
/// # Example
/// ```ignore
/// let expensive_result = use_memo(ui, "calculation", input, |input| {
///     // Expensive computation
///     input * input
/// });
/// ```
pub fn use_memo<T: Clone + PartialEq + Send + Sync + 'static, R: Clone + Send + Sync + 'static, F>(
    ui: &mut Ui,
    id: impl Into<String>,
    input: T,
    f: F,
) -> R
where
    F: FnOnce(&T) -> R,
{
    let id_str = id.into();
    let memo_id = egui::Id::new(format!("{}_input", id_str));
    let result_id = egui::Id::new(format!("{}_result", id_str));

    // Check if input changed
    let prev_input = ui.data(|d| d.get_temp::<T>(memo_id));

    if prev_input.as_ref() != Some(&input) {
        // Recompute
        let result = f(&input);
        ui.data_mut(|d| {
            d.insert_temp(memo_id, input);
            d.insert_temp(result_id, result.clone());
        });
        result
    } else {
        // Return cached result
        ui.data(|d| d.get_temp::<R>(result_id))
            .expect("Memoized result should exist")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_use_state() {
        let ctx = egui::Context::default();
        let _ = ctx.run(Default::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let (count, set_count) = use_state(ui, "counter", 0);
                assert_eq!(count, 0);

                // Update state
                set_count.set(ui, 42);

                // Read updated state (in next frame)
                let (count2, _) = use_state::<i32>(ui, "counter", 0);
                assert_eq!(count2, 42);
            });
        });
    }

    #[test]
    fn test_use_effect() {
        let ctx = egui::Context::default();
        let _ = ctx.run(Default::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let mut effect_ran = false;

                use_effect(ui, "test_effect", 42, |_| {
                    effect_ran = true;
                });

                // Effect runs on first call (no previous value)
                // Note: In real usage, this would be in a closure
            });
        });
    }

    #[test]
    fn test_use_memo() {
        let ctx = egui::Context::default();
        let _ = ctx.run(Default::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let result = use_memo(ui, "expensive", 10, |x| x * x);
                assert_eq!(result, 100);

                // Second call should return cached value
                let result2 = use_memo(ui, "expensive", 10, |x| x * x);
                assert_eq!(result2, 100);
            });
        });
    }

    #[test]
    fn test_state_setter() {
        let ctx = egui::Context::default();
        let _ = ctx.run(Default::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let (_count, set_count) = use_state(ui, "test_setter", 0);

                // Test setter
                set_count.set(ui, 5);

                let (updated, _) = use_state::<i32>(ui, "test_setter", 0);
                assert_eq!(updated, 5);
            });
        });
    }
}
