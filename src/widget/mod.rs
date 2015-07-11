
use elmesque::Element;
use graphics::character::CharacterCache;
use position::{Depth, Dimensions, Padding, Point, Positionable, Sizeable};
use std::any::Any;
use std::fmt::Debug;
use theme::Theme;
use ui::{self, GlyphCache, Ui, UserInput};

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
///
/// Methods that *must* be overridden:
/// - unique_kind
/// - init_state
/// - style
/// - update
/// - draw
///
/// Methods that can be optionally overridden:
/// - parent_id
/// - capture_mouse
/// - uncapture_mouse
/// - capture_keyboard
/// - uncapture_keyboard
///
/// Methods that should not be overridden:
/// - set
///
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

    /// Return the kind of the widget as a &'static str. Note that this must be unique from all
    /// other widgets' "unique kinds". This is used by conrod to help avoid WidgetId errors and to
    /// provide better messages for those that do occur.
    fn unique_kind(&self) -> &'static str;

    /// Return the initial `State` of the Widget. The `Ui` will only call this once.
    fn init_state(&self) -> Self::State;

    /// Return the styling of the widget. The `Ui` will call this once prior to each `update`. It
    /// does this so that it can check for differences in `Style` in case a new `Element` needs to
    /// be constructed.
    fn style(&self) -> Self::Style;

    /// Your widget's previous state is given to you as a parameter and it is your job to
    /// construct and return an Update that will be used to update the widget's cached state.
    /// You only have to return `Some` state if the resulting state would be different to `prev`.
    /// If `Some` new state was returned, `Widget::draw` will be called in order to construct an
    /// up to date `Element`.
    ///
    /// # Arguments
    /// * prev - The previous state of the Widget. If none existed, `Widget::init_state` will be
    /// used to pass the initial state instead.
    /// * xy - The coordinates representing the middle of the widget.
    /// * dim - The dimensions of the widget.
    /// * input - A view into the current state of the user input (i.e. mouse and keyboard).
    /// * current_style - The style just produced by the `Widget::style` method.
    /// * theme - The currently active `Theme` within the `Ui`.
    /// * glyph_cache - Used for determining the size of rendered text if necessary.
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
    ///
    /// # Arguments
    /// * new_state - The freshly produced State which contains the unique widget info necessary
    /// for rendering.
    /// * current_style - The freshly produced `Style` of the widget.
    /// * theme - The currently active `Theme` within the `Ui`.
    /// * glyph_cache - Used for determining the size of rendered text if necessary.
    fn draw<C>(new_state: &State<Self::State>,
               current_style: &Self::Style,
               theme: &Theme,
               glyph_cache: &GlyphCache<C>) -> Element
        where C: CharacterCache;

    /// Return the parent to which the Widget will be attached, if there is one. Note that the
    /// WidgetId can also normally be inferred by the widget's `Position`, however calling this
    /// method will override this behaviour.
    fn parent_id(&self) -> Option<WidgetId> { None }

    /// Optionally override with the case that the widget should capture the mouse.
    fn capture_mouse(_prev: &Self::State, _new: &Self::State) -> bool { false }

    /// Optionally override with the case that the widget should capture the mouse.
    fn uncapture_mouse(_prev: &Self::State, _new: &Self::State) -> bool { false }

    /// Optionally override with the case that the widget should capture the mouse.
    fn capture_keyboard(_prev: &Self::State, _new: &Self::State) -> bool { false }

    /// Optionally override with the case that the widget should capture the mouse.
    fn uncapture_keyboard(_prev: &Self::State, _new: &Self::State) -> bool { false }

    /// Note: There should be no need to override this method.
    ///
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
        let maybe_prev_state = ui::get_widget_state::<C, Self>(ui, id, kind).map(|prev|{
            let Cached { state, style, xy, dim, depth, kid_area_xy, kid_area_dim, kid_area_pad } =
                prev;
            (Some(style), State { state: state, xy: xy, dim: dim, depth: depth, })
        });
        let (maybe_prev_style, prev_state) = maybe_prev_state.unwrap_or_else(|| {
            (None, State { state: self.init_state(), dim: [0.0, 0.0], xy: [0.0, 0.0], depth: 0.0 })
        });

        // Determine the id of the canvas that the widget is attached to. If not given explicitly,
        // check the positioning to retrieve the Id from there.
        let maybe_parent_id = self.parent_id().or_else(|| ui::parent_from_position(ui, pos));

        // Update the widget's state.
        let maybe_new_state = {
            // Construct a UserInput for the widget.
            let user_input = ui::user_input(ui, id);
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
                ui::mouse_captured_by(ui, id);
            }
            if Self::uncapture_mouse(&prev_state.state, new_state) {
                ui::mouse_uncaptured_by(ui, id);
            }
            if Self::capture_keyboard(&prev_state.state, new_state) {
                ui::keyboard_captured_by(ui, id);
            }
            if Self::uncapture_keyboard(&prev_state.state, new_state) {
                ui::keyboard_uncaptured_by(ui, id);
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
        let cached: Cached<Self> = Cached {
            state: state,
            style: new_style,
            dim: dim,
            xy: xy,
            depth: depth,
            // TODO: Implement these proplerly
            kid_area_xy: xy,
            kid_area_dim: dim,
            kid_area_pad: Padding::none(),
        };
        ui::update_widget(ui, id, maybe_parent_id, kind, cached, maybe_new_element);
    }

}

// /// A struct containing builder data common to all Widget types.
// pub struct CommonBuilder {
//     pub maybe_width: Option<Scalar>,
//     pub maybe_height: Option<Scalar>,
//     pub maybe_pos: Option<Position>,
//     pub maybe_h_align: Option<HorizontalAlign>,
//     pub maybe_v_align: Option<VerticalAlign>,
// }

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
pub struct Cached<W> where W: Widget {
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
    /// The position of the area in which child widgets are placed.
    pub kid_area_xy: Point,
    /// The dimensions of the area in which child widgets are placed.
    pub kid_area_dim: Dimensions,
    /// Padding of the area in which child widgets are placed.
    pub kid_area_pad: Padding,
}

