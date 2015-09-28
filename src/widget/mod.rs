
use Scalar;
use clock_ticks::precise_time_ns;
use elmesque::Element;
use graph::NodeIndex;
use graphics::character::CharacterCache;
use position::{Depth, Dimensions, Direction, Padding, Point, Position, Positionable, Sizeable,
               HorizontalAlign, VerticalAlign};
use std::any::Any;
use theme::Theme;
use ui::{self, GlyphCache, Ui, UserInput};

pub use self::index::Index;

pub mod drag;
mod index;
pub mod scroll;

// Widget Modules.
pub mod button;
pub mod canvas;
pub mod drop_down_list;
pub mod envelope_editor;
pub mod label;
pub mod matrix;
pub mod number_dialer;
pub mod slider;
pub mod split;
pub mod tabs;
pub mod text_box;
pub mod toggle;
pub mod xy_pad;


/// Unique widget identifier. Each widget must use a unique `WidgetId` so that it's state can be
/// cached within the `Ui` type. The reason we use a usize is because widgets are cached within
/// a `Vec`, which is limited to a size of `usize` elements.
pub type WidgetId = usize;


/// Arguments for the `Widget::update` method in a struct to simplify the method signature.
pub struct UpdateArgs<'a, W, C: 'a> where W: Widget {
    /// W's unique index.
    pub idx: Index,
    /// The Widget's state that was last returned by the update method.
    pub prev_state: &'a State<W::State>,
    /// The absolute (centered origin) screen position of the widget.
    pub xy: Point,
    /// The dimensions of the Widget.
    pub dim: Dimensions,
    /// The Widget's current Style.
    pub style: &'a W::Style,
    /// Restricted access to the `Ui`.
    /// Provides methods for immutably accessing the `Ui`'s `Theme` and `GlyphCache`.
    /// Also allows calling `Widget::set_internal` within the `Widget::update` method.
    pub ui: UiCell<'a, C>,
}

/// A wrapper around a `Ui` that only exposes the functionality necessary for the `Widget::update`
/// method. Its primary role is to allow for widget designers to compose their own unique widgets
/// from other widgets by calling the `Widget::set_internal` method within their own `Widget`'s
/// update method. It also provides methods for accessing the `Ui`'s `Theme` and `GlyphCache` via
/// immutable reference.
///
/// BTW - if you have a better name for this type, please post an issue or PR! "Cell" was the best
/// I could come up with as it's kind of like a jail cell for the `Ui` - restricting a user's
/// access to it.
pub struct UiCell<'a, C: 'a> {
    /// A mutable reference to a `Ui`.
    ui: &'a mut Ui<C>,
    /// The index of the Widget that "owns" the `UiCell`. The index is needed so that we can
    /// correctly retrieve user input information for the specific widget.
    idx: Index,
}

/// Arguments for the `Widget::draw` method in a struct to simplify the method signature.
pub struct DrawArgs<'a, W, C: 'a> where W: Widget {
    /// The current state of the Widget.
    pub state: &'a State<W::State>,
    /// The current style of the Widget.
    pub style: &'a W::Style,
    /// The active `Theme` within the `Ui`.
    pub theme: &'a Theme,
    /// The `Ui`'s GlyphCache (for determining text width, etc).
    pub glyph_cache: &'a GlyphCache<C>,
}

/// The area upon which a Widget's child widgets will be placed.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct KidArea {
    /// The coords of the centre of the rectangle.
    pub xy: Point,
    /// The dimensions of the area.
    pub dim: Dimensions,
    /// The distance between the edge of the area and where the widgets will be placed.
    pub pad: Padding,
}

/// The builder argument for the widget's Parent.
#[derive(Copy, Clone, Debug, PartialEq, RustcEncodable, RustcDecodable)]
pub enum MaybeParent {
    /// The user specified the widget should not have any parents, so the Root will be used.
    None,
    /// The user gave a specific parent widget.
    Some(Index),
    /// No parent widget was specified, so we will assume they want the last parent.
    Unspecified,
}

