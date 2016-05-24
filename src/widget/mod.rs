use {Backend, CharacterCache, Dimension, GlyphCache};
use graph::NodeIndex;
use position::{Align, Depth, Dimensions, Padding, Position, Positionable, Rect, Sizeable};
use std;
use theme::{self, Theme};
use ui::{self, Ui, UiCell};

pub use self::id::Id;
pub use self::index::Index;

// Macro providing modules.
#[macro_use] mod builder;
#[macro_use] mod style;

// Widget functionality modules.
mod id;
mod index;
pub mod scroll;

// Primitive widget modules.
pub mod primitive;

// Widget modules.
pub mod button;
pub mod canvas;
pub mod drop_down_list;
pub mod envelope_editor;
pub mod matrix;
pub mod number_dialer;
pub mod slider;
pub mod tabs;
pub mod text_box;
pub mod text_edit;
pub mod title_bar;
pub mod toggle;
pub mod xy_pad;


/// Arguments for the [**Widget::update**](./trait.Widget#method.update) method in a struct to
/// simplify the method signature.
pub struct UpdateArgs<'a, 'b: 'a, W, B: 'a>
    where W: Widget,
          B: Backend,
{
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
    /// If **State::update** is called, we assume that there has been some mutation and in turn
    /// will require re-drawing the **Widget**. Thus, it is recommended that you *only* call
    /// **State::update** if you need to update the unique state in some way.
    pub state: &'a mut State<'b, W::State>,
    /// The rectangle describing the **Widget**'s area.
    pub rect: Rect,
    /// The **Widget**'s current **Widget::Style**.
    pub style: &'a W::Style,
    /// Restricted access to the `Ui`.
    ///
    /// Provides methods for immutably accessing the `Ui`'s `Theme` and `GlyphCache`.  Also allows
    /// calling `Widget::set` within the `Widget::update` method.
    pub ui: UiCell<'a, B>,
}

/// A small cache for a single unique **NodeIndex**.
///
/// This should be used by **Widget**s within their unique **State** for instantiating their own
/// unique widgets.
///
/// This should reduce the need for users to directly call `UiCell::new_unique_node_index` and in
/// turn reduce related mistakes (i.e. accidentally calling it and growing the graph unnecessarily).
#[derive(Clone, Debug, PartialEq)]
pub struct IndexSlot {
    maybe_idx: std::cell::Cell<Option<NodeIndex>>,
}

/// Arguments to the [**Widget::kid_area**](./trait.Widget#method.kid_area) method in a struct to
/// simplify the method signature.
pub struct KidAreaArgs<'a, W, C: 'a>
    where W: Widget,
{
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
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum MaybeParent {
    /// The user specified the widget should not have any parents, so the Root will be used.
    None,
    /// The user gave a specific parent widget.
    Some(Index),
    /// No parent widget was specified, so we will assume they want the last parent.
    Unspecified,
}

impl MaybeParent {
    /// Convert the **MaybeParent** into an **Option<Index>**.
    ///
    /// If `Unspecified`, check the positioning to retrieve the **Index** from there.
    ///
    /// If `None`, the `Ui`'s `window` widget will be used.
    ///
    /// **Note:** This method does not check whether or not using the `window` widget as the parent
    /// would cause a cycle. If it is important that the inferred parent should not cause a cycle,
    /// use `get` instead.
    pub fn get_unchecked<B>(&self, ui: &Ui<B>, x_pos: Position, y_pos: Position) -> Index
        where B: Backend,
    {
        match *self {
            MaybeParent::Some(idx) => idx,
            MaybeParent::None => ui.window.into(),
            MaybeParent::Unspecified => ui::infer_parent_unchecked(ui, x_pos, y_pos),
        }
    }

    /// The same as `get_unchecked`, but checks whether or not the widget that we're inferring the
    /// parent for is the `Ui`'s window (which cannot have a parent, without creating a cycle).
    pub fn get<B>(&self, idx: Index, ui: &Ui<B>, x_pos: Position, y_pos: Position) -> Option<Index>
        where B: Backend,
    {
        if idx == ui.window.into() {
            None
        } else {
            Some(self.get_unchecked(ui, x_pos, y_pos))
        }
    }
}

/// State necessary for "floating" (pop-up style) widgets.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Floating {
    /// The time the **Widget** was last clicked (used for depth sorting in the widget **Graph**).
    pub time_last_clicked: std::time::Instant,
}

