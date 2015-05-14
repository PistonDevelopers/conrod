
use canvas::CanvasId;
use elmesque::Element;
use graphics::character::CharacterCache;
use position::{Depth, Dimensions, Point, Positionable, Sizeable};
use std::any::Any;
use std::fmt::Debug;
use theme::Theme;
use ui::{GlyphCache, Ui, UiId, UserInput};

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

/// Unique widget identifier. Each widget must use a unique `WidgetId` so that it's state can be
/// cached within the `Ui` type. The reason we use a usize is because widgets are cached within
/// a `Vec`, which is limited to a size of `usize` elements.
pub type WidgetId = usize;

/// A trait to be implemented by all Widget types.
pub trait Widget: Positionable + Sizeable + Sized {
    /// State to be stored within the `Ui`s widget cache. Take advantage of this type for any large
    /// allocations that you would like to avoid repeating between updates, or any calculations
    /// that you'd like to avoid repeating between calls to `update` and `draw`. Conrod will never
    /// clone the state, it will only ever be moved.
    type State: Any + PartialEq + ::std::fmt::Debug;
    /// Styling used by the widget to construct an Element. Styling is useful to have in its own
    /// abstraction in order to making Theme serializing easier. Conrod doesn't yet support
    /// serializing non-internal widget styling with the `Theme` type, but we hope to soon.
    type Style: Any + PartialEq + ::std::fmt::Debug;

    /// After building the widget, you call this method to set its current state into the given
    /// `Ui`. More precisely, the following will occur when calling this method:
    /// - The widget's previous state and style will be retrieved.
    /// - The widget's current `Style` will be retrieved (from the `Widget::style` method).
    /// - The widget's state will be updated (using the `Widget::udpate` method).
    /// - If the widget's state or style has changed, `Widget::draw` will be called to create the
    /// new Element for rendering.
    /// - The new State, Style and Element (if there is one) will be cached within the `Ui`.
    fn set<C>(self, id: WidgetId, ui: &mut Ui<C>) where C: CharacterCache {
        let kind = self.unique_kind();
        let new_style = self.style();
        let depth = self.get_depth();
        let pos = self.get_position();
        let (h_align, v_align) = self.get_alignment(&ui.theme);
        let dim = {
            let Ui { ref theme, ref glyph_cache, .. } = *ui;
            self.get_dimensions(theme, glyph_cache)
        };
        let xy = ui.get_xy(pos, dim, h_align, v_align);

        // Collect the previous state and style or initialise both if none exist.
        let maybe_prev_state = ui.get_widget_state::<Self>(id, kind).map(|prev|{
            let PrevState { state, style, xy, dim, depth } = prev;
            (Some(style), State { state: state, xy: xy, dim: dim, depth: depth, })
        });
        let (maybe_prev_style, prev_state) = maybe_prev_state.unwrap_or_else(|| {
            (None, State { state: self.init_state(), dim: [0.0, 0.0], xy: [0.0, 0.0], depth: 0.0 })
        });

        // Determine the id of the canvas that the widget is attached to. If not given explicitly,
        // check the positioning to retrieve the Id from there.
        let maybe_canvas_id = self.canvas_id().or_else(|| ui.canvas_from_position(pos));

        // Update the widget's state.
        let maybe_new_state = {
            // Construct a UserInput for the widget.
            let user_input = ui.user_input(UiId::Widget(id), maybe_canvas_id);
            let Ui { ref theme, ref glyph_cache, .. } = *ui;
            self.update(&prev_state, xy, dim, user_input, &new_style, theme, glyph_cache)
        };

        // Check for whether or not the user input needs to be captured or uncaptured.
        {
            let new_state = match maybe_new_state {
                Some(ref new_state) => new_state,
                None => &prev_state.state
            };
            if Self::capture_mouse(&prev_state.state, new_state) {
                ui.mouse_captured_by(UiId::Widget(id));
            }
            if Self::uncapture_mouse(&prev_state.state, new_state) {
                ui.mouse_uncaptured_by(UiId::Widget(id));
            }
            if Self::capture_keyboard(&prev_state.state, new_state) {
                ui.keyboard_captured_by(UiId::Widget(id));
            }
            if Self::uncapture_keyboard(&prev_state.state, new_state) {
                ui.keyboard_uncaptured_by(UiId::Widget(id));
            }
        }

        // Determine whether or not the `State` has changed.
        let (state_has_changed, new_state) = {
            match maybe_new_state {
                Some(new_state) =>
                    (true, State { dim: dim, xy: xy, depth: depth, state: new_state }),
                None => {
                    let has_changed = xy != prev_state.xy
                        || dim != prev_state.dim
                        || depth != prev_state.depth;
                    (has_changed, State { dim: dim, xy: xy, depth: depth, ..prev_state })
                },
            }
        };

        // Determine whether or not the widget's `Style` has changed.
        let style_has_changed = match maybe_prev_style {
            Some(prev_style) => prev_style != new_style,
            None => false,
        };

        // Construct the widget's element.
        let maybe_new_element = if style_has_changed || state_has_changed {
            let Ui { ref theme, ref glyph_cache, .. } = *ui;
            Some(Self::draw(&new_state, &new_style, theme, glyph_cache))
        } else {
            None
        };

        // Store the new `State` and `Style` within the cache.
        let State { state, dim, xy, depth } = new_state;
        let store: Store<Self::State, Self::Style> = Store { state: state, style: new_style };
        ui.update_widget(id, maybe_canvas_id, kind, store, dim, xy, depth, maybe_new_element);
    }

