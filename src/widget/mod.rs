
use ::{CharacterCache, Scalar};
use clock_ticks::precise_time_ns;
use elmesque::Element;
use graph::NodeIndex;
use position::{Depth, Dimensions, Direction, Padding, Point, Position, Positionable, Sizeable,
               HorizontalAlign, VerticalAlign};
use std::any::Any;
use theme::Theme;
use ui::{self, GlyphCache, Ui, UserInput};

pub use self::id::Id;
pub use self::index::Index;

pub mod drag;
mod id;
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
    /// Also allows calling `Widget::set` within the `Widget::update` method.
    pub ui: UiCell<'a, C>,
}

/// A wrapper around a `Ui` that only exposes the functionality necessary for the `Widget::update`
/// method. Its primary role is to allow for widget designers to compose their own unique `Widget`s
/// from other `Widget`s by calling the `Widget::set` method within their own `Widget`'s
/// update method. It also provides methods for accessing the `Ui`'s `Theme`, `GlyphCache` and
/// `UserInput` via immutable reference.
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
    pub state: &'a W::State,
    /// The current style of the Widget.
    pub style: &'a W::Style,
    /// The active `Theme` within the `Ui`.
    pub theme: &'a Theme,
    /// The `Ui`'s GlyphCache (for determining text width, etc).
    pub glyph_cache: &'a GlyphCache<C>,
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

/// Arguments to the `Widget::kid_area` method in a struct to simplify the method signature.
pub struct KidAreaArgs<'a, W, C: 'a> where W: Widget {
    /// Current position of the Widget.
    pub xy: Point,
    /// Current Widget dimensions.
    pub dim: Dimensions,
    /// Current Style of the Widget.
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

/// Widget data to be cached prior to the `Widget::update` call in the `set_widget` function.
/// We do this so that if this Widget were to internally `set` some other `Widget`s, this
/// `Widget`s positioning and dimension data already exists within the `Graph` for reference.
pub struct PreUpdateCache {
    /// The `Widget`'s unique kind.
    pub kind: &'static str,
    /// The `Widget`'s unique Index.
    pub idx: Index,
    /// The widget's parent's unique index (if it has a parent).
    pub maybe_parent_idx: Option<Index>,
    /// If this widget is relatively positioned to another `Widget`, this will be the index of
    /// the `Widget` to which this `Widget` is relatively positioned
    pub maybe_positioned_relatively_idx: Option<Index>,
    /// The new position of the Widget.
    pub xy: Point,
    /// The new dimensions of the Widget.
    pub dim: Dimensions,
    /// The z-axis depth - affects the render order of sibling widgets.
    pub depth: Depth,
    /// The new KidArea for the Widget.
    pub kid_area: KidArea,
    /// The current state of the dragged widget, if it is draggable.
    pub drag_state: drag::State,
    /// Scrolling data for the Widget if there is some.
    pub maybe_floating: Option<Floating>,
    /// Scrolling data for the Widget if there is some.
    pub maybe_scrolling: Option<scroll::State>,
}

/// Widget data to be cached after the `Widget::update` call in the `set_widget` function.
/// We do this so that if this Widget were to internally `set` some other `Widget`s, this
/// `Widget`s positioning and dimension data already exists within the `Graph` for reference.
pub struct PostUpdateCache<W> where W: Widget {
    /// The `Widget`'s unique Index.
    pub idx: Index,
    /// The widget's parent's unique index (if it has a parent).
    pub maybe_parent_idx: Option<Index>,
    /// The newly produced unique `State` associated with the `Widget`.
    pub state: W::State,
    /// The newly produced unique `Style` associated with the `Widget`.
    pub style: W::Style,
    /// A new `Element` to use for the `Widget` if a new one has been produced.
    pub maybe_element: Option<Element>,
}


/// A trait that allows us to be generic over both Ui and UiCell in the `Widget::set` arguments.
trait UiRefMut {
    type CharacterCache: CharacterCache;
    /// A mutable reference to the `Ui`.
    fn ui_ref_mut(&mut self) -> &mut Ui<Self::CharacterCache>;
}

impl<C> UiRefMut for Ui<C> where C: CharacterCache {
    type CharacterCache = C;
    fn ui_ref_mut(&mut self) -> &mut Ui<C> { self }
}

impl<'a, C> UiRefMut for UiCell<'a, C> where C: CharacterCache {
    type CharacterCache = C;
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
    /// * idx - The `Widget`'s unique index (whether `Public` or `Internal`).
    /// * prev - The previous state of the Widget. If none existed, `Widget::init_state` will be
    /// used to pass the initial state instead.
    /// * xy - The coordinates representing the middle of the widget.
    /// * dim - The dimensions of the widget.
    /// * style - The style produced by the `Widget::style` method.
    /// * ui - A wrapper around the `Ui`, offering restricted access to its functionality. See the
    /// docs for `UiCell` for more details.
    fn update<C>(self, args: UpdateArgs<Self, C>) -> Option<Self::State>
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
    fn draw<C>(args: DrawArgs<Self, C>) -> Element
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
    fn kid_area<C: CharacterCache>(&self, args: KidAreaArgs<Self, C>) -> KidArea {
        KidArea {
            xy: args.xy,
            dim: args.dim,
            pad: Padding::none(),
        }
    }


