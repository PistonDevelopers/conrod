
use {CharacterCache, FontSize, Scalar};
use color::Color;
use elmesque::Element;
use graph::{self, Graph, NodeIndex, Walker};
use graphics::{Context, Graphics};
use mouse::{self, Mouse};
use input;
use input::{
    GenericEvent,
    MouseCursorEvent,
    MouseScrollEvent,
    PressEvent,
    ReleaseEvent,
    RenderEvent,
    TextEvent,
};
use position::{Dimensions, HorizontalAlign, Padding, Point, Position, Rect, VerticalAlign};
use std::cell::RefCell;
use std::collections::HashSet;
use std::io::Write;
use theme::Theme;
use widget::{self, Widget};


/// Indicates whether or not the Mouse has been captured by a widget.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum Capturing {
    /// The Ui is captured by the Ui element with the given widget::Index.
    Captured(widget::Index),
    /// The Ui has just been uncaptured.
    JustReleased,
}

/// `Ui` is the most important type within Conrod and is necessary for rendering and maintaining
/// widget state.
/// # Ui Handles the following:
/// * Contains the state of all widgets which can be indexed via their widget::Index.
/// * Stores rendering state for each widget until the end of each render cycle.
/// * Contains the theme used for default styling of the widgets.
/// * Maintains the latest user input state (for mouse and keyboard).
/// * Maintains the latest window dimensions.
pub struct Ui<C> {
    /// The theme used to set default styling for widgets.
    pub theme: Theme,
    /// Cache for character textures, used for label width calculation and glyph rendering.
    pub glyph_cache: GlyphCache<C>,
    /// An index into the root widget of the graph, representing the entire window.
    pub window: NodeIndex,
    /// Window width.
    pub win_w: f64,
    /// Window height.
    pub win_h: f64,
    /// The Widget cache, storing state for all widgets.
    widget_graph: Graph,
    /// The latest received mouse state.
    mouse: Mouse,
    /// Keys that have been pressed since the end of the last render cycle.
    keys_just_pressed: Vec<input::keyboard::Key>,
    /// Keys that have been released since the end of the last render cycle.
    keys_just_released: Vec<input::keyboard::Key>,
    /// Text that has been entered since the end of the last render cycle.
    text_just_entered: Vec<String>,
    /// The widget::Index of the widget that was last updated/set.
    maybe_prev_widget_idx: Option<widget::Index>,
    /// The widget::Index of the last widget used as a parent for another widget.
    maybe_current_parent_idx: Option<widget::Index>,
    /// If the mouse is currently over a widget, its ID will be here.
    maybe_widget_under_mouse: Option<widget::Index>,
    /// The ID of the top-most scrollable widget under the cursor (if there is one).
    maybe_top_scrollable_widget_under_mouse: Option<widget::Index>,
    /// The widget::Index of the widget currently capturing mouse input if there is one.
    maybe_captured_mouse: Option<Capturing>,
    /// The widget::Index of the widget currently capturing keyboard input if there is one.
    maybe_captured_keyboard: Option<Capturing>,
    /// The number of frames that that will be used for the `redraw_count` when `need_redraw` is
    /// triggered.
    num_redraw_frames: u8,
    /// Whether or not the `Ui` needs to be re-drawn to screen.
    redraw_count: u8,
    /// A background color to clear the screen with before drawing if one was given.
    maybe_background_color: Option<Color>,
    /// The latest element returned/drawn that represents the entire widget graph.
    maybe_element: Option<Element>,

    /// The order in which widgets from the `widget_graph` are drawn.
    depth_order: graph::DepthOrder,
    /// The set of widgets that have been updated since the beginning of the last `set_widgets`
    /// stage.
    updated_widgets: HashSet<NodeIndex>,
}

/// A wrapper over the current user input state.
#[derive(Clone, Debug)]
pub struct UserInput<'a> {
    /// Mouse state only if it is currently available to the widget after considering capturing.
    pub maybe_mouse: Option<Mouse>,
    /// The universal state of the Mouse, regardless of capturing.
    pub global_mouse: Mouse,
    /// Keys pressed since the last cycle.
    pub pressed_keys: &'a [input::keyboard::Key],
    /// Keys released since the last cycle.
    pub released_keys: &'a [input::keyboard::Key],
    /// Text entered since the last cycle.
    pub entered_text: &'a [String],
    /// Current dimensions of the window.
    pub window_dim: Dimensions,
}

