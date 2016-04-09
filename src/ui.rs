
use {CharacterCache, Scalar};
use backend::{self, Backend, ToRawEvent};
use backend::graphics::{Context, Graphics};
use color::Color;
use glyph_cache::GlyphCache;
use graph::{self, Graph, NodeIndex};
use mouse::{self, Mouse};
use position::{Align, Direction, Dimensions, Padding, Place, Point, Position, Range, Rect};
use std::collections::HashSet;
use std::io::Write;
use std::marker::PhantomData;
use theme::Theme;
use widget::{self, Widget};
use input;
use event::UiEvent;


/// Indicates whether or not the Mouse has been captured by a widget.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum Capturing {
    /// The Ui is captured by the Widget with the given widget::Index.
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
pub struct Ui<B>
    where B: Backend,
{
    /// The backend used by the `Ui`, providing the `Graphics` and `CharacterCache` types.
    backend: PhantomData<B>,
    /// The theme used to set default styling for widgets.
    pub theme: Theme,
    /// Cache for character textures, used for label width calculation and glyph rendering.
    pub glyph_cache: GlyphCache<B::CharacterCache>,
    /// An index into the root widget of the graph, representing the entire window.
    pub window: NodeIndex,
    /// Handles aggregation of events and providing them to Widgets
    pub global_input: input::Global,
    /// The Widget cache, storing state for all widgets.
    widget_graph: Graph,
    /// The widget::Index of the widget that was last updated/set.
    maybe_prev_widget_idx: Option<widget::Index>,
    /// The widget::Index of the last widget used as a parent for another widget.
    maybe_current_parent_idx: Option<widget::Index>,
    /// The number of frames that that will be used for the `redraw_count` when `need_redraw` is
    /// triggered.
    num_redraw_frames: u8,
    /// Whether or not the `Ui` needs to be re-drawn to screen.
    redraw_count: u8,
    /// A background color to clear the screen with before drawing if one was given.
    maybe_background_color: Option<Color>,
    /// The order in which widgets from the `widget_graph` are drawn.
    depth_order: graph::DepthOrder,
    /// The set of widgets that have been updated since the beginning of the `set_widgets` stage.
    updated_widgets: HashSet<NodeIndex>,
    /// The `updated_widgets` for the previous `set_widgets` stage.
    ///
    /// We use this to compare against the newly generated `updated_widgets` to see whether or not
    /// we require re-drawing.
    prev_updated_widgets: HashSet<NodeIndex>,

    // TODO: Remove the following fields as they should now be handled by `input::Global`.

    /// The latest received mouse state.
    mouse: Mouse,
    /// Keys that have been pressed since the end of the last render cycle.
    keys_just_pressed: Vec<input::keyboard::Key>,
    /// Keys that have been released since the end of the last render cycle.
    keys_just_released: Vec<input::keyboard::Key>,
    /// Text that has been entered since the end of the last render cycle.
    text_just_entered: Vec<String>,
    /// If the mouse is currently over a widget, its ID will be here.
    maybe_widget_under_mouse: Option<widget::Index>,
    /// The ID of the top-most scrollable widget under the cursor (if there is one).
    maybe_top_scrollable_widget_under_mouse: Option<widget::Index>,
    /// The widget::Index of the widget currently capturing mouse input if there is one.
    maybe_captured_mouse: Option<Capturing>,
    /// The widget::Index of the widget currently capturing keyboard input if there is one.
    maybe_captured_keyboard: Option<Capturing>,
    /// Window width.
    pub win_w: f64,
    /// Window height.
    pub win_h: f64,
}

/// A wrapper around the `Ui` that restricts the user from mutating the `Ui` in certain ways while
/// in the scope of the `Ui::set_widgets` function and within `Widget`s' `update` methods. Using
/// the `UiCell`, users may access the `Ui` immutably (via `Deref`) however they wish, however they
/// may only mutate the `Ui` via the `&mut self` methods provided by the `UiCell`.
///
/// The name came from its likening to a "jail cell for the `Ui`", as it restricts a user's access
/// to it. However, we realise that the name may also cause ambiguity with the std `Cell` and
/// `RefCell` types (which `UiCell` has nothing to do with). Thus, if you have a better name for
/// this type in mind, please let us know at the github repo via an issue or PR sometime before we
/// hit 1.0.0!
pub struct UiCell<'a, B: 'a>
    where B: Backend,
{
    /// A mutable reference to a **Ui**.
    ui: &'a mut Ui<B>,
}

/// A wrapper over the current user input state.
///
/// NOTE: This is deprecated in favour of the new `events` API introduced in PR #684 and will be
/// removed once all internal widgets have been ported over to the new API.
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


