use ui::{Ui, UiId};

/// A trait to be implemented for Custom widget types.
///
/// If you think your widget might be useful enough for conrod's official widget library, Feel free
/// to submit a PR at https://github.com/PistonDevelopers/conrod.
pub trait Custom: Clone + ::std::fmt::Debug {
    /// After building the widget, we use this method to set its current state into the given `Ui`.
    /// The `Ui` will cache this state and use it for rendering the next time `ui.draw(graphics)`
    /// is called.
    ///
    /// See one of the internal widgets for an example of how to implement this method.
    fn set<C>(mut self, ui_id: UiId, ui: &mut Ui<C>);
}
