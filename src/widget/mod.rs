
use ::{CharacterCache, Scalar};
use clock_ticks::precise_time_ns;
use elmesque::Element;
use graph::NodeIndex;
use position::{Depth, Dimensions, Direction, Padding, Position, Positionable, Rect, Sizeable,
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


/// Arguments for the [**Widget::update**](./trait.Widget#method.update) method in a struct to
/// simplify the method signature.
pub struct UpdateArgs<'a, 'b: 'a, W, C: 'a> where W: Widget {
    /// The **Widget**'s unique index.
    pub idx: Index,
    /// The **Widget**'s parent unique index, if there is one.
    pub maybe_parent_idx: Option<Index>,
    /// The **Widget**'s previous state. Specifically, the state that is common between all widgets,
    /// such as positioning, floatability, draggability, etc.
    pub prev: &'a CommonState,
    /// A wrapper around the **Widget**'s unique state, providing methods for both immutably viewing
    /// and mutably updating the state.
    ///
    /// We wrap mutation in a method so that we can keep track of whether or not the unique state
    /// has been updated.
    ///
    /// If **State::update** is called, we assume that there has been some mutation and in turn will
    /// produce a new **Element** for the **Widget**. Thus, it is recommended that you *only* call
    /// **State::update** if you need to update the unique state in some way.
    pub state: &'a mut State<'b, W::State>,
    /// The rectangle describing the **Widget**'s area.
    pub rect: Rect,
    /// The **Widget**'s current **Widget::Style**.
    pub style: &'a W::Style,
    /// Restricted access to the `Ui`.
    ///
    /// Provides methods for immutably accessing the `Ui`'s `Theme` and `GlyphCache`.
    /// Also allows calling `Widget::set` within the `Widget::update` method.
    pub ui: UiCell<'a, C>,
}

/// A wrapper around a `Ui` that only exposes the functionality necessary for the
/// **Widget::update** method.
///
/// Its primary role is to allow for widget designers to compose their own unique **Widget**s from
/// other **Widget**s by calling the **Widget::set** method within their own **Widget**'s
/// update method.
///
/// It also provides methods for accessing the **Ui**'s **Theme**, **GlyphCache** and **UserInput**
/// via immutable reference.
///
/// BTW - if you have a better name for this type, please post an issue or PR! "Cell" was the best
/// I could come up with as it's kind of like a jail cell for the **Ui** - restricting a user's
/// access to it.
pub struct UiCell<'a, C: 'a> {
    /// A mutable reference to a **Ui**.
    ui: &'a mut Ui<C>,
    /// The index of the Widget that "owns" the **UiCell**. The index is needed so that we can
    /// correctly retrieve user input information for the specific widget.
    idx: Index,
}

/// Arguments for the **Widget::draw** method in a struct to simplify the method signature.
pub struct DrawArgs<'a, W, C: 'a> where W: Widget {
    /// The current state of the Widget.
    pub state: &'a W::State,
    /// The current style of the Widget.
    pub style: &'a W::Style,
    /// The active **Theme** within the **Ui**.
    pub theme: &'a Theme,
    /// The **Ui**'s **GlyphCache** (for determining text width).
    pub glyph_cache: &'a GlyphCache<C>,
    /// The **Widget**'s z-axis position relative to its sibling widgets.
    pub depth: Depth,
    /// The rectangle describing the **Widget**'s area.
    pub rect: Rect,
    /// The current state of the dragged **Widget**, if it is draggable.
    pub drag_state: drag::State,
    /// Floating state for the widget if it is floating.
    pub maybe_floating: Option<Floating>,
}

/// Arguments to the [**Widget::kid_area**](./trait.Widget#method.kid_area) method in a struct to
/// simplify the method signature.
pub struct KidAreaArgs<'a, W, C: 'a> where W: Widget {
    /// The **Rect** describing the **Widget**'s position and dimensions.
    pub rect: Rect,
    /// Current **Widget::Style** of the **Widget**.
    pub style: &'a W::Style,
    /// The active **Theme** within the **Ui**.
    pub theme: &'a Theme,
    /// The **Ui**'s **GlyphCache** (for determining text width).
    pub glyph_cache: &'a GlyphCache<C>,
}

/// The area upon which a **Widget**'s child widgets will be placed.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct KidArea {
    /// The **Rect** bounds describing the position and area.
    pub rect: Rect,
    /// The distance between the edge of the area and where the widgets will be placed.
    pub pad: Padding,
}