/// A wrapper over some CharacterCache, exposing it's functionality via a RefCell.
pub struct GlyphCache<C>(RefCell<C>);


impl<C> GlyphCache<C> where C: CharacterCache {
    /// Return the width of a character.
    pub fn char_width(&self, font_size: FontSize, ch: char) -> f64 {
        self.0.borrow_mut().character(font_size, ch).width()
    }
    /// Return the width of the given text.
    pub fn width(&self, font_size: FontSize, text: &str) -> f64 {
        self.0.borrow_mut().width(font_size, text)
    }
}

impl<C> ::std::ops::Deref for GlyphCache<C> {
    type Target = RefCell<C>;
    fn deref<'a>(&'a self) -> &'a RefCell<C> { &self.0 }
}

impl<C> ::std::ops::DerefMut for GlyphCache<C> {
    fn deref_mut<'a>(&'a mut self) -> &'a mut RefCell<C> { &mut self.0 }
}


/// Each time conrod is required to redraw the GUI, it must draw for at least the next three frames
/// to ensure that, in the case that graphics buffers are being swapped, we have filled each
/// buffer. Otherwise if we don't draw into each buffer, we will probably be subject to flickering.
pub const SAFE_REDRAW_COUNT: u8 = 3;


impl<C> Ui<C> {


    /// A new, empty **Ui**.
    pub fn new(character_cache: C, theme: Theme) -> Self {
        let widget_graph = Graph::new();
        let depth_order = graph::DepthOrder::new();
        let updated_widgets = HashSet::new();
        Self::new_internal(character_cache, theme, widget_graph, depth_order, updated_widgets)
    }

    /// A new **Ui** with the capacity given as a number of widgets.
    pub fn with_capacity(character_cache: C, theme: Theme, n_widgets: usize) -> Self {
        let widget_graph = Graph::with_node_capacity(n_widgets);
        let depth_order = graph::DepthOrder::with_node_capacity(n_widgets);
        let updated_widgets = HashSet::with_capacity(n_widgets);
        Self::new_internal(character_cache, theme, widget_graph, depth_order, updated_widgets)
    }

    /// An internal constructor to share logic between the `new` and `with_capacity` constructors.
    fn new_internal(character_cache: C,
                    theme: Theme,
                    mut widget_graph: Graph,
                    depth_order: graph::DepthOrder,
                    updated_widgets: HashSet<NodeIndex>) -> Self
    {
        let window = widget_graph.add_placeholder();
        Ui {
            widget_graph: widget_graph,
            theme: theme,
            window: window,
            mouse: Mouse::new(),
            keys_just_pressed: Vec::with_capacity(10),
            keys_just_released: Vec::with_capacity(10),
            text_just_entered: Vec::with_capacity(10),
            glyph_cache: GlyphCache(RefCell::new(character_cache)),
            win_w: 0.0,
            win_h: 0.0,
            maybe_prev_widget_idx: None,
            maybe_current_parent_idx: None,
            maybe_widget_under_mouse: None,
            maybe_top_scrollable_widget_under_mouse: None,
            maybe_captured_mouse: None,
            maybe_captured_keyboard: None,
            num_redraw_frames: SAFE_REDRAW_COUNT,
            redraw_count: SAFE_REDRAW_COUNT,
            maybe_background_color: None,
            maybe_element: None,
            depth_order: depth_order,
            updated_widgets: updated_widgets,
        }
    }

    /// The **Rect** for the widget at the given index.
    ///
    /// Returns `None` if there is no widget for the given index.
    pub fn rect_of<I: Into<widget::Index>>(&self, idx: I) -> Option<Rect> {
        let idx: widget::Index = idx.into();
        self.widget_graph.widget(idx).map(|widget| widget.rect)
    }

    /// The absolute width of the widget at the given index.
    ///
    /// Returns `None` if there is no widget for the given index.
    pub fn width_of<I: Into<widget::Index>>(&self, idx: I) -> Option<Scalar> {
        self.rect_of(idx).map(|rect| rect.w())
    }

    /// The absolute height of the widget at the given index.
    ///
    /// Returns `None` if there is no widget for the given index.
    pub fn height_of<I: Into<widget::Index>>(&self, idx: I) -> Option<Scalar> {
        self.rect_of(idx).map(|rect| rect.h())
    }

