
use graphics::character::CharacterCache;
use super::Kind;
use super::Update;
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
    fn set<C>(self, ui_id: UiId, ui: &mut Ui<C, Self>) where C: CharacterCache {
        let state = match *ui.get_widget_mut(ui_id, Kind::Custom(Self::State::init())) {
            ::widget::Kind::Custom(ref state) => state.clone(),
            _ => panic!("The Kind variant returned by Ui is different to that which \
                        was requested (Check that there are no UiId conflicts)."),
        };
        let Update { new_state, xy, depth, element } = self.update(state, ui_id, ui);
        ui.update_widget(ui_id, Kind::Custom(new_state), xy, depth, Some(element));
    }

    /// This is the method you have to implement! Your widget's previous state is given to you as a
    /// parameter and it is your job to construct and return an Update that will be used to update
    /// the widget's cached state.
    fn update<C>(mut self, prev: Self::State, ui_id: UiId, ui: &mut Ui<C, Self>) -> Update<Self::State>;

}

/// The state to be stored within the `Ui`s widget cache.
pub trait State: Clone + ::std::fmt::Debug {
    /// Whether or not the state matches some other state.
    fn matches(&self, other: &Self) -> bool;
    /// The inital state.
    fn init() -> Self;
}

impl Custom for () {
    type State = ();
    fn set<C>(self, _: UiId, _ui: &mut Ui<C, ()>) {}
    fn update<C>(self, _: (), _: UiId, _: &mut Ui<C, ()>) -> Update<()> {
        Update {
            new_state: (),
            xy: [0.0, 0.0],
            depth: 0.0,
            element: ::elmesque::element::empty(),
        }
    }
}

impl State for () {
    fn matches(&self, _other: &()) -> bool { true }
    fn init() -> Self { () }
}