/// The builder argument for the **Widget**'s parent.
#[derive(Copy, Clone, Debug, PartialEq, RustcEncodable, RustcDecodable)]
pub enum MaybeParent {
    /// The user specified the widget should not have any parents, so the Root will be used.
    None,
    /// The user gave a specific parent widget.
    Some(Index),
    /// No parent widget was specified, so we will assume they want the last parent.
    Unspecified,
}

/// State necessary for "floating" (pop-up style) widgets.
#[derive(Copy, Clone, Debug, PartialEq, RustcEncodable, RustcDecodable)]
pub struct Floating {
    /// The time the **Widget** was last clicked (used for depth sorting in the widget **Graph**).
    pub time_last_clicked: u64,
}

/// A struct containing builder data common to all **Widget** types.
/// This type allows us to do a blanket impl of **Positionable** and **Sizeable** for `T: Widget`.
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

/// A wrapper around a **Widget**'s unique **Widget::State**.
///
/// This type is used to provide limited access to the **Widget::State** within the
/// [**Widget::update**](./trait.Widget#method.update) method (to which it is passed via the
/// [**UpdateArgs**](./struct.UpdateArgs)).
///
/// The type provides only two methods. One for viewing the state, the other for mutating it.
///
/// We do this so that we can keep track of whether or not the **Widget::State** has been mutated
/// (using an internal `has_updated` flag). This allows us to know whether or not we need to
/// produce a new **Element** for the **Widget**, without having to compare the previous and
/// new **Widget::State**s.
#[derive(Debug)]
pub struct State<'a, T: 'a> {
    state: &'a mut T,
    /// A flag indicating whether or not the widget's State has been updated.
    has_updated: bool,
}

/// A wrapper around state that is common to all **Widget** types.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct CommonState {
    /// The rectangle describing the `Widget`'s area.
    pub rect: Rect,
    /// The rendering depth for the Widget (the default is 0.0).
    pub depth: Depth,
    /// The current state of the dragged widget, if it is draggable.
    pub drag_state: drag::State,
    /// Floating state for the widget if it is floating.
    pub maybe_floating: Option<Floating>,
}

/// A **Widget**'s state in a form that is retrievable from the **Ui**'s widget cache.
pub struct Cached<W> where W: Widget {
    /// State that is unique to the Widget.
    pub state: W::State,
    /// Unique styling state for the Widget.
    pub style: W::Style,
    /// The rectangle representing the Widget's area.
    pub rect: Rect,
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

/// **Widget** data to be cached prior to the **Widget::update** call in the **set_widget**
/// function.
///
/// We do this so that if this **Widget** were to internally `set` some other **Widget**s, this
/// **Widget**'s positioning and dimension data already exists within the widget **Graph** for
/// reference.
pub struct PreUpdateCache {
    /// The **Widget**'s unique kind.
    pub kind: &'static str,
    /// The **Widget**'s unique Index.
    pub idx: Index,
    /// The **Widget**'s parent's unique index (if it has a parent).
    pub maybe_parent_idx: Option<Index>,
    /// If this **Widget** is relatively positioned to another **Widget**, this will be the index
    /// of the **Widget** to which this **Widget** is relatively positioned
    pub maybe_positioned_relatively_idx: Option<Index>,
    /// The **Rect** describing the **Widget**'s position and dimensions.
    pub rect: Rect,
    /// The z-axis depth - affects the render order of sibling widgets.
    pub depth: Depth,
    /// The area upon which the **Widget**'s children widgets will be placed.
    pub kid_area: KidArea,
    /// The current state of the dragged **Widget**, if it is draggable.
    pub drag_state: drag::State,
    /// Floating data for the **Widget** if there is some.
    pub maybe_floating: Option<Floating>,
    /// Scrolling data for the **Widget** if there is some.
    pub maybe_scrolling: Option<scroll::State>,
}

/// **Widget** data to be cached after the **Widget::update** call in the **set_widget**
/// function.
///
/// We do this so that if this **Widget** were to internally **Widget::set** some other
/// **Widget**s, this **Widget**'s positioning and dimension data will already exist within the
/// widget **Graph** for reference.
pub struct PostUpdateCache<W> where W: Widget {
    /// The **Widget**'s unique **Index**.
    pub idx: Index,
    /// The **Widget**'s parent's unique **Index** (if it has a parent).
    pub maybe_parent_idx: Option<Index>,
    /// The newly produced unique **Widget::State** associated with the **Widget**.
    pub state: W::State,
    /// The newly produced unique **Widget::Style** associated with the **Widget**.
    pub style: W::Style,
    /// A new **Element** to use for the **Widget** if a new one has been produced.
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


/// A trait to be implemented by all **Widget** types.
///
/// A type that implements **Widget** can be thought of as a collection of arguments to the
/// **Widget**'s **Widget::update** method. They type itself is not stored between updates, but
/// rather is used to update an instance of the **Widget**'s **Widget::State**, which *is* stored.
///
/// Methods that *must* be overridden:
/// - common
/// - common_mut
/// - unique_kind
/// - init_state
/// - style
/// - update
/// - draw
///
/// Methods that can be optionally overridden:
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