/// State necessary for Floating widgets.
#[derive(Copy, Clone, Debug, PartialEq, RustcEncodable, RustcDecodable)]
pub struct Floating {
    /// The time the canvas was last clicked (used for depth sorting in graph).
    pub time_last_clicked: u64,
}

/// A struct containing builder data common to all Widget types.
/// This type allows us to do a blanket impl of Positionable and Sizeable for T: Widget.
#[derive(Clone, Copy, Debug, RustcEncodable, RustcDecodable)]
pub struct CommonBuilder {
    /// The width of a Widget.
    pub maybe_width: Option<Scalar>,
    /// The height of a Widget.
    pub maybe_height: Option<Scalar>,
    /// The position of a Widget.
    pub maybe_position: Option<Position>,
    /// The horizontal alignment of a Widget.
    pub maybe_h_align: Option<HorizontalAlign>,
    /// The vertical alignment of a Widget.
    pub maybe_v_align: Option<VerticalAlign>,
    /// The rendering Depth of the Widget.
    pub maybe_depth: Option<Depth>,
    /// The parent widget of the Widget.
    pub maybe_parent_idx: MaybeParent,
    /// Whether or not the Widget is a "floating" Widget.
    pub is_floating: bool,
    /// Builder data for scrollable widgets.
    pub scrolling: scroll::Scrolling,
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
    /// The current state of the dragged widget, if it is draggable.
    pub drag_state: drag::State,
    /// Floating state for the widget if it is floating.
    pub maybe_floating: Option<Floating>,
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
    /// The current state of the dragged widget, if it is draggable.
    pub drag_state: drag::State,
    /// The area in which child widgets are placed.
    pub kid_area: KidArea,
    /// Whether or not the Widget is a "floating" Widget.
    pub maybe_floating: Option<Floating>,
    /// The state for scrollable widgets.
    pub maybe_scrolling: Option<scroll::State>,
}


/// This allows us to be generic over both Ui and UiCell in the `Widget::set` arguments.
trait UiRefMut<C> {
    /// A mutable reference to the `Ui`.
    fn ui_ref_mut(&mut self) -> &mut Ui<C>;
}

impl<C> UiRefMut<C> for Ui<C> {
    fn ui_ref_mut(&mut self) -> &mut Ui<C> { self }
}

impl<'a, C> UiRefMut<C> for UiCell<'a, C> {
    fn ui_ref_mut(&mut self) -> &mut Ui<C> { self.ui }
}


/// A trait to be implemented by all Widget types.
///
/// Methods that *must* be overridden:
/// - common_mut
/// - common
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
/// - default_position
/// - default_width
/// - default_height
/// - default_h_align
/// - default_v_align
///
/// Methods that should not be overridden:
/// - parent
/// - set
/// - set_internal
pub trait Widget: Sized {
    /// State to be stored within the `Ui`s widget cache. Take advantage of this type for any large
    /// allocations that you would like to avoid repeating between updates, or any calculations
    /// that you'd like to avoid repeating between calls to `update` and `draw`. Conrod will never
    /// clone the state, it will only ever be moved.
    type State: Any + PartialEq + ::std::fmt::Debug;
    /// Styling used by the widget to construct an Element. Styling is useful to have in its own
    /// abstraction in order to making Theme serializing easier. Conrod doesn't yet support
    /// serializing non-internal widget styling with the `Theme` type, but we hope to soon.
    type Style: Any + PartialEq + ::std::fmt::Debug;

    /// Return a reference to a CommonBuilder struct owned by the Widget.
    /// This method allows us to do a blanket impl of Positionable and Sizeable for T: Widget.
    fn common(&self) -> &CommonBuilder;

