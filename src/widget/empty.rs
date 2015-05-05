
use elmesque::Element;
use graphics::character::CharacterCache;
use super::{State, Widget};
use ui::{Ui, UiId};

impl Widget for () {
    type State = ();
    type Style = ();
    fn unique_kind(&self) -> &'static str { "EMPTY" }
    fn init_state(&self) -> () { () }
    fn style(&self) -> () { () }
    fn update<C>(&mut self, _: &State<()>, _: &(), _: UiId, _: &mut Ui<C>) -> State<()>
        where C: CharacterCache
    {
        State { state: (), xy: [0.0, 0.0], depth: 0.0 }
    }
    fn draw<C>(&mut self, _: &State<()>, _: &(), _: UiId, _: &mut Ui<C>) -> Element
        where C: CharacterCache
    {
        ::elmesque::element::empty()
    }
}

