//! A demonstration of designing a custom, third-party widget.
//!
//! In this case, we'll design a simple circular button.
//!
//! All of the custom widget design will occur within the `circular_button` module.
//!
//! We'll *use* our fancy circular button in the `main` function (below the circular_button module).
//!
//! Note that in this case, we use `backend::src` to draw our widget, however in practise you may
//! use any backend you wish.
//!
//! For more information, please see the `Widget` trait documentation.

#[macro_use]
extern crate conrod_core;
extern crate conrod_glium;
#[macro_use]
extern crate conrod_winit;
extern crate find_folder;
extern crate glium;

mod support;

/// The module in which we'll implement our own custom circular button.
mod circular_button {
    use conrod_core::{
        self, widget, widget_ids, Colorable, Labelable, Point, Positionable, Widget,
    };

    /// The type upon which we'll implement the `Widget` trait.
    #[derive(WidgetCommon)]
    pub struct CircularButton<'a> {
        /// An object that handles some of the dirty work of rendering a GUI. We don't
        /// really have to worry about it.
        #[conrod(common_builder)]
        common: widget::CommonBuilder,
        /// Optional label string for the button.
        maybe_label: Option<&'a str>,
        /// See the Style struct below.
        style: Style,
        /// Whether the button is currently enabled, i.e. whether it responds to
        /// user input.
        enabled: bool,
    }

    // We use `#[derive(WidgetStyle)] to vastly simplify the definition and implementation of the
    // widget's associated `Style` type. This generates an implementation that automatically
    // retrieves defaults from the provided theme in the following order:
    //
    // 1. If the field is `None`, falls back to the style stored within the `Theme`.
    // 2. If there are no style defaults for the widget in the `Theme`, or if the
    //    default field is also `None`, falls back to the expression specified within
    //    the field's `#[conrod(default = "expr")]` attribute.

    /// Represents the unique styling for our CircularButton widget.
    #[derive(Copy, Clone, Debug, Default, PartialEq, WidgetStyle)]
    pub struct Style {
        /// Color of the button.
        #[conrod(default = "theme.shape_color")]
        pub color: Option<conrod_core::Color>,
        /// Color of the button's label.
        #[conrod(default = "theme.label_color")]
        pub label_color: Option<conrod_core::Color>,
        /// Font size of the button's label.
        #[conrod(default = "theme.font_size_medium")]
        pub label_font_size: Option<conrod_core::FontSize>,
        /// Specify a unique font for the label.
        #[conrod(default = "theme.font_id")]
        pub label_font_id: Option<Option<conrod_core::text::font::Id>>,
    }

    // We'll create the widget using a `Circle` widget and a `Text` widget for its label.
    //
    // Here is where we generate the type that will produce these identifiers.
    widget_ids! {
        struct Ids {
            circle,
            text,
        }
    }

    /// Represents the unique, cached state for our CircularButton widget.
    pub struct State {
        ids: Ids,
    }

    impl<'a> CircularButton<'a> {
        /// Create a button context to be built upon.
        pub fn new() -> Self {
            CircularButton {
                common: widget::CommonBuilder::default(),
                style: Style::default(),
                maybe_label: None,
                enabled: true,
            }
        }

        /// Specify the font used for displaying the label.
        pub fn label_font_id(mut self, font_id: conrod_core::text::font::Id) -> Self {
            self.style.label_font_id = Some(Some(font_id));
            self
        }

        /// If true, will allow user inputs.  If false, will disallow user inputs.  Like
        /// other Conrod configs, this returns self for chainability. Allow dead code
        /// because we never call this in the example.
        #[allow(dead_code)]
        pub fn enabled(mut self, flag: bool) -> Self {
            self.enabled = flag;
            self
        }
    }

    /// A custom Conrod widget must implement the Widget trait. See the **Widget** trait
    /// documentation for more details.
    impl<'a> Widget for CircularButton<'a> {
        /// The State struct that we defined above.
        type State = State;
        /// The Style struct that we defined using the `widget_style!` macro.
        type Style = Style;
        /// The event produced by instantiating the widget.
        ///
        /// `Some` when clicked, otherwise `None`.
        type Event = Option<()>;

        fn init_state(&self, id_gen: widget::id::Generator) -> Self::State {
            State {
                ids: Ids::new(id_gen),
            }
        }

        fn style(&self) -> Self::Style {
            self.style.clone()
        }

        /// Optionally specify a function to use for determining whether or not a point is over a
        /// widget, or if some other widget's function should be used to represent this widget.
        ///
        /// This method is optional to implement. By default, the bounding rectangle of the widget
        /// is used.
        fn is_over(&self) -> widget::IsOverFn {
            use conrod_core::graph::Container;
            use conrod_core::Theme;
            fn is_over_widget(widget: &Container, _: Point, _: &Theme) -> widget::IsOver {
                let unique = widget.state_and_style::<State, Style>().unwrap();
                unique.state.ids.circle.into()
            }
            is_over_widget
        }

        /// Update the state of the button by handling any input that has occurred since the last
        /// update.
        fn update(self, args: widget::UpdateArgs<Self>) -> Self::Event {
            let widget::UpdateArgs {
                id,
                state,
                rect,
                ui,
                style,
                ..
            } = args;

            let (color, event) = {
                let input = ui.widget_input(id);

                // If the button was clicked, produce `Some` event.
                let event = input.clicks().left().next().map(|_| ());

                let color = style.color(&ui.theme);
                let color = input.mouse().map_or(color, |mouse| {
                    if mouse.buttons.left().is_down() {
                        color.clicked()
                    } else {
                        color.highlighted()
                    }
                });

                (color, event)
            };

            // Finally, we'll describe how we want our widget drawn by simply instantiating the
            // necessary primitive graphics widgets.
            //
            // Conrod will automatically determine whether or not any changes have occurred and
            // whether or not any widgets need to be re-drawn.
            //
            // The primitive graphics widgets are special in that their unique state is used within
            // conrod's backend to do the actual drawing. This allows us to build up more complex
            // widgets by using these simple primitives with our familiar layout, coloring, etc
            // methods.
            //
            // If you notice that conrod is missing some sort of primitive graphics that you
            // require, please file an issue or open a PR so we can add it! :)

            // First, we'll draw the **Circle** with a radius that is half our given width.
            let radius = rect.w() / 2.0;
            widget::Circle::fill(radius)
                .middle_of(id)
                .graphics_for(id)
                .color(color)
                .set(state.ids.circle, ui);

            // Now we'll instantiate our label using the **Text** widget.
            if let Some(ref label) = self.maybe_label {
                let label_color = style.label_color(&ui.theme);
                let font_size = style.label_font_size(&ui.theme);
                let font_id = style.label_font_id(&ui.theme).or(ui.fonts.ids().next());
                widget::Text::new(label)
                    .and_then(font_id, widget::Text::font_id)
                    .middle_of(id)
                    .font_size(font_size)
                    .graphics_for(id)
                    .color(label_color)
                    .set(state.ids.text, ui);
            }

            event
        }
    }

    /// Provide the chainable color() configuration method.
    impl<'a> Colorable for CircularButton<'a> {
        fn color(mut self, color: conrod_core::Color) -> Self {
            self.style.color = Some(color);
            self
        }
    }

    /// Provide the chainable label(), label_color(), and label_font_size()
    /// configuration methods.
    impl<'a> Labelable<'a> for CircularButton<'a> {
        fn label(mut self, text: &'a str) -> Self {
            self.maybe_label = Some(text);
            self
        }
        fn label_color(mut self, color: conrod_core::Color) -> Self {
            self.style.label_color = Some(color);
            self
        }
        fn label_font_size(mut self, size: conrod_core::FontSize) -> Self {
            self.style.label_font_size = Some(size);
            self
        }
    }
}