/// A struct containing builder data common to all **Widget** types.
///
/// This type also allows us to do a blanket impl of **Positionable** and **Sizeable** for `T: Widget`.
///
/// When Rust gets some sort of field inheritance feature, this will most likely be refactored to
/// take advantage of that.
#[derive(Clone, Copy, Debug)]
pub struct CommonBuilder {
    /// Styling and positioning data that is common between all widget types.
    pub style: CommonStyle,
    /// The parent widget of the Widget.
    pub maybe_parent_idx: MaybeParent,
    /// Whether or not the Widget is a "floating" Widget.
    pub is_floating: bool,
    /// Arguments to the scrolling of the widget's *x* axis.
    pub maybe_x_scroll: Option<scroll::Scroll>,
    /// Arguments to the scrolling of the widget's *y* axis.
    pub maybe_y_scroll: Option<scroll::Scroll>,
    /// Whether or not the **Widget** should be placed on the kid_area.
    ///
    /// If `true`, the **Widget** will be placed on the `kid_area` of the parent **Widget** if the
    /// **Widget** is given a **Place** variant for its **Position**.
    ///
    /// If `false`, the **Widget** will be placed on the parent **Widget**'s *total* area.
    pub place_on_kid_area: bool,
    /// Describes whether or not the **Widget** is instantiated as a graphical element for some
    /// other **Widget**.
    ///
    /// When adding an edge *a -> b* where *b* is considered to be a graphical element of *a*,
    /// several things are implied about *b*:
    ///
    /// - If *b* is picked within either **Graph::pick_widget** or
    /// **Graph::pick_top_scrollable_widget**, it will instead return the index for *a*.
    /// - When determining the **Graph::scroll_offset** for *b*, *a*'s scrolling (if it is
    /// scrollable, that is) will be skipped.
    /// - *b* will always be placed upon *a*'s total area, rather than its kid_area which is the
    /// default.
    /// - Any **Graphic** child of *b* will be considered as a **Graphic** child of *a*.
    pub maybe_graphics_for: Option<Index>,
}

/// Styling and positioning data that is common between all widget types.
#[derive(Clone, Copy, Debug)]
pub struct CommonStyle {
    /// The width of a Widget.
    pub maybe_x_dimension: Option<Dimension>,
    /// The height of a Widget.
    pub maybe_y_dimension: Option<Dimension>,
    /// The position of a Widget along the *x* axis.
    pub maybe_x_position: Option<Position>,
    /// The position of a Widget along the *y* axis.
    pub maybe_y_position: Option<Position>,
    /// The rendering Depth of the Widget.
    pub maybe_depth: Option<Depth>,
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
/// re-draw the **Widget**, without having to compare the previous and new **Widget::State**s.
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
    /// Floating state for the widget if it is floating.
    pub maybe_floating: Option<Floating>,
    /// The area of the widget upon which kid widgets are placed.
    pub kid_area: KidArea,
}

/// A **Widget**'s state in a form that is retrievable from the **Ui**'s widget cache.
pub struct Cached<W>
    where W: Widget,
{
    /// State that is unique to the Widget.
    pub state: W::State,
    /// Unique styling state for the Widget.
    pub style: W::Style,
    /// The rectangle representing the Widget's area.
    pub rect: Rect,
    /// Previous rendering depth of the Widget.
    pub depth: Depth,
    /// The area in which child widgets are placed.
    pub kid_area: KidArea,
    /// Whether or not the Widget is a "floating" Widget.
    pub maybe_floating: Option<Floating>,
    /// The state for a widget's scrollable *x* axis.
    pub maybe_x_scroll_state: Option<scroll::StateX>,
    /// The state for a widget's scrollable *y* axis.
    pub maybe_y_scroll_state: Option<scroll::StateY>,
}

/// A unique identifier for a **Widget** type.
///
/// Note: This might be replaced with **Any::get_type_id** when it stabilises.
pub type Kind = &'static str;

/// **Widget** data to be cached prior to the **Widget::update** call in the **set_widget**
/// function.
///
/// We do this so that if this **Widget** were to internally `set` some other **Widget**s, this
/// **Widget**'s positioning and dimension data already exists within the widget **Graph** for
/// reference.
#[allow(missing_copy_implementations)]
pub struct PreUpdateCache {
    /// The **Widget**'s unique kind.
    pub kind: Kind,
    /// The **Widget**'s unique Index.
    pub idx: Index,
    /// The **Widget**'s parent's unique index (if it has a parent).
    pub maybe_parent_idx: Option<Index>,
    /// If this **Widget** is relatively positioned to another **Widget**, this will be the index
    /// of the **Widget** to which this **Widget** is relatively positioned along the *x* axis.
    pub maybe_x_positioned_relatively_idx: Option<Index>,
    /// If this **Widget** is relatively positioned to another **Widget**, this will be the index
    /// of the **Widget** to which this **Widget** is relatively positioned along the *y* axis.
    pub maybe_y_positioned_relatively_idx: Option<Index>,
    /// The **Rect** describing the **Widget**'s position and dimensions.
    pub rect: Rect,
    /// The z-axis depth - affects the render order of sibling widgets.
    pub depth: Depth,
    /// The area upon which the **Widget**'s children widgets will be placed.
    pub kid_area: KidArea,
    /// Floating data for the **Widget** if there is some.
    pub maybe_floating: Option<Floating>,
    /// Scrolling data for the **Widget**'s *x* axis if there is some.
    pub maybe_x_scroll_state: Option<scroll::StateX>,
    /// Scrolling data for the **Widget**'s *y* axis if there is some.
    pub maybe_y_scroll_state: Option<scroll::StateY>,
    /// Whether or not the **Widget** has been instantiated as a graphical element for some other
    /// widget.
    pub maybe_graphics_for: Option<Index>,
}

