
use canvas::{self, Canvas, CanvasId};
use elmesque::Element;
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
use position::{Depth, Dimensions, HorizontalAlign, Padding, Point, Position, VerticalAlign};
use std::any::Any;
use std::cell::RefCell;
use std::cmp::Ordering;
use theme::Theme;
use widget::{self, Widget, WidgetId};
use ::std::io::Write;


/// For functions that may take either a WidgetId or a CanvasId.
#[derive(Copy, Clone, Debug, PartialEq, Eq, RustcEncodable, RustcDecodable)]
pub enum UiId {
    /// The ID for a Canvas.
    Canvas(CanvasId),
    /// The ID for a Widget.
    Widget(WidgetId),
}

/// Indicates whether or not the Mouse has been captured by a widget.
#[derive(Copy, Clone, Debug)]
enum Capturing {
    /// The Ui is captured by the Ui element with the given UiId.
    Captured(UiId),
    /// The Ui has just been uncaptured.
    JustReleased,
}

/// `Ui` is the most important type within Conrod and is necessary for rendering and maintaining
/// widget state.
/// # Ui Handles the following:
/// * Contains the state of all widgets which can be indexed via their UiId.
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
    /// Stores the state of all canvasses.
    canvas_cache: Vec<Canvas>,
    /// The Widget cache, storing state for all widgets.
    widget_cache: Vec<widget::Cached>,
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
    /// The UiId of the previously set Widget.
    maybe_prev_widget_id: Option<WidgetId>,
    /// The Id of the current canvas.
    maybe_current_canvas_id: Option<CanvasId>,
    /// The UiId of the widget currently capturing mouse input if there is one.
    maybe_captured_mouse: Option<Capturing>,
    /// The UiId of the widget currently capturing keyboard input if there is one.
    maybe_captured_keyboard: Option<Capturing>,
    /// The Canvas that is currently under the mouse.
    maybe_canvas_under_mouse: Option<CanvasId>,
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
        const CANVAS_RESERVATION: usize = 64;
        const WIDGET_RESERVATION: usize = 512;
        Ui {
            canvas_cache: (0..CANVAS_RESERVATION).map(|_| Canvas::empty()).collect(),
            widget_cache: (0..WIDGET_RESERVATION).map(|_| widget::Cached::empty()).collect(),
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
            maybe_current_canvas_id: None,
            maybe_captured_mouse: None,
            maybe_captured_keyboard: None,
            maybe_canvas_under_mouse: None,
        }
    }

    /// Return the dimensions of a Canvas.
    pub fn widget_size(&self, id: WidgetId) -> Dimensions {
        let (w, h) = self.widget_cache[id].element.get_size();
        [w as f64, h as f64]
    }

    /// Return the dimensions of a Canvas.
    pub fn canvas_size(&self, id: CanvasId) -> Dimensions {
        let (w, h) = self.canvas_cache[id].element.get_size();
        [w as f64, h as f64]
    }

    /// Handle game events and update the state.
    pub fn handle_event<E: GenericEvent>(&mut self, event: &E) {
        if self.prev_event_was_render {

            // Flush input.
            self.keys_just_pressed.clear();
            self.keys_just_released.clear();
            self.text_just_entered.clear();
            self.mouse.scroll = Scroll { x: 0.0, y: 0.0 };

            self.maybe_prev_widget_id = None;
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
        });

        event.mouse_cursor(|x, y| {
            // Convert mouse coords to (0, 0) origin.
            self.mouse.xy = [x - self.win_w / 2.0, -(y - self.win_h / 2.0)];
            self.maybe_canvas_under_mouse = self.pick_canvas(self.mouse.xy);
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

    /// If the given Point is currently on a Canvas, return the Id of that canvas.
    pub fn pick_canvas(&self, xy: Point) -> Option<CanvasId> {
        let mut canvasses = self.canvas_cache.iter().enumerate().filter(|&(_, ref canvas)| {
            if let canvas::Kind::NoCanvas = canvas.kind { false } else { true }
        }).collect::<Vec<_>>();
        canvasses.sort_by(|&(_, ref a), &(_, ref b)| compare_canvasses(a, b));
        canvasses.iter().rev().find(|&&(_, ref canvas)| {
            use utils::is_over_rect;
            let (w, h) = canvas.element.get_size();
            is_over_rect(canvas.xy, xy, [w as f64, h as f64])
        }).map(|&(id, _)| id)
    }

    /// Get the centred xy coords for some given `Dimension`s, `Position` and alignment.
    pub fn get_xy(&self,
                  position: Position,
                  dim: Dimensions,
                  h_align: HorizontalAlign,
                  v_align: VerticalAlign) -> Point {
        match position {

            Position::Absolute(x, y) => [x, y],

            Position::Relative(x, y, maybe_ui_id) => {
                match maybe_ui_id.or(self.maybe_prev_widget_id.map(|id| UiId::Widget(id))) {
                    None => [0.0, 0.0],
                    Some(UiId::Widget(id)) => ::vecmath::vec2_add(self.widget_cache[id].xy, [x, y]),
                    Some(UiId::Canvas(id)) => ::vecmath::vec2_add(self.canvas_cache[id].xy, [x, y]),
                }
            },

            Position::Direction(direction, px, maybe_ui_id) => {
                match maybe_ui_id.or(self.maybe_prev_widget_id.map(|id| UiId::Widget(id))) {
                    None => [0.0, 0.0],
                    Some(rel_ui_id) => {
                        use position::Direction;
                        let (rel_xy, element) = match rel_ui_id {
                            UiId::Widget(id) =>
                                (self.widget_cache[id].xy, &self.widget_cache[id].element),
                            UiId::Canvas(id) =>
                                (self.canvas_cache[id].xy, &self.canvas_cache[id].element),
                        };
                        let (rel_w, rel_h) = element.get_size();
                        let (rel_w, rel_h) = (rel_w as f64, rel_h as f64);

                        match direction {

                            // For vertical directions, we must consider horizontal alignment.
                            Direction::Up | Direction::Down => {
                                // Check whether or not we are aligning to a specific `Ui` element.
                                let (other_x, other_w) = match h_align.1 {
                                    Some(other_ui_id) => {
                                        let (x, elem) = match other_ui_id {
                                            UiId::Widget(id) =>
                                                (self.widget_cache[id].xy[0],
                                                 &self.widget_cache[id].element),
                                            UiId::Canvas(id) =>
                                                (self.canvas_cache[id].xy[0],
                                                 &self.canvas_cache[id].element),
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
                                    Some(other_ui_id) => {
                                        let (y, elem) = match other_ui_id {
                                            UiId::Widget(id) =>
                                                (self.widget_cache[id].xy[1],
                                                 &self.widget_cache[id].element),
                                            UiId::Canvas(id) =>
                                                (self.canvas_cache[id].xy[1],
                                                 &self.canvas_cache[id].element),
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

            Position::Place(place, maybe_canvas_id) => {
                let (xy, target_dim, pad) = match maybe_canvas_id.or(self.maybe_current_canvas_id) {
                    Some(canvas_id) => {
                        let canvas = &self.canvas_cache[canvas_id];
                        (canvas.widget_area_xy, canvas.widget_area_dim, canvas.padding.clone())
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
            ref mut canvas_cache,
            ref mut widget_cache,
            ref glyph_cache,
            ..
        } = *self;

        // Collect references to the widgets so that we can sort them without changing cache order.
        let mut widgets: Vec<_> = widget_cache.iter_mut().enumerate()
            .filter(|&(_, ref widget)| widget.has_updated)
            .collect();

        for &mut (_, ref mut widget) in widgets.iter_mut() {
            widget.has_updated = false;
        }

        // Check for captured widgets and take them from the Vec (we want to draw them last).
        let maybe_mouse = self.maybe_captured_mouse;
        let maybe_keyboard = self.maybe_captured_keyboard;

        // Sort the rest of the widgets by rendering depth.
        widgets.sort_by(|&(id_a, ref a), &(id_b, ref b)| {
            match (a.maybe_canvas_id, b.maybe_canvas_id) {
                (Some(canvas_id_a), Some(canvas_id_b)) => {
                    match (&canvas_cache[canvas_id_a].kind, &canvas_cache[canvas_id_b].kind) {
                        (&canvas::Kind::Split(_), &canvas::Kind::Split(_)) => (),
                        (&canvas::Kind::Split(_), _) => return Ordering::Less,
                        (_, &canvas::Kind::Split(_)) => return Ordering::Greater,
                        _ => (),
                    }
                },
                (Some(_), None) => return Ordering::Greater,
                (None, Some(_)) => return Ordering::Less,
                _ => (),
            };
            if let Some(Capturing::Captured(UiId::Widget(id))) = maybe_mouse {
                if      id == id_a { return Ordering::Greater }
                else if id == id_b { return Ordering::Less }
            }
            if let Some(Capturing::Captured(UiId::Widget(id))) = maybe_keyboard {
                if      id == id_a { return Ordering::Greater }
                else if id == id_b { return Ordering::Less }
            }
            if      a.depth < b.depth { Ordering::Greater }
            else if a.depth > b.depth { Ordering::Less }
            else                      { Ordering::Equal }
        });

        // Construct the elmesque Renderer for rendering the Elements.
        let mut ref_mut_character_cache = glyph_cache.0.borrow_mut();
        let character_cache = ref_mut_character_cache.deref_mut();
        let mut renderer = Renderer::new(context, graphics).character_cache(character_cache);

        // Collect references to the canvasses for sorting.
        let mut canvasses: Vec<_> = canvas_cache.iter_mut().enumerate()
            .filter(|&(_, ref canvas)| canvas.has_updated).collect();

        // Sort canvasses into correct rendering depth.
        canvasses.sort_by(|&(id_a, ref a), &(id_b, ref b)| {
            let cmp = compare_canvasses(a, b);
            match cmp {
                Ordering::Less | Ordering::Greater => cmp,
                Ordering::Equal => id_a.cmp(&id_b),
            }
        });


        // Print all the widgets that aren't attached to any canvas.
        for &mut (_, ref widget) in widgets.iter_mut()
            .filter(|&&mut(_, ref widget)| widget.maybe_canvas_id == None) {
            widget.element.draw(&mut renderer);
        }

        // Print all of the canvasses in order.
        for (id, canvas) in canvasses.into_iter() {
            canvas.element.draw(&mut renderer);

            // Print each widget attached to the current canvas.
            for &mut (_, ref widget) in widgets.iter_mut()
                .filter(|&&mut(_, ref widget)| widget.maybe_canvas_id == Some(id)) {
                widget.element.draw(&mut renderer);
            }

            canvas.has_updated = false;
        }

    }

}


/// Compare two canvasses in order to determine their rendering depth.
fn compare_canvasses(a: &Canvas, b: &Canvas) -> Ordering {
    match (&a.kind, &b.kind) {
        (&canvas::Kind::Split(_), &canvas::Kind::Floating(_)) => Ordering::Less,
        (&canvas::Kind::Floating(_), &canvas::Kind::Split(_)) => Ordering::Greater,
        (&canvas::Kind::Floating(ref a_state), &canvas::Kind::Floating(ref b_state)) =>
            a_state.time_last_clicked.cmp(&b_state.time_last_clicked),
        (&canvas::Kind::NoCanvas, &canvas::Kind::Split(_)) => Ordering::Less,
        (&canvas::Kind::Split(_), &canvas::Kind::NoCanvas) => Ordering::Greater,
        _ => Ordering::Equal,
    }
}


/// Set the ID of the current canvas.
pub fn set_current_canvas_id<C>(ui: &mut Ui<C>, id: CanvasId) {
    ui.maybe_current_canvas_id = Some(id);
}


/// Check the given position for a attached Canvas.
pub fn canvas_from_position<C>(ui: &Ui<C>, position: Position) -> Option<CanvasId> {
    match position {
        Position::Relative(_, _, ui_id) => match ui_id {
            Some(UiId::Widget(id)) => ui.widget_cache[id].maybe_canvas_id,
            Some(UiId::Canvas(id)) => Some(id),
            None => match ui.maybe_prev_widget_id {
                Some(id) => ui.widget_cache[id].maybe_canvas_id,
                None => ui.maybe_current_canvas_id,
            },
        },
        Position::Direction(_, _, ui_id) => match ui_id {
            Some(UiId::Widget(id)) => ui.widget_cache[id].maybe_canvas_id,
            Some(UiId::Canvas(id)) => Some(id),
            None => match ui.maybe_prev_widget_id {
                Some(id) => ui.widget_cache[id].maybe_canvas_id,
                None => ui.maybe_current_canvas_id,
            },
        },
        Position::Place(_, maybe_canvas_id) => maybe_canvas_id.or(ui.maybe_current_canvas_id),
        _ => ui.maybe_current_canvas_id,
    }
}


/// Return the user input state available for the widget with the given ID.
/// Take into consideration whether or not each input type is captured.
pub fn user_input<'a, C>(ui: &'a Ui<C>, ui_id: UiId, maybe_canvas_id: Option<CanvasId>) -> UserInput<'a> {
    let maybe_mouse = get_mouse_state(ui, ui_id, maybe_canvas_id);
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
        Some(Capturing::Captured(captured_ui_id)) => if ui_id == captured_ui_id {
            with_keys()
        } else {
            without_keys()
        },
        Some(Capturing::JustReleased) => without_keys(),
        None => with_keys(),
    }
}


/// Return the current mouse state.
///
/// If the Ui has been captured and the given ui_id doesn't match the captured ui_id, return
/// None.
///
/// If no widget is capturing the mouse and a canvas id was given, check that the mouse is over
/// the same canvas.
pub fn get_mouse_state<C>(ui: &Ui<C>, ui_id: UiId, maybe_canvas_id: Option<CanvasId>) -> Option<Mouse> {
    match ui.maybe_captured_mouse {
        Some(Capturing::Captured(captured_ui_id)) => {
            match ui_id == captured_ui_id {
                true => Some(ui.mouse),
                false => None,
            }
        },
        Some(Capturing::JustReleased) => None,
        None => match ui.maybe_canvas_under_mouse == maybe_canvas_id {
            true => Some(ui.mouse),
            false => None,
        },
    }
}


/// Get the state of a widget with the given type and WidgetId.
///
/// If the widget doesn't already have a position within the Cache, Create and initialise a
/// cache position before returning None.
pub fn get_widget_state<C, W>(ui: &mut Ui<C>,
                          id: WidgetId,
                          kind: &'static str) -> Option<widget::PrevState<W>>
    where
        W: Widget,
        W::State: Any + 'static,
        W::Style: Any + 'static,
{

    // If the cache is not big enough, extend it.
    if ui.widget_cache.len() <= id {
        let num_to_extend = id + 1 - ui.widget_cache.len();
        let extension = (0..num_to_extend).map(|_| widget::Cached::empty());
        ui.widget_cache.extend(extension);
    }

    // If the cache is empty, return None.
    if ui.widget_cache[id].kind == "EMPTY" {
        None
    }

    // Else if the cache is already initialised for a widget of a different kind, warn the user.
    else if ui.widget_cache[id].kind != kind {
        writeln!(::std::io::stderr(),
                 "A widget of a different kind already exists at the given UiId ({:?}).
                  You tried to insert a {:?}, however the existing widget is a {:?}.
                  Check your widgets' `UiId`s for errors.",
                  id, kind, &ui.widget_cache[id].kind).unwrap();
        None
    }

    // Otherwise we've successfully found our state!
    else {
        let cached_widget = &mut ui.widget_cache[id];
        if let Some(any_state) = cached_widget.maybe_state.take() {
            let dim = cached_widget.dim;
            let xy = cached_widget.xy;
            let depth = cached_widget.depth;
            let store: Box<widget::Store<W::State, W::Style>> = any_state.downcast()
                .ok().expect("Failed to downcast from Box<Any> to required widget::Store.");
            let store: widget::Store<W::State, W::Style> = *store;
            let widget::Store { state, style } = store;
            Some(widget::PrevState {
                state: state,
                style: style,
                dim: dim,
                xy: xy,
                depth: depth,
            })
        } else {
            None
        }
    }
}


/// Get the state of a canvas with the given Id.
///
/// If the canvas doesn't already have a position within the Cache, Create and initialise a
/// cache position before returning None.
pub fn get_canvas_state<C>(ui: &mut Ui<C>, id: CanvasId) -> Option<canvas::Kind> {

    // If the cache is not big enough, extend it.
    if ui.canvas_cache.len() <= id {
        let num_to_extend = id + 1 - ui.canvas_cache.len();
        let extension = (0..num_to_extend).map(|_| Canvas::empty());
        ui.canvas_cache.extend(extension);
    }

    // If the cache is empty, return None.
    if let &canvas::Kind::NoCanvas = &ui.canvas_cache[id].kind {
        None
    }

    // Otherwise, return the unique state of the Canvas.
    else {
        Some(ui.canvas_cache[id].kind.clone())
    }
}

/// Indicate that the widget with the given UiId has captured the mouse.
pub fn mouse_captured_by<C>(ui: &mut Ui<C>, ui_id: UiId) {
    match ui.maybe_captured_mouse {
        Some(Capturing::Captured(captured_ui_id)) => if ui_id != captured_ui_id {
            writeln!(::std::io::stderr(),
                    "Warning: {:?} tried to capture the mouse, however it is \
                     already captured by {:?}.", ui_id, captured_ui_id).unwrap();
        },
        Some(Capturing::JustReleased) => {
            writeln!(::std::io::stderr(),
                    "Warning: {:?} tried to capture the mouse, however it was \
                     already captured.", ui_id).unwrap();
        },
        None => ui.maybe_captured_mouse = Some(Capturing::Captured(ui_id)),
    }
}

/// Indicate that the widget is no longer capturing the mouse.
pub fn mouse_uncaptured_by<C>(ui: &mut Ui<C>, ui_id: UiId) {
    match ui.maybe_captured_mouse {
        Some(Capturing::Captured(captured_ui_id)) => if ui_id != captured_ui_id {
            writeln!(::std::io::stderr(),
                    "Warning: {:?} tried to uncapture the mouse, however it is \
                     actually captured by {:?}.", ui_id, captured_ui_id).unwrap();
        } else {
            ui.maybe_captured_mouse = Some(Capturing::JustReleased);
        },
        Some(Capturing::JustReleased) => {
            writeln!(::std::io::stderr(),
                    "Warning: {:?} tried to uncapture the mouse, however it had \
                     already been released this cycle.", ui_id).unwrap();
        },
        None => {
            writeln!(::std::io::stderr(),
                    "Warning: {:?} tried to uncapture the mouse, however the mouse \
                     was not captured", ui_id).unwrap();
        },
    }
}

/// Indicate that the widget with the given UiId has captured the keyboard.
pub fn keyboard_captured_by<C>(ui: &mut Ui<C>, ui_id: UiId) {
    match ui.maybe_captured_keyboard {
        Some(Capturing::Captured(captured_ui_id)) => if ui_id != captured_ui_id {
            writeln!(::std::io::stderr(),
                    "Warning: {:?} tried to capture the keyboard, however it is \
                     already captured by {:?}.", ui_id, captured_ui_id).unwrap();
        },
        Some(Capturing::JustReleased) => {
            writeln!(::std::io::stderr(),
                    "Warning: {:?} tried to capture the keyboard, however it was \
                     already captured.", ui_id).unwrap();
        },
        None => ui.maybe_captured_keyboard = Some(Capturing::Captured(ui_id)),
    }
}

/// Indicate that the widget is no longer capturing the keyboard.
pub fn keyboard_uncaptured_by<C>(ui: &mut Ui<C>, ui_id: UiId) {
    match ui.maybe_captured_keyboard {
        Some(Capturing::Captured(captured_ui_id)) => if ui_id != captured_ui_id {
            writeln!(::std::io::stderr(),
                    "Warning: {:?} tried to uncapture the keyboard, however it is \
                     actually captured by {:?}.", ui_id, captured_ui_id).unwrap();
        } else {
            ui.maybe_captured_keyboard = Some(Capturing::JustReleased);
        },
        Some(Capturing::JustReleased) => {
            writeln!(::std::io::stderr(),
                    "Warning: {:?} tried to uncapture the keyboard, however it had \
                     already been released this cycle.", ui_id).unwrap();
        },
        None => {
            writeln!(::std::io::stderr(),
                    "Warning: {:?} tried to uncapture the keyboard, however the mouse \
                     was not captured", ui_id).unwrap();
        },
    }
}


/// Update the given canvas.
pub fn update_canvas<C>(ui: &mut Ui<C>,
                        id: CanvasId,
                        kind: canvas::Kind,
                        xy: Point,
                        widget_area_xy: Point,
                        widget_area_dim: Dimensions,
                        padding: Padding,
                        maybe_new_element: Option<Element>) {
    if ui.canvas_cache[id].kind.matches(&kind)
    || ui.canvas_cache[id].kind.matches(&canvas::Kind::NoCanvas) {
        if ui.canvas_cache[id].has_updated {
            writeln!(::std::io::stderr(),
                     "Warning: The canvas with CanvasId {:?} has already been set within the \
                      `Ui` since the last time that `Ui::draw` was called (you probably don't \
                      want this). Perhaps check that your CanvasIds are correct, that you're \
                      calling `Ui::draw` after constructing your widgets and that you haven't \
                      accidentally set the same canvas twice.", id).unwrap();
        }
        let canvas = &mut ui.canvas_cache[id];
        canvas.kind = kind;
        canvas.xy = xy;
        canvas.widget_area_xy = widget_area_xy;
        canvas.widget_area_dim = widget_area_dim;
        canvas.padding = padding;
        if let Some(new_element) = maybe_new_element {
            canvas.element = new_element;
        }
        canvas.has_updated = true;
    } else {
        panic!("A canvas of a different kind already exists at the given CanvasId ({:?}).
                You tried to insert a {:?}, however the existing canvas is a {:?}.
                Check your widgets' `CanvasId`s for errors.",
                id, &kind, &ui.canvas_cache[id].kind);
    }
}


/// Update the given widget at the given UiId.
pub fn update_widget<C, Sta, Sty>(ui: &mut Ui<C>,
                                  id: WidgetId,
                                  maybe_canvas_id: Option<CanvasId>,
                                  kind: &'static str,
                                  store: widget::Store<Sta, Sty>,
                                  dim: Dimensions,
                                  xy: Point,
                                  depth: Depth,
                                  maybe_new_element: Option<Element>)
    where
        Sta: Any + ::std::fmt::Debug + 'static,
        Sty: Any + ::std::fmt::Debug + 'static,
{
    if ui.widget_cache[id].kind == kind
    || ui.widget_cache[id].kind == "EMPTY" {
        if ui.widget_cache[id].has_updated {
            writeln!(::std::io::stderr(),
                     "Warning: The widget with UiId {:?} has already been set within the `Ui` \
                      since the last time that `Ui::draw` was called (you probably don't want \
                      this). Perhaps check that your UiIds are correct, that you're calling \
                      `Ui::draw` after constructing your widgets and that you haven't \
                      accidentally set the same widget twice.", id).unwrap();
        }
        let cached_widget = &mut ui.widget_cache[id];
        let state: Box<Any> = Box::new(store);
        cached_widget.maybe_state = Some(state);
        cached_widget.kind = kind;
        cached_widget.xy = xy;
        cached_widget.dim = dim;
        cached_widget.depth = depth;
        cached_widget.maybe_canvas_id = maybe_canvas_id.or(ui.maybe_current_canvas_id);
        if let Some(new_element) = maybe_new_element {
            cached_widget.element = new_element;
        }
        cached_widget.has_updated = true;
        ui.maybe_prev_widget_id = Some(id);
        if let Some(id) = cached_widget.maybe_canvas_id {
            ui.maybe_current_canvas_id = Some(id);
        }
    } else {
        panic!("A widget of a different kind already exists at the given UiId ({:?}).
                You tried to insert a {:?}, however the existing widget is a {:?}.
                Check your widgets' `UiId`s for errors.",
                id, &kind, &ui.widget_cache[id].kind);
    }
}


