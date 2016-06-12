use backend::{self, Backend, ToRawEvent};
use color::Color;
use event;
use graph::{self, Graph, NodeIndex};
use position::{Align, Direction, Dimensions, Padding, Place, Point, Position, Range, Rect, Scalar};
use render;
use rusttype;
use std;
use text;
use theme::Theme;
use utils;
use widget::{self, Widget};
use input;


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
    backend: std::marker::PhantomData<B>,
    /// The theme used to set default styling for widgets.
    pub theme: Theme,
    /// An index into the root widget of the graph, representing the entire window.
    pub window: NodeIndex,
    /// Handles aggregation of events and providing them to Widgets
    pub global_input: input::Global,
    /// Manages all fonts that have been loaded by the user.
    pub fonts: text::font::Map,
    /// A cache to use for rendering glyphs.
    glyph_cache: rusttype::gpu_cache::Cache,
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
    updated_widgets: std::collections::HashSet<NodeIndex>,
    /// The `updated_widgets` for the previous `set_widgets` stage.
    ///
    /// We use this to compare against the newly generated `updated_widgets` to see whether or not
    /// we require re-drawing.
    prev_updated_widgets: std::collections::HashSet<NodeIndex>,
    /// Scroll events that have been emitted during a call to `Ui::set_widgets`. These are usually
    /// emitted by some widget like the `Scrollbar`.
    ///
    /// These events will be drained and pushed onto the end of the `global_input` event buffer at
    /// the end of the `Ui::set_widgets` method. This ensures that the events are received by the
    /// target widgets during the next call to `Ui::set_widgets`.
    pending_scroll_events: Vec<event::Ui>,

    // TODO: Remove the following fields as they should now be handled by `input::Global`.

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


/// Each time conrod is required to redraw the GUI, it must draw for at least the next three frames
/// to ensure that, in the case that graphics buffers are being swapped, we have filled each
/// buffer. Otherwise if we don't draw into each buffer, we will probably be subject to flickering.
pub const SAFE_REDRAW_COUNT: u8 = 3;


