
use canvas::{Canvas, CanvasId};
use canvas::Kind as CanvasKind;
use elmesque::Element;
use graphics::Graphics;
use graphics::character::{Character, CharacterCache};
use label::FontSize;
use mouse::{ButtonState, Mouse};
use piston::input;
use piston::event::{
    GenericEvent,
    MouseCursorEvent,
    PressEvent,
    ReleaseEvent,
    RenderEvent,
    TextEvent,
};
use position::{Depth, Dimensions, HorizontalAlign, Padding, Point, Position, VerticalAlign};
use std::any::Any;
use theme::Theme;
use widget::{self, Widget};

/// User interface identifier. Each widget must use a unique `UiId` so that it's state can be
/// cached within the `Ui` type. The reason we use a usize is because widgets are cached within
/// a `Vec`, which is limited to a size of `usize` elements.
pub type UiId = usize;

/// Indicates whether or not the Mouse has been captured by a widget.
#[derive(Copy, Clone, Debug)]
enum Capturing {
    /// The Ui is captured by the widget with UiId with the mouse state `Mouse`.
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
    /// Stores the state of all canvasses.
    canvas_cache: Vec<Canvas>,
    /// The Widget cache, storing state for all widgets.
    widget_cache: Vec<widget::Cached>,
    /// The theme used to set default styling for widgets.
    pub theme: Theme,
    /// The latest received mouse state.
    pub mouse: Mouse,
    /// Keys that have been pressed since the end of the last render cycle.
    pub keys_just_pressed: Vec<input::keyboard::Key>,
    /// Keys that have been released since the end of the last render cycle.
    pub keys_just_released: Vec<input::keyboard::Key>,
    /// Text that has been entered since the end of the last render cycle.
    pub text_just_entered: Vec<String>,
    /// Cache for character textures, used for label width calculation and glyph rendering.
    pub character_cache: C,
    prev_event_was_render: bool,
    /// Window width.
    pub win_w: f64,
    /// Window height.
    pub win_h: f64,
    /// The UiId of the previously drawn Widget.
    maybe_prev_ui_id: Option<UiId>,
    /// The Id of the current canvas.
    maybe_current_canvas_id: Option<CanvasId>,
    /// The captured Mouse and the UiId of the widget who has captured it.
    maybe_captured_mouse: Option<(Capturing, Mouse)>,
    /// The UiId of the widget currently keyboard input if there is one.
    maybe_captured_keyboard: Option<Capturing>
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
            character_cache: character_cache,
            prev_event_was_render: false,
            win_w: 0.0,
            win_h: 0.0,
            maybe_prev_ui_id: None,
            maybe_current_canvas_id: None,
            maybe_captured_mouse: None,
            maybe_captured_keyboard: None,
        }
    }

    /// Return the dimensions of a Canvas.
    pub fn widget_size(&self, ui_id: UiId) -> Dimensions {
        let (w, h) = self.widget_cache[ui_id].element.get_size();
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
            self.flush_input();
            self.maybe_prev_ui_id = None;
            self.prev_event_was_render = false;
            if let Some((Capturing::JustReleased, _)) = self.maybe_captured_mouse {
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

    /// Return a reference to a `Character` from the GlyphCache.
    pub fn get_character(&mut self,
                         size: FontSize,
                         ch: char) -> &Character<C::Texture>
        where
            C: CharacterCache
    {
        self.character_cache.character(size, ch)
    }

    /// Return the width of a 'Character'.
    pub fn get_character_w(&mut self, size: FontSize, ch: char) -> f64
        where
            C: CharacterCache
    {
        self.get_character(size, ch).width()
    }

    /// Flush all stored keys.
    pub fn flush_input(&mut self) {
        self.keys_just_pressed.clear();
        self.keys_just_released.clear();
        self.text_just_entered.clear();
    }

    /// Return the current mouse state. If the Ui has been captured and the given ui_id doesn't
    /// match the captured ui_id, return the captured mouse state.
    pub fn get_mouse_state(&self, ui_id: UiId) -> Mouse {
        match self.maybe_captured_mouse {
            Some((Capturing::Captured(captured_ui_id), captured_mouse)) => {
                match ui_id == captured_ui_id {
                    true => self.mouse,
                    false => captured_mouse,
                }
            },
            Some((Capturing::JustReleased, captured_mouse)) => captured_mouse,
            None => self.mouse,
        }
    }

    /// Return the vector of recently pressed keys.
    pub fn get_pressed_keys(&self, ui_id: UiId) -> &[input::keyboard::Key] {
        match self.maybe_captured_keyboard {
            Some(Capturing::Captured(captured_ui_id)) => if ui_id == captured_ui_id {
                &self.keys_just_pressed
            } else {
                &[]
            },
            Some(Capturing::JustReleased) => &[],
            None => &self.keys_just_pressed,
        }
    }

    /// Return the vector of recently entered text.
    pub fn get_entered_text(&self, ui_id: UiId) -> &[String] {
        match self.maybe_captured_keyboard {
            Some(Capturing::Captured(captured_ui_id)) => if ui_id == captured_ui_id {
                &self.text_just_entered
            } else {
                &[]
            },
            Some(Capturing::JustReleased) => &[],
            None => &self.text_just_entered,
        }
    }

    /// Get the state of a widget with the given type and UiId.
    ///
    /// If the widget doesn't already have a position within the Cache, Create and initialise a
    /// cache position before returning the default initialised state.
    pub fn get_widget_state<W>(&mut self,
                               ui_id: UiId,
                               kind: &'static str,
                               style: &W::Style,
                               widget: &W) -> widget::PrevState<W>
        where
            W: Widget,
            W::State: Any + 'static,
            W::Style: Any + 'static,
    {
        fn init_cache<W>(kind: &'static str, style: &W::Style, widget: &W) -> widget::Cached
            where
                W: Widget,
                W::State: Any + 'static,
                W::Style: Any + 'static,
        {
            let store: widget::Store<W::State, W::Style> = widget::Store {
                state: W::init_state(&widget),
                style: style.clone()
            };
            widget::Cached::new(kind, store)
        }

        // If the cache is not big enough, extend it.
        if self.widget_cache.len() <= ui_id {
            let num_to_extend = ui_id - self.widget_cache.len();
            let init = init_cache(kind, style, widget);
            let extension = (0..num_to_extend)
                .map(|_| widget::Cached::empty())
                .chain(Some(init).into_iter());
            self.widget_cache.extend(extension);
        }

        // If the cache is empty, initialise it.
        if self.widget_cache[ui_id].kind == Widget::unique_kind(&()) {
            self.widget_cache[ui_id] = init_cache(kind, style, widget);
        // Else if the cache is already initialised for a widget of a different kind, warn the user.
        } else if self.widget_cache[ui_id].kind != kind {
            println!("A widget of a different kind already exists at the given UiId ({:?}).
                      You tried to insert a {:?}, however the existing widget is a {:?}.
                      Check your widgets' `UiId`s for errors.",
                      ui_id, kind, &self.widget_cache[ui_id].kind);
            self.widget_cache[ui_id] = init_cache(kind, style, widget);
        }

        let cached_widget = &mut self.widget_cache[ui_id];
        let xy = cached_widget.xy;
        let depth = cached_widget.depth;
        let store: &widget::Store<W::State, W::Style> =
            cached_widget.state.downcast_ref().unwrap();
        let state = store.state.clone();
        let style = store.style.clone();
        widget::PrevState { state: state, style: style, xy: xy, depth: depth }
    }

    // /// Return a mutable reference to the widget that matches the given ui_id
    // pub fn get_widget_mut(&mut self, ui_id: UiId, default: WidgetKind<W>) -> &mut WidgetKind<W> {
    //     if self.widget_cache.len() > ui_id {
    //         match &mut self.widget_cache[ui_id].kind {
    //             &mut WidgetKind::NoWidget => {
    //                 let mut widget = &mut self.widget_cache[ui_id].kind;
    //                 *widget = default;
    //                 widget
    //             },
    //             _ => &mut self.widget_cache[ui_id].kind,
    //         }
    //     } else {
    //         if ui_id >= self.widget_cache.len() {
    //             let num_to_extend = ui_id - self.widget_cache.len();
    //             self.widget_cache.extend(repeat(Widget::empty())
    //                 .take(num_to_extend)
    //                 .chain(Some(Widget::new(default)).into_iter()));
    //         } else {
    //             self.widget_cache[ui_id] = Widget::<W>::new(default);
    //         }
    //         &mut self.widget_cache[ui_id].kind
    //     }
    // }

    /// Update the given canvas.
    pub fn update_canvas(&mut self,
                         id: CanvasId,
                         kind: CanvasKind,
                         xy: Point,
                         padding: Padding,
                         maybe_new_element: Option<Element>) {
        if self.canvas_cache[id].kind.matches(&kind)
        || self.canvas_cache[id].kind.matches(&CanvasKind::NoCanvas) {
            if self.canvas_cache[id].set_since_last_drawn {
                println!("Warning: The canvas with CanvasId {:?} has already been set within the \
                          `Ui` since the last time that `Ui::draw` was called (you probably don't \
                          want this). Perhaps check that your CanvasIds are correct, that you're \
                          calling `Ui::draw` after constructing your widgets and that you haven't \
                          accidentally set the same canvas twice.", id);
            }
            let canvas = &mut self.canvas_cache[id];
            canvas.kind = kind;
            canvas.xy = xy;
            canvas.padding = padding;
            if let Some(new_element) = maybe_new_element {
                canvas.element = new_element;
            }
            canvas.set_since_last_drawn = true;
            self.maybe_current_canvas_id = Some(id);
        } else {
            panic!("A canvas of a different kind already exists at the given CanvasId ({:?}).
                    You tried to insert a {:?}, however the existing canvas is a {:?}.
                    Check your widgets' `CanvasId`s for errors.",
                    id, &kind, &self.canvas_cache[id].kind);
        }
    }

    /// Update the given widget at the given UiId.
    pub fn update_widget<Sta, Sty>(&mut self,
                                   ui_id: UiId,
                                   kind: &'static str,
                                   store: widget::Store<Sta, Sty>,
                                   xy: Point,
                                   depth: Depth,
                                   maybe_new_element: Option<Element>)
        where
            Sta: Any + Clone + ::std::fmt::Debug + 'static,
            Sty: Any + Clone + ::std::fmt::Debug + 'static,
    {
        if self.widget_cache[ui_id].kind == kind
        || self.widget_cache[ui_id].kind == ().unique_kind() {
            if self.widget_cache[ui_id].set_since_last_drawn {
                println!("Warning: The widget with UiId {:?} has already been set within the `Ui` \
                          since the last time that `Ui::draw` was called (you probably don't want \
                          this). Perhaps check that your UiIds are correct, that you're calling \
                          `Ui::draw` after constructing your widgets and that you haven't \
                          accidentally set the same widget twice.", ui_id);
            }
            let cached_widget = &mut self.widget_cache[ui_id];
            {
                let state: &mut widget::Store<Sta, Sty> = cached_widget.state.downcast_mut()
                    .expect("Mismatched type when casting to Store<W> from Any");
                *state = store;
            }
            cached_widget.kind = kind;
            cached_widget.xy = xy;
            cached_widget.depth = depth;
            if let Some(new_element) = maybe_new_element {
                cached_widget.element = new_element;
            }
            cached_widget.set_since_last_drawn = true;
            self.maybe_prev_ui_id = Some(ui_id);
        } else {
            panic!("A widget of a different kind already exists at the given UiId ({:?}).
                    You tried to insert a {:?}, however the existing widget is a {:?}.
                    Check your widgets' `UiId`s for errors.",
                    ui_id, &kind, &self.widget_cache[ui_id].kind);
        }
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
                match maybe_ui_id.or(self.maybe_prev_ui_id) {
                    None => [0.0, 0.0],
                    Some(rel_ui_id) => ::vecmath::vec2_add(self.widget_cache[rel_ui_id].xy, [x, y]),
                }
            },

            Position::Direction(direction, px, maybe_ui_id) => {
                use position::{align_left_of, align_right_of, align_top_of, align_bottom_of};
                match maybe_ui_id.or(self.maybe_prev_ui_id) {
                    None => [0.0, 0.0],
                    Some(rel_ui_id) => {
                        use position::Direction;
                        let rel_xy = self.widget_cache[rel_ui_id].xy;
                        let element = &self.widget_cache[rel_ui_id].element;
                        let (rel_w, rel_h) = element.get_size();
                        let (rel_w, rel_h) = (rel_w as f64, rel_h as f64);
                        match direction {

                            Direction::Up => {
                                let x = rel_xy[0] + match h_align {
                                    HorizontalAlign::Middle => 0.0,
                                    HorizontalAlign::Left   => align_left_of(rel_w, dim[0]),
                                    HorizontalAlign::Right  => align_right_of(rel_w, dim[0]),
                                };
                                let y = rel_xy[1] + rel_h / 2.0 + dim[1] / 2.0 + px;
                                [x, y]
                            },

                            Direction::Down => {
                                let x = rel_xy[0] + match h_align {
                                    HorizontalAlign::Middle => 0.0,
                                    HorizontalAlign::Left   => align_left_of(rel_w, dim[0]),
                                    HorizontalAlign::Right  => align_right_of(rel_w, dim[0]),
                                };
                                let y = rel_xy[1] - rel_h / 2.0 - dim[1] / 2.0 - px;
                                [x, y]
                            },

                            Direction::Left => {
                                let y = rel_xy[1] + match v_align {
                                    VerticalAlign::Middle => 0.0,
                                    VerticalAlign::Bottom => align_bottom_of(rel_h, dim[1]),
                                    VerticalAlign::Top    => align_top_of(rel_h, dim[1]),
                                };
                                let x = rel_xy[0] - rel_w / 2.0 - dim[0] / 2.0 - px;
                                [x, y]
                            },

                            Direction::Right => {
                                let y = rel_xy[1] + match v_align {
                                    VerticalAlign::Middle => 0.0,
                                    VerticalAlign::Bottom => align_bottom_of(rel_h, dim[1]),
                                    VerticalAlign::Top    => align_top_of(rel_h, dim[1]),
                                };
                                let x = rel_xy[0] + rel_w / 2.0 + dim[0] / 2.0 + px;
                                [x, y]
                            },

                        }
                    },
                }
            },

            Position::Place(place, maybe_canvas_id) => {
                use position::{self, Place};
                let (xy, target_dim, pad) = match maybe_canvas_id.or(self.maybe_current_canvas_id) {
                    Some(canvas_id) => {
                        let canvas = &self.canvas_cache[canvas_id];
                        let (w, h) = canvas.element.get_size();
                        (canvas.xy, [w as f64, h as f64], canvas.padding.clone())
                    },
                    None => ([0.0, 0.0], [self.win_w, self.win_h], Padding::none()),
                };
                let place_xy = match place {
                    Place::Middle      => position::middle_of(target_dim, dim),
                    Place::TopLeft     => position::top_left_of(target_dim, dim),
                    Place::TopRight    => position::top_right_of(target_dim, dim),
                    Place::BottomLeft  => position::bottom_left_of(target_dim, dim),
                    Place::BottomRight => position::bottom_right_of(target_dim, dim),
                    Place::MidTop      => position::mid_top_of(target_dim, dim),
                    Place::MidBottom   => position::mid_bottom_of(target_dim, dim),
                    Place::MidLeft     => position::mid_left_of(target_dim, dim),
                    Place::MidRight    => position::mid_right_of(target_dim, dim),
                };
                let relative_xy = ::vecmath::vec2_add(place_xy, pad.offset_from(place));
                ::vecmath::vec2_add(xy, relative_xy)
            },

        }
    }

    /// Indicate that the widget with the given UiId has captured the mouse.
    pub fn mouse_captured_by(&mut self, ui_id: UiId) {
        match self.maybe_captured_mouse {
            Some((Capturing::Captured(captured_ui_id), _)) => if ui_id != captured_ui_id {
                println!("Warning: Widget {:?} tried to capture the mouse, however it is \
                         already captured by {:?}.", ui_id, captured_ui_id);
            },
            Some((Capturing::JustReleased, _)) => {
                println!("Warning: Widget {:?} tried to capture the mouse, however it was \
                         already captured.", ui_id);
            },
            None => self.maybe_captured_mouse = Some((Capturing::Captured(ui_id), self.mouse)),
        }
    }

    /// Indicate that the widget is no longer capturing the mouse.
    pub fn mouse_uncaptured_by(&mut self, ui_id: UiId) {
        match self.maybe_captured_mouse {
            Some((Capturing::Captured(captured_ui_id), mouse)) => if ui_id != captured_ui_id {
                println!("Warning: Widget {:?} tried to uncapture the mouse, however it is \
                         actually captured by {:?}.", ui_id, captured_ui_id);
            } else {
                self.maybe_captured_mouse = Some((Capturing::JustReleased, mouse));
            },
            Some((Capturing::JustReleased, _)) => {
                println!("Warning: Widget {:?} tried to uncapture the mouse, however it had \
                         already been released this cycle.", ui_id);
            },
            None => {
                println!("Warning: Widget {:?} tried to uncapture the mouse, however the mouse \
                         was not captured", ui_id);
            },
        }
    }

    /// Indicate that the widget with the given UiId has captured the keyboard.
    pub fn keyboard_captured_by(&mut self, ui_id: UiId) {
        match self.maybe_captured_keyboard {
            Some(Capturing::Captured(captured_ui_id)) => if ui_id != captured_ui_id {
                println!("Warning: Widget {:?} tried to capture the keyboard, however it is \
                         already captured by {:?}.", ui_id, captured_ui_id);
            },
            Some(Capturing::JustReleased) => {
                println!("Warning: Widget {:?} tried to capture the keyboard, however it was \
                         already captured.", ui_id);
            },
            None => self.maybe_captured_keyboard = Some(Capturing::Captured(ui_id)),
        }
    }

    /// Indicate that the widget is no longer capturing the keyboard.
    pub fn keyboard_uncaptured_by(&mut self, ui_id: UiId) {
        match self.maybe_captured_keyboard {
            Some(Capturing::Captured(captured_ui_id)) => if ui_id != captured_ui_id {
                println!("Warning: Widget {:?} tried to uncapture the keyboard, however it is \
                         actually captured by {:?}.", ui_id, captured_ui_id);
            } else {
                self.maybe_captured_keyboard = Some(Capturing::JustReleased);
            },
            Some(Capturing::JustReleased) => {
                println!("Warning: Widget {:?} tried to uncapture the keyboard, however it had \
                         already been released this cycle.", ui_id);
            },
            None => {
                println!("Warning: Widget {:?} tried to uncapture the keyboard, however the mouse \
                         was not captured", ui_id);
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
    pub fn draw<G>(&mut self, graphics: &mut G)
        where
            C: CharacterCache,
            G: Graphics<Texture = C::Texture>,
    {
        use elmesque::Renderer;
        use std::cmp::Ordering;

        let Ui {
            ref mut canvas_cache,
            ref mut widget_cache,
            ref win_w, ref win_h,
            ref mut character_cache,
            ..
        } = *self;

        // Collect references to the widgets so that we can sort them without changing cache order.
        let mut widgets: Vec<_> = widget_cache.iter_mut()
            .filter(|widget| widget.set_since_last_drawn)
            .collect();

        for widget in widgets.iter_mut() {
            widget.set_since_last_drawn = false;
        }

        // Check for captured widgets and take them from the Vec (we want to draw them last).
        let maybe_mouse = self.maybe_captured_mouse.map(|(capturing, _)| capturing);
        let maybe_keyboard = self.maybe_captured_keyboard;
        let (maybe_mouse_widget, maybe_keyboard_widget) = match (maybe_mouse, maybe_keyboard) {
            (Some(Capturing::Captured(mouse_ui_id)), Some(Capturing::Captured(keyboard_ui_id))) => {
                if mouse_ui_id == keyboard_ui_id {
                    (Some(widgets.swap_remove(mouse_ui_id)), None)
                } else if mouse_ui_id > keyboard_ui_id {
                    let mouse_widget = widgets.swap_remove(mouse_ui_id);
                    let keyboard_widget = widgets.swap_remove(keyboard_ui_id);
                    (Some(mouse_widget), Some(keyboard_widget))
                }
                else {
                    let keyboard_widget = widgets.swap_remove(keyboard_ui_id);
                    let mouse_widget = widgets.swap_remove(mouse_ui_id);
                    (Some(mouse_widget), Some(keyboard_widget))
                }
            },
            (Some(Capturing::Captured(mouse_ui_id)), _) => {
                (Some(widgets.swap_remove(mouse_ui_id)), None)
            },
            (_, Some(Capturing::Captured(keyboard_ui_id))) => {
                (None, Some(widgets.swap_remove(keyboard_ui_id)))
            },
            (_, _) => (None, None),
        };

        // Sort the rest of the widgets by rendering depth.
        widgets.sort_by(|a, b| if      a.depth < b.depth { Ordering::Greater }
                               else if a.depth > b.depth { Ordering::Less }
                               else                      { Ordering::Equal });

        // Construct the elmesque Renderer for rendering the Elements.
        let mut renderer = Renderer::new(*win_w, *win_h, graphics).character_cache(character_cache);

        // Chain our widgets with the captured widgets and take their Elements.
        let elements = widgets.iter()
            .chain(maybe_keyboard_widget.iter())
            .chain(maybe_mouse_widget.iter())
            .map(|widget| &widget.element);

        // Draw all Canvas Splits.
        for canvas in canvas_cache.iter().filter(|canvas| canvas.set_since_last_drawn) {
            canvas.element.draw(&mut renderer);
        }

        // Draw all Elements.
        for element in elements {
            element.draw(&mut renderer);
        }

        // Indicate that the canvasses and widgets have now been drawn since the last time it was set.
        for canvas in canvas_cache.iter_mut() {
            canvas.set_since_last_drawn = false;
        }

    }

}