/// **Widget** data to be cached after the **Widget::update** call in the **set_widget**
/// function.
///
/// We do this so that if this **Widget** were to internally **Widget::set** some other
/// **Widget**s, this **Widget**'s positioning and dimension data will already exist within the
/// widget **Graph** for reference.
pub struct PostUpdateCache<W>
    where W: Widget,
{
    /// The **Widget**'s unique **Index**.
    pub idx: Index,
    /// The **Widget**'s parent's unique **Index** (if it has a parent).
    pub maybe_parent_idx: Option<Index>,
    /// The newly produced unique **Widget::State** associated with the **Widget**.
    pub state: W::State,
    /// The newly produced unique **Widget::Style** associated with the **Widget**.
    pub style: W::Style,
}


/// The necessary bounds for a **Widget**'s associated **Style** type.
pub trait Style: std::any::Any + std::fmt::Debug + PartialEq + Sized {}

/// Auto-implement the **Style** trait for all applicable types.
impl<T> Style for T where T: std::any::Any + std::fmt::Debug + PartialEq + Sized {}


/// Determines the default **Dimension** for a **Widget**.
///
/// This function checks for a default dimension in the following order.
/// 1. Check for a default value within the **Ui**'s **Theme**.
/// 2. Otherwise attempts to copy the dimension of the previously set widget if there is one.
/// 3. Otherwise attempts to copy the dimension of our parent widget.
/// 4. If no parent widget can be inferred, the window dimensions are used.
fn default_dimension<W, B, F>(widget: &W, ui: &Ui<B>, f: F) -> Dimension
    where W: Widget,
          B: Backend,
          F: FnOnce(theme::UniqueDefault<W::Style>) -> Option<Dimension>,
{
    ui.theme.widget_style::<W::Style>(widget.unique_kind())
        .and_then(f)
        .or_else(|| ui.maybe_prev_widget().map(|idx| Dimension::Of(idx, None)))
        .unwrap_or_else(|| {
            let x_pos = widget.get_x_position(ui);
            let y_pos = widget.get_y_position(ui);
            let parent_idx = widget.common().maybe_parent_idx.get_unchecked(ui, x_pos, y_pos);
            Dimension::Of(parent_idx, None)
        })
}

/// Determines the default **Dimension** for a **Widget**.
///
/// This function checks for a default dimension in the following order.
/// 1. Check for a default value within the **Ui**'s **Theme**.
/// 2. Otherwise attempts to copy the dimension of the previously set widget if there is one.
/// 3. Otherwise attempts to copy the dimension of our parent widget.
/// 4. If no parent widget can be inferred, the window dimensions are used.
///
/// This is called by the default implementations of **Widget::default_x_dimension**.
///
/// If you wish to override **Widget::default_x_dimension**, feel free to call this function
/// internally if you partly require the bahaviour of the default implementations.
pub fn default_x_dimension<W, B>(widget: &W, ui: &Ui<B>) -> Dimension
    where W: Widget,
          B: Backend,
{
    default_dimension(widget, ui, |default| default.common.maybe_x_dimension)
}

/// Determines the default **Dimension** for a **Widget**.
///
/// This function checks for a default dimension in the following order.
/// 1. Check for a default value within the **Ui**'s **Theme**.
/// 2. Otherwise attempts to copy the dimension of the previously set widget if there is one.
/// 3. Otherwise attempts to copy the dimension of our parent widget.
/// 4. If no parent widget can be inferred, the window dimensions are used.
///
/// This is called by the default implementations of **Widget::default_y_dimension**.
///
/// If you wish to override **Widget::default_y_dimension**, feel free to call this function
/// internally if you partly require the bahaviour of the default implementations.
pub fn default_y_dimension<W, B>(widget: &W, ui: &Ui<B>) -> Dimension
    where W: Widget,
          B: Backend,
{
    default_dimension(widget, ui, |default| default.common.maybe_y_dimension)
}


