
use elmesque::Element;
use graph::Graph;
use graphics::{Context, Graphics};
use graphics::character::CharacterCache;
use label::FontSize;
use mouse::{ButtonState, Mouse, Scroll};
use piston::input;
use piston::event::{
    GenericEvent,
    MouseCursorEvent,
    MouseScrollEvent,
    PressEvent,
    ReleaseEvent,
    RenderEvent,
    TextEvent,
};
use position::{Dimensions, HorizontalAlign, Padding, Point, Position, VerticalAlign};
use std::any::Any;
use std::cell::RefCell;
use std::io::Write;
use theme::Theme;
use widget::{self, Widget, WidgetId};


/// Indicates whether or not the Mouse has been captured by a widget.
#[derive(Copy, Clone, Debug)]
enum Capturing {
    /// The Ui is captured by the Ui element with the given WidgetId.
    Captured(WidgetId),
    /// The Ui has just been uncaptured.
    JustReleased,
}

/// `Ui` is the most important type within Conrod and is necessary for rendering and maintaining
/// widget state.
/// # Ui Handles the following:
/// * Contains the state of all widgets which can be indexed via their WidgetId.
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
    /// The WidgetId of the widget that was last updated/set.
    maybe_prev_widget_id: Option<WidgetId>,
    /// The WidgetId of the last widget used as a parent for another widget.
    maybe_current_parent_id: Option<WidgetId>,
    /// If the mouse is currently over a widget, its ID will be here.
    maybe_widget_under_mouse: Option<WidgetId>,
    /// The WidgetId of the widget currently capturing mouse input if there is one.
    maybe_captured_mouse: Option<Capturing>,
    /// The WidgetId of the widget currently capturing keyboard input if there is one.
    maybe_captured_keyboard: Option<Capturing>,
}

/// A wrapper over the current user input state.
#[derive(Clone, Debug)]
pub struct UserInput<'a> {
    /// Mouse state if it is available.
    pub maybe_mouse: Option<Mouse>,
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


impl<C> Ui<C> {

    /// Constructor for a UiContext.
    pub fn new(character_cache: C, theme: Theme) -> Ui<C> {
        const GRAPH_CAPACITY: usize = 512;
        Ui {
            widget_graph: Graph::with_capacity(GRAPH_CAPACITY),
            theme: theme,
            mouse: Mouse::new([0.0, 0.0], ButtonState::Up, ButtonState::Up, ButtonState::Up),
            keys_just_pressed: Vec::with_capacity(10),
            keys_just_released: Vec::with_capacity(10),
            text_just_entered: Vec::with_capacity(10),
            glyph_cache: GlyphCache(RefCell::new(character_cache)),
            prev_event_was_render: false,
            win_w: 0.0,
            win_h: 0.0,
            maybe_prev_widget_id: None,
            maybe_current_parent_id: None,
            maybe_widget_under_mouse: None,
            maybe_captured_mouse: None,
            maybe_captured_keyboard: None,
        }
    }

    /// Return the dimensions of a widget.
    pub fn widget_size(&self, id: WidgetId) -> Dimensions {
        self.widget_graph[id].dim
    }

    /// Handle game events and update the state.
    pub fn handle_event<E: GenericEvent>(&mut self, event: &E) {

        // The `Ui` tracks various things during a frame - we'll reset those things here.
        if self.prev_event_was_render {

            // Flush input.
            self.keys_just_pressed.clear();
            self.keys_just_released.clear();
            self.text_just_entered.clear();
            self.mouse.scroll = Scroll { x: 0.0, y: 0.0 };

            self.maybe_prev_widget_id = None;
            self.maybe_current_parent_id = None;
            self.prev_event_was_render = false;
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

            let maybe_new_picked_widget = self.widget_graph.pick_widget(self.mouse.xy);
            if maybe_new_picked_widget != self.maybe_widget_under_mouse {
                println!("Widget under mouse: {:?}", &maybe_new_picked_widget);
            }
            self.maybe_widget_under_mouse = maybe_new_picked_widget;
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
            use piston::input::Button;
            use piston::input::MouseButton::{Left, Middle, Right};

            match button_type {
                Button::Mouse(button) => {
                    *match button {
                        Left => &mut self.mouse.left,
                        Right => &mut self.mouse.right,
                        Middle => &mut self.mouse.middle,
                        _ => &mut self.mouse.unknown,
                    } = ButtonState::Down;
                },
                Button::Keyboard(key) => self.keys_just_pressed.push(key),
            }
        });

