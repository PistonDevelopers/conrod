
use color::Color;
use elmesque::Element;
use graph::Graph;
use graphics::{Context, Graphics};
use graphics::character::CharacterCache;
use label::FontSize;
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
    /// Tracks whether or not the previous event was a Render event.
    prev_event_was_render: bool,
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


    /// Constructor for a UiContext.
    pub fn new(character_cache: C, theme: Theme) -> Ui<C> {
        const GRAPH_CAPACITY: usize = 512;
        Ui {
            widget_graph: Graph::with_capacity(GRAPH_CAPACITY),
            theme: theme,
            mouse: Mouse::new(),
            keys_just_pressed: Vec::with_capacity(10),
            keys_just_released: Vec::with_capacity(10),
            text_just_entered: Vec::with_capacity(10),
            glyph_cache: GlyphCache(RefCell::new(character_cache)),
            prev_event_was_render: false,
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
        }
    }


    /// Return the dimensions of a widget.
    pub fn widget_size(&self, id: widget::Id) -> Dimensions {
        self.widget_graph[id].rect.dim()
    }


    /// An index to the previously updated widget if there is one.
    pub fn maybe_prev_widget(&self) -> Option<widget::Index> {
        self.maybe_prev_widget_idx
    }


    /// Handle game events and update the state.
    pub fn handle_event<E: GenericEvent>(&mut self, event: &E) {

        // The `Ui` tracks various things during a frame - we'll reset those things here.
        if self.prev_event_was_render {
            self.prev_event_was_render = false;

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
        }

        event.render(|args| {
            self.win_w = args.width as f64;
            self.win_h = args.height as f64;
            self.prev_event_was_render = true;
            self.maybe_widget_under_mouse = self.widget_graph.pick_widget(self.mouse.xy);
            self.maybe_top_scrollable_widget_under_mouse =
                self.widget_graph.pick_top_scrollable_widget(self.mouse.xy);
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
    /// If getting the xy for a widget, its ID should be specified so that we can also consider the
    /// scroll offset of the scrollable parent widgets.
    pub fn get_xy(&self,
                  maybe_idx: Option<widget::Index>,
                  position: Position,
                  dim: Dimensions,
                  h_align: HorizontalAlign,
                  v_align: VerticalAlign) -> Point
    {
        use vecmath::vec2_add;

        let xy = match position {

            Position::Absolute(x, y) => [x, y],

            Position::Relative(x, y, maybe_idx) => match maybe_idx.or(self.maybe_prev_widget()) {
                None => [x, y],
                Some(idx) => vec2_add(self.widget_graph[idx].rect.xy(), [x, y]),
            },

            Position::Direction(direction, px, maybe_idx) => {
                match maybe_idx.or(self.maybe_prev_widget()) {
                    None => [0.0, 0.0],
                    Some(rel_idx) => {
                        use position::Direction;
                        let (rel_xy, rel_w, rel_h) = {
                            let widget = &self.widget_graph[rel_idx];
                            (widget.rect.xy(), widget.rect.w(), widget.rect.h())
                        };

                        match direction {

                            // For vertical directions, we must consider horizontal alignment.
                            Direction::Up | Direction::Down => {
                                // Check whether or not we are aligning to a specific `Ui` element.
                                let (other_x, other_w) = match h_align.1 {
                                    Some(other_idx) => {
                                        let widget = &self.widget_graph[other_idx];
                                        (widget.rect.x(), widget.rect.w())
                                    },
                                    None => (rel_xy[0], rel_w),
                                };
                                let x = other_x + h_align.0.to(other_w, dim[0]);
                                let y = match direction {
                                    Direction::Up   => rel_xy[1] + rel_h / 2.0 + dim[1] / 2.0 + px,
                                    Direction::Down => rel_xy[1] - rel_h / 2.0 - dim[1] / 2.0 - px,
                                    _ => unreachable!(),
                                };
                                [x, y]
                            },

                            // For horizontal directions, we must consider vertical alignment.
                            Direction::Left | Direction::Right => {
                                // Check whether or not we are aligning to a specific `Ui` element.
                                let (other_y, other_h) = match h_align.1 {
                                    Some(other_idx) => {
                                        let widget = &self.widget_graph[other_idx];
                                        (widget.rect.y(), widget.rect.h())
                                    },
                                    None => (rel_xy[1], rel_h),
                                };
                                let y = other_y + v_align.0.to(other_h, dim[1]);
                                let x = match direction {
                                    Direction::Left  => rel_xy[0] - rel_w / 2.0 - dim[0] / 2.0 - px,
                                    Direction::Right => rel_xy[0] + rel_w / 2.0 + dim[0] / 2.0 + px,
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
                    Some(parent_idx) => match self.widget_graph.get_widget(parent_idx) {
                        Some(parent) =>
                            (parent.kid_area.rect.xy_dim(), parent.kid_area.pad),
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
            .map(|idx| vec2_add(xy, self.widget_graph.scroll_offset(idx)))
            .unwrap_or(xy)
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


    /// Helper method for logic shared between draw() and element().
    /// Returns (maybe_captured_mouse, maybe_captured_keyboard).
    fn captures_for_draw(&self) -> (Option<widget::Index>, Option<widget::Index>) {
        let maybe_captured_mouse = match self.maybe_captured_mouse {
            Some(Capturing::Captured(id)) => Some(id),
            _                             => None,
        };
        let maybe_captured_keyboard = match self.maybe_captured_keyboard {
            Some(Capturing::Captured(id)) => Some(id),
            _                             => None,
        };
        (maybe_captured_mouse, maybe_captured_keyboard)
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
        // The graph needs to consider captured widgets when calculating the render depth order.
        let (maybe_captured_mouse, maybe_captured_keyboard) = self.captures_for_draw();

        let Ui {
            ref mut widget_graph,
            ref mut maybe_background_color,
            ref mut maybe_element,
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
        match widget_graph.element_if_changed(maybe_captured_mouse, maybe_captured_keyboard) {

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

                    let element = widget_graph
                        .element(maybe_captured_mouse, maybe_captured_keyboard);

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
        // The graph needs to consider captured widgets when calculating the render depth order.
        let (maybe_captured_mouse, maybe_captured_keyboard) = self.captures_for_draw();

        let Ui {
            ref mut widget_graph,
            ref mut maybe_background_color,
            ref mut maybe_element,
            ..
        } = *self;

        // Request a new `Element` from the graph if there has been some change.
        widget_graph
            .element_if_changed(maybe_captured_mouse, maybe_captured_keyboard)
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
        self.widget_graph.kids_bounding_box(idx)
    }


    /// The **Rect** that represents the maximum fully visible area for the widget with the given
    /// index, including consideration of cropped scroll area.
    ///
    /// Otherwise, return None if the widget is not visible.
    pub fn visible_area<I: Into<widget::Index>>(&self, idx: I) -> Option<Rect> {
        let idx: widget::Index = idx.into();
        self.widget_graph.visible_area(idx)
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
            Some(idx) => ui.widget_graph.parent_of(idx),
            None     => match ui.maybe_prev_widget_idx {
                Some(idx) => ui.widget_graph.parent_of(idx),
                None     => ui.maybe_current_parent_idx,
            },
        },
        Position::Direction(_, _, maybe_idx) => match maybe_idx {
            Some(idx) => ui.widget_graph.parent_of(idx),
            None     => match ui.maybe_prev_widget_idx {
                Some(idx) => ui.widget_graph.parent_of(idx),
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
    ui.widget_graph.pre_update_cache(widget);
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
    ui.widget_graph.post_update_cache(widget);
}


/// Clear the background with the given color.
pub fn clear_with<C>(ui: &mut Ui<C>, color: Color) {
    ui.maybe_background_color = Some(color);
}