    /// The absolute dimensions for the widget at the given index.
    ///
    /// Returns `None` if there is no widget for the given index.
    pub fn dim_of<I: Into<widget::Index>>(&self, idx: I) -> Option<Dimensions> {
        self.rect_of(idx).map(|rect| rect.dim())
    }

    /// The coordinates for the widget at the given index.
    ///
    /// Returns `None` if there is no widget for the given index.
    pub fn xy_of<I: Into<widget::Index>>(&self, idx: I) -> Option<Point> {
        self.rect_of(idx).map(|rect| rect.xy())
    }

    /// An index to the previously updated widget if there is one.
    pub fn maybe_prev_widget(&self) -> Option<widget::Index> {
        self.maybe_prev_widget_idx
    }

    /// Borrow the **Ui**'s `widget_graph`.
    pub fn widget_graph(&self) -> &Graph {
        &self.widget_graph
    }

    /// Handle game events and update the state.
    pub fn handle_event<E: GenericEvent>(&mut self, event: &E) {

        event.render(|args| {
            self.win_w = args.width as f64;
            self.win_h = args.height as f64;
        });

        event.mouse_cursor(|x, y| {
            // Convert mouse coords to (0, 0) origin.
            self.mouse.xy = [x - self.win_w / 2.0, -(y - self.win_h / 2.0)];
        });

        event.mouse_scroll(|x, y| {
            self.mouse.scroll.x += x;
            self.mouse.scroll.y += y;
        });

        event.press(|button_type| {
            use input::Button;
            use input::MouseButton::{Left, Middle, Right};

            match button_type {
                Button::Mouse(button) => {
                    let mouse_button = match button {
                        Left => &mut self.mouse.left,
                        Right => &mut self.mouse.right,
                        Middle => &mut self.mouse.middle,
                        _ => &mut self.mouse.unknown,
                    };
                    mouse_button.position = mouse::ButtonPosition::Down;
                    mouse_button.was_just_pressed = true;
                },
                Button::Keyboard(key) => self.keys_just_pressed.push(key),
                _ => {}
            }
        });

        event.release(|button_type| {
            use input::Button;
            use input::MouseButton::{Left, Middle, Right};
            match button_type {
                Button::Mouse(button) => {
                    let mouse_button = match button {
                        Left => &mut self.mouse.left,
                        Right => &mut self.mouse.right,
                        Middle => &mut self.mouse.middle,
                        _ => &mut self.mouse.unknown,
                    };
                    mouse_button.position = mouse::ButtonPosition::Up;
                    mouse_button.was_just_released = true;
                },
                Button::Keyboard(key) => self.keys_just_released.push(key),
                _ => {}
            }
        });

        event.text(|text| {
            self.text_just_entered.push(text.to_string())
        });
    }