    /// Optionally override with the case that the widget should capture the mouse.
    fn capture_mouse(_prev: &Self::State, _new: &Self::State) -> bool { false }

    /// Optionally override with the case that the widget should capture the mouse.
    fn uncapture_mouse(_prev: &Self::State, _new: &Self::State) -> bool { false }

    /// Optionally override with the case that the widget should capture the mouse.
    fn capture_keyboard(_prev: &Self::State, _new: &Self::State) -> bool { false }

    /// Optionally override with the case that the widget should capture the mouse.
    fn uncapture_keyboard(_prev: &Self::State, _new: &Self::State) -> bool { false }

    /// Return the kind of the widget as a &'static str. Note that this must be unique from all
    /// other widgets' "unique kinds". This is used by conrod to help avoid UiId errors.
    fn unique_kind(&self) -> &'static str;

    /// Return the initial `State` of the Widget. The `Ui` will only call this once.
    fn init_state(&self) -> Self::State;

    /// Return the styling of the widget. The `Ui` will call this once prior to each `update`. It
    /// does this so that it can check for differences in `Style` in case a new `Element` needs to
    /// be constructed.
    fn style(&self) -> Self::Style;

    /// Return the canvas to which the Widget will be attached, if there is one.
    fn canvas_id(&self) -> Option<CanvasId> { None }

    /// Your widget's previous state is given to you as a parameter and it is your job to
    /// construct and return an Update that will be used to update the widget's cached state.
    fn update<'a, C>(self,
                     prev: &State<Self::State>,
                     xy: Point,
                     dim: Dimensions,
                     input: UserInput<'a>,
                     current_style: &Self::Style,
                     theme: &Theme,
                     glyph_cache: &GlyphCache<C>) -> Option<Self::State>
        where C: CharacterCache;

    /// Construct a renderable Element from the current styling and new state. This will *only* be
    /// called on the occasion that the widget's `Style` or `State` has changed. Keep this in mind
    /// when designing your widget's `Style` and `State` types.
    fn draw<C>(new_state: &State<Self::State>,
               current_style: &Self::Style,
               theme: &Theme,
               glyph_cache: &GlyphCache<C>) -> Element
        where C: CharacterCache;

}

/// Represents the unique cached state of a widget.
#[derive(PartialEq)]
pub struct State<T> {
    /// The state of the Widget.
    pub state: T,
    /// The rectangular dimensions of the Widget.
    pub dim: Dimensions,
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
    /// Previous dimensions of the Widget.
    pub dim: Dimensions,
    /// Previous position of the Widget.
    pub xy: Point,
    /// Previous rendering depth of the Widget.
    pub depth: Depth,
}

/// The state type that we'll dynamically cast to and from Any for storage within the Cache.
#[derive(Debug)]
pub struct Store<Sta, Sty>
    where
        Sta: Any  + Debug,
        Sty: Any  + Debug,
{
    pub state: Sta,
    pub style: Sty,
}

/// A widget element for storage within the Ui's `widget_cache`.
#[derive(Debug)]
pub struct Cached {
    pub maybe_state: Option<Box<Any>>,
    pub kind: &'static str,
    pub dim: Dimensions,
    pub xy: Point,
    pub depth: Depth,
    pub element: Element,
    pub has_updated: bool,
    pub maybe_canvas_id: Option<CanvasId>,
}

impl Cached {

    /// Construct an empty Widget for a vacant widget position within the Ui.
    pub fn empty() -> Cached {
        Cached {
            maybe_state: None,
            kind: "EMPTY",
            dim: [0.0, 0.0],
            xy: [0.0, 0.0],
            depth: 0.0,
            element: ::elmesque::element::empty(),
            has_updated: false,
            maybe_canvas_id: None,
        }
    }

}

