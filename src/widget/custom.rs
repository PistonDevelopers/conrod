
use ui::{Ui, UiId};

/// A trait to be implemented for Custom widget types.
///
/// If you think your widget might be useful enough for conrod's official widget library, Feel free
/// to submit a PR at https://github.com/PistonDevelopers/conrod.
pub trait Custom: Clone + ::std::fmt::Debug {
    /// State to be stored within the `Ui`s widget cache.
    type State: State;

    /// After building the widget, we use this method to set its current state into the given `Ui`.
    /// The `Ui` will cache this state and use it for rendering the next time `ui.draw(graphics)`
    /// is called.
    ///
    /// See one of the internal widgets for an example of how to implement this method.
    fn set<C>(mut self, ui_id: UiId, ui: &mut Ui<C>);

}

/// The state to be stored within the `Ui`s widget cache.
pub trait State: Copy + Clone + ::std::fmt::Debug {
    /// Whether or not the state matches some other state.
    fn matches(&self, other: &Self) -> bool;
}

impl Custom for () {
    type State = ();
    fn set<C>(self, _ui_id: UiId, _ui: &mut Ui<C>) {}
}

impl State for () {
    fn matches(&self, _other: &()) -> bool { true }
}