    /// Return a mutable reference to a CommonBuilder struct owned by the Widget.
    /// This method allows us to do a blanket impl of Positionable and Sizeable for T: Widget.
    fn common_mut(&mut self) -> &mut CommonBuilder;

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
    fn update<'a, C>(self, args: UpdateArgs<'a, Self, C>) -> Option<Self::State>
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
    fn draw<'a, C>(args: DrawArgs<'a, Self, C>) -> Element
        where C: CharacterCache;

    /// The default Position for the widget.
    /// This is used when no Position is explicitly given when instantiating the Widget.
    fn default_position(&self, _theme: &Theme) -> Position {
        Position::Direction(Direction::Down, 20.0, None)
    }

    /// The default horizontal alignment for the widget.
    /// This is used when no HorizontalAlign is explicitly given when instantiating a Widget.
    fn default_h_align(&self, theme: &Theme) -> HorizontalAlign {
        theme.align.horizontal
    }

    /// The default vertical alignment for the widget.
    /// This is used when no VerticalAlign is explicitly given when instantiating a Widget.
    fn default_v_align(&self, theme: &Theme) -> VerticalAlign {
        theme.align.vertical
    }

    /// The default width of the widget.
    /// A reference to the GlyphCache is provided in case the width should adjust to some text len.
    /// This method is only used if no width or dimensions are given.
    fn default_width<C: CharacterCache>(&self, _theme: &Theme, _: &GlyphCache<C>) -> Scalar {
        0.0
    }

    /// The default height of the widget.
    /// This method is only used if no height or dimensions are given.
    fn default_height(&self, _theme: &Theme) -> Scalar {
        0.0
    }

    /// Optionally override with the case that the widget should capture the mouse.
    fn capture_mouse(_prev: &Self::State, _new: &Self::State) -> bool { false }

    /// Optionally override with the case that the widget should capture the mouse.
    fn uncapture_mouse(_prev: &Self::State, _new: &Self::State) -> bool { false }

    /// Optionally override with the case that the widget should capture the mouse.
    fn capture_keyboard(_prev: &Self::State, _new: &Self::State) -> bool { false }

    /// Optionally override with the case that the widget should capture the mouse.
    fn uncapture_keyboard(_prev: &Self::State, _new: &Self::State) -> bool { false }

    /// If the widget is draggable, implement this method and return the position an dimensions
    /// of the draggable space. The position should be relative to the center of the widget.
    fn drag_area(&self,
                 _dim: Dimensions,
                 _style: &Self::Style,
                 _theme: &Theme) -> Option<drag::Area>
    {
        None
    }

    /// The area on which child widgets will be placed when using the `Place` `Position` methods.
    fn kid_area(state: &State<Self::State>, _style: &Self::Style, _theme: &Theme) -> KidArea {
        KidArea {
            xy: state.xy,
            dim: state.dim,
            pad: Padding::none(),
        }
    }

    /// Set the parent widget for this Widget by passing the WidgetId of the parent.
    /// This will attach this Widget to the parent widget.
    fn parent<I: Into<Index>>(mut self, maybe_parent_idx: Option<I>) -> Self {
        self.common_mut().maybe_parent_idx = match maybe_parent_idx {
            None => MaybeParent::None,
            Some(idx) => MaybeParent::Some(idx.into()),
        };
        self
    }

    /// Set whether or not the widget is floating (the default is `false`).
    /// A typical example of a floating widget would be a pop-up or alert window.
    ///
    /// A "floating" widget will always be rendered *after* its parent tree and all widgets
    /// connected to its parent tree. If two sibling widgets are both floating, then the one that
    /// was last clicked will be rendered last. If neither are clicked, they will be rendered in
    /// the order in which they were cached into the `Ui`.
    fn floating(mut self, is_floating: bool) -> Self {
        self.common_mut().is_floating = is_floating;
        self
    }

    /// Set whether or not the widget's `KidArea` is scrollable (the default is false).
    /// If a widget is scrollable and it has children widgets that fall outside of its `KidArea`,
    /// the `KidArea` will become scrollable.
    fn scrolling(mut self, scrollable: bool) -> Self {
        self.common_mut().scrolling.vertical = scrollable;
        self.common_mut().scrolling.horizontal = scrollable;
        self
    }

    /// Set whether or not the widget's `KidArea` is scrollable (the default is false).
    /// If a widget is scrollable and it has children widgets that fall outside of its `KidArea`,
    /// the `KidArea` will become scrollable.
    fn vertical_scrolling(mut self, scrollable: bool) -> Self {
        self.common_mut().scrolling.vertical = scrollable;
        self
    }

    /// Set whether or not the widget's `KidArea` is scrollable (the default is false).
    /// If a widget is scrollable and it has children widgets that fall outside of its `KidArea`,
    /// the `KidArea` will become scrollable.
    fn horizontal_scrolling(mut self, scrollable: bool) -> Self {
        self.common_mut().scrolling.horizontal = scrollable;
        self
    }

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
    fn set<C, U>(self, id: WidgetId, ui: &mut U) where
        C: CharacterCache,
        U: UiRefMut<C>,
    {
        set_widget(self, Index::Public(id), ui.ui_ref_mut());
    }

    /// Set the widget within the `Ui` that is stored within the given `UiCell` without occupying
    /// the `WidgetId` space.
    ///
    /// This method is designed to be an alternative to `Widget::set`, to be used by Widget
    /// designers when composing `Widget`s out of other `Widgets`.
    fn set_internal<'a, C>(self, maybe_idx: &mut Option<NodeIndex>, ui_cell: &mut UiCell<'a, C>)
        where C: CharacterCache
    {

        let idx = match maybe_idx {
            // If we don't yet have a NodeIndex for our internal `Widget` we'll add a placeholder
            // to the `Graph` and use the returned `NodeIndex`.
            maybe_idx @ &mut None => {
                let idx = ui::widget_graph_mut(&mut ui_cell.ui).add_placeholder();
                *maybe_idx = Some(idx);
                idx
            },
            // Otherwise, we'll use the index we already have.
            &mut Some(idx) => idx,
        };

        set_widget(self, Index::Internal(idx), &mut ui_cell.ui);
    }

}