/// A trait to be implemented by all **Widget** types.
///
/// A type that implements **Widget** can be thought of as a collection of arguments to the
/// **Widget**'s **Widget::update** method. They type itself is not stored between updates, but
/// rather is used to update an instance of the **Widget**'s **Widget::State**, which *is* stored.
///
/// Methods that *must* be overridden:
///
/// - common
/// - common_mut
/// - unique_kind
/// - init_state
/// - style
/// - update
///
/// Methods that can be optionally overridden:
///
/// - default_x_position
/// - default_y_position
/// - default_width
/// - default_height
/// - drag_area
/// - kid_area
///
/// Methods that should not be overridden:
///
/// - floating
/// - scroll_kids
/// - scroll_kids_vertically
/// - scroll_kids_horizontally
/// - place_widget_on_kid_area
/// - parent
/// - no_parent
/// - set
pub trait Widget: Sized {
    /// State to be stored within the `Ui`s widget cache.
    ///
    /// Take advantage of this type for any large allocations that you would like to avoid
    /// repeating between updates, or any calculations that you'd like to avoid repeating between
    /// calls to `update`.
    ///
    /// Conrod will never clone the state, it will only ever be moved.
    type State: std::any::Any + PartialEq + std::fmt::Debug;
    /// Every widget is required to have its own associated `Style` type. This type is intended to
    /// contain high-level styling information for the widget that can be *optionally specified* by
    /// a user of the widget.
    ///
    /// All `Style` structs are typically `Copy` and contain simple, descriptive fields like
    /// `color`, `font_size`, `line_spacing`, `frame_width`, etc. These types are also required to
    /// be `PartialEq`. This is so that the `Ui` may automatically compare the previous style to
    /// the new style each time `.set` is called, allowing conrod to automatically determine
    /// whether or not something has changed and if a re-draw is required.
    ///
    /// Each field in a `Style` struct is typically an `Option<T>`. This is so that each field may
    /// be *optionally specified*, indicating to fall back to defaults if the fields are `None`
    /// upon style retrieval.
    ///
    /// The reason this data is required to be in its own `Style` type (rather than in the widget
    /// type itself) is so that conrod can distinguish between default style data that may be
    /// stored within the `Theme`'s `widget_styling`, and other data that is necessary for the
    /// widget's behaviour logic. Having `Style` be an associated type makes it trivial to retrieve
    /// unique, widget-specific styling data for each widget from a single method (see
    /// [`Theme::widget_style`](./theme/struct.Theme.html#method.widget_style)).
    ///
    /// These types are often quite similar and can involve a lot of boilerplate when written by
    /// hand due to rust's lack of field inheritance. To get around this, conrod provides
    /// [`widget_style!`][1] - a macro that vastly simplifies the definition and implementation of
    /// widget `Style` types.
    ///
    /// Conrod doesn't yet support serializing widget styling with the `Theme` type, but we hope to
    /// soon.
    ///
    /// # Examples
    ///
    /// ```
    /// # extern crate conrod;
    /// # use conrod::{Color, FontSize, Scalar};
    /// # fn main() {}
    /// /// Unique styling for a Button widget.
    /// pub struct Style {
    ///     /// Color of the Button's pressable area.
    ///     pub color: Option<Color>,
    ///     /// Width of the frame surrounding the button.
    ///     pub frame: Option<Scalar>,
    ///     /// The color of the Button's rectangular frame.
    ///     pub frame_color: Option<Color>,
    ///     /// The color of the Button's label.
    ///     pub label_color: Option<Color>,
    ///     /// The font size for the Button's label.
    ///     pub label_font_size: Option<FontSize>,
    /// }
    /// ```
    ///
    /// Note: It is recommended that you don't write these types yourself as it can get tedious.
    /// Instead, we suggest using the [`widget_style!`][1] macro which also provides all necessary
    /// style retrieval method implementations.
    ///
    /// [1]: ./macro.widget_style!.html
    type Style: Style;

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
    fn unique_kind(&self) -> Kind;

    /// Return the initial **State** of the Widget.
    ///
    /// The `Ui` will only call this once, shortly prior to the first time that **Widget::update**
    /// is first called.
    fn init_state(&self) -> Self::State;

    /// Return the styling of the widget.
    ///
    /// The `Ui` will call this once prior to each `update`. It does this so that it can check for
    /// differences in `Style` in case we need to re-draw the widget.
    fn style(&self) -> Self::Style;

    /// Update our **Widget**'s unique **Widget::State** via the **State** wrapper type (the
    /// `state` field within the [**UpdateArgs**](./struct.UpdateArgs)).
    ///
    /// Whenever [**State::update**](./struct.State.html#method.update) is called, a `has_updated`
    /// flag is set within the **State**, indicating that there has been some change to the unique
    /// **Widget::State** and that we require re-drawing the **Widget**. As a result, widget
    /// designers should only call **State::update** when necessary, checking whether or not the
    /// state has changed before invoking the method. See the custom_widget.rs example for a
    /// demonstration of this.
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
    fn update<B: Backend>(self, args: UpdateArgs<Self, B>);

    /// The default **Position** for the widget along the *x* axis.
    ///
    /// This is used when no **Position** is explicitly given when instantiating the Widget.
    fn default_x_position<B: Backend>(&self, ui: &Ui<B>) -> Position {
        ui.theme.widget_style::<Self::Style>(self.unique_kind())
            .and_then(|style| style.common.maybe_x_position)
            .unwrap_or(ui.theme.x_position)
    }

    /// The default **Position** for the widget along the *y* axis.
    ///
    /// This is used when no **Position** is explicitly given when instantiating the Widget.
    fn default_y_position<B: Backend>(&self, ui: &Ui<B>) -> Position {
        ui.theme.widget_style::<Self::Style>(self.unique_kind())
            .and_then(|style| style.common.maybe_y_position)
            .unwrap_or(ui.theme.y_position)
    }