fn main() {
    use self::circular_button::CircularButton;
    use conrod_core::{widget, Colorable, Labelable, Positionable, Sizeable, Widget};
    use glium::Surface;

    const WIDTH: u32 = 1200;
    const HEIGHT: u32 = 800;

    // Build the window.
    let event_loop = glium::glutin::event_loop::EventLoop::new();
    let window = glium::glutin::window::WindowBuilder::new()
        .with_title("Control Panel")
        .with_inner_size(glium::glutin::dpi::LogicalSize::new(WIDTH, HEIGHT));
    let context = glium::glutin::ContextBuilder::new()
        .with_vsync(true)
        .with_multisampling(4);
    let display = glium::Display::new(window, context, &event_loop).unwrap();

    // construct our `Ui`.
    let mut ui = conrod_core::UiBuilder::new([WIDTH as f64, HEIGHT as f64]).build();

    // The `widget_ids` macro is a easy, safe way of generating a type for producing `widget::Id`s.
    widget_ids! {
        struct Ids {
            // An ID for the background widget, upon which we'll place our custom button.
            background,
            // The WidgetId we'll use to plug our widget into the `Ui`.
            circle_button,
        }
    }
    let ids = Ids::new(ui.widget_id_generator());

    // Add a `Font` to the `Ui`'s `font::Map` from file.
    let assets = find_folder::Search::KidsThenParents(3, 5)
        .for_folder("assets")
        .unwrap();
    let font_path = assets.join("fonts/NotoSans/NotoSans-Regular.ttf");
    let regular = ui.fonts.insert_from_file(font_path).unwrap();

    // A type used for converting `conrod_core::render::Primitives` into `Command`s that can be used
    // for drawing to the glium `Surface`.
    let mut renderer = conrod_glium::Renderer::new(&display).unwrap();

    // The image map describing each of our widget->image mappings (in our case, none).
    let image_map = conrod_core::image::Map::<glium::texture::Texture2d>::new();

    // Poll events from the window.
    support::run_loop(display, event_loop, move |request, display| {
        match request {
            support::Request::Event {
                event,
                should_update_ui,
                should_exit,
            } => {
                // Use the `winit` backend feature to convert the winit event to a conrod one.
                if let Some(event) = support::convert_event(&event, &display.gl_window().window()) {
                    ui.handle_event(event);
                    *should_update_ui = true;
                }

                match event {
                    glium::glutin::event::Event::WindowEvent { event, .. } => match event {
                        // Break from the loop upon `Escape`.
                        glium::glutin::event::WindowEvent::CloseRequested
                        | glium::glutin::event::WindowEvent::KeyboardInput {
                            input:
                                glium::glutin::event::KeyboardInput {
                                    virtual_keycode:
                                        Some(glium::glutin::event::VirtualKeyCode::Escape),
                                    ..
                                },
                            ..
                        } => *should_exit = true,
                        _ => {}
                    },
                    _ => {}
                }
            }
            support::Request::SetUi { needs_redraw } => {
                // Instantiate the widgets.
                let ui = &mut ui.set_widgets();

                // Sets a color to clear the background with before the Ui draws our widget.
                widget::Canvas::new()
                    .color(conrod_core::color::DARK_RED)
                    .set(ids.background, ui);

                // Instantiate of our custom widget.
                for _click in CircularButton::new()
                    .color(conrod_core::color::rgb(0.0, 0.3, 0.1))
                    .middle_of(ids.background)
                    .w_h(256.0, 256.0)
                    .label_font_id(regular)
                    .label_color(conrod_core::color::WHITE)
                    .label("Circular Button")
                    // Add the widget to the conrod_core::Ui. This schedules the widget it to be
                    // drawn when we call Ui::draw.
                    .set(ids.circle_button, ui)
                {
                    println!("Click!");
                }

                *needs_redraw = ui.has_changed();
            }
            support::Request::Redraw => {
                // Render the `Ui` and then display it on the screen.
                let primitives = ui.draw();

                renderer.fill(display, primitives, &image_map);
                let mut target = display.draw();
                target.clear_color(0.0, 0.0, 0.0, 1.0);
                renderer.draw(display, &mut target, &image_map).unwrap();
                target.finish().unwrap();
            }
        }
    })
}