impl<B> Ui<B>
    where B: Backend,
{

    /// A new, empty **Ui**.
    pub fn new(theme: Theme) -> Self {
        let widget_graph = Graph::new();
        let depth_order = graph::DepthOrder::new();
        let updated_widgets = std::collections::HashSet::new();
        Self::new_internal(theme, widget_graph, depth_order, updated_widgets)
    }

    /// A new **Ui** with the capacity given as a number of widgets.
    pub fn with_capacity(theme: Theme, n_widgets: usize) -> Self {
        let widget_graph = Graph::with_node_capacity(n_widgets);
        let depth_order = graph::DepthOrder::with_node_capacity(n_widgets);
        let updated_widgets = std::collections::HashSet::with_capacity(n_widgets);
        Self::new_internal(theme, widget_graph, depth_order, updated_widgets)
    }

    /// An internal constructor to share logic between the `new` and `with_capacity` constructors.
    fn new_internal(theme: Theme,
                    mut widget_graph: Graph,
                    depth_order: graph::DepthOrder,
                    updated_widgets: std::collections::HashSet<NodeIndex>) -> Self
    {
        use rusttype::gpu_cache::Cache;
        let window = widget_graph.add_placeholder();
        let prev_updated_widgets = updated_widgets.clone();
        let cache = new_glyph_cache(0, 0);
        Ui {
            backend: std::marker::PhantomData,
            widget_graph: widget_graph,
            theme: theme,
            fonts: text::font::Map::new(),
            glyph_cache: cache,
            window: window,
            win_w: 0.0,
            win_h: 0.0,
            maybe_prev_widget_idx: None,
            maybe_current_parent_idx: None,
            num_redraw_frames: SAFE_REDRAW_COUNT,
            redraw_count: SAFE_REDRAW_COUNT,
            maybe_background_color: None,
            depth_order: depth_order,
            updated_widgets: updated_widgets,
            prev_updated_widgets: prev_updated_widgets,
            global_input: input::Global::new(),
            pending_scroll_events: Vec::new(),
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
    pub fn updated_widgets(&self) -> &std::collections::HashSet<NodeIndex> {
        &self.updated_widgets
    }

    /// Borrow the **Ui**'s set of updated widgets.
    ///
    /// This set indicates which widgets have were instantiated during the previous call to
    /// `Ui::set_widgets`.
    pub fn prev_updated_widgets(&self) -> &std::collections::HashSet<NodeIndex> {
        &self.prev_updated_widgets
    }

    /// Scroll the widget at the given index by the given offset amount.
    ///
    /// The produced `Scroll` event will be applied upon the next call to `Ui::set_widgets`.
    pub fn scroll_widget<I>(&mut self, widget_idx: I, offset: [Scalar; 2])
        where I: Into<widget::Index>,
    {
        let widget_idx = widget_idx.into();
        let (x, y) = (offset[0], offset[1]);

        if x != 0.0 || y != 0.0 {
            let event = event::Ui::Scroll(Some(widget_idx), event::Scroll {
                x: x,
                y: y,
                modifiers: self.global_input.current.modifiers,
            }).into();
            self.global_input.push_event(event);
        }
    }

    /// Handle raw window events and update the `Ui` state accordingly.
    ///
    /// This occurs within several stages:
    ///
    /// 1. Convert the user's given `event` to a `RawEvent` so that the `Ui` may use it.
    /// 2. Interpret the `RawEvent` for higher-level `Event`s such as `DoubleClick`,
    ///    `WidgetCapturesKeyboard`, etc.
    /// 3. Update the `Ui`'s `global_input` `State` accordingly, depending on the `RawEvent`.
    /// 4. Store newly produced `event::Ui`s within the `global_input` so that they may be filtered
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
                graph::algo::pick_widgets(&ui.depth_order.indices,
                                          ui.global_input.current.mouse.xy)
                                          .next(&ui.widget_graph, &ui.depth_order.indices);

            // If MouseButton::Left is up and `widget_under_mouse` has changed, capture new widget
            // under mouse.
            if ui.global_input.current.mouse.buttons.left().is_up() {
                let widget_under_mouse = ui.global_input.current.widget_under_mouse;

                // Check to see if we need to uncapture a widget.
                if let Some(idx) = ui.global_input.current.widget_capturing_mouse {
                    if widget_under_mouse != Some(idx) {
                        let event = event::Ui::WidgetUncapturesMouse(idx).into();
                        ui.global_input.push_event(event);
                        ui.global_input.current.widget_capturing_mouse = None;
                    }
                }

                // Check to see if there is a new widget capturing the mouse.
                if ui.global_input.current.widget_capturing_mouse.is_none() {
                    if let Some(idx) = widget_under_mouse {
                        let event = event::Ui::WidgetCapturesMouse(idx).into();
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

                    // If either dimension is greater, re-make the glyph cache.
                    if w > self.win_w || h > self.win_h {
                        self.glyph_cache = new_glyph_cache(w as u32, h as u32);
                    }

                    self.win_w = w;
                    self.win_h = h;
                    track_widget_under_mouse_and_update_capturing(self);
                }
            },

            // Here we handle all user input given to conrod.
            //
            // Not only do we store the `Input` event as a `Event::Raw`, we also use them to
            // interpret higher level events such as `Click` or `Drag`.
            //
            // Finally, we also ensure that the `current_state` is up-to-date.
            backend::event::Event::Input(input_event) => {
                use event;
                use input::Button;
                use input::state::mouse::Button as MouseButton;

                // Update our global_input with the raw input event.
                self.global_input.push_event(input_event.clone().into());

                match input_event {

                    // Some button was pressed, whether keyboard, mouse or some other device.
                    Input::Press(button_type) => match button_type {

                        // Check to see whether we need to (un)capture the keyboard or mouse.
                        Button::Mouse(mouse_button) => {

                            // Create a mouse `Press` event.
                            let mouse_xy = self.global_input.current.mouse.xy;
                            let press = event::Press {
                                button: event::Button::Mouse(mouse_button, mouse_xy),
                                modifiers: self.global_input.current.modifiers,
                            };
                            let widget = self.global_input.current.widget_capturing_mouse;
                            let press_event = event::Ui::Press(widget, press).into();
                            self.global_input.push_event(press_event);

                            if let MouseButton::Left = mouse_button {
                                // Check to see if we need to uncapture the keyboard.
                                if let Some(idx) = self.global_input.current.widget_capturing_keyboard {
                                    if Some(idx) != self.global_input.current.widget_under_mouse {
                                        let event = event::Ui::WidgetUncapturesKeyboard(idx).into();
                                        self.global_input.push_event(event);
                                        self.global_input.current.widget_capturing_keyboard = None;
                                    }
                                }

                                // Check to see if we need to capture the keyboard.
                                if let Some(idx) = self.global_input.current.widget_under_mouse {
                                    let event = event::Ui::WidgetCapturesKeyboard(idx).into();
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

                            // Create a keyboard `Press` event.
                            let press = event::Press {
                                button: event::Button::Keyboard(key),
                                modifiers: self.global_input.current.modifiers,
                            };
                            let widget = self.global_input.current.widget_capturing_keyboard;
                            let press_event = event::Ui::Press(widget, press).into();
                            self.global_input.push_event(press_event);

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
                    // 2. DoubleClick
                    // 2. WidgetUncapturesMouse
                    Input::Release(button_type) => match button_type {
                        Button::Mouse(mouse_button) => {

                            // Create a `Release` event.
                            let mouse_xy = self.global_input.current.mouse.xy;
                            let release = event::Release {
                                button: event::Button::Mouse(mouse_button, mouse_xy),
                                modifiers: self.global_input.current.modifiers,
                            };
                            let widget = self.global_input.current.widget_capturing_mouse;
                            let release_event = event::Ui::Release(widget, release).into();
                            self.global_input.push_event(release_event);

                            // Check for `Click` and `DoubleClick` events.
                            let down = self.global_input.current.mouse.buttons[mouse_button].if_down();
                            if let Some((_, widget)) = down {

                                // The widget that's being clicked.
                                let clicked_widget = self.global_input.current.widget_under_mouse
                                    .and_then(|released| widget.and_then(|pressed| {
                                        if pressed == released { Some(released) } else { None }
                                    }));

                                let click = event::Click {
                                    button: mouse_button,
                                    xy: self.global_input.current.mouse.xy,
                                    modifiers: self.global_input.current.modifiers,
                                };

                                let click_event = event::Ui::Click(clicked_widget, click).into();
                                self.global_input.push_event(click_event);

                                let now = std::time::Instant::now();
                                let double_click = self.global_input.last_click
                                    .and_then(|(last_time, last_click)| {

                                        // If the button of this click is different to the button
                                        // of last click, don't create a `DoubleClick`.
                                        if click.button != last_click.button {
                                            return None;
                                        }

                                        // If the mouse has moved since the last click, don't
                                        // create a `DoubleClick`.
                                        if click.xy != last_click.xy {
                                            return None;
                                        }

                                        // If the duration since the last click is longer than the
                                        // double_click_threshold, don't create a `DoubleClick`.
                                        let duration = now.duration_since(last_time);
                                        // TODO: Work out how to get this threshold from the user's
                                        // system preferences.
                                        let threshold = self.theme.double_click_threshold;
                                        if duration >= threshold {
                                            return None;
                                        }

                                        Some(event::DoubleClick {
                                            button: click.button,
                                            xy: click.xy,
                                            modifiers: click.modifiers,
                                        })
                                    });

                                if let Some(double_click) = double_click {
                                    // Reset the `last_click` to `None`, as to not register another
                                    // `DoubleClick` on the next consecutive `Click`.
                                    self.global_input.last_click = None;
                                    let double_click_event =
                                        event::Ui::DoubleClick(clicked_widget, double_click).into();
                                    self.global_input.push_event(double_click_event);

                                } else {
                                    // Set the `Click` that we just stored as the `last_click`.
                                    self.global_input.last_click = Some((now, click));
                                }
                            }

                            // Uncapture widget capturing mouse if MouseButton::Left is down and
                            // widget_under_mouse != capturing widget.
                            if let MouseButton::Left = mouse_button {
                                if let Some(idx) = self.global_input.current.widget_capturing_mouse {
                                    if Some(idx) != self.global_input.current.widget_under_mouse {
                                        let event = event::Ui::WidgetUncapturesMouse(idx).into();
                                        self.global_input.push_event(event);
                                        self.global_input.current.widget_capturing_mouse = None;
                                    }
                                }
                            }

                            // Release the given mouse_button from the input::State.
                            self.global_input.current.mouse.buttons.release(mouse_button);
                        },
                        
                        Button::Keyboard(key) => {

                            // Create a `Release` event.
                            let release = event::Release {
                                button: event::Button::Keyboard(key),
                                modifiers: self.global_input.current.modifiers,
                            };
                            let widget = self.global_input.current.widget_capturing_keyboard;
                            let release_event = event::Ui::Release(widget, release).into();
                            self.global_input.push_event(release_event);

                            // If a modifier key was released, remove it from the current set.
                            if let Some(modifier) = filter_modifier(key) {
                                self.global_input.current.modifiers.remove(modifier);
                            }
                        },

                        _ => (),
                    },

                    // The window was resized.
                    Input::Resize(w, h) => {

                        // Create a `WindowResized` event.
                        let (w, h) = (w as Scalar, h as Scalar);
                        let window_resized = event::Ui::WindowResized([w, h]).into();
                        self.global_input.push_event(window_resized);

                        // If either dimension is greater, re-make the glyph cache.
                        if w > self.win_w || h > self.win_h {
                            self.glyph_cache = new_glyph_cache(w as u32, h as u32);
                        }

                        self.win_w = w;
                        self.win_h = h;
                        self.needs_redraw();
                    },

                    // The mouse cursor was moved to a new position.
                    //
                    // Checks for events in the following order:
                    // 1. `Drag`
                    // 2. `WidgetUncapturesMouse`
                    // 3. `WidgetCapturesMouse`
                    Input::Move(motion) => {

                        // Create a `Move` event.
                        let move_ = event::Move {
                            motion: motion,
                            modifiers: self.global_input.current.modifiers,
                        };
                        let widget = self.global_input.current.widget_capturing_mouse;
                        let move_event = event::Ui::Move(widget, move_).into();
                        self.global_input.push_event(move_event);

                        match motion {

                            Motion::MouseCursor(x, y) => {

                                // Check for drag events.
                                let last_mouse_xy = self.global_input.current.mouse.xy;
                                let mouse_xy = [x, y];
                                let delta_xy = utils::vec2_sub(mouse_xy, last_mouse_xy);
                                let distance = (delta_xy[0] + delta_xy[1]).abs().sqrt();
                                if distance > self.theme.mouse_drag_threshold {
                                    // For each button that is down, trigger a drag event.
                                    let buttons = self.global_input.current.mouse.buttons.clone();
                                    for (btn, btn_xy, widget) in buttons.pressed() {
                                        let total_delta_xy = utils::vec2_sub(mouse_xy, btn_xy);
                                        let event = event::Ui::Drag(widget, event::Drag {
                                            button: btn,
                                            origin: btn_xy,
                                            from: last_mouse_xy,
                                            to: mouse_xy,
                                            delta_xy: delta_xy,
                                            total_delta_xy: total_delta_xy,
                                            modifiers: self.global_input.current.modifiers,
                                        }).into();
                                        self.global_input.push_event(event);
                                    }
                                }

                                // Update the position of the mouse within the global_input's
                                // input::State.
                                self.global_input.current.mouse.xy = mouse_xy;

                                track_widget_under_mouse_and_update_capturing(self);
                            },

                            // The mouse was scrolled.
                            Motion::MouseScroll(x, y) => {

                                let mut scrollable_widgets = {
                                    let depth_order = &self.depth_order.indices;
                                    let mouse_xy = self.global_input.current.mouse.xy;
                                    graph::algo::pick_scrollable_widgets(depth_order, mouse_xy)
                                };

                                // Iterate through the scrollable widgets from top to bottom.
                                //
                                // A scroll event will be created for the first scrollable widget
                                // that hasn't already reached the bound of the scroll event's
                                // direction.
                                while let Some(idx) =
                                    scrollable_widgets.next(&self.widget_graph,
                                                            &self.depth_order.indices)
                                {

                                    let (kid_area, maybe_x_scroll, maybe_y_scroll) =
                                        match self.widget_graph.widget(idx) {
                                            Some(widget) => {
                                                (widget.kid_area,
                                                 widget.maybe_x_scroll_state,
                                                 widget.maybe_y_scroll_state)
                                            },
                                            None => continue,
                                        };

                                    fn offset_is_at_bound<A>(scroll: &widget::scroll::State<A>,
                                                             additional_offset: Scalar) -> bool
                                    {
                                        use utils;

                                        fn approx_eq(a: Scalar, b: Scalar) -> bool {
                                            (a - b).abs() < 0.000001
                                        }

                                        if additional_offset.is_sign_positive() {
                                            let max = utils::partial_max(scroll.offset_bounds.start,
                                                                         scroll.offset_bounds.end);
                                            approx_eq(scroll.offset, max)
                                        } else {
                                            let min = utils::partial_min(scroll.offset_bounds.start,
                                                                         scroll.offset_bounds.end);
                                            approx_eq(scroll.offset, min)
                                        }
                                    }

                                    let mut scroll_x = false;
                                    let mut scroll_y = false;

                                    // Check whether the x axis is scrollable.
                                    if x != 0.0 {
                                        let new_scroll =
                                            widget::scroll::State::update(self, idx, &kid_area,
                                                                          maybe_x_scroll, x);
                                        if let Some(prev_scroll) = maybe_x_scroll {
                                            let (prev_is_at_bound, new_is_at_bound) =
                                                (offset_is_at_bound(&prev_scroll, x),
                                                 offset_is_at_bound(&new_scroll, x));
                                            scroll_x = !prev_is_at_bound || !new_is_at_bound;
                                        }
                                    }

                                    // Check whether the y axis is scrollable.
                                    if y != 0.0 {
                                        let new_scroll =
                                            widget::scroll::State::update(self, idx, &kid_area,
                                                                          maybe_y_scroll, y);
                                        if let Some(prev_scroll) = maybe_y_scroll {
                                            let (prev_is_at_bound, new_is_at_bound) =
                                                (offset_is_at_bound(&prev_scroll, y),
                                                 offset_is_at_bound(&new_scroll, y));
                                            scroll_y = !prev_is_at_bound || !new_is_at_bound;
                                        }
                                    }

                                    // Create a `Scroll` event if either axis is scrollable.
                                    if scroll_x || scroll_y {
                                        let event = event::Ui::Scroll(Some(idx), event::Scroll {
                                            x: x,
                                            y: y,
                                            modifiers: self.global_input.current.modifiers,
                                        }).into();
                                        self.global_input.push_event(event);

                                        // Now that we've scrolled the top, scrollable widget,
                                        // we're done with the loop.
                                        break;
                                    }
                                }

                                // If no scrollable widgets could be scrolled, emit the event to
                                // the widget that currently captures the mouse.
                                if x != 0.0 || y != 0.0 {
                                    let widget = self.global_input.current.widget_capturing_mouse;
                                    if let Some(idx) = widget {
                                        if let Some(widget) = self.widget_graph.widget(idx) {
                                            // Only create the event if the widget is not
                                            // scrollable, as the event would have already been
                                            // created within the above loop.
                                            if widget.maybe_x_scroll_state.is_none()
                                            && widget.maybe_y_scroll_state.is_none() {
                                                let scroll = event::Scroll {
                                                    x: x,
                                                    y: y,
                                                    modifiers: self.global_input.current.modifiers,
                                                };
                                                let event = event::Ui::Scroll(Some(idx), scroll);
                                                self.global_input.push_event(event.into());
                                            }
                                        }
                                    }
                                }

                                // Now that there might be a different widget under the mouse, we
                                // must update the capturing state.
                                track_widget_under_mouse_and_update_capturing(self);
                            },

                            _ => (),

                        }
                    },

                    Input::Text(string) => {
                        // Create a `Text` event.
                        let text = event::Text {
                            string: string,
                            modifiers: self.global_input.current.modifiers,
                        };
                        let widget = self.global_input.current.widget_capturing_keyboard;
                        let text_event = event::Ui::Text(widget, text).into();
                        self.global_input.push_event(text_event);
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
        use utils::vec2_add;

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

        // Move the previous `updated_widgets` to `prev_updated_widgets` and clear
        // `updated_widgets` so that we're ready to store the newly updated widgets.
        {
            let Ui { ref mut updated_widgets, ref mut prev_updated_widgets, .. } = *self;
            std::mem::swap(updated_widgets, prev_updated_widgets);
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

        // Update the **DepthOrder** so that it reflects the **Graph**'s current state.
        {
            let Ui {
                ref widget_graph,
                ref mut depth_order,
                window,
                ref updated_widgets,
                ..
            } = *self;

            depth_order.update(widget_graph, window, updated_widgets);
        }

        // Reset the global input state. Note that this is the **only** time this should be called.
        self.global_input.clear_events_and_update_start_state();

        // Move all pending `Scroll` events that have been produced since the start of this method
        // into the `global_input` event buffer.
        for scroll_event in self.pending_scroll_events.drain(0..) {
            self.global_input.push_event(scroll_event.into());
        }
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
    pub fn draw(&mut self) -> render::Primitives {
        let Ui {
            ref mut redraw_count,
            ref widget_graph,
            ref depth_order,
            ref theme,
            win_w, win_h,
            ..
        } = *self;

        // Use the depth_order indices as the order for drawing.
        let indices = &depth_order.indices;

        // We're about to draw everything, so take one from the redraw count.
        if *redraw_count > 0 {
            *redraw_count -= 1;
        }

        render::Primitives::new(widget_graph, indices, theme, [win_w, win_h])
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
    pub fn draw_if_changed(&mut self) -> Option<render::Primitives> {
        if self.redraw_count > 0 {
            return Some(self.draw())
        }
        None
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

    /// A convenience method for borrowing the `Font` for the given `Id` if it exists.
    pub fn font(&self, id: text::font::Id) -> Option<&text::Font> {
        self.ui.fonts.get(id)
    }

    /// Returns the dimensions of the window
    pub fn window_dim(&self) -> Dimensions {
        [self.ui.win_w, self.ui.win_h]
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

    /// Scroll the widget at the given index by the given offset amount.
    ///
    /// The produced `Scroll` event will be pushed to the `pending_scroll_events` and will be
    /// applied to the widget during the next call to `Ui::set_widgets`.
    pub fn scroll_widget<I>(&mut self, widget_idx: I, offset: [Scalar; 2])
        where I: Into<widget::Index>
    {
        let widget_idx = widget_idx.into();
        let (x, y) = (offset[0], offset[1]);

        if x != 0.0 || y != 0.0 {
            let event = event::Ui::Scroll(Some(widget_idx), event::Scroll {
                x: x,
                y: y,
                modifiers: self.ui.global_input.current.modifiers,
            });
            self.ui.pending_scroll_events.push(event);
        }
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

/// Construct a new RustType GPU cache with the given dimensions.
fn new_glyph_cache(w: u32, h: u32) -> rusttype::gpu_cache::Cache {
    const POSITION_TOLERANCE: f32 = 0.1;
    const SCALE_TOLERANCE: f32 = 0.1;
    rusttype::gpu_cache::Cache::new(w, h, SCALE_TOLERANCE, POSITION_TOLERANCE)
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