    /// The default width for the **Widget**.
    ///
    /// This method is only used if no height is explicitly given.
    ///
    /// By default, this simply calls [**default_dimension**](./fn.default_dimension) with a
    /// fallback absolute dimension of 0.0.
    fn default_x_dimension<B: Backend>(&self, ui: &Ui<B>) -> Dimension {
        default_x_dimension(self, ui)
    }

    /// The default height of the widget.
    ///
    /// By default, this simply calls [**default_dimension**](./fn.default_dimension) with a
    /// fallback absolute dimension of 0.0.
    fn default_y_dimension<B: Backend>(&self, ui: &Ui<B>) -> Dimension {
        default_y_dimension(self, ui)
    }

    /// If the widget is draggable, implement this method and return the position and dimensions of
    /// the draggable space. The position should be relative to the center of the widget.
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
    // Most of them would benefit by some sort of field inheritance as they are mainly just used to
    // set sommon data.


    /// Set the parent widget for this Widget by passing the WidgetId of the parent.
    ///
    /// This will attach this Widget to the parent widget.
    fn parent<I: Into<Index>>(mut self, parent_idx: I) -> Self {
        self.common_mut().maybe_parent_idx = MaybeParent::Some(parent_idx.into());
        self
    }

    /// Specify that this widget has no parent widgets.
    fn no_parent(mut self) -> Self {
        self.common_mut().maybe_parent_idx = MaybeParent::None;
        self
    }

    /// Set whether or not the **Widget** should be placed on the kid_area.
    ///
    /// If `true`, the **Widget** will be placed on the `kid_area` of the parent **Widget** if the
    /// **Widget** is given a **Place** variant for its **Position**.
    ///
    /// If `false`, the **Widget** will be placed on the parent **Widget**'s *total* area.
    ///
    /// By default, conrod will automatically determine this for you by checking whether or not the
    /// **Widget** that our **Widget** is being placed upon returns `Some` from its
    /// **Widget::kid_area** method.
    fn place_on_kid_area(mut self, b: bool) -> Self {
        self.common_mut().place_on_kid_area = b;
        self
    }

