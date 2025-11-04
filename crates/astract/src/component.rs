// astract/src/component.rs - Component trait and helpers

use egui::Ui;

/// Core trait for reusable UI components
///
/// Components encapsulate rendering logic and can accept props.
///
/// # Example
/// ```ignore
/// struct Counter;
///
/// impl Component for Counter {
///     type Props = i32;
///     
///     fn render(&self, ui: &mut Ui, count: Self::Props) {
///         ui.label(format!("Count: {}", count));
///         if ui.button("Increment").clicked() {
///             // Update state (via parent)
///         }
///     }
/// }
/// ```
pub trait Component {
    /// Props passed to the component
    type Props;

    /// Render the component
    fn render(&self, ui: &mut Ui, props: Self::Props);
}

/// Create a stateless function component
///
/// # Example
/// ```ignore
/// let greeter = stateless(|ui, name: &str| {
///     ui.label(format!("Hello, {}!", name));
/// });
///
/// greeter.render(ui, "World");
/// ```
pub fn stateless<P, F>(f: F) -> StatelessComponent<P, F>
where
    F: Fn(&mut Ui, P),
{
    StatelessComponent {
        f,
        _phantom: std::marker::PhantomData,
    }
}

/// Stateless function component wrapper
pub struct StatelessComponent<P, F>
where
    F: Fn(&mut Ui, P),
{
    f: F,
    _phantom: std::marker::PhantomData<P>,
}

impl<P, F> Component for StatelessComponent<P, F>
where
    F: Fn(&mut Ui, P),
{
    type Props = P;

    fn render(&self, ui: &mut Ui, props: Self::Props) {
        (self.f)(ui, props);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stateless_component() {
        let ctx = egui::Context::default();
        let _ = ctx.run(Default::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let greeter = stateless(|ui: &mut Ui, name: &str| {
                    ui.label(format!("Hello, {}!", name));
                });

                greeter.render(ui, "World");
            });
        });
    }

    #[test]
    fn test_component_trait() {
        struct Counter;

        impl Component for Counter {
            type Props = i32;

            fn render(&self, ui: &mut Ui, count: Self::Props) {
                ui.label(format!("Count: {}", count));
            }
        }

        let ctx = egui::Context::default();
        let _ = ctx.run(Default::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let counter = Counter;
                counter.render(ui, 42);
            });
        });
    }
}