/// Sets a given widget within the given `Ui`.
/// If it is the first time a widget has been set, it will be cached into the `Ui`'s widget_graph.
/// For all following occasions, the pre-existing cached state will be compared and updated.
fn set_widget<'a, W, C>(widget: W, idx: Index, ui: &mut Ui<C>) where
    W: Widget,
    C: CharacterCache,
{
    let kind = widget.unique_kind();

    // Collect the previous state of the widget if there is some to collect.
    let maybe_widget_state: Option<Cached<W>> = {

        // If the cache is already initialised for a widget of a different kind, warn the user.
        let check_container_kind = |container: &mut ::graph::Container| {
            use std::io::Write;
            if container.kind != kind {
                writeln!(::std::io::stderr(),
                         "A widget of a different kind already exists at the given WidgetId \
                         ({:?}). You tried to insert a {:?}, however the existing widget is a \
                         {:?}. Check your widgets' `WidgetId`s for errors.",
                          idx, kind, container.kind).unwrap();
                return None;
            } else {
                container.take_widget_state()
            }
        };

        ui::widget_graph_mut(ui).get_widget_mut(idx).and_then(check_container_kind)
    };

    // Seperate the Widget's previous state into it's unique state, style and scrolling.
    let (maybe_prev_state, maybe_prev_style, maybe_scrolling) = maybe_widget_state.map(|prev|{

        // Destructure the cached state.
        let Cached {
            state,
            style,
            xy,
            dim,
            depth,
            drag_state,
            maybe_floating,
            maybe_scrolling,
            //kid_area,
            ..
        } = prev;

        // Use the cached state to construct the prev_state (to be fed to Widget::update).
        let prev_state = State {
            state: state,
            xy: xy,
            dim: dim,
            depth: depth,
            drag_state: drag_state,
            maybe_floating: maybe_floating,
        };

        (Some(prev_state), Some(style), maybe_scrolling)
    }).unwrap_or_else(|| (None, None, None));

    let new_style = widget.style();
    let depth = widget.get_depth();
    let dim = widget.get_dimensions(&ui.theme, &ui.glyph_cache);
    let pos = widget.get_position(&ui.theme);

    let (xy, drag_state) = {
        // A function for generating the xy coords from the given alignment and Position.
        let gen_xy = || {
            let (h_align, v_align) = widget.get_alignment(&ui.theme);
            let new_xy = ui.get_xy(Some(idx), pos, dim, h_align, v_align);
            new_xy
        };

        // Check to see if the widget is currently being dragged and return the new xy / drag.
        match maybe_prev_state {
            // If there is no previous state to compare for dragging, return an initial state.
            None => (gen_xy(), drag::State::Normal),
            Some(ref prev_state) => {
                let maybe_mouse = ui::get_mouse_state(ui, idx);
                let maybe_drag_area = widget.drag_area(dim, &new_style, &ui.theme);
                match maybe_drag_area {
                    // If the widget isn't draggable, generate its position the normal way.
                    // FIXME: This may cause issues in the case that a widget's draggable area
                    // is dynamic (i.e. sometimes its Some, other times its None).
                    // Specifically, if a widget is dragged somewhere and then it returns None,
                    // it will snap back to the position produced by gen_xy. We should keep
                    // track of whether or not a widget `has_been_dragged` to see if we should
                    // leave it at its previous xy or use gen_xy.
                    None => (gen_xy(), drag::State::Normal),
                    Some(drag_area) => match maybe_mouse {
                        // If there is some draggable area and mouse, drag the xy.
                        Some(mouse) => {
                            let drag_state = prev_state.drag_state;

                            // Drag the xy of the widget and return the new xy.
                            drag::drag_widget(prev_state.xy, drag_area, drag_state, mouse)
                        },
                        // Otherwise just return the regular xy and drag state.
                        None => (prev_state.xy, drag::State::Normal),
                    },
                }
            },
        }
    };

    // Check whether we have stopped / started dragging the widget and in turn whether or not
    // we need to capture the mouse.
    match (maybe_prev_state.as_ref().map(|prev| prev.drag_state), drag_state) {
        (Some(drag::State::Highlighted), drag::State::Clicked(_)) =>
            ui::mouse_captured_by(ui, idx),
        (Some(drag::State::Clicked(_)), drag::State::Highlighted) |
        (Some(drag::State::Clicked(_)), drag::State::Normal)      =>
            ui::mouse_uncaptured_by(ui, idx),
        _ => (),
    }

    // Determine the id of the canvas that the widget is attached to. If not given explicitly,
    // check the positioning to retrieve the Id from there.
    let maybe_parent_idx = match widget.common().maybe_parent_idx {
        MaybeParent::Some(idx) => Some(idx),
        MaybeParent::None => None,
        MaybeParent::Unspecified => ui::parent_from_position(ui, pos),
    };

    // Collect whether or not the widget is "floating" before `widget` gets consumed so that we
    // can store it in our widget::Cached later in the function.
    let is_floating = widget.common().is_floating;

    // If it is floating, check to see if we need to update the last time it was clicked.
    let new_floating = || Floating { time_last_clicked: precise_time_ns() };
    let maybe_floating = match (is_floating, maybe_prev_state.as_ref()) {
        (false, _) => None,
        (true, Some(prev)) => {
            let maybe_mouse = ui::get_mouse_state(ui, idx);
            match (prev.maybe_floating, maybe_mouse) {
                (Some(prev_floating), Some(mouse)) => {
                    if mouse.left == ::mouse::ButtonState::Down {
                        Some(new_floating())
                    } else {
                        Some(prev_floating)
                    }
                },
                (Some(prev_floating), None) => Some(prev_floating),
                _ => Some(new_floating()),
            }
        },
        (true, None) => Some(new_floating()),
    };

    // Determine whether or not this is the first time set has been called.
    // We'll use this to determine whether or not we need to call Widget::draw.
    let is_first_set = maybe_prev_state.is_none();

    // Check whether or not our widget is scrollable before widget is moved into `update`.
    let scrolling = widget.common().scrolling;

    // Unwrap the previous state. If there is no previous state to unwrap, call the
    // `init_state` method to use the initial state as the prev_state.
    let prev_state = maybe_prev_state.unwrap_or_else(|| State {
        state: widget.init_state(),
        xy: xy,
        dim: dim,
        depth: depth,
        drag_state: drag_state,
        maybe_floating: maybe_floating,
    });

    // Update the widget's state.
    let maybe_new_state = {
        // Construct a UserInput for the widget.
        let args = UpdateArgs {
            idx: idx,
            prev_state: &prev_state,
            xy: xy,
            dim: dim,
            style: &new_style,
            ui: UiCell { ui: ui, idx: idx },
        };
        widget.update(args)
    };

    // Check for whether or not the user input needs to be captured or uncaptured.
    {
        let new_state = match maybe_new_state {
            Some(ref new_state) => new_state,
            None => &prev_state.state
        };
        if W::capture_mouse(&prev_state.state, new_state) {
            ui::mouse_captured_by(ui, idx);
        }
        if W::uncapture_mouse(&prev_state.state, new_state) {
            ui::mouse_uncaptured_by(ui, idx);
        }
        if W::capture_keyboard(&prev_state.state, new_state) {
            ui::keyboard_captured_by(ui, idx);
        }
        if W::uncapture_keyboard(&prev_state.state, new_state) {
            ui::keyboard_uncaptured_by(ui, idx);
        }
    }

    // Determine whether or not the `State` has changed.
    let state_has_changed = match maybe_new_state {
        Some(_) => true,
        None => xy != prev_state.xy
            || dim != prev_state.dim
            || depth != prev_state.depth
            || is_first_set,
    };

    // Determine whether or not the widget's `Style` has changed.
    let style_has_changed = match maybe_prev_style {
        Some(prev_style) => prev_style != new_style,
        None => false,
    };

    // Construct the resulting new State to be passed to the `draw` method.
    let new_state = State {
        state: maybe_new_state.unwrap_or_else(move || {
            let State { state, .. } = prev_state;
            state
        }),
        dim: dim,
        xy: xy,
        depth: depth,
        drag_state: drag_state,
        maybe_floating: maybe_floating,
    };

    // Retrieve the area upon which kid widgets will be placed.
    let kid_area = W::kid_area(&new_state, &new_style, &ui.theme);

    // Calc the max offset given the length of the visible area along with the total length.
    fn calc_max_offset(visible_len: Scalar, total_len: Scalar) -> Scalar {
        visible_len - (visible_len / total_len) * visible_len
    }

    // Determine whether or not we have state for scrolling.
    let maybe_new_scrolling = {

        let maybe_mouse = ui::get_mouse_state(ui, idx);

        // If we haven't been placed in the graph yet (and bounding_box returns None),
        // we'll just use our current dimensions as the bounding box.
        let init_bounds = || {
            let half_h = kid_area.dim[1] / 2.0;
            let half_w = kid_area.dim[0] / 2.0;
            (half_h, -half_h, -half_w, half_w)
        };

        // If we have neither vertical or horizontal scrolling, return None.
        if !scrolling.horizontal && !scrolling.vertical {
            None

        // Else if we have some previous scrolling, use it in determining the new scrolling.
        } else if let Some(prev_scrollable) = maybe_scrolling {
            let (top_y, bottom_y, left_x, right_x) = ui::widget_graph(ui)
                .bounding_box(false, None, true, idx)
                .unwrap_or_else(init_bounds);

            // The total length of the area occupied by child widgets that is scrolled.
            let total_v_length = top_y - bottom_y;
            let total_h_length = right_x - left_x;

            let scroll_state = scroll::State {

                // Vertical scrollbar state.
                maybe_vertical: if scrolling.vertical {
                    Some(scroll::Bar {
                        interaction: prev_scrollable.maybe_vertical.as_ref()
                            .map(|bar| bar.interaction)
                            .unwrap_or(scroll::Interaction::Normal),
                        offset: prev_scrollable.maybe_vertical.as_ref()
                            .map(|bar| bar.offset)
                            .unwrap_or_else(|| {
                                top_y - (kid_area.xy[1] + kid_area.dim[1] / 2.0)
                            }),
                        max_offset: calc_max_offset(kid_area.dim[1], total_v_length),
                        total_length: total_v_length,
                    })
                } else {
                    None
                },

                // Horizontal scrollbar state.
                maybe_horizontal: if scrolling.horizontal {
                    Some(scroll::Bar {
                        interaction: prev_scrollable.maybe_horizontal.as_ref()
                            .map(|bar| bar.interaction)
                            .unwrap_or(scroll::Interaction::Normal),
                        offset: prev_scrollable.maybe_horizontal.as_ref()
                            .map(|bar| bar.offset)
                            .unwrap_or_else(|| {
                                (kid_area.xy[0] - kid_area.dim[0] / 2.0) - left_x
                            }),
                        max_offset: calc_max_offset(kid_area.dim[0], total_h_length),
                        total_length: total_h_length,
                    })
                } else {
                    None
                },

                width: scrolling.style.width(&ui.theme),
                color: scrolling.style.color(&ui.theme),
            };

            Some(scroll::update(&kid_area, &scroll_state, maybe_mouse)
                .unwrap_or_else(|| scroll_state))

        // Otherwise, we'll make a brand new scrolling.
        } else {
            let (top_y, bottom_y, left_x, right_x) = ui::widget_graph(ui)
                .bounding_box(false, None, true, idx)
                .unwrap_or_else(init_bounds);

            // The total length of the area occupied by child widgets that is scrolled.
            let total_v_length = top_y - bottom_y;
            let total_h_length = right_x - left_x;

            let scroll_state = scroll::State {

                // The initial vertical scrollbar state.
                maybe_vertical: if scrolling.vertical {
                    Some(scroll::Bar {
                        interaction: scroll::Interaction::Normal,
                        offset: 0.0,
                        max_offset: calc_max_offset(kid_area.dim[1], total_v_length),
                        total_length: total_v_length,
                    })
                } else {
                    None
                },

                // The initial horizontal scrollbar state.
                maybe_horizontal: if scrolling.horizontal {
                    Some(scroll::Bar {
                        interaction: scroll::Interaction::Normal,
                        offset: 0.0,
                        max_offset: calc_max_offset(kid_area.dim[0], total_h_length),
                        total_length: total_h_length,
                    })
                } else {
                    None
                },

                width: scrolling.style.width(&ui.theme),
                color: scrolling.style.color(&ui.theme),
            };

            Some(scroll::update(&kid_area, &scroll_state, maybe_mouse)
                .unwrap_or_else(|| scroll_state))
        }
    };

    // Check whether or not our new scrolling state should capture or uncapture the mouse.
    if let (Some(ref prev), Some(ref new)) = (maybe_scrolling, maybe_new_scrolling) {
        if scroll::capture_mouse(prev, new) {
            ui::mouse_captured_by(ui, idx);
        }
        if scroll::uncapture_mouse(prev, new) {
            ui::mouse_uncaptured_by(ui, idx);
        }
    }

    // We need to know if the scroll state has changed to see if we need to redraw.
    let scroll_has_changed = maybe_new_scrolling != maybe_scrolling;

    // Construct the widget's element.
    let maybe_new_element = if style_has_changed || state_has_changed || scroll_has_changed {

        // Inform the `Ui` that we'll need a redraw.
        ui.needs_redraw();

        let args = DrawArgs {
            state: &new_state,
            style: &new_style,
            theme: &ui.theme,
            glyph_cache: &ui.glyph_cache,
        };
        Some(W::draw(args))
    } else {
        None
    };

    // Store the new `State` and `Style` within the cache.
    let State {
        state,
        dim,
        xy,
        depth,
        drag_state,
        maybe_floating,
        //maybe_scrolling,
    } = new_state;

    let cached: Cached<W> = Cached {
        state: state,
        style: new_style,
        dim: dim,
        xy: xy,
        depth: depth,
        drag_state: drag_state,
        kid_area: kid_area,
        maybe_floating: maybe_floating,
        maybe_scrolling: maybe_new_scrolling,
    };

    ui::update_widget(ui, idx, maybe_parent_idx, kind, cached, maybe_new_element);
}