    /// Indicates that the **Widget** is used as a non-interactive graphical element for some other
    /// widget.
    ///
    /// This is useful for **Widget**s that are used to compose some other **Widget**.
    ///
    /// When adding an edge *a -> b* where *b* is considered to be a graphical element of *a*,
    /// several things are implied about *b*:
    ///
    /// - If *b* is picked within either **Graph::pick_widget** or
    /// **Graph::pick_top_scrollable_widget**, it will instead return the index for *a*.
    /// - When determining the **Graph::scroll_offset** for *b*, *a*'s scrolling (if it is
    /// scrollable, that is) will be skipped.
    /// - *b* will always be placed upon *a*'s total area, rather than its kid_area which is the
    /// default.
    /// - Any **Graphic** child of *b* will be considered as a **Graphic** child of *a*.
    fn graphics_for<I: Into<Index>>(mut self, idx: I) -> Self {
        self.common_mut().maybe_graphics_for = Some(idx.into());
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
    ///
    /// If a widget is scrollable and it has children widgets that fall outside of its `KidArea`,
    /// the `KidArea` will become scrollable.
    fn scroll_kids(self) -> Self {
        self.scroll_kids_vertically().scroll_kids_horizontally()
    }

    /// Set whether or not the widget's `KidArea` is scrollable (the default is false).
    ///
    /// If a widget is scrollable and it has children widgets that fall outside of its `KidArea`,
    /// the `KidArea` will become scrollable.
    fn scroll_kids_vertically(mut self) -> Self {
        self.common_mut().maybe_y_scroll = Some(scroll::Scroll::new());
        self
    }

    /// Set whether or not the widget's `KidArea` is scrollable (the default is false).
    ///
    /// If a widget is scrollable and it has children widgets that fall outside of its `KidArea`,
    /// the `KidArea` will become scrollable.
    fn scroll_kids_horizontally(mut self) -> Self {
        self.common_mut().maybe_x_scroll = Some(scroll::Scroll::new());
        self
    }

    /// A builder method that "lifts" the **Widget** through the given `build` function.
    ///
    /// This method is solely for providing slight ergonomic improvement by helping to maintain
    /// the symmetry of the `builder` pattern in some cases.
    #[inline]
    fn and<F>(self, build: F) -> Self
        where F: FnOnce(Self) -> Self,
    {
        build(self)
    }

    /// A builder method that mutates the **Widget** with the given `mutate` function.
    ///
    /// This method is solely for providing slight ergonomic improvement by helping to maintain
    /// the symmetry of the `builder` pattern in some cases.
    #[inline]
    fn and_mut<F>(mut self, mutate: F) -> Self
        where F: FnOnce(&mut Self),
    {
        mutate(&mut self);
        self
    }

    /// A method that conditionally builds the **Widget** with the given `build` function.
    ///
    /// If `cond` is `true`, `build(self)` is evaluated and returned.
    ///
    /// If `false`, `self` is returned.
    #[inline]
    fn and_if<F>(self, cond: bool, build: F) -> Self
        where F: FnOnce(Self) -> Self,
    {
        if cond { build(self) } else { self }
    }

    /// A method that optionally builds the the **Widget** with the given `build` function.
    ///
    /// If `maybe` is `Some(t)`, `build(self, t)` is evaluated and returned.
    ///
    /// If `None`, `self` is returned.
    #[inline]
    fn and_then<T, F>(self, maybe: Option<T>, build: F) -> Self
        where F: FnOnce(Self, T) -> Self,
    {
        if let Some(t) = maybe { build(self, t) } else { self }
    }

    /// Note: There should be no need to override this method.
    ///
    /// After building the widget, you call this method to set its current state into the given
    /// `Ui`. More precisely, the following will occur when calling this method:
    /// - The widget's previous state and style will be retrieved.
    /// - The widget's current `Style` will be retrieved (from the `Widget::style` method).
    /// - The widget's state will be updated (using the `Widget::udpate` method).
    /// - If the widget's state or style has changed, the **Ui** will be notified that the widget
    /// needs to be re-drawn.
    /// - The new State and Style will be cached within the `Ui`.
    fn set<'a, 'b, I, B>(self, idx: I, ui_cell: &'a mut UiCell<'b, B>)
        where I: Into<Index>,
              B: Backend,
    {
        set_widget(self, idx.into(), ui::ref_mut_from_ui_cell(ui_cell));
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
fn set_widget<'a, B, W>(widget: W, idx: Index, ui: &mut Ui<B>)
    where B: Backend,
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

        ui::widget_graph_mut(ui).widget_mut(idx).and_then(check_container_kind)
    };

    // Seperate the Widget's previous state into it's unique state, style and scrolling.
    let (maybe_prev_unique_state,
         maybe_prev_common,
         maybe_prev_style,
         maybe_prev_x_scroll_state,
         maybe_prev_y_scroll_state) =
        maybe_widget_state.map(|prev| {

            // Destructure the cached state.
            let Cached {
                state,
                style,
                rect,
                depth,
                maybe_floating,
                maybe_x_scroll_state,
                maybe_y_scroll_state,
                kid_area,
                ..
            } = prev;

            // Use the cached state to construct the prev_state (to be fed to Widget::update).
            let prev_common = CommonState {
                rect: rect,
                depth: depth,
                maybe_floating: maybe_floating,
                kid_area: kid_area,
            };

            (Some(state),
             Some(prev_common),
             Some(style),
             maybe_x_scroll_state,
             maybe_y_scroll_state)
        }).unwrap_or_else(|| (None, None, None, None, None));

    // We need to hold onto the current "previously set widget", as this may change during our
    // `Widget`'s update method (i.e. if it sets any of its own widgets, they will become the last
    // previous widget).
    let maybe_prev_widget_idx = ui.maybe_prev_widget();

    let new_style = widget.style();
    let depth = widget.get_depth();
    let dim = widget.get_wh(&ui).unwrap_or([0.0, 0.0]);
    let x_pos = widget.get_x_position(ui);
    let y_pos = widget.get_y_position(ui);
    let place_on_kid_area = widget.common().place_on_kid_area;

    // Determine the id of the canvas that the widget is attached to. If not given explicitly,
    // check the positioning to retrieve the Id from there.
    let maybe_parent_idx = widget.common().maybe_parent_idx.get(idx, ui, x_pos, y_pos);

    // Calculate the `xy` location of the widget, considering drag.
    let xy = maybe_prev_common
        .as_ref()
        .and_then(|prev| {
            let maybe_drag_area = widget.drag_area(dim, &new_style, &ui.theme);
            maybe_drag_area.map(|drag_area| {
                let mut left_mouse_drags = ui.widget_input(idx).drags().left();
                let maybe_first_drag = left_mouse_drags.next();
                let prev_xy = prev.rect.xy();
                maybe_first_drag
                    .and_then(|first_drag| {
                        if drag_area.is_over(first_drag.from) {
                            let total_drag_xy = left_mouse_drags
                                .fold(first_drag.delta_xy, |total, drag| {
                                    [total[0] + drag.delta_xy[0], total[1] + drag.delta_xy[1]]
                                });
                            Some([prev_xy[0] + total_drag_xy[0], prev_xy[1] + total_drag_xy[1]])
                        } else {
                            None
                        }
                    })
                    .unwrap_or(prev_xy)
            })
        })
        // If there is no previous state to compare for dragging, return an initial state.
        //
        // A function for generating the xy coords from the given alignment and Position.
        .unwrap_or_else(|| ui.calc_xy(Some(idx), x_pos, y_pos, dim, place_on_kid_area));

    // Construct the rectangle describing our Widget's area.
    let rect = Rect::from_xy_dim(xy, dim);

    // Check whether or not the widget is a "floating" (hovering / pop-up style) widget.
    let maybe_floating = if widget.common().is_floating {

        fn new_floating() -> Floating {
            Floating { time_last_clicked: std::time::Instant::now() }
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
        let args: KidAreaArgs<W, B::CharacterCache> = KidAreaArgs {
            rect: rect,
            style: &new_style,
            theme: &ui.theme,
            glyph_cache: &ui.glyph_cache,
        };
        widget.kid_area(args)
    };

    // If either axis is scrollable, retrieve the up-to-date `scroll::State` for that axis.
    //
    // We must step the scrolling using the previous `kid_area` state so that the bounding box
    // around our kid widgets is in sync with the position of the `kid_area`.
    let prev_kid_area = maybe_prev_common.map(|common| common.kid_area)
        .unwrap_or_else(|| kid_area);

    let maybe_x_scroll_state = widget.common().maybe_x_scroll.map(|scroll_args| {
        scroll::State::update(ui, idx, &prev_kid_area, maybe_prev_x_scroll_state, 0.0)
    });
    let maybe_y_scroll_state = widget.common().maybe_y_scroll.map(|scroll_args| {
        scroll::State::update(ui, idx, &prev_kid_area, maybe_prev_y_scroll_state, 0.0)
    });

    // Determine whether or not this is the first time set has been called.
    // We'll use this to determine whether or not we need to draw for the first time.
    let is_first_set = maybe_prev_common.is_none();

    // Update all positioning and dimension related data prior to calling `Widget::update`.
    // We do this so that if this widget were to internally `set` some other `Widget`s, this
    // `Widget`s positioning and dimension data already exists within the `Graph`.
    {
        use Position::{Place, Relative, Direction, Align, Absolute};

        // Some widget to which this widget is relatively positioned (if there is one).
        let maybe_positioned_relatively_idx = |pos: Position| match pos {
            Place(_, maybe_idx) | Relative(_, maybe_idx) |
            Direction(_, _, maybe_idx) | Align(_, maybe_idx) =>
                maybe_idx.or(maybe_prev_widget_idx),
            Absolute(_) => None,
        };

        let maybe_x_positioned_relatively_idx = maybe_positioned_relatively_idx(x_pos);
        let maybe_y_positioned_relatively_idx = maybe_positioned_relatively_idx(y_pos);

        // This will cache the given data into the `ui`'s `widget_graph`.
        ui::pre_update_cache(ui, PreUpdateCache {
            kind: kind,
            idx: idx,
            maybe_parent_idx: maybe_parent_idx,
            maybe_x_positioned_relatively_idx: maybe_x_positioned_relatively_idx,
            maybe_y_positioned_relatively_idx: maybe_y_positioned_relatively_idx,
            rect: rect,
            depth: depth,
            kid_area: kid_area,
            maybe_floating: maybe_floating,
            maybe_y_scroll_state: maybe_y_scroll_state,
            maybe_x_scroll_state: maybe_x_scroll_state,
            maybe_graphics_for: widget.common().maybe_graphics_for,
        });
    }

    // Unwrap the widget's previous common state. If there is no previous common state, we'll
    // use the new state in it's place.
    let prev_common = maybe_prev_common.unwrap_or_else(|| CommonState {
        rect: rect,
        depth: depth,
        maybe_floating: maybe_floating,
        kid_area: kid_area,
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
                ui: ui::new_ui_cell(ui),
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
    let scroll_has_changed = maybe_x_scroll_state != maybe_prev_x_scroll_state
        || maybe_y_scroll_state != maybe_prev_y_scroll_state;

    // We only need to redraw if some visible part of our widget has changed.
    let requires_redraw = style_has_changed || state_has_changed || scroll_has_changed;

    // If we require a redraw, we should notify the `Ui`.
    if requires_redraw {
        ui.needs_redraw();
    }

    // Finally, cache the `Widget`'s newly updated `State` and `Style` within the `ui`'s
    // `widget_graph`.
    ui::post_update_cache::<B, W>(ui, PostUpdateCache {
        idx: idx,
        maybe_parent_idx: maybe_parent_idx,
        state: unique_state,
        style: new_style,
    });
}




impl IndexSlot {

    /// Construct a new empty **IndexSlot**.
    pub fn new() -> Self {
        IndexSlot {
            maybe_idx: ::std::cell::Cell::new(None),
        }
    }

    /// Returns the **NodeIndex** held by the **IndexSlot**.
    ///
    /// If the **IndexSlot** does not yet hold a **NodeIndex**, the **UiCell** will be used to
    /// produce a `new_unique_node_index`.
    pub fn get<B>(&self, ui: &mut UiCell<B>) -> NodeIndex
        where B: Backend,
    {
        if self.maybe_idx.get().is_none() {
            let new_idx = ui.new_unique_node_index();
            self.maybe_idx.set(Some(new_idx));
        }
        self.maybe_idx.get().unwrap()
    }

}


impl<'a, T> State<'a, T> {

    /// Immutably borrow the internal widget state.
    #[inline]
    pub fn view(&self) -> &T { &self.state }

    /// Mutate the internal widget state and set a flag notifying us that there has been a mutation.
    ///
    /// If this method is *not* called, we assume that there has been no mutation and in turn we do
    /// not need to re-draw the Widget.
    ///
    /// If this method *is* called, we assume that there has been some mutation and in turn will
    /// need to re-draw the Widget. Thus, it is recommended that you *only* call this method if you
    /// need to update the unique state in some way.
    pub fn update<F>(&mut self, f: F) where F: FnOnce(&mut T) {
        self.has_updated = true;
        f(self.state);
    }

}


impl CommonBuilder {
    /// Construct an empty, initialised CommonBuilder.
    pub fn new() -> CommonBuilder {
        CommonBuilder {
            style: CommonStyle::new(),
            maybe_parent_idx: MaybeParent::Unspecified,
            place_on_kid_area: true,
            maybe_graphics_for: None,
            is_floating: false,
            maybe_x_scroll: None,
            maybe_y_scroll: None,
        }
    }
}

impl CommonStyle {
    /// A new default CommonStyle.
    pub fn new() -> Self {
        CommonStyle {
            maybe_x_dimension: None,
            maybe_y_dimension: None,
            maybe_x_position: None,
            maybe_y_position: None,
            maybe_depth: None,
        }
    }
}


impl<W> Positionable for W
    where W: Widget,
{
    #[inline]
    fn x_position(mut self, x: Position) -> Self {
        self.common_mut().style.maybe_x_position = Some(x);
        self
    }
    #[inline]
    fn y_position(mut self, y: Position) -> Self {
        self.common_mut().style.maybe_y_position = Some(y);
        self
    }
    #[inline]
    fn get_x_position<B: Backend>(&self, ui: &Ui<B>) -> Position {
        let from_y_position = || self.common().style.maybe_y_position
            .and_then(|y_pos| infer_position_from_other_position(y_pos, Align::Start));
        self.common().style.maybe_x_position
            .or_else(from_y_position)
            .unwrap_or(self.default_x_position(ui))
    }
    #[inline]
    fn get_y_position<B: Backend>(&self, ui: &Ui<B>) -> Position {
        let from_x_position = || self.common().style.maybe_x_position
            .and_then(|x_pos| infer_position_from_other_position(x_pos, Align::End));
        self.common().style.maybe_y_position
            .or_else(from_x_position)
            .unwrap_or(self.default_y_position(ui))
    }
    #[inline]
    fn depth(mut self, depth: Depth) -> Self {
        self.common_mut().style.maybe_depth = Some(depth);
        self
    }
    #[inline]
    fn get_depth(&self) -> Depth {
        const DEFAULT_DEPTH: Depth = 0.0;
        self.common().style.maybe_depth.unwrap_or(DEFAULT_DEPTH)
    }
}


/// In the case that a position hasn't been given for one of the axes, we must first check to see
/// if we can infer the missing axis position from the other axis.
///
/// This is used within the impl of **Positionable** for **Widget**.
fn infer_position_from_other_position(other_pos: Position, dir_align: Align) -> Option<Position> {
    match other_pos {
        Position::Direction(_, _, maybe_idx) => Some(Position::Align(dir_align, maybe_idx)),
        Position::Place(_, maybe_idx) => Some(Position::Align(Align::Middle, maybe_idx)),
        Position::Relative(_, maybe_idx) => Some(Position::Relative(0.0, maybe_idx)),
        Position::Align(_, _) | Position::Absolute(_) => None,
    }
}


impl<W> Sizeable for W
    where W: Widget,
{
    #[inline]
    fn x_dimension(mut self, w: Dimension) -> Self {
        self.common_mut().style.maybe_x_dimension = Some(w);
        self
    }
    #[inline]
    fn y_dimension(mut self, h: Dimension) -> Self {
        self.common_mut().style.maybe_y_dimension = Some(h);
        self
    }
    #[inline]
    /// We attempt to retrieve the `x` **Dimension** for the widget via the following:
    /// - Check for specified value at `maybe_x_dimension`
    /// - Otherwise, use the default returned by **Widget::default_x_dimension**.
    fn get_x_dimension<B: Backend>(&self, ui: &Ui<B>) -> Dimension {
        self.common().style.maybe_x_dimension.unwrap_or_else(|| self.default_x_dimension(ui))
    }
    #[inline]
    /// We attempt to retrieve the `y` **Dimension** for the widget via the following:
    /// - Check for specified value at `maybe_y_dimension`
    /// - Otherwise, use the default returned by **Widget::default_y_dimension**.
    fn get_y_dimension<B: Backend>(&self, ui: &Ui<B>) -> Dimension {
        self.common().style.maybe_y_dimension.unwrap_or_else(|| self.default_y_dimension(ui))
    }
}