    /// Get the centred xy coords for some given `Dimension`s, `Position` and alignment.
    ///
    /// If getting the xy for a specific widget, its `widget::Index` should be specified so that we
    /// can also consider the scroll offset of the scrollable parent widgets.
    ///
    /// The `place_on_kid_area` argument specifies whether or not **Place** **Position** variants
    /// should target a **Widget**'s `kid_area`, or simply the **Widget**'s total area.
    pub fn calc_xy(&self,
                   maybe_idx: Option<widget::Index>,
                   position: Position,
                   dim: Dimensions,
                   h_align: HorizontalAlign,
                   v_align: VerticalAlign,
                   place_on_kid_area: bool) -> Point
    {
        use vecmath::vec2_add;

        let xy = match position {

            Position::Absolute(x, y) => [x, y],

            Position::Relative(x, y, maybe_idx) => match maybe_idx.or(self.maybe_prev_widget()) {
                None => [x, y],
                Some(idx) => self.widget_graph.widget(idx)
                    .map(|w| vec2_add(w.rect.xy(), [x, y]))
                    .unwrap_or([x, y]),
            },

            Position::Direction(direction, px, maybe_idx) => {
                match maybe_idx.or(self.maybe_prev_widget()) {
                    None => [0.0, 0.0],
                    Some(rel_idx) => {
                        use position::Direction;
                        let (rel_x, rel_y, rel_w, rel_h) = self.widget_graph.widget(rel_idx)
                            .map(|widget| widget.rect.x_y_w_h())
                            .unwrap_or_else(|| (0.0, 0.0, 0.0, 0.0));

                        match direction {

                            // For vertical directions, we must consider horizontal alignment.
                            Direction::Up | Direction::Down => {
                                // Check whether or not we are aligning to a specific `Ui` element.
                                let (other_x, other_w) = match h_align.1 {
                                    Some(other_idx) => self.widget_graph.widget(other_idx)
                                        .map(|widget| (widget.rect.x(), widget.rect.w()))
                                        .unwrap_or_else(|| (rel_x, rel_w)),
                                    None => (rel_x, rel_w),
                                };
                                let x = other_x + h_align.0.to(other_w, dim[0]);
                                let y = match direction {
                                    Direction::Up   => rel_y + rel_h / 2.0 + dim[1] / 2.0 + px,
                                    Direction::Down => rel_y - rel_h / 2.0 - dim[1] / 2.0 - px,
                                    _ => unreachable!(),
                                };
                                [x, y]
                            },

                            // For horizontal directions, we must consider vertical alignment.
                            Direction::Left | Direction::Right => {
                                // Check whether or not we are aligning to a specific `Ui` element.
                                let (other_y, other_h) = match h_align.1 {
                                    Some(other_idx) => self.widget_graph.widget(other_idx)
                                        .map(|widget| (widget.rect.y(), widget.rect.h()))
                                        .unwrap_or_else(|| (rel_y, rel_h)),
                                    None => (rel_y, rel_h),
                                };
                                let y = other_y + v_align.0.to(other_h, dim[1]);
                                let x = match direction {
                                    Direction::Left  => rel_x - rel_w / 2.0 - dim[0] / 2.0 - px,
                                    Direction::Right => rel_x + rel_w / 2.0 + dim[0] / 2.0 + px,
                                    _ => unreachable!(),
                                };
                                [x, y]
                            },

                        }
                    },
                }
            },

            Position::Place(place, maybe_parent_idx) => {
                let window = || (([0.0, 0.0], [self.win_w, self.win_h]), Padding::none());
                let maybe_parent = maybe_parent_idx.or(self.maybe_current_parent_idx);
                let ((target_xy, target_dim), pad) = match maybe_parent {
                    Some(parent_idx) => match self.widget_graph.widget(parent_idx) {
                        Some(parent) => match place_on_kid_area {
                            true => (parent.kid_area.rect.xy_dim(), parent.kid_area.pad),
                            false => (parent.rect.xy_dim(), Padding::none()),
                        },
                        // Sometimes the children are placed prior to their parents being set for
                        // the first time. If this is the case, we'll just place them on the window
                        // until we have information about the parents on the next update.
                        None => window(),
                    },
                    None => window(),
                };
                let place_xy = place.within(target_dim, dim);
                let relative_xy = vec2_add(place_xy, pad.offset_from(place));
                vec2_add(target_xy, relative_xy)
            },

        };

        // Add the widget's parents' total combined scroll offset to the given xy.
        maybe_idx
            .map(|idx| vec2_add(xy, graph::algo::scroll_offset(&self.widget_graph, idx)))
            .unwrap_or(xy)
    }