impl<'a, C> UiCell<'a, C> {

    /// A reference to the `Theme` that is currently active within the `Ui`.
    pub fn theme(&self) -> &Theme { &self.ui.theme }

    /// A reference to the `Ui`'s `GlyphCache`.
    pub fn glyph_cache(&self) -> &GlyphCache<C> { &self.ui.glyph_cache }

    /// A struct representing the user input that has occurred since the last update.
    pub fn input(&self) -> UserInput {
        ui::user_input(self.ui, self.idx)
    }

}


impl CommonBuilder {
    /// Construct an empty, initialised CommonBuilder.
    pub fn new() -> CommonBuilder {
        CommonBuilder {
            maybe_width: None,
            maybe_height: None,
            maybe_position: None,
            maybe_h_align: None,
            maybe_v_align: None,
            maybe_depth: None,
            maybe_parent_idx: MaybeParent::Unspecified,
            is_floating: false,
            scrolling: scroll::Scrolling::new(),
        }
    }
}


impl<T> Positionable for T where T: Widget {
    #[inline]
    fn position(mut self, pos: Position) -> Self {
        self.common_mut().maybe_position = Some(pos);
        self
    }
    #[inline]
    fn get_position(&self, theme: &Theme) -> Position {
        self.common().maybe_position.unwrap_or(self.default_position(theme))
    }
    #[inline]
    fn horizontal_align(mut self, h_align: HorizontalAlign) -> Self {
        self.common_mut().maybe_h_align = Some(h_align);
        self
    }
    #[inline]
    fn vertical_align(mut self, v_align: VerticalAlign) -> Self {
        self.common_mut().maybe_v_align = Some(v_align);
        self
    }
    #[inline]
    fn get_horizontal_align(&self, theme: &Theme) -> HorizontalAlign {
        self.common().maybe_h_align.unwrap_or(self.default_h_align(theme))
    }
    #[inline]
    fn get_vertical_align(&self, theme: &Theme) -> VerticalAlign {
        self.common().maybe_v_align.unwrap_or(self.default_v_align(theme))
    }
    #[inline]
    fn depth(mut self, depth: Depth) -> Self {
        self.common_mut().maybe_depth = Some(depth);
        self
    }
    #[inline]
    fn get_depth(&self) -> Depth {
        const DEFAULT_DEPTH: Depth = 0.0;
        self.common().maybe_depth.unwrap_or(DEFAULT_DEPTH)
    }
}

impl<T> Sizeable for T where T: Widget {
    #[inline]
    fn width(mut self, w: f64) -> Self {
        self.common_mut().maybe_width = Some(w);
        self
    }
    #[inline]
    fn height(mut self, h: f64) -> Self {
        self.common_mut().maybe_height = Some(h);
        self
    }
    #[inline]
    fn get_width<C: CharacterCache>(&self, theme: &Theme, glyph_cache: &GlyphCache<C>) -> f64 {
        self.common().maybe_width.unwrap_or(self.default_width(theme, glyph_cache))
    }
    #[inline]
    fn get_height(&self, theme: &Theme) -> f64 {
        self.common().maybe_height.unwrap_or(self.default_height(theme))
    }
}