    /// Return a reference to a **CommonBuilder** struct owned by the Widget.
    /// This method allows us to do a blanket impl of Positionable and Sizeable for T: Widget.
    ///
    /// Note: When rust introduces field inheritance, we will move the **CommonBuilder**
    /// accordingly (perhaps under a different name).
    fn common(&self) -> &CommonBuilder;

    /// Return a mutable reference to a CommonBuilder struct owned by the Widget.
    /// This method allows us to do a blanket impl of Positionable and Sizeable for T: Widget.
    ///
    /// Note: When rust introduces field inheritance, we will move the **CommonBuilder**
    /// accordingly (perhaps under a different name).
    fn common_mut(&mut self) -> &mut CommonBuilder;

    /// Return the kind of the widget as a &'static str.
    ///
    /// Note that this must be unique from all other widgets' "unique kinds".
    ///
    /// This is used by conrod to help avoid WidgetId errors and to provide better messages for
    /// those that do occur.
    fn unique_kind(&self) -> &'static str;

    /// Return the initial **State** of the Widget.
    ///
    /// The `Ui` will only call this once, shortly prior to the first time that **Widget::update**
    /// is first called.
    fn init_state(&self) -> Self::State;

    /// Return the styling of the widget. The `Ui` will call this once prior to each `update`. It
    /// does this so that it can check for differences in `Style` in case a new `Element` needs to
    /// be constructed.
    fn style(&self) -> Self::Style;

    /// Update our **Widget**'s unique **Widget::State** via the **State** wrapper type (the
    /// `state` field within the [**UpdateArgs**](./struct.UpdateArgs)).
    ///
    /// Whenever [**State::update**](./struct.State.html#method.update) is called, a `has_updated`
    /// flag is set within the **State**, indicating that there has been some change to the unique
    /// **Widget::State** and that we require re-drawing the **Widget**'s **Element** (i.e. calling
    /// [**Widget::draw**](./trait.Widget#method.draw). As a result, widget designers should only
    /// call **State::update** when necessary, checking whether or not the state has changed before
    /// invoking the the method. See the custom_widget.rs example for a demonstration
    /// of this.
    ///
    /// # Arguments
    /// * idx - The `Widget`'s unique index (whether `Public` or `Internal`).
    /// * prev - The previous common state of the Widget. If this is the first time **update** is
    /// called, `Widget::init_state` will be used to produce some intial state instead.
    /// * state - A wrapper around the `Widget::State`. See the [**State** docs](./struct.State)
    /// for more details.
    /// * rect - The position (centered) and dimensions of the widget.
    /// * style - The style produced by the `Widget::style` method.
    /// * ui - A wrapper around the `Ui`, offering restricted access to its functionality. See the
    /// docs for `UiCell` for more details.
    fn update<C: CharacterCache>(self, args: UpdateArgs<Self, C>);

    /// Construct a renderable Element from the current styling and new state. This will *only* be
    /// called on the occasion that the widget's `Style` or `State` has changed. Keep this in mind
    /// when designing your widget's `Style` and `State` types.
    ///
    /// # Arguments
    /// * state - The current **Widget::State** which should contain all unique state necessary for
    /// rendering the **Widget**.
    /// * style - The current **Widget::Style** of the **Widget**.
    /// * theme - The currently active **Theme** within the `Ui`.
    /// * glyph_cache - Used for determining the size of rendered text if necessary.
    fn draw<C: CharacterCache>(args: DrawArgs<Self, C>) -> Element;

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

    /// If the widget is draggable, implement this method and return the position an dimensions
    /// of the draggable space. The position should be relative to the center of the widget.
    fn drag_area(&self,
                 _dim: Dimensions,
                 _style: &Self::Style,
                 _theme: &Theme) -> Option<Rect>
    {
        None
    }

    /// The area on which child widgets will be placed when using the `Place` `Position` methods.
    fn kid_area<C: CharacterCache>(&self, args: KidAreaArgs<Self, C>) -> KidArea {
        KidArea {
            rect: args.rect,
            pad: Padding::none(),
        }
    }