    // None of the following methods should require overriding.


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
    fn set<I, U>(self, idx: I, ui: &mut U) where
        I: Into<Index>,
        U: UiRefMut,
    {
        set_widget(self, idx.into(), ui.ui_ref_mut());
    }

}



/// Updates the given widget and caches it within the given `Ui`'s `widget_graph`.
///
/// If it is the first time a widget has been set, it will be cached into the `Ui`'s widget_graph.
/// For all following occasions, the pre-existing cached state will be compared and updated.
///
/// Note that this is a very imperative, mutation oriented segment of code. We try to move as much
/// imperativeness and mutation out of the users hands and into this function as possible, so that 
/// users have a clear, consise, purely functional `Widget` API. As a result, we try to keep this
/// as verbosely annotated as possible. If anything is unclear, feel free to post an issue or PR
/// with concerns/improvements to the github repo.
fn set_widget<'a, C, W>(widget: W, idx: Index, ui: &mut Ui<C>) where
    C: CharacterCache,
    W: Widget,
{
    let kind = widget.unique_kind();

    // Take the previous state of the widget from the cache if there is some to collect.
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

    // We need to hold onto the current "previously set widget", as this may change during our
    // `Widget`'s update method (i.e. if it sets any of its own widgets, they will become the last
    // previous widget).
    let maybe_prev_widget_idx = ui.maybe_prev_widget();

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

    // Check whether or not the widget is a "floating" (hovering / pop-up style) widget.
    let maybe_floating = if widget.common().is_floating {

        fn new_floating() -> Floating {
            Floating { time_last_clicked: precise_time_ns() }
        }

        // If it is floating, check to see if we need to update the last time it was clicked.
        match maybe_prev_state.as_ref() {
            Some(prev) => {
                let maybe_mouse = ui::get_mouse_state(ui, idx);
                match (prev.maybe_floating, maybe_mouse) {
                    (Some(prev_floating), Some(mouse)) => {
                        if mouse.left.position == ::mouse::ButtonPosition::Down {
                            Some(new_floating())
                        } else {
                            Some(prev_floating)
                        }
                    },
                    (Some(prev_floating), None) => Some(prev_floating),
                    _ => Some(new_floating()),
                }
            },
            None => Some(new_floating()),
        }
    } else {
        None
    };

    // Retrieve the area upon which kid widgets will be placed.
    let kid_area = {
        let args = KidAreaArgs {
            xy: xy,
            dim: dim,
            style: &new_style,
            theme: &ui.theme,
            glyph_cache: &ui.glyph_cache,
        };
        widget.kid_area(args)
    };

    // Determine whether or not we have state for scrolling.
    let maybe_new_scrolling = {

        // Collect the scrolling input given via the widgets builder methods.
        let scrolling = widget.common().scrolling;

        // Calc the max offset given the length of the visible area along with the total length.
        fn calc_max_offset(visible_len: Scalar, total_len: Scalar) -> Scalar {
            visible_len - (visible_len / total_len) * visible_len
        }

        let maybe_mouse = ui::get_mouse_state(ui, idx);

        // If we haven't been placed in the graph yet (and bounding_box returns None),
        // we'll just use our current dimensions as the bounding box.
        let self_bounds = || {
            let half_h = kid_area.dim[1] / 2.0;
            let half_w = kid_area.dim[0] / 2.0;
            (half_h, -half_h, -half_w, half_w)
        };

        // Calculate the scroll bounds for the widget.
        let bounds = || ui::widget_graph(ui)
            .bounding_box(false, None, true, idx)
            .unwrap_or_else(self_bounds);

        // If we have neither vertical or horizontal scrolling, return None.
        if !scrolling.horizontal && !scrolling.vertical {
            None

        // Else if we have some previous scrolling, use it in determining the new scrolling.
        } else if let Some(prev_scrollable) = maybe_scrolling {
            let (top_y, bottom_y, left_x, right_x) = bounds();

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

                thickness: scrolling.style.thickness(&ui.theme),
                color: scrolling.style.color(&ui.theme),
            };

            Some(scroll::update(&kid_area, &scroll_state, maybe_mouse)
                .unwrap_or_else(|| scroll_state))

        // Otherwise, we'll make a brand new scrolling.
        } else {
            let (top_y, bottom_y, left_x, right_x) = bounds();

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

                thickness: scrolling.style.thickness(&ui.theme),
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

    // Determine whether or not this is the first time set has been called.
    // We'll use this to determine whether or not we need to call Widget::draw.
    let is_first_set = maybe_prev_state.is_none();

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

    // Update all positioning and dimension related data prior to calling `Widget::update`.
    // We do this so that if this widget were to internally `set` some other `Widget`s, this
    // `Widget`s positioning and dimension data already exists within the `Graph`.
    {
        // Some widget to which this widget is relatively positioned (if there is one).
        let maybe_positioned_relatively_idx = match pos {
            Position::Relative(_, _, maybe_idx)  |
            Position::Direction(_, _, maybe_idx) => maybe_idx.or(maybe_prev_widget_idx),
            _ => None,
        };

        // This will cache the given data into the `ui`'s `widget_graph`.
        ui::pre_update_cache(ui, PreUpdateCache {
            kind: kind,
            idx: idx,
            maybe_parent_idx: maybe_parent_idx,
            maybe_positioned_relatively_idx: maybe_positioned_relatively_idx,
            dim: dim,
            xy: xy,
            depth: depth,
            drag_state: drag_state,
            kid_area: kid_area,
            maybe_floating: maybe_floating,
            maybe_scrolling: maybe_new_scrolling,
        });
    }

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

    // Check for whether or not the user input needs to be captured or uncaptured. We check after
    // the widget update so that child widgets get the first oppotunity to capture input.
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
    let state_has_changed = maybe_new_state.is_some()
            || xy != prev_state.xy
            || dim != prev_state.dim
            || depth != prev_state.depth
            || is_first_set;

    // Determine whether or not the widget's `Style` has changed.
    let style_has_changed = maybe_prev_style.map(|style| style != new_style).unwrap_or(false);

    // We need to know if the scroll state has changed to see if we need to redraw.
    let scroll_has_changed = maybe_new_scrolling != maybe_scrolling;

    // We only need to redraw the `Element` if some visible part of our widget has changed.
    let requires_redraw = style_has_changed || state_has_changed || scroll_has_changed;

    // Our resulting `State` after having updated.
    let resulting_state = maybe_new_state
        .unwrap_or_else(move || { let State { state, .. } = prev_state; state });

    // If we require a redraw, we should draw a new `Element`.
    let maybe_new_element = if requires_redraw {
        Some(W::draw(DrawArgs {
            state: &resulting_state,
            style: &new_style,
            theme: &ui.theme,
            glyph_cache: &ui.glyph_cache,
            dim: dim,
            xy: xy,
            depth: depth,
            drag_state: drag_state,
            maybe_floating: maybe_floating,
        }))
    } else {
        None
    };

    // Finally, cache the `Widget`'s newly updated `State` and `Style` within the `ui`'s
    // `widget_graph`.
    ui::post_update_cache::<C, W>(ui, PostUpdateCache {
        idx: idx,
        maybe_parent_idx: maybe_parent_idx,
        state: resulting_state,
        style: new_style,
        maybe_element: maybe_new_element,
    });
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

    /// A struct representing the user input that has occurred since the last update for the
    /// `Widget` with the given index..
    pub fn input_for<I: Into<Index>>(&self, idx: I) -> UserInput {
        ui::user_input(self.ui, idx.into())
    }

    /// Generate a new, unique NodeIndex into a Placeholder node within the `Ui`'s widget graph.
    /// This should only be called once for each unique widget needed to avoid unnecessary bloat
    /// within the `Ui`'s widget graph.
    ///
    /// When using this method in your `Widget`'s `update` method, be sure to store the returned
    /// NodeIndex somewhere within your `Widget::State` so that it can be re-used on next update.
    pub fn new_unique_node_index(&mut self) -> NodeIndex {
        ui::widget_graph_mut(&mut self.ui).add_placeholder()
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


impl<W> Positionable for W where W: Widget {
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

impl<W> Sizeable for W where W: Widget {
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