    /// A function within which all widgets are instantiated by the user, normally situated within
    /// the "update" stage of an event loop.
    pub fn set_widgets<F>(&mut self, user_widgets_fn: F)
        where C: CharacterCache,
              F: FnOnce(&mut Self),
    {
        self.maybe_prev_widget_idx = None;
        self.maybe_current_parent_idx = None;

        // If the mouse / keyboard capturing was just released by a widget, reset to None ready
        // for capturing once again.
        if let Some(Capturing::JustReleased) = self.maybe_captured_mouse {
            self.maybe_captured_mouse = None;
        }
        if let Some(Capturing::JustReleased) = self.maybe_captured_keyboard {
            self.maybe_captured_keyboard = None;
        }

        self.maybe_widget_under_mouse =
            graph::algo::pick_widget(&self.widget_graph, &self.depth_order.indices, self.mouse.xy);
        self.maybe_top_scrollable_widget_under_mouse =
            graph::algo::pick_scrollable_widget(&self.widget_graph, &self.depth_order.indices, self.mouse.xy);
        self.updated_widgets.clear();

        // Instantiate the root `Window` `Widget`.
        //
        // This widget acts as the parent-most widget and root node for the Ui's `widget_graph`,
        // upon which all other widgets are placed.
        {
            use ::{color, Colorable, Frameable, FramedRectangle, Positionable, Widget};
            type Window = FramedRectangle;
            Window::new([self.win_w, self.win_h])
                .parent(None::<widget::Index>)
                .xy(0.0, 0.0)
                .frame(0.0)
                .frame_color(color::black().alpha(0.0))
                .color(self.maybe_background_color.unwrap_or(color::black().alpha(0.0)))
                .set(self.window, self);
        }

        self.maybe_current_parent_idx = Some(self.window.into());

        // Call the given user function for instantiating Widgets.
        user_widgets_fn(self);

        // Reset all widgets that were not updated during the given `user_widget_fn`.
        fn reset_non_updated_widgets(graph: &mut Graph, updated: &HashSet<NodeIndex>) -> bool {
            (0..graph.node_count())
                .map(|i| NodeIndex::new(i))
                .filter(|idx| !updated.contains(idx))
                // TODO: should also remove all **Depth** children here (need a **Depth** toposort).
                .map(|idx| graph.reset_node(idx))
                .count() > 0
        }

        // Reset all non-updated widgets.
        let one_or_more_widgets_were_reset = {
            let Ui { ref mut widget_graph, ref updated_widgets, .. } = *self;
            reset_non_updated_widgets(widget_graph, updated_widgets)
        };

        // If one or more widgets were reset, we need to re-draw our GUI.
        if one_or_more_widgets_were_reset {
            self.needs_redraw();
        }

        // Update the graph's internal depth_order while considering the captured input.
        let maybe_captured_mouse = match self.maybe_captured_mouse {
            Some(Capturing::Captured(id)) => Some(id),
            _                             => None,
        };
        let maybe_captured_keyboard = match self.maybe_captured_keyboard {
            Some(Capturing::Captured(id)) => Some(id),
            _                             => None,
        };

        // Update the **DepthOrder** so that it reflects the **Graph**'s current state.
        {
            let Ui { ref widget_graph, ref mut depth_order, window, .. } = *self;
            depth_order.update(widget_graph, window, maybe_captured_mouse, maybe_captured_keyboard);
        }

        // Clear text and key buffers.
        self.keys_just_pressed.clear();
        self.keys_just_released.clear();
        self.text_just_entered.clear();

        // Reset the mouse state.
        self.mouse.scroll = mouse::Scroll { x: 0.0, y: 0.0 };
        self.mouse.left.reset_pressed_and_released();
        self.mouse.middle.reset_pressed_and_released();
        self.mouse.right.reset_pressed_and_released();
        self.mouse.unknown.reset_pressed_and_released();
    }


    /// Set the number of frames that the `Ui` should draw in the case that `needs_redraw` is
    /// called. The default is `3` (see the SAFE_REDRAW_COUNT docs for details).
    pub fn set_num_redraw_frames(&mut self, num_frames: u8) {
        self.num_redraw_frames = num_frames;
    }


    /// Tells the `Ui` that it needs to be re-draw everything. It does this by setting the redraw
    /// count to `num_redraw_frames`. See the docs for `set_num_redraw_frames`, SAFE_REDRAW_COUNT
    /// or `draw_if_changed` for more info on how/why the redraw count is used.
    pub fn needs_redraw(&mut self) {
        self.redraw_count = self.num_redraw_frames;
    }


    /// Compiles the `Ui`'s entire widget `Graph` in its current state into a single
    /// `elmesque::Element` and returns a reference to it.
    ///
    /// This allows a user to take all information necessary for drawing within a single type,
    /// which can be sent across threads or used to draw later on rather than drawing the whole
    /// graph immediately (as does the `Ui::draw` method).
    ///
    /// Producing an `Element` also allows for simpler interoperation with `elmesque` (a purely
    /// functional graphics layout crate which conrod uses internally).
    pub fn element(&mut self) -> &Element {
        let Ui {
            ref mut widget_graph,
            ref mut maybe_background_color,
            ref mut maybe_element,
            ref depth_order,
            ..
        } = *self;

        let maybe_bg_color = maybe_background_color.take();

        // A function to simplify combining an element with the given background color.
        let with_background_color = |element: Element| -> Element {
            match maybe_bg_color {
                Some(color) => element.clear(color),
                None => element,
            }
        };

        // Check to see whether or not there is a new element to be stored.
        match widget_graph.element_if_changed(&depth_order.indices) {

            // If there's been some change in element, we'll store the new one and return a
            // reference to it.
            Some(new_element) => {
                *maybe_element = Some(with_background_color(new_element));
                maybe_element.as_ref().unwrap()
            },

            // Otherwise, we'll check to see if we have a pre-stored one that we can return a
            // reference to.
            None => match maybe_element {

                // If we do have some stored element, we'll clone that.
                &mut Some(ref element) => element,

                // Otherwise this must be the first time an `element` is requested so we'll
                // request a new `Element` from our widget graph.
                maybe_element @ &mut None => {

                    let element = widget_graph.element(&depth_order.indices);

                    // Store the new `Element` (along with it's background color).
                    *maybe_element = Some(with_background_color(element));

                    // We can unwrap here as we *know* that we have `Some` element.
                    maybe_element.as_ref().unwrap()
                },
            },
        }
    }


