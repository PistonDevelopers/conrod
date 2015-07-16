
use Scalar;
use elmesque::Element;
use graphics::character::CharacterCache;
use position::{Depth, Dimensions, Direction, Padding, Point, Position, Positionable, Sizeable,
               HorizontalAlign, VerticalAlign};
use std::any::Any;
use theme::Theme;
use ui::{self, GlyphCache, Ui, UserInput};

pub mod button;
pub mod canvas;
pub mod drag;
pub mod drop_down_list;
pub mod envelope_editor;
pub mod label;
pub mod matrix;
pub mod number_dialer;
pub mod slider;
pub mod split;
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
    /// * prev - The previous state of the Widget. If none existed, `Widget::init_state` will be
    /// used to pass the initial state instead.
    /// * xy - The coordinates representing the middle of the widget.
    /// * dim - The dimensions of the widget.
    /// * input - A view into the current state of the user input (i.e. mouse and keyboard).
    /// * current_style - The style just produced by the `Widget::style` method.
    /// * theme - The currently active `Theme` within the `Ui`.
    /// * glyph_cache - Used for determining the size of rendered text if necessary.
    fn update<'a, 'b, C>(self, args: UpdateArgs<'a, 'b, Self, C>) -> Option<Self::State>
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
    /// of the draggable space.
    fn drag_area(&self,
                 _xy: Point,
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
    fn parent(mut self, parent_id: WidgetId) -> Self {
        self.common_mut().maybe_parent_id = Some(parent_id);
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
    fn set<C>(self, id: WidgetId, ui: &mut Ui<C>) where C: CharacterCache {

        let kind = self.unique_kind();

        // Collect the previous state and style or initialise both if none exist.
        let maybe_widget_state = ui::get_widget_state::<C, Self>(ui, id, kind);
        let (maybe_prev_state, maybe_prev_style) = maybe_widget_state.map(|prev|{

            // Destructure the cached state.
            let Cached {
                state,
                style,
                xy,
                dim,
                depth,
                drag_state,
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
            };

            (Some(prev_state), Some(style))
        }).unwrap_or_else(|| (None, None));

        let new_style = self.style();
        let depth = self.get_depth();
        let dim = self.get_dimensions(&ui.theme, &ui.glyph_cache);
        let pos = self.get_position(&ui.theme);

        let (xy, drag_state) = {
            // A function for producing the initial xy coords.
            let init_xy = || {
                let (h_align, v_align) = self.get_alignment(&ui.theme);
                let new_xy = ui.get_xy(pos, dim, h_align, v_align);
                new_xy
            };

            // Check to see if the widget is currently being dragged and return the new xy / drag.
            match maybe_prev_state {
                None => (init_xy(), drag::State::Normal),
                Some(ref prev_state) => {
                    let maybe_mouse = ui::get_mouse_state(ui, id);
                    let maybe_drag_area = self.drag_area(prev_state.xy, dim, &new_style, &ui.theme);
                    match (maybe_drag_area, maybe_mouse) {
                        // If there is some draggable area and mouse, drag the xy.
                        (Some(drag_area), Some(mouse)) => {
                            let drag_state = prev_state.drag_state;

                            // Drag the xy of the widget and return the new xy.
                            drag::drag_widget(prev_state.xy, drag_area, drag_state, mouse)
                        },
                        // Otherwise just return the regular xy and drag state.
                        _ => (init_xy(), drag::State::Normal),
                    }
                },
            }
        };

        // Determine the id of the canvas that the widget is attached to. If not given explicitly,
        // check the positioning to retrieve the Id from there.
        let maybe_parent_id = self.common().maybe_parent_id
            .or_else(|| ui::parent_from_position(ui, pos));

        // Determine whether or not this is the first time set has been called.
        // We'll use this to determine whether or not we need to call Widget::draw.
        let is_first_set = maybe_prev_state.is_none();

        // Unwrap the previous state. If there is no previous state to unwrap, call the
        // `init_state` method to use the initial state as the prev_state.
        let prev_state = maybe_prev_state.unwrap_or_else(|| State {
            state: self.init_state(),
            xy: xy,
            dim: dim,
            depth: depth,
            drag_state: drag_state,
        });

        // Update the widget's state.
        let maybe_new_state = {
            // Construct a UserInput for the widget.
            let input = ui::user_input(ui, id);
            let args = UpdateArgs {
                prev_state: &prev_state,
                xy: xy,
                dim: dim,
                input: input,
                style: &new_style,
                theme: &ui.theme,
                glyph_cache: &ui.glyph_cache,
            };
            self.update(args)
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
        };

        // Retrieve the area upon which kid widgets will be placed.
        let kid_area = Self::kid_area(&new_state, &new_style, &ui.theme);

        // Construct the widget's element.
        let maybe_new_element = if style_has_changed || state_has_changed {
            let args = DrawArgs {
                state: &new_state,
                style: &new_style,
                theme: &ui.theme,
                glyph_cache: &ui.glyph_cache,
            };
            Some(Self::draw(args))
        } else {
            None
        };

        // Store the new `State` and `Style` within the cache.
        let State { state, dim, xy, depth, drag_state } = new_state;
        let cached: Cached<Self> = Cached {
            state: state,
            style: new_style,
            dim: dim,
            xy: xy,
            depth: depth,
            drag_state: drag_state,
            kid_area: kid_area,
        };
        ui::update_widget(ui, id, maybe_parent_id, kind, cached, maybe_new_element);

    }

}

/// Arguments for the `Widget::update` method in a struct to simplify the method signature.
pub struct UpdateArgs<'a, 'b, W, C: 'a> where W: Widget {
    /// The Widget's state that was last returned by the update method.
    pub prev_state: &'a State<W::State>,
    /// The absolute (centered origin) screen position of the widget.
    pub xy: Point,
    /// The dimensions of the Widget.
    pub dim: Dimensions,
    /// The current state of user input (i.e. mouse, keys pressed, etc).
    pub input: UserInput<'b>,
    /// The Widget's current Style.
    pub style: &'a W::Style,
    /// The active `Theme` within the `Ui`.
    pub theme: &'a Theme,
    /// The `Ui`'s GlyphCache (for determining text width, etc).
    pub glyph_cache: &'a GlyphCache<C>,
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

/// A struct containing builder data common to all Widget types.
/// This type allows us to do a blanket impl of Positionable and Sizeable for T: Widget.
#[derive(Clone, Debug, RustcEncodable, RustcDecodable)]
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
    pub maybe_parent_id: Option<WidgetId>,
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
            maybe_parent_id: None,
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

