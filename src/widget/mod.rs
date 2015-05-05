
use elmesque::Element;
use graphics::character::CharacterCache;
use position::{Depth, Point};
use std::any::Any;
use std::fmt::Debug;
use ui::{UiId, Ui};

pub mod empty;

pub mod button;
pub mod drop_down_list;
pub mod envelope_editor;
pub mod label;
pub mod matrix;
pub mod number_dialer;
pub mod slider;
pub mod text_box;
pub mod toggle;
pub mod xy_pad;


/// A trait to be implemented by all Widget types.
pub trait Widget: Sized {
    /// State to be stored within the `Ui`s widget cache. Take advantage of this type for any large
    /// allocations that you would like to avoid repeating between updates, or any calculations
    /// that you'd like to avoid repeating between calls to `update` and `draw`. Conrod will never
    /// clone the state, it will only ever be moved.
    type State: Any + PartialEq + Clone + ::std::fmt::Debug;
    /// Styling used by the widget to construct an Element. Styling is useful to have in its own
    /// abstraction in order to making Theme serializing easier. Conrod doesn't yet support
    /// serializing non-internal widget styling with the `Theme` type, but we hope to soon.
    type Style: Any + PartialEq + Clone + ::std::fmt::Debug;


    /// After building the widget, you call this method to set its current state into the given
    /// `Ui`. More precisely, the following will occur when calling this method:
    /// - The widget's previous state and style will be retrieved.
    /// - The widget's current `Style` will be retrieved (from the `Widget::style` method).
    /// - The widget's state will be updated (using the `Widget::udpate` method).
    /// - If the widget's state or style has changed, `Widget::draw` will be called to create the
    /// new Element for rendering.
    /// - The new State, Style and Element (if there is one) will be cached within the `Ui`.
    fn set<C>(mut self, ui_id: UiId, ui: &mut Ui<C>) where C: CharacterCache {
        let kind = self.unique_kind();
        let new_style = self.style();
        let prev_state = ui.get_widget_state(ui_id, kind, &new_style, &self);
        let PrevState { state, style, xy, depth } = prev_state;
        let prev = State { state: state, xy: xy, depth: depth };
        let new_state = self.update(&prev, &new_style, ui_id, ui);
        let maybe_new_element = if new_style != style || new_state != prev {
            Some(self.draw(&new_state, &new_style, ui_id, ui))
        } else {
            Some(self.draw(&new_state, &new_style, ui_id, ui))
        };
        let State { state, xy, depth } = new_state;
        let store: Store<Self::State, Self::Style> = Store { state: state, style: style };
        ui.update_widget(ui_id, kind, store, xy, depth, maybe_new_element);
    }

    /// Return the kind of the widget as a &'static str. Note that this must be unique from all
    /// other widgets' "unique kinds". This is used by conrod to help avoid UiId errors.
    fn unique_kind(&self) -> &'static str;

    /// Return the initial `State` of the Widget. The `Ui` will only call this once.
    fn init_state(&self) -> Self::State;

    /// Return the styling of the widget. The `Ui` will call this once prior to each `update`. It
    /// does this so that it can check for differences in `Style` in case a new `Element` needs to
    /// be constructed.
    fn style(&self) -> Self::Style;

    /// Your widget's previous state is given to you as a parameter and it is your job to
    /// construct and return an Update that will be used to update the widget's cached state.
    fn update<C>(&mut self,
                 prev: &State<Self::State>,
                 current_style: &Self::Style,
                 ui_id: UiId,
                 ui: &mut Ui<C>) -> State<Self::State>
        where C: CharacterCache;

    /// Construct a renderable Element from the current styling and new state. This will *only* be
    /// called on the occasion that the widget's `Style` or `State` has changed. Keep this in mind
    /// when designing your widget's `Style` and `State` types.
    fn draw<C>(&mut self,
               new_state: &State<Self::State>,
               current_style: &Self::Style,
               ui_id: UiId,
               ui: &mut Ui<C>) -> Element
        where C: CharacterCache;
}


// /// A blanket trait implemented for all `Widget` types that enables them to be set within the `Ui`.
// pub trait Set: Widget {
// 
// }
// 
// impl<W> Set for W where W: Widget {}


/// Represents the unique cached state of a widget.
#[derive(PartialEq, Clone)]
pub struct State<T> {
    /// The state of the Widget.
    pub state: T,
    /// The position of the Widget given as [x, y] coordinates.
    pub xy: Point,
    /// The rendering depth for the Widget (the default is 0.0).
    pub depth: Depth,
}

/// The previous widget state to be returned by the Ui prior to a widget updating it's new state.
pub struct PrevState<W> where W: Widget {
    /// State that is unique to the widget.
    pub state: W::State,
    /// Unique styling state for the widget.
    pub style: W::Style,
    /// Previous position of the Widget.
    pub xy: Point,
    /// Previous rendering depth of the Widget.
    pub depth: Depth,
}

/// The state type that we'll dynamically cast to and from Any for storage within the Cache.
#[derive(Clone, Debug)]
pub struct Store<Sta, Sty>
    where
        Sta: Any + Clone + Debug,
        Sty: Any + Clone + Debug,
{
    pub state: Sta,
    pub style: Sty,
}

/// A widget element for storage within the Ui's `widget_cache`.
#[derive(Debug)]
pub struct Cached {
    pub state: Box<Any>,
    pub kind: &'static str,
    pub xy: Point,
    pub depth: Depth,
    pub element: Element,
    pub set_since_last_drawn: bool,
}

impl Cached {

    /// Construct an empty Widget for a vacant widget position within the Ui.
    pub fn empty() -> Cached {
        Cached::new(().unique_kind(), Store { state: (), style: () })
    }

    /// Construct a Widget from a given kind.
    pub fn new<Sta, Sty>(kind: &'static str, store: Store<Sta, Sty>) -> Cached
        where
            Sta: Any + Clone + Debug + 'static,
            Sty: Any + Clone + Debug + 'static,
    {
        let state: Box<Any> = Box::new(store);
        Cached {
            state: state,
            kind: kind,
            xy: [0.0, 0.0],
            depth: 0.0,
            element: ::elmesque::element::empty(),
            set_since_last_drawn: false,
        }
    }

}