    /// Same as `Ui::element`, but only returns an `&Element` if the stored `&Element` has changed
    /// since the last time `Ui::element` or `Ui::element_if_changed` was called.
    pub fn element_if_changed(&mut self) -> Option<&Element> {
        let Ui {
            ref mut widget_graph,
            ref mut maybe_background_color,
            ref mut maybe_element,
            ref depth_order,
            ..
        } = *self;

        // Request a new `Element` from the graph if there has been some change.
        widget_graph
            .element_if_changed(&depth_order.indices)
            .map(move |element| {
                let element = match maybe_background_color.take() {
                    // If we've been given a background color for the gui, construct a Cleared Element.
                    Some(color) => element.clear(color),
                    None => element
                };
                // If we have a new `Element` we'll update our stored `Element`.
                *maybe_element = Some(element.clone());
                maybe_element.as_ref().unwrap()
            })
    }


    /// Draw the `Ui` in it's current state.
    /// NOTE: If you don't need to redraw your conrod GUI every frame, it is recommended to use the
    /// `Ui::draw_if_changed` method instead.
    /// See the `Graph::draw` method for more details on how Widgets are drawn.
    /// See the `graph::update_visit_order` function for details on how the render order is
    /// determined.
    pub fn draw<G>(&mut self, context: Context, graphics: &mut G)
        where
            C: CharacterCache,
            G: Graphics<Texture = C::Texture>,
    {
        use elmesque::Renderer;
        use std::ops::DerefMut;

        // Ensure that `maybe_element` is `Some(element)` with the latest `Element`.
        self.element();

        let Ui {
            ref mut glyph_cache,
            ref mut redraw_count,
            ref maybe_element,
            ..
        } = *self;

        // We know that `maybe_element` is `Some` due to calling `self.element` above, thus we can
        // safely unwrap our reference to it to use for drawing.
        let element = maybe_element.as_ref().unwrap();

        // Construct the elmesque Renderer for rendering the Elements.
        let mut ref_mut_character_cache = glyph_cache.0.borrow_mut();
        let character_cache = ref_mut_character_cache.deref_mut();
        let mut renderer = Renderer::new(context, graphics).character_cache(character_cache);

        // Renderer the `Element` to the screen.
        element.draw(&mut renderer);

        // Because we're about to draw everything, take one from the redraw count.
        if *redraw_count > 0 {
            *redraw_count = *redraw_count - 1;
        }
    }


    /// Same as the `Ui::draw` method, but *only* draws if the `redraw_count` is greater than 0.
    /// The `redraw_count` is set to `SAFE_REDRAW_COUNT` whenever a `Widget` produces a new
    /// `Element` because its state has changed.
    /// It can also be triggered manually by the user using the `Ui::needs_redraw` method.
    ///
    /// This method is generally preferred over `Ui::draw` as it requires far less CPU usage, only
    /// redrawing to the screen if necessary.
    ///
    /// Note that when `Ui::needs_redraw` is triggered, it sets the `redraw_count` to 3 by default.
    /// This ensures that conrod is drawn to each buffer in the case that there is buffer swapping
    /// happening. Let us know if you need finer control over this and we'll expose a way for you
    /// to set the redraw count manually.
    pub fn draw_if_changed<G>(&mut self, context: Context, graphics: &mut G)
        where
            C: CharacterCache,
            G: Graphics<Texture = C::Texture>,
    {
        if self.widget_graph.have_any_elements_changed() {
            self.redraw_count = self.num_redraw_frames;
        }
        if self.redraw_count > 0 {
            self.draw(context, graphics);
        }
    }


    /// The **Rect** that bounds the kids of the widget with the given index.
    pub fn kids_bounding_box<I: Into<widget::Index>>(&self, idx: I) -> Option<Rect> {
        let idx: widget::Index = idx.into();
        graph::algo::bounding_box(&self.widget_graph, false, None, true, idx, None)
    }