        event.release(|button_type| {
            use piston::input::Button;
            use piston::input::MouseButton::Left;
            match button_type {
                Button::Mouse(button) => {
                    *match button {
                        Left => &mut self.mouse.left,
                        _/*input::mouse::Right*/ => &mut self.mouse.right,
                        //Middle => &mut self.mouse.middle,
                    } = ButtonState::Up;
                },
                Button::Keyboard(key) => self.keys_just_released.push(key),
            }
        });

        event.text(|text| {
            self.text_just_entered.push(text.to_string())
        });
    }

    /// Get the centred xy coords for some given `Dimension`s, `Position` and alignment.
    pub fn get_xy(&self,
                  position: Position,
                  dim: Dimensions,
                  h_align: HorizontalAlign,
                  v_align: VerticalAlign) -> Point {
        match position {

            Position::Absolute(x, y) => [x, y],

            Position::Relative(x, y, maybe_id) => {
                match maybe_id.or(self.maybe_prev_widget_id.map(|id| id)) {
                    None => [0.0, 0.0],
                    Some(id) => ::vecmath::vec2_add(self.widget_graph[id].xy, [x, y]),
                }
            },

            Position::Direction(direction, px, maybe_id) => {
                match maybe_id.or(self.maybe_prev_widget_id.map(|id| id)) {
                    None => [0.0, 0.0],
                    Some(rel_id) => {
                        use position::Direction;
                        let (rel_xy, element) = {
                            let widget = &self.widget_graph[rel_id];
                            (widget.xy, &widget.element)
                        };
                        let (rel_w, rel_h) = element.get_size();
                        let (rel_w, rel_h) = (rel_w as f64, rel_h as f64);

                        match direction {

                            // For vertical directions, we must consider horizontal alignment.
                            Direction::Up | Direction::Down => {
                                // Check whether or not we are aligning to a specific `Ui` element.
                                let (other_x, other_w) = match h_align.1 {
                                    Some(other_id) => {
                                        let (x, elem) = {
                                            let widget = &self.widget_graph[other_id];
                                            (widget.xy[0], &widget.element)
                                        };
                                        let w = elem.get_width() as f64;
                                        (x, w)
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
                                    Some(other_id) => {
                                        let (y, elem) = {
                                            let widget = &self.widget_graph[other_id];
                                            (widget.xy[1], &widget.element)
                                        };
                                        let h = elem.get_height() as f64;
                                        (y, h)
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

            Position::Place(place, maybe_parent_id) => {
                let (xy, target_dim, pad) = match maybe_parent_id.or(self.maybe_current_parent_id) {
                    Some(parent_id) => {
                        let parent = &self.widget_graph[parent_id];
                        (parent.kid_area.xy, parent.kid_area.dim, parent.kid_area.pad)
                    },
                    None => ([0.0, 0.0], [self.win_w, self.win_h], Padding::none()),
                };
                let place_xy = place.within(target_dim, dim);
                let relative_xy = ::vecmath::vec2_add(place_xy, pad.offset_from(place));
                ::vecmath::vec2_add(xy, relative_xy)
            },

        }
    }

    /// Draw the `Ui` in it's current state.
    /// - The order of drawing is as follows:
    ///     1. Canvas splits.
    ///     2. Widgets on Canvas splits.
    ///     3. Floating Canvasses.
    ///     4. Widgets on Floating Canvasses.
    /// - Widgets are sorted by capturing and then render depth (depth first).
    /// - Construct the elmesque `Renderer` for rendering the elm `Element`s.
    /// - Render all widgets.
    pub fn draw<G>(&mut self, context: Context, graphics: &mut G)
        where
            C: CharacterCache,
            G: Graphics<Texture = C::Texture>,
    {
        use elmesque::Renderer;
        use std::ops::DerefMut;

        let Ui {
            ref mut widget_graph,
            ref glyph_cache,
            maybe_captured_mouse,
            maybe_captured_keyboard,
            ..
        } = *self;

        let maybe_captured_mouse = match maybe_captured_mouse {
            Some(Capturing::Captured(id)) => Some(id),
            _                             => None,
        };

        let maybe_captured_keyboard = match maybe_captured_keyboard {
            Some(Capturing::Captured(id)) => Some(id),
            _                             => None,
        };

        // Construct the elmesque Renderer for rendering the Elements.
        let mut ref_mut_character_cache = glyph_cache.0.borrow_mut();
        let character_cache = ref_mut_character_cache.deref_mut();
        let mut renderer = Renderer::new(context, graphics).character_cache(character_cache);

        widget_graph.draw(maybe_captured_mouse, maybe_captured_keyboard, &mut renderer);
    }

}


/// Set the ID of the current canvas.
pub fn set_current_parent_id<C>(ui: &mut Ui<C>, id: WidgetId) {
    ui.maybe_current_parent_id = Some(id);
}


/// Check the given position for an attached parent widget.
pub fn parent_from_position<C>(ui: &Ui<C>, position: Position) -> Option<WidgetId> {
    match position {
        Position::Relative(_, _, maybe_id) => match maybe_id {
            Some(id) => ui.widget_graph.parent_of(id),
            None     => match ui.maybe_prev_widget_id {
                Some(id) => ui.widget_graph.parent_of(id),
                None     => ui.maybe_current_parent_id,
            },
        },
        Position::Direction(_, _, maybe_id) => match maybe_id {
            Some(id) => ui.widget_graph.parent_of(id),
            None     => match ui.maybe_prev_widget_id {
                Some(id) => ui.widget_graph.parent_of(id),
                None     => ui.maybe_current_parent_id,
            },
        },
        Position::Place(_, maybe_parent_id) => maybe_parent_id.or(ui.maybe_current_parent_id),
        _ => ui.maybe_current_parent_id,
    }
}


/// Return the user input state available for the widget with the given ID.
/// Take into consideration whether or not each input type is captured.
pub fn user_input<'a, C>(ui: &'a Ui<C>, id: WidgetId) -> UserInput<'a> {
    let maybe_mouse = get_mouse_state(ui, id);
    let without_keys = || UserInput {
        maybe_mouse: maybe_mouse,
        pressed_keys: &[],
        released_keys: &[],
        entered_text: &[],
        window_dim: [ui.win_w, ui.win_h],
    };
    let with_keys = || UserInput {
        maybe_mouse: maybe_mouse,
        pressed_keys: &ui.keys_just_pressed,
        released_keys: &ui.keys_just_released,
        entered_text: &ui.text_just_entered,
        window_dim: [ui.win_w, ui.win_h],
    };
    match ui.maybe_captured_keyboard {
        Some(Capturing::Captured(captured_id)) => if id == captured_id { with_keys()    }
                                                  else                 { without_keys() },
        Some(Capturing::JustReleased) => without_keys(),
        None => with_keys(),
    }
}


/// Return the current mouse state.
///
/// If the Ui has been captured and the given id doesn't match the captured id, return None.
pub fn get_mouse_state<C>(ui: &Ui<C>, id: WidgetId) -> Option<Mouse> {
    match ui.maybe_captured_mouse {
        Some(Capturing::Captured(captured_id)) =>
            if id == captured_id { Some(ui.mouse) } else { None },
        Some(Capturing::JustReleased) =>
            None,
        None =>
            if Some(id) == ui.maybe_widget_under_mouse { Some(ui.mouse) } else { None },
    }
}


/// Get the state of a widget with the given type and WidgetId.
///
/// If the widget doesn't already have a position within the Cache, Create and initialise a
/// cache position before returning None.
pub fn get_widget_state<C, W>(ui: &mut Ui<C>,
                              id: WidgetId,
                              kind: &'static str) -> Option<widget::Cached<W>>
    where
        W: Widget,
        W::State: Any + 'static,
        W::Style: Any + 'static,
{
    ui.widget_graph.get_widget_mut(id).and_then(|container| {
        // If the cache is already initialised for a widget of a different kind, warn the user.
        if container.kind != kind {
            writeln!(::std::io::stderr(),
                     "A widget of a different kind already exists at the given WidgetId ({:?}).
                      You tried to insert a {:?}, however the existing widget is a {:?}.
                      Check your widgets' `WidgetId`s for errors.",
                      id, kind, container.kind).unwrap();
            return None;
        } else {
            container.take_widget_state()
        }
    })
}


/// Indicate that the widget with the given WidgetId has captured the mouse.
pub fn mouse_captured_by<C>(ui: &mut Ui<C>, id: WidgetId) {
    match ui.maybe_captured_mouse {
        Some(Capturing::Captured(captured_id)) => if id != captured_id {
            writeln!(::std::io::stderr(),
                    "Warning: {:?} tried to capture the mouse, however it is \
                     already captured by {:?}.", id, captured_id).unwrap();
        },
        Some(Capturing::JustReleased) => {
            writeln!(::std::io::stderr(),
                    "Warning: {:?} tried to capture the mouse, however it was \
                     already captured.", id).unwrap();
        },
        None => ui.maybe_captured_mouse = Some(Capturing::Captured(id)),
    }
}

/// Indicate that the widget is no longer capturing the mouse.
pub fn mouse_uncaptured_by<C>(ui: &mut Ui<C>, id: WidgetId) {
    match ui.maybe_captured_mouse {
        Some(Capturing::Captured(captured_id)) => if id != captured_id {
            writeln!(::std::io::stderr(),
                    "Warning: {:?} tried to uncapture the mouse, however it is \
                     actually captured by {:?}.", id, captured_id).unwrap();
        } else {
            ui.maybe_captured_mouse = Some(Capturing::JustReleased);
        },
        Some(Capturing::JustReleased) => {
            writeln!(::std::io::stderr(),
                    "Warning: {:?} tried to uncapture the mouse, however it had \
                     already been released this cycle.", id).unwrap();
        },
        None => {
            writeln!(::std::io::stderr(),
                    "Warning: {:?} tried to uncapture the mouse, however the mouse \
                     was not captured", id).unwrap();
        },
    }
}

/// Indicate that the widget with the given WidgetId has captured the keyboard.
pub fn keyboard_captured_by<C>(ui: &mut Ui<C>, id: WidgetId) {
    match ui.maybe_captured_keyboard {
        Some(Capturing::Captured(captured_id)) => if id != captured_id {
            writeln!(::std::io::stderr(),
                    "Warning: {:?} tried to capture the keyboard, however it is \
                     already captured by {:?}.", id, captured_id).unwrap();
        },
        Some(Capturing::JustReleased) => {
            writeln!(::std::io::stderr(),
                    "Warning: {:?} tried to capture the keyboard, however it was \
                     already captured.", id).unwrap();
        },
        None => ui.maybe_captured_keyboard = Some(Capturing::Captured(id)),
    }
}

/// Indicate that the widget is no longer capturing the keyboard.
pub fn keyboard_uncaptured_by<C>(ui: &mut Ui<C>, id: WidgetId) {
    match ui.maybe_captured_keyboard {
        Some(Capturing::Captured(captured_id)) => if id != captured_id {
            writeln!(::std::io::stderr(),
                    "Warning: {:?} tried to uncapture the keyboard, however it is \
                     actually captured by {:?}.", id, captured_id).unwrap();
        } else {
            ui.maybe_captured_keyboard = Some(Capturing::JustReleased);
        },
        Some(Capturing::JustReleased) => {
            writeln!(::std::io::stderr(),
                    "Warning: {:?} tried to uncapture the keyboard, however it had \
                     already been released this cycle.", id).unwrap();
        },
        None => {
            writeln!(::std::io::stderr(),
                    "Warning: {:?} tried to uncapture the keyboard, however the mouse \
                     was not captured", id).unwrap();
        },
    }
}


/// Update the given widget at the given WidgetId.
pub fn update_widget<C, W>(ui: &mut Ui<C>,
                           id: WidgetId,
                           maybe_parent_id: Option<WidgetId>,
                           kind: &'static str,
                           cached: widget::Cached<W>,
                           maybe_new_element: Option<Element>)
    where
        W: Widget,
        W::State: 'static,
        W::Style: 'static,
{
    ui.widget_graph.update_widget(id, maybe_parent_id, kind, cached, maybe_new_element);
    ui.maybe_prev_widget_id = Some(id);
    ui.maybe_current_parent_id = maybe_parent_id;
}