/// Each time conrod is required to redraw the GUI, it must draw for at least the next three frames
/// to ensure that, in the case that graphics buffers are being swapped, we have filled each
/// buffer. Otherwise if we don't draw into each buffer, we will probably be subject to flickering.
pub const SAFE_REDRAW_COUNT: u8 = 3;


impl<B> Ui<B>
    where B: Backend,
{

    /// A new, empty **Ui**.
    pub fn new(character_cache: B::CharacterCache, theme: Theme) -> Self {
        let widget_graph = Graph::new();
        let depth_order = graph::DepthOrder::new();
        let updated_widgets = HashSet::new();
        Self::new_internal(character_cache, theme, widget_graph, depth_order, updated_widgets)
    }

    /// A new **Ui** with the capacity given as a number of widgets.
    pub fn with_capacity(character_cache: B::CharacterCache,
                         theme: Theme,
                         n_widgets: usize) -> Self
    {
        let widget_graph = Graph::with_node_capacity(n_widgets);
        let depth_order = graph::DepthOrder::with_node_capacity(n_widgets);
        let updated_widgets = HashSet::with_capacity(n_widgets);
        Self::new_internal(character_cache, theme, widget_graph, depth_order, updated_widgets)
    }

    /// An internal constructor to share logic between the `new` and `with_capacity` constructors.
    fn new_internal(character_cache: B::CharacterCache,
                    theme: Theme,
                    mut widget_graph: Graph,
                    depth_order: graph::DepthOrder,
                    updated_widgets: HashSet<NodeIndex>) -> Self
    {
        let window = widget_graph.add_placeholder();
        let prev_updated_widgets = updated_widgets.clone();
        Ui {
            backend: PhantomData,
            widget_graph: widget_graph,
            theme: theme,
            window: window,
            mouse: Mouse::new(),
            keys_just_pressed: Vec::with_capacity(10),
            keys_just_released: Vec::with_capacity(10),
            text_just_entered: Vec::with_capacity(10),
            glyph_cache: GlyphCache::new(character_cache),
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
            depth_order: depth_order,
            updated_widgets: updated_widgets,
            prev_updated_widgets: prev_updated_widgets,
            global_input: input::Global::new(),
        }
    }

    /// Returns a `input::Widget` for the given widget
    pub fn widget_input<I: Into<widget::Index>>(&self, widget: I) -> input::Widget {
        let idx = widget.into();

        // If there's no rectangle for a given widget, then we use one with zero area.
        // This means that the resulting `input::Widget` will not include any mouse events
        // unless it has captured the mouse, since none will have occured over that area.
        let rect = self.rect_of(idx).unwrap_or_else(|| {
            let right_edge = self.win_w / 2.0;
            let bottom_edge = self.win_h / 2.0;
            Rect::from_xy_dim([right_edge, bottom_edge], [0.0, 0.0])
        });
        input::Widget::for_widget(idx, rect, &self.global_input)
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
    pub fn w_of<I: Into<widget::Index>>(&self, idx: I) -> Option<Scalar> {
        self.rect_of(idx).map(|rect| rect.w())
    }

    /// The absolute height of the widget at the given index.
    ///
    /// Returns `None` if there is no widget for the given index.
    pub fn h_of<I: Into<widget::Index>>(&self, idx: I) -> Option<Scalar> {
        self.rect_of(idx).map(|rect| rect.h())
    }

    /// The absolute dimensions for the widget at the given index.
    ///
    /// Returns `None` if there is no widget for the given index.
    pub fn wh_of<I: Into<widget::Index>>(&self, idx: I) -> Option<Dimensions> {
        self.rect_of(idx).map(|rect| rect.dim())
    }

    /// The coordinates for the widget at the given index.
    ///
    /// Returns `None` if there is no widget for the given index.
    pub fn xy_of<I: Into<widget::Index>>(&self, idx: I) -> Option<Point> {
        self.rect_of(idx).map(|rect| rect.xy())
    }

    /// The `kid_area` of the widget at the given index.
    ///
    /// Returns `None` if there is no widget for the given index.
    pub fn kid_area_of<I: Into<widget::Index>>(&self, idx: I) -> Option<Rect> {
        let idx: widget::Index = idx.into();
        self.widget_graph.widget(idx).map(|widget| {
            widget.kid_area.rect.padding(widget.kid_area.pad)
        })
    }

    /// An index to the previously updated widget if there is one.
    pub fn maybe_prev_widget(&self) -> Option<widget::Index> {
        self.maybe_prev_widget_idx
    }

    /// Borrow the **Ui**'s `widget_graph`.
    pub fn widget_graph(&self) -> &Graph {
        &self.widget_graph
    }

    /// Borrow the **Ui**'s set of updated widgets.
    ///
    /// This set indicates which widgets have been instantiated since the beginning of the most
    /// recent `Ui::set_widgets` call.
    pub fn updated_widgets(&self) -> &HashSet<NodeIndex> {
        &self.updated_widgets
    }

    /// Borrow the **Ui**'s set of updated widgets.
    ///
    /// This set indicates which widgets have were instantiated during the previous call to
    /// `Ui::set_widgets`.
    pub fn prev_updated_widgets(&self) -> &HashSet<NodeIndex> {
        &self.prev_updated_widgets
    }

    /// Handle raw window events and update the `Ui` state accordingly.
    ///
    /// This occurs within several stages:
    ///
    /// 1. Convert the given `event` to a `RawEvent` so that the `Ui` may use it.
    /// 2. Interpret the `RawEvent` for higher-level `UiEvent`s such as `DoubleClick`,
    ///    `WidgetCapturesKeyboard`, etc.
    /// 3. Update the `Ui`'s `global_input` `State` accordingly, depending on the `RawEvent`.
    /// 4. Store newly produced `UiEvent`s within the `global_input` so that they may be filtered
    ///    and fed to `Widget`s next time `Ui::set_widget` is called.
    ///
    /// This method *drives* the `Ui` forward, and is what allows for using conrod's `Ui` with any
    /// window event stream.
    ///
    /// The given `event` must implement the **ToRawEvent** trait so that it can be converted to a
    /// `RawEvent` that can be used by the `Ui`.
    pub fn handle_event<E>(&mut self, event: E)
        where E: ToRawEvent,
    {
        use backend::event::{Input, Motion, Key, ModifierKey, RawEvent};

        // Determines which widget is currently under the mouse and sets it within the `Ui`'s
        // `input::Global`'s `input::State`.
        //
        // If the `widget_under_mouse` has changed, this function will also update the
        // `widget_capturing_mouse`.
        //
        // If the left mouse button is up, we assume that the widget directly under the
        // mouse cursor captures all input from the mouse.
        //
        // If the left mouse button is down, we assume that the widget that was clicked
        // remains "pinned" and will continue to capture the mouse until it is
        // released.
        //
        // Note: This function expects that `ui.global_input.current.mouse.xy` is up-to-date.
        fn track_widget_under_mouse_and_update_capturing<B: Backend>(ui: &mut Ui<B>) {
            ui.global_input.current.widget_under_mouse =
                graph::algo::pick_widget(&ui.widget_graph,
                                         &ui.depth_order.indices,
                                         ui.global_input.current.mouse.xy);

            // If MouseButton::Left is up and `widget_under_mouse` has changed, capture new widget
            // under mouse.
            if ui.global_input.current.mouse.buttons.left().is_up() {
                let widget_under_mouse = ui.global_input.current.widget_under_mouse;

                // Check to see if we need to uncapture a widget.
                if let Some(idx) = ui.global_input.current.widget_capturing_mouse {
                    if widget_under_mouse != Some(idx) {
                        let event = UiEvent::WidgetUncapturesMouse(idx);
                        ui.global_input.push_event(event);
                        ui.global_input.current.widget_capturing_mouse = None;
                    }
                }

                // Check to see if there is a new widget capturing the mouse.
                if ui.global_input.current.widget_capturing_mouse.is_none() {
                    if let Some(idx) = widget_under_mouse {
                        let event = UiEvent::WidgetCapturesMouse(idx);
                        ui.global_input.push_event(event);
                        ui.global_input.current.widget_capturing_mouse = Some(idx);
                    }
                }
            }
        }

        // A function for filtering `ModifierKey`s.
        fn filter_modifier(key: Key) -> Option<ModifierKey> {
            use backend::event::keyboard::{CTRL, SHIFT, ALT, GUI};
            match key {
                Key::LCtrl | Key::RCtrl => Some(CTRL),
                Key::LShift | Key::RShift => Some(SHIFT),
                Key::LAlt | Key::RAlt => Some(ALT),
                Key::LGui | Key::RGui => Some(GUI),
                _ => None
            }
        }

        // Convert the user given event to a `RawEvent` or return early if we cannot.
        let event: RawEvent = match event.to_raw_event(self.win_w, self.win_h) {
            Some(event) => event,
            None => return,
        };

        match event {

            // On each `Render` we should check that our window dimensions are up to date.
            //
            // This event is also the first time that we receive the proper dimensions of the
            // window (when the `Ui` is created, the dimensions are set to `0`).
            backend::event::Event::Render(args) => {
                let (w, h) = (args.width as Scalar, args.height as Scalar);
                if self.win_w != w || self.win_h != h {
                    self.win_w = w;
                    self.win_h = h;
                    track_widget_under_mouse_and_update_capturing(self);
                }
            },

            // Here we handle all user input given to conrod.
            //
            // Not only do we store the `Input` event as a `UiEvent::Raw`, we also use them to
            // interpret higher level events such as `Click` or `Drag`.
            //
            // Finally, we also ensure that the `current_state` is up-to-date.
            backend::event::Event::Input(input_event) => {
                use event::{self, UiEvent};
                use input::Button;
                use input::state::mouse::Button as MouseButton;

                // Update our global_input with the raw input event.
                self.global_input.push_event(input_event.clone().into());

                match input_event {

                    // Some button was pressed, whether keyboard, mouse or some other device.
                    Input::Press(button_type) => match button_type {

                        // Check to see whether we need to (un)capture the keyboard or mouse.
                        Button::Mouse(mouse_button) => {
                            if let MouseButton::Left = mouse_button {
                                // Check to see if we need to uncapture the keyboard.
                                if let Some(idx) = self.global_input.current.widget_capturing_keyboard {
                                    if Some(idx) != self.global_input.current.widget_under_mouse {
                                        let event = UiEvent::WidgetUncapturesKeyboard(idx);
                                        self.global_input.push_event(event);
                                        self.global_input.current.widget_capturing_keyboard = None;
                                    }
                                }

                                // Check to see if we need to capture the keyboard.
                                if let Some(idx) = self.global_input.current.widget_under_mouse {
                                    let event = UiEvent::WidgetCapturesKeyboard(idx);
                                    self.global_input.push_event(event);
                                    self.global_input.current.widget_capturing_keyboard = Some(idx);
                                }
                            }

                            // Keep track of pressed buttons in the current input::State.
                            let xy = self.global_input.current.mouse.xy;
                            let widget = self.global_input.current.widget_under_mouse;
                            self.global_input.current.mouse.buttons.press(mouse_button, xy, widget);
                        },

                        Button::Keyboard(key) => {
                            // If some modifier key was pressed, add it to the current modifiers.
                            if let Some(modifier) = filter_modifier(key) {
                                self.global_input.current.modifiers.insert(modifier);
                            }

                            // If `Esc` was pressed, check to see if we need to cancel a `Drag` or
                            // uncapture a widget.
                            if let Key::Escape = key {
                                // TODO:
                                // 1. Cancel `Drag` if currently under way.
                                // 2. If mouse is captured due to pinning widget with left mouse button,
                                //    cancel capturing.
                            }
                        },

                        _ => {}
                    },

                    // Some button was released.
                    //
                    // Checks for events in the following order:
                    // 1. Click
                    // 2. WidgetUncapturesMouse
                    Input::Release(button_type) => match button_type {
                        Button::Mouse(mouse_button) => {

                            // Check for a `Click` event.
                            let down = self.global_input.current.mouse.buttons[mouse_button].if_down();
                            if let Some((_, widget)) = down {
                                let clicked_widget = self.global_input.current.widget_under_mouse
                                    .and_then(|released| widget.and_then(|pressed| {
                                        if pressed == released { Some(released) } else { None }
                                    }));
                                let event = UiEvent::Click(event::Click {
                                    button: mouse_button,
                                    xy: self.global_input.current.mouse.xy,
                                    modifiers: self.global_input.current.modifiers,
                                    widget: clicked_widget,
                                });
                                self.global_input.push_event(event);
                            }

                            // Uncapture widget capturing mouse if MouseButton::Left is down and
                            // widget_under_mouse != capturing widget.
                            if let MouseButton::Left = mouse_button {
                                if let Some(idx) = self.global_input.current.widget_capturing_mouse {
                                    if Some(idx) != self.global_input.current.widget_under_mouse {
                                        let event = UiEvent::WidgetUncapturesMouse(idx);
                                        self.global_input.push_event(event);
                                        self.global_input.current.widget_capturing_mouse = None;
                                    }
                                }
                            }

                            // Release the given mouse_button from the input::State.
                            self.global_input.current.mouse.buttons.release(mouse_button);
                        },
                        
                        // If a modifier key was released, remove it from the current modifiers.
                        Button::Keyboard(key) => if let Some(modifier) = filter_modifier(key) {
                            self.global_input.current.modifiers.remove(modifier);
                        },

                        _ => (),
                    },

                    // The window was resized.
                    Input::Resize(w, h) => {
                        self.win_w = w as Scalar;
                        self.win_h = h as Scalar;
                        self.needs_redraw();
                    },

                    // The mouse cursor was moved to a new position.
                    //
                    // Checks for events in the following order:
                    // 1. `Drag`
                    // 2. `WidgetUncapturesMouse`
                    // 3. `WidgetCapturesMouse`
                    Input::Move(Motion::MouseCursor(x, y)) => {

                        // Check for drag events.
                        let last_xy = self.global_input.current.mouse.xy;
                        let delta_xy = [x - last_xy[0], y - last_xy[1]];
                        let distance = (delta_xy[0] + delta_xy[1]).abs().sqrt();
                        if distance > self.theme.mouse_drag_threshold {
                            // For each button that is down, trigger a drag event.
                            let buttons = self.global_input.current.mouse.buttons.clone();
                            for (btn, btn_xy, widget) in buttons.pressed() {
                                let event = UiEvent::Drag(event::Drag {
                                    button: btn,
                                    start: btn_xy,
                                    end: [x, y],
                                    modifiers: self.global_input.current.modifiers,
                                    widget: widget,
                                });
                                self.global_input.push_event(event);
                            }
                        }

                        // Update the position of the mouse within the global_input's input::State.
                        self.global_input.current.mouse.xy = [x, y];

                        track_widget_under_mouse_and_update_capturing(self);
                    },

                    // The mouse was scrolled.
                    Input::Move(Motion::MouseScroll(x, y)) => {

                        let event = UiEvent::Scroll(event::Scroll {
                            x: x,
                            y: y,
                            modifiers: self.global_input.current.modifiers,
                        });
                        self.global_input.push_event(event);

                        track_widget_under_mouse_and_update_capturing(self);
                    },

                    _ => (),
                }

            },

            _ => (),
        }
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
                   x_position: Position,
                   y_position: Position,
                   dim: Dimensions,
                   place_on_kid_area: bool) -> Point
    {
        use vecmath::vec2_add;

        // Retrieves the absolute **Scalar** position from the given position for a single axis.
        //
        // The axis used is specified by the given range_from_rect function which, given some
        // **Rect**, returns the relevant **Range**.
        fn abs_from_position<B, R, P>(ui: &Ui<B>,
                                      position: Position,
                                      dim: Scalar,
                                      place_on_kid_area: bool,
                                      range_from_rect: R,
                                      start_and_end_pad: P) -> Scalar
            where B: Backend,
                  R: FnOnce(Rect) -> Range,
                  P: FnOnce(Padding) -> Range,
        {
            match position {

                Position::Absolute(abs) => abs,

                Position::Relative(rel, maybe_idx) =>
                    maybe_idx.or(ui.maybe_prev_widget_idx).or(Some(ui.window.into()))
                        .and_then(|idx| ui.rect_of(idx).map(range_from_rect))
                        .map(|other_range| other_range.middle() + rel)
                        .unwrap_or(rel),

                Position::Direction(direction, amt, maybe_idx) =>
                    maybe_idx.or(ui.maybe_prev_widget_idx)
                        .and_then(|idx| ui.rect_of(idx).map(range_from_rect))
                        .map(|other_range| {
                            let range = Range::from_pos_and_len(0.0, dim);
                            match direction {
                                Direction::Forwards => range.align_after(other_range).middle() + amt,
                                Direction::Backwards => range.align_before(other_range).middle() - amt,
                            }
                        })
                        .unwrap_or_else(|| match direction {
                            Direction::Forwards => amt,
                            Direction::Backwards => -amt,
                        }),

                Position::Align(align, maybe_idx) =>
                    maybe_idx.or(ui.maybe_prev_widget_idx).or(Some(ui.window.into()))
                        .and_then(|idx| ui.rect_of(idx).map(range_from_rect))
                        .map(|other_range| {
                            let range = Range::from_pos_and_len(0.0, dim);
                            match align {
                                Align::Start => range.align_start_of(other_range).middle(),
                                Align::Middle => other_range.middle(),
                                Align::End => range.align_end_of(other_range).middle(),
                            }
                        })
                        .unwrap_or(0.0),

                Position::Place(place, maybe_idx) => {
                    let parent_idx = maybe_idx
                        .or(ui.maybe_current_parent_idx)
                        .unwrap_or(ui.window.into());
                    let maybe_area = match place_on_kid_area {
                        true => ui.widget_graph.widget(parent_idx)
                            .map(|w| w.kid_area)
                            .map(|k| (range_from_rect(k.rect), start_and_end_pad(k.pad))),
                        false => ui.rect_of(parent_idx)
                            .map(|rect| (range_from_rect(rect), Range::new(0.0, 0.0))),
                    };
                    maybe_area
                        .map(|(parent_range, pad)| {
                            let range = Range::from_pos_and_len(0.0, dim);
                            let parent_range = parent_range.pad_start(pad.start).pad_end(pad.end);
                            match place {
                                Place::Start(maybe_mgn) =>
                                    range.align_start_of(parent_range).middle() + maybe_mgn.unwrap_or(0.0),
                                Place::Middle =>
                                    parent_range.middle(),
                                Place::End(maybe_mgn) =>
                                    range.align_end_of(parent_range).middle() - maybe_mgn.unwrap_or(0.0),
                            }
                        })
                        .unwrap_or(0.0)
                },

            }
        }

        fn x_range(rect: Rect) -> Range { rect.x }
        fn y_range(rect: Rect) -> Range { rect.y }
        fn x_pad(pad: Padding) -> Range { pad.x }
        fn y_pad(pad: Padding) -> Range { pad.y }
        let x = abs_from_position(self, x_position, dim[0], place_on_kid_area, x_range, x_pad);
        let y = abs_from_position(self, y_position, dim[1], place_on_kid_area, y_range, y_pad);
        let xy = [x, y];

        // Add the widget's parents' total combined scroll offset to the given xy.
        maybe_idx
            .map(|idx| vec2_add(xy, graph::algo::scroll_offset(&self.widget_graph, idx)))
            .unwrap_or(xy)
    }


    /// A function within which all widgets are instantiated by the user, normally situated within
    /// the "update" stage of an event loop.
    pub fn set_widgets<F>(&mut self, user_widgets_fn: F)
        where F: FnOnce(UiCell<B>),
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


        // Move the previous `updated_widgets` to `prev_updated_widgets` and clear
        // `updated_widgets` so that we're ready to store the newly updated widgets.
        {
            let Ui { ref mut updated_widgets, ref mut prev_updated_widgets, .. } = *self;
            ::std::mem::swap(updated_widgets, prev_updated_widgets);
            updated_widgets.clear();
        }

        // Instantiate the root `Window` `Widget`.
        //
        // This widget acts as the parent-most widget and root node for the Ui's `widget_graph`,
        // upon which all other widgets are placed.
        {
            use {color, Colorable, Frameable, FramedRectangle, Positionable, Widget};
            type Window = FramedRectangle;
            Window::new([self.win_w, self.win_h])
                .no_parent()
                .x_y(0.0, 0.0)
                .frame(0.0)
                .frame_color(color::BLACK.alpha(0.0))
                .color(self.maybe_background_color.unwrap_or(color::BLACK.alpha(0.0)))
                .set(self.window, &mut UiCell { ui: self });
        }

        self.maybe_current_parent_idx = Some(self.window.into());

        // Call the given user function for instantiating Widgets.
        user_widgets_fn(UiCell { ui: self });

        // We'll need to re-draw if we have gained or lost widgets.
        if self.updated_widgets != self.prev_updated_widgets {
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
            let Ui {
                ref widget_graph,
                ref mut depth_order,
                window,
                ref updated_widgets,
                ..
            } = *self;

            depth_order.update(widget_graph,
                               window,
                               updated_widgets,
                               maybe_captured_mouse,
                               maybe_captured_keyboard);
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

        // Reset the global input state. Note that this is the **only** time this should be called.
        self.global_input.clear_events_and_update_start_state();
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


    /// Draw the `Ui` in it's current state.
    ///
    /// NOTE: If you don't need to redraw your conrod GUI every frame, it is recommended to use the
    /// `Ui::draw_if_changed` method instead.
    pub fn draw<G>(&mut self, context: Context, graphics: &mut G)
        where G: Graphics<Texture=B::Texture>,
    {
        use backend::graphics::{draw_from_graph, Transformed};
        use std::ops::{Deref, DerefMut};

        let Ui {
            ref mut glyph_cache,
            ref mut redraw_count,
            ref widget_graph,
            ref depth_order,
            ref theme,
            ..
        } = *self;

        let view_size = context.get_view_size();
        let context = context.trans(view_size[0] / 2.0, view_size[1] / 2.0).scale(1.0, -1.0);

        // Retrieve the `CharacterCache` from the `Ui`'s `GlyphCache`.
        let mut ref_mut_character_cache = glyph_cache.deref().borrow_mut();
        let character_cache = ref_mut_character_cache.deref_mut();

        // Use the depth_order indices as the order for drawing.
        let indices = &depth_order.indices;

        // Draw the `Ui` from the `widget_graph`.
        draw_from_graph::<B, G>(context, graphics, character_cache, widget_graph, indices, theme);

        // Because we just drew everything, take one from the redraw count.
        if *redraw_count > 0 {
            *redraw_count -= 1;
        }
    }


    /// Same as the `Ui::draw` method, but *only* draws if the `redraw_count` is greater than 0.
    ///
    /// The `redraw_count` is set to `SAFE_REDRAW_COUNT` whenever a `Widget` indicates that it
    /// needs to be re-drawn.
    ///
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
        where G: Graphics<Texture=B::Texture>,
    {
        if self.redraw_count > 0 {
            self.draw(context, graphics);
        }
    }


    /// The **Rect** that bounds the kids of the widget with the given index.
    pub fn kids_bounding_box<I: Into<widget::Index>>(&self, idx: I) -> Option<Rect> {
        let idx: widget::Index = idx.into();
        graph::algo::kids_bounding_box(&self.widget_graph, &self.prev_updated_widgets, idx)
    }


    /// The **Rect** that represents the maximum fully visible area for the widget with the given
    /// index, including consideration of cropped scroll area.
    ///
    /// Otherwise, return None if the widget is not visible.
    pub fn visible_area<I: Into<widget::Index>>(&self, idx: I) -> Option<Rect> {
        let idx: widget::Index = idx.into();
        graph::algo::cropped_area_of_widget(&self.widget_graph, idx)
    }

}


impl<'a, B> UiCell<'a, B>
    where B: Backend,
{

    /// A reference to the `Theme` that is currently active within the `Ui`.
    pub fn theme(&self) -> &Theme { &self.ui.theme }

    /// A reference to the `Ui`'s `GlyphCache`.
    pub fn glyph_cache(&self) -> &GlyphCache<B::CharacterCache> { &self.ui.glyph_cache }

    /// Returns the dimensions of the window
    pub fn window_dim(&self) -> Dimensions {
        [self.ui.win_w, self.ui.win_h]
    }

    /// A struct representing the user input that has occurred since the last update, relevant to
    /// the widget with the given index.
    ///
    /// NOTE: This method is deprecated (following #684) and will be removed in favour of the
    /// `widget_input` and `global_input` methods.
    pub fn input<I: Into<widget::Index>>(&self, idx: I) -> UserInput {
        user_input(self.ui, idx.into())
    }

    /// Returns an immutable reference to the `input::Global` of the `Ui`.
    ///
    /// All coordinates here will be relative to the center of the window.
    pub fn global_input(&self) -> &input::Global {
        &self.ui.global_input
    }

    /// Returns a `input::Widget` with input events for the widget.
    ///
    /// All coordinates in the `input::Widget` will be relative to the widget at the given index.
    pub fn widget_input<I: Into<widget::Index>>(&self, idx: I) -> input::Widget {
        self.ui.widget_input(idx.into())
    }

    /// Have the widget capture the mouse input. The mouse state will be hidden from other
    /// widgets while captured.
    ///
    /// Returns true if the mouse was successfully captured.
    ///
    /// Returns false if it was already captured by some other widget.
    pub fn capture_mouse<I: Into<widget::Index>>(&mut self, idx: I) -> bool {
        mouse_captured_by(self.ui, idx.into())
    }

    /// Uncapture the mouse input.
    ///
    /// Returns true if the mouse was successfully uncaptured.
    ///
    /// Returns false if the mouse wasn't captured by our widget in the first place.
    pub fn uncapture_mouse<I: Into<widget::Index>>(&mut self, idx: I) -> bool {
        mouse_uncaptured_by(self.ui, idx.into())
    }

    /// Have the widget capture the keyboard input. The keyboard state will be hidden from other
    /// widgets while captured.
    ///
    /// Returns true if the keyboard was successfully captured.
    ///
    /// Returns false if it was already captured by some other widget.
    pub fn capture_keyboard<I: Into<widget::Index>>(&mut self, idx: I) -> bool {
        keyboard_captured_by(self.ui, idx.into())
    }

    /// Uncapture the keyboard input.
    ///
    /// Returns true if the keyboard was successfully uncaptured.
    ///
    /// Returns false if the keyboard wasn't captured by our widget in the first place.
    pub fn uncapture_keyboard<I: Into<widget::Index>>(&mut self, idx: I) -> bool {
        keyboard_uncaptured_by(self.ui, idx.into())
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
        widget_graph_mut(&mut self.ui).add_placeholder()
    }

    /// The **Rect** that bounds the kids of the widget with the given index.
    ///
    /// Returns `None` if the widget has no children or if there's is no widget for the given index.
    pub fn kids_bounding_box<I: Into<widget::Index>>(&self, idx: I) -> Option<Rect> {
        self.ui.kids_bounding_box(idx)
    }

}

impl<'a, B> ::std::ops::Deref for UiCell<'a, B>
    where B: Backend,
{
    type Target = Ui<B>;
    fn deref(&self) -> &Ui<B> {
        self.ui
    }
}

impl<'a, B> AsRef<Ui<B>> for UiCell<'a, B>
    where B: Backend,
{
    fn as_ref(&self) -> &Ui<B> {
        &self.ui
    }
}

/// A private constructor for the `UiCell` for internal use.
pub fn new_ui_cell<B>(ui: &mut Ui<B>) -> UiCell<B>
    where B: Backend,
{
    UiCell { ui: ui }
}

/// A function for retrieving the `&mut Ui<B>` from a `UiCell<B>`.
///
/// This function is only for internal use to allow for some `Ui` type acrobatics in order to
/// provide a nice *safe* API for the user.
pub fn ref_mut_from_ui_cell<'a, 'b, B>(ui_cell: &'a mut UiCell<'b, B>) -> &'a mut Ui<B>
    where 'b: 'a,
          B: Backend
{
    ui_cell.ui
}

/// A mutable reference to the given `Ui`'s widget `Graph`.
pub fn widget_graph_mut<B>(ui: &mut Ui<B>) -> &mut Graph
    where B: Backend,
{
    &mut ui.widget_graph
}


/// Infer a widget's `Depth` parent by examining it's *x* and *y* `Position`s.
///
/// When a different parent may be inferred from either `Position`, the *x* `Position` is favoured.
pub fn infer_parent_from_position<B>(ui: &Ui<B>, x_pos: Position, y_pos: Position)
    -> Option<widget::Index>
    where B: Backend,
{
    use Position::{Place, Relative, Direction, Align};
    match (x_pos, y_pos) {
        (Place(_, maybe_parent_idx), _) | (_, Place(_, maybe_parent_idx)) =>
            maybe_parent_idx,
        (Direction(_, _, maybe_idx), _) | (_, Direction(_, _, maybe_idx)) |
        (Align(_, maybe_idx), _)        | (_, Align(_, maybe_idx))        |
        (Relative(_, maybe_idx), _)     | (_, Relative(_, maybe_idx))     =>
            maybe_idx.or(ui.maybe_prev_widget_idx)
                .and_then(|idx| ui.widget_graph.depth_parent(idx)),
        _ => None,
    }
}


/// Attempts to infer the parent of a widget from its *x*/*y* `Position`s and the current state of
/// the `Ui`.
///
/// If no parent can be inferred via the `Position`s, the `maybe_current_parent_idx` will be used.
///
/// If `maybe_current_parent_idx` is `None`, the `Ui`'s `window` widget will be used.
///
/// **Note:** This function does not check whether or not using the `window` widget would cause a
/// cycle.
pub fn infer_parent_unchecked<B>(ui: &Ui<B>, x_pos: Position, y_pos: Position) -> widget::Index
    where B: Backend,
{
    infer_parent_from_position(ui, x_pos, y_pos)
        .or(ui.maybe_current_parent_idx)
        .unwrap_or(ui.window.into())
}


/// Return the user input state available for the widget with the given ID.
/// Take into consideration whether or not each input type is captured.
pub fn user_input<'a, B>(ui: &'a Ui<B>, idx: widget::Index) -> UserInput<'a>
    where B: Backend,
{
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
pub fn get_mouse_state<B>(ui: &Ui<B>, idx: widget::Index) -> Option<Mouse>
    where B: Backend,
{
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
pub fn mouse_captured_by<B>(ui: &mut Ui<B>, idx: widget::Index) -> bool
    where B: Backend,
{
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
pub fn mouse_uncaptured_by<B>(ui: &mut Ui<B>, idx: widget::Index) -> bool
    where B: Backend,
{
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
pub fn keyboard_captured_by<B>(ui: &mut Ui<B>, idx: widget::Index) -> bool
    where B: Backend,
{
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
pub fn keyboard_uncaptured_by<B>(ui: &mut Ui<B>, idx: widget::Index) -> bool
    where B: Backend,
{
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
pub fn pre_update_cache<B>(ui: &mut Ui<B>, widget: widget::PreUpdateCache)
    where B: Backend,
{
    ui.maybe_prev_widget_idx = Some(widget.idx);
    ui.maybe_current_parent_idx = widget.maybe_parent_idx;
    let widget_idx = widget.idx;
    ui.widget_graph.pre_update_cache(ui.window, widget, ui.updated_widgets.len());

    // Add the widget's `NodeIndex` to the set of updated widgets.
    let node_idx = ui.widget_graph.node_index(widget_idx).expect("No NodeIndex");
    ui.updated_widgets.insert(node_idx);
}

/// Cache some `PostUpdateCache` widget data into the widget graph.
/// Set the widget that is being cached as the new `prev_widget`.
/// Set the widget's parent as the new `current_parent`.
pub fn post_update_cache<B, W>(ui: &mut Ui<B>, widget: widget::PostUpdateCache<W>)
    where B: Backend,
          W: Widget,
          W::State: 'static,
          W::Style: 'static,
{
    ui.maybe_prev_widget_idx = Some(widget.idx);
    ui.maybe_current_parent_idx = widget.maybe_parent_idx;
    ui.widget_graph.post_update_cache(widget);
}


/// Clear the background with the given color.
pub fn clear_with<B>(ui: &mut Ui<B>, color: Color)
    where B: Backend,
{
    ui.maybe_background_color = Some(color);
}