    /// The **Rect** that represents the maximum fully visible area for the widget with the given
    /// index, including consideration of cropped scroll area.
    ///
    /// Otherwise, return None if the widget is not visible.
    pub fn visible_area<I: Into<widget::Index>>(&self, idx: I) -> Option<Rect> {
        let idx: widget::Index = idx.into();
        graph::algo::visible_area_of_widget(&self.widget_graph, idx)
    }

}


/// A mutable reference to the given `Ui`'s widget `Graph`.
pub fn widget_graph_mut<C>(ui: &mut Ui<C>) -> &mut Graph {
    &mut ui.widget_graph
}


/// Check the given position for an attached parent widget.
pub fn parent_from_position<C>(ui: &Ui<C>, position: Position) -> Option<widget::Index> {
    match position {
        Position::Relative(_, _, maybe_idx) => match maybe_idx {
            Some(idx) => ui.widget_graph.depth_parent(idx),
            None     => match ui.maybe_prev_widget_idx {
                Some(idx) => ui.widget_graph.depth_parent(idx),
                None     => ui.maybe_current_parent_idx,
            },
        },
        Position::Direction(_, _, maybe_idx) => match maybe_idx {
            Some(idx) => ui.widget_graph.depth_parent(idx),
            None     => match ui.maybe_prev_widget_idx {
                Some(idx) => ui.widget_graph.depth_parent(idx),
                None     => ui.maybe_current_parent_idx,
            },
        },
        Position::Place(_, maybe_parent_idx) => maybe_parent_idx.or(ui.maybe_current_parent_idx),
        _ => ui.maybe_current_parent_idx,
    }
}


/// A function to allow the position matrix to set the current parent within the `Ui`.
pub fn set_current_parent_idx<C>(ui: &mut Ui<C>, idx: widget::Index) {
    ui.maybe_current_parent_idx = Some(idx);
}


/// Return the user input state available for the widget with the given ID.
/// Take into consideration whether or not each input type is captured.
pub fn user_input<'a, C>(ui: &'a Ui<C>, idx: widget::Index) -> UserInput<'a> {
    let maybe_mouse = get_mouse_state(ui, idx);
    let global_mouse = ui.mouse;
    let without_keys = || UserInput {
        maybe_mouse: maybe_mouse,
        global_mouse: global_mouse,
        pressed_keys: &[],
        released_keys: &[],
        entered_text: &[],
        window_dim: [ui.win_w, ui.win_h],
    };
    let with_keys = || UserInput {
        maybe_mouse: maybe_mouse,
        global_mouse: global_mouse,
        pressed_keys: &ui.keys_just_pressed,
        released_keys: &ui.keys_just_released,
        entered_text: &ui.text_just_entered,
        window_dim: [ui.win_w, ui.win_h],
    };
    match ui.maybe_captured_keyboard {
        Some(Capturing::Captured(captured_idx)) =>
            if idx == captured_idx { with_keys()    }
            else                   { without_keys() },
        Some(Capturing::JustReleased) => without_keys(),
        None => with_keys(),
    }
}


/// Return the current mouse state.
///
/// If the Ui has been captured and the given id doesn't match the captured id, return None.
pub fn get_mouse_state<C>(ui: &Ui<C>, idx: widget::Index) -> Option<Mouse> {
    match ui.maybe_captured_mouse {
        Some(Capturing::Captured(captured_idx)) =>
            if idx == captured_idx { Some(ui.mouse) } else { None },
        Some(Capturing::JustReleased) =>
            None,
        None => match ui.maybe_captured_keyboard {
            Some(Capturing::Captured(captured_idx)) =>
                if idx == captured_idx { Some(ui.mouse) } else { None },
            _ =>
                if Some(idx) == ui.maybe_widget_under_mouse 
                || Some(idx) == ui.maybe_top_scrollable_widget_under_mouse {
                    Some(ui.mouse)
                } else {
                    None
                },
        },
    }
}


/// Indicate that the widget with the given widget::Index has captured the mouse.
///
/// Returns true if the mouse was successfully captured.
/// 
/// Returns false if the mouse was already captured.
pub fn mouse_captured_by<C>(ui: &mut Ui<C>, idx: widget::Index) -> bool {
    // If the mouse isn't already captured, set idx as the capturing widget.
    if let None = ui.maybe_captured_mouse {
        ui.maybe_captured_mouse = Some(Capturing::Captured(idx));
        return true;
    }
    false
}