    // None of the following methods should require overriding. Perhaps they should be split off
    // into a separate trait which is impl'ed for W: Widget to make this clearer?


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
    let (maybe_prev_unique_state, maybe_prev_common, maybe_prev_style, maybe_prev_scrolling) =
        maybe_widget_state.map(|prev| {

            // Destructure the cached state.
            let Cached {
                state,
                style,
                rect,
                depth,
                drag_state,
                maybe_floating,
                maybe_scrolling,
                ..
            } = prev;

            // Use the cached state to construct the prev_state (to be fed to Widget::update).
            let prev_common = CommonState {
                rect: rect,
                depth: depth,
                drag_state: drag_state,
                maybe_floating: maybe_floating,
            };

            (Some(state), Some(prev_common), Some(style), maybe_scrolling)
        }).unwrap_or_else(|| (None, None, None, None));

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
        match maybe_prev_common {
            // If there is no previous state to compare for dragging, return an initial state.
            None => (gen_xy(), drag::State::Normal),
            Some(ref prev) => {
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
                            let drag_state = prev.drag_state;

                            // Drag the xy of the widget and return the new xy.
                            drag::drag_widget(prev.rect.xy(), drag_area, drag_state, mouse)
                        },
                        // Otherwise just return the regular xy and drag state.
                        None => (prev.rect.xy(), drag::State::Normal),
                    },
                }
            },
        }
    };

    // Construct the rectangle describing our Widget's area.
    let rect = Rect::from_xy_dim(xy, dim);

    // Check whether we have stopped / started dragging the widget and in turn whether or not
    // we need to capture the mouse.
    match (maybe_prev_common.as_ref().map(|prev| prev.drag_state), drag_state) {
        (Some(drag::State::Highlighted), drag::State::Clicked(_)) => {
            ui::mouse_captured_by(ui, idx);
        },
        (Some(drag::State::Clicked(_)), drag::State::Highlighted) |
        (Some(drag::State::Clicked(_)), drag::State::Normal)      => {
            ui::mouse_uncaptured_by(ui, idx);
        },
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
        match maybe_prev_common.as_ref() {
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
            rect: rect,
            style: &new_style,
            theme: &ui.theme,
            glyph_cache: &ui.glyph_cache,
        };
        widget.kid_area(args)
    };

    // Determine whether or not we have some state for scrolling.
    let maybe_new_scrolling = {
        let scrolling = widget.common().scrolling;
        if !scrolling.horizontal && !scrolling.vertical {
            None
        // Otherwise, construct our new Scroll state!
        } else {
            let maybe_mouse = ui::get_mouse_state(ui, idx);
            let visible = kid_area.rect;
            let kids = ui.kids_bounding_box(idx)
                .map(|kids| kids.shift(visible.xy()))
                .unwrap_or_else(|| kid_area.rect);
            let maybe_prev = maybe_prev_scrolling.as_ref();
            let scroll_state = scroll::State::new(scrolling, visible, kids, &ui.theme, maybe_prev);
            Some(maybe_mouse.map(|mouse| scroll_state.handle_input(mouse))
                .unwrap_or_else(|| scroll_state))
        }
    };

    // Check whether or not our new scrolling state should capture or uncapture the mouse.
    if let (Some(ref prev), Some(ref new)) = (maybe_prev_scrolling, maybe_new_scrolling) {
        if scroll::capture_mouse(prev, new) {
            ui::mouse_captured_by(ui, idx);
        }
        if scroll::uncapture_mouse(prev, new) {
            ui::mouse_uncaptured_by(ui, idx);
        }
    }

    // Determine whether or not this is the first time set has been called.
    // We'll use this to determine whether or not we need to call Widget::draw.
    let is_first_set = maybe_prev_common.is_none();

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
            rect: rect,
            depth: depth,
            drag_state: drag_state,
            kid_area: kid_area,
            maybe_floating: maybe_floating,
            maybe_scrolling: maybe_new_scrolling,
        });
    }

    // Unwrap the widget's previous common state. If there is no previous common state, we'll
    // use the new state in it's place.
    let prev_common = maybe_prev_common.unwrap_or_else(|| CommonState {
        rect: rect,
        depth: depth,
        drag_state: drag_state,
        maybe_floating: maybe_floating,
    });

    // Retrieve the widget's unique state and update it via `Widget::update`.
    let (unique_state, has_state_updated) = {

        // Unwrap our unique widget state. If there is no previous state to unwrap, call the
        // `init_state` method to construct some initial state.
        let mut unique_state = maybe_prev_unique_state.unwrap_or_else(|| widget.init_state());
        let has_updated = {

            // A wrapper around the widget's unique state in order to keep track of whether or not it
            // has been updated during the `Widget::update` method.
            let mut state = State {
                state: &mut unique_state,
                has_updated: false,
            };

            widget.update(UpdateArgs {
                idx: idx,
                maybe_parent_idx: maybe_parent_idx,
                state: &mut state,
                prev: &prev_common,
                rect: rect,
                style: &new_style,
                ui: UiCell { ui: ui, idx: idx },
            });

            state.has_updated
        };

        (unique_state, has_updated)
    };

    // Determine whether or not the `State` has changed.
    let state_has_changed = has_state_updated
        || rect != prev_common.rect
        || depth != prev_common.depth
        || is_first_set;

    // Determine whether or not the widget's `Style` has changed.
    let style_has_changed = maybe_prev_style.map(|style| style != new_style).unwrap_or(false);

    // We need to know if the scroll state has changed to see if we need to redraw.
    let scroll_has_changed = maybe_new_scrolling != maybe_prev_scrolling;

    // We only need to redraw the `Element` if some visible part of our widget has changed.
    let requires_redraw = style_has_changed || state_has_changed || scroll_has_changed;

    // If we require a redraw, we should draw a new `Element`.
    let maybe_new_element = if requires_redraw {
        Some(W::draw(DrawArgs {
            state: &unique_state,
            style: &new_style,
            theme: &ui.theme,
            glyph_cache: &ui.glyph_cache,
            rect: rect,
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
        state: unique_state,
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

    /// Have the widget capture the mouse input. The mouse state will be hidden from other
    /// widgets while captured.
    ///
    /// Returns true if the mouse was successfully captured.
    ///
    /// Returns false if it was already captured by some other widget.
    pub fn capture_mouse(&mut self) -> bool {
        ui::mouse_captured_by(self.ui, self.idx)
    }

    /// Uncapture the mouse input.
    ///
    /// Returns true if the mouse was successfully uncaptured.
    ///
    /// Returns false if the mouse wasn't captured by our widget in the first place.
    pub fn uncapture_mouse(&mut self) -> bool {
        ui::mouse_uncaptured_by(self.ui, self.idx)
    }

    /// Have the widget capture the keyboard input. The keyboard state will be hidden from other
    /// widgets while captured.
    ///
    /// Returns true if the keyboard was successfully captured.
    ///
    /// Returns false if it was already captured by some other widget.
    pub fn capture_keyboard(&mut self) -> bool {
        ui::keyboard_captured_by(self.ui, self.idx)
    }

    /// Uncapture the keyboard input.
    ///
    /// Returns true if the keyboard was successfully uncaptured.
    ///
    /// Returns false if the keyboard wasn't captured by our widget in the first place.
    pub fn uncapture_keyboard(&mut self) -> bool {
        ui::keyboard_uncaptured_by(self.ui, self.idx)
    }

    /// Generate a new, unique NodeIndex into a Placeholder node within the `Ui`'s widget graph.
    /// This should only be called once for each unique widget needed to avoid unnecessary bloat
    /// within the `Ui`'s widget graph.
    ///
    /// When using this method in your `Widget`'s `update` method, be sure to store the returned
    /// NodeIndex somewhere within your `Widget::State` so that it can be re-used on next update.
    ///
    /// **Panics** if adding another node would exceed the maximum capacity for node indices.
    pub fn new_unique_node_index(&mut self) -> NodeIndex {
        ui::widget_graph_mut(&mut self.ui).add_placeholder()
    }

    /// The **Rect** that bounds the kids of the widget with the given index.
    ///
    /// Returns `None` if the widget has no children or if there's is no widget for the given index.
    pub fn kids_bounding_box<I: Into<Index>>(&self, idx: I) -> Option<Rect> {
        self.ui.kids_bounding_box(idx)
    }

}


impl<'a, T> State<'a, T> {

    /// Immutably borrow the internal widget state.
    #[inline]
    pub fn view(&self) -> &T { &self.state }

    /// Mutate the internal widget state and set a flag notifying us that there has been a mutation.
    ///
    /// If this method is *not* called, we assume that there has been no mutation and in turn we
    /// will re-use the Widget's pre-existing `Element`.
    ///
    /// If this method *is* called, we assume that there has been some mutation and in turn will
    /// produce a new `Element` for the `Widget`. Thus, it is recommended that you *only* call
    /// this method if you need to update the unique state in some way.
    pub fn update<F>(&mut self, f: F) where F: FnOnce(&mut T) {
        self.has_updated = true;
        f(self.state);
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