/// Indicate that the widget is no longer capturing the mouse.
///
/// Returns true if the mouse was sucessfully released.
///
/// Returns false if the mouse wasn't captured by the widget in the first place.
pub fn mouse_uncaptured_by<C>(ui: &mut Ui<C>, idx: widget::Index) -> bool {
    // Check that we are indeed the widget that is currently capturing the Mouse before releasing.
    if ui.maybe_captured_mouse == Some(Capturing::Captured(idx)) {
        ui.maybe_captured_mouse = Some(Capturing::JustReleased);
        return true;
    }
    false
}

/// Indicate that the widget with the given widget::Index has captured the keyboard.
///
/// Returns true if the keyboard was successfully captured.
///
/// Returns false if the keyboard was already captured by another widget.
pub fn keyboard_captured_by<C>(ui: &mut Ui<C>, idx: widget::Index) -> bool {
    match ui.maybe_captured_keyboard {
        Some(Capturing::Captured(captured_idx)) => if idx != captured_idx {
            writeln!(::std::io::stderr(),
                    "Warning: {:?} tried to capture the keyboard, however it is \
                     already captured by {:?}.", idx, captured_idx).unwrap();
        },
        Some(Capturing::JustReleased) => {
            writeln!(::std::io::stderr(),
                    "Warning: {:?} tried to capture the keyboard, however it was \
                     already captured.", idx).unwrap();
        },
        None => {
            ui.maybe_captured_keyboard = Some(Capturing::Captured(idx));
            return true;
        },
    }
    false
}


/// Indicate that the widget is no longer capturing the keyboard.
///
/// Returns true if the keyboard was successfully released.
///
/// Returns false if the keyboard wasn't captured by the given widget in the first place.
pub fn keyboard_uncaptured_by<C>(ui: &mut Ui<C>, idx: widget::Index) -> bool {
    match ui.maybe_captured_keyboard {
        Some(Capturing::Captured(captured_idx)) => if idx != captured_idx {
            writeln!(::std::io::stderr(),
                    "Warning: {:?} tried to uncapture the keyboard, however it is \
                     actually captured by {:?}.", idx, captured_idx).unwrap();
        } else {
            ui.maybe_captured_keyboard = Some(Capturing::JustReleased);
            return true;
        },
        Some(Capturing::JustReleased) => {
            writeln!(::std::io::stderr(),
                    "Warning: {:?} tried to uncapture the keyboard, however it had \
                     already been released this cycle.", idx).unwrap();
        },
        None => {
            writeln!(::std::io::stderr(),
                    "Warning: {:?} tried to uncapture the keyboard, however the mouse \
                     was not captured", idx).unwrap();
        },
    }
    false
}


/// Cache some `PreUpdateCache` widget data into the widget graph.
/// Set the widget that is being cached as the new `prev_widget`.
/// Set the widget's parent as the new `current_parent`.
pub fn pre_update_cache<C>(ui: &mut Ui<C>, widget: widget::PreUpdateCache) where
    C: CharacterCache,
{
    ui.maybe_prev_widget_idx = Some(widget.idx);
    ui.maybe_current_parent_idx = widget.maybe_parent_idx;
    ui.widget_graph.pre_update_cache(ui.window, widget);
}

/// Cache some `PostUpdateCache` widget data into the widget graph.
/// Set the widget that is being cached as the new `prev_widget`.
/// Set the widget's parent as the new `current_parent`.
pub fn post_update_cache<C, W>(ui: &mut Ui<C>, widget: widget::PostUpdateCache<W>) where
    C: CharacterCache,
    W: Widget,
    W::State: 'static,
    W::Style: 'static,
{
    ui.maybe_prev_widget_idx = Some(widget.idx);
    ui.maybe_current_parent_idx = widget.maybe_parent_idx;
    let widget_idx = widget.idx;
    ui.widget_graph.post_update_cache(widget);

    // Add the widget's `NodeIndex` to the set of updated widgets.
    let node_idx = ui.widget_graph.node_index(widget_idx).expect("No NodeIndex");
    ui.updated_widgets.insert(node_idx);
}


/// Clear the background with the given color.
pub fn clear_with<C>(ui: &mut Ui<C>, color: Color) {
    ui.maybe_background_color = Some(color);
}

