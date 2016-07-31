//! A demonstration of designing a custom, third-party widget.
//!
//! In this case, we'll design a simple circular button.
//!
//! All of the custom widget design will occur within the `circular_button` module.
//!
//! We'll *use* our fancy circular button in the `main` function (below the circular_button module).
//!
//! Note that in this case, we use `piston_window` to draw our widget, however in practise you may
//! use any backend you wish.
//!
//! For more information, please see the `Widget` trait documentation.

#[macro_use] extern crate conrod;
extern crate find_folder;
extern crate piston_window;


/// The module in which we'll implement our own custom circular button.
mod circular_button {
    use conrod::{self, Colorable, Dimensions, Labelable, Point, Positionable, Widget};

    /// The type upon which we'll implement the `Widget` trait.
    pub struct CircularButton<'a, F> {
        /// An object that handles some of the dirty work of rendering a GUI. We don't
        /// really have to worry about it.
        common: conrod::CommonBuilder,
        /// Optional label string for the button.
        maybe_label: Option<&'a str>,
        /// Optional callback for when the button is pressed. If you want the button to
        /// do anything, this callback must exist.
        maybe_react: Option<F>,
        /// See the Style struct below.
        style: Style,
        /// Whether the button is currently enabled, i.e. whether it responds to
        /// user input.
        enabled: bool
    }

    // We use the `widget_style!` macro to vastly simplify the definition and implementation of the
    // widget's associated `Style` type. This generates both a `Style` struct, as well as an
    // implementation that automatically retrieves defaults from the provided theme.
    //
    // See the documenation of the macro for a more details.
    widget_style!{
        /// Represents the unique styling for our CircularButton widget.
        style Style {
            /// Color of the button.
            - color: conrod::Color { theme.shape_color }
            /// Color of the button's label.
            - label_color: conrod::Color { theme.label_color }
            /// Font size of the button's label.
            - label_font_size: conrod::FontSize { theme.font_size_medium }
        }
    }

    /// Represents the unique, cached state for our CircularButton widget.
    #[derive(Clone, Debug, PartialEq)]
    pub struct State {
        /// An index to use for our **Circle** primitive graphics widget.
        circle_idx: conrod::IndexSlot,
        /// An index to use for our **Text** primitive graphics widget (for the label).
        text_idx: conrod::IndexSlot,
    }

    /// Return whether or not a given point is over a circle at a given point on a
    /// Cartesian plane. We use this to determine whether the mouse is over the button.
    pub fn is_over_circ(circ_center: Point, mouse_point: Point, dim: Dimensions) -> bool {
        // Offset vector from the center of the circle to the mouse.
        let offset = conrod::utils::vec2_sub(mouse_point, circ_center);

        // If the length of the offset vector is less than or equal to the circle's
        // radius, then the mouse is inside the circle. We assume that dim is a square
        // bounding box around the circle, thus 2 * radius == dim[0] == dim[1].
        let distance = (offset[0].powf(2.0) + offset[1].powf(2.0)).sqrt();
        let radius = dim[0] / 2.0;
        distance <= radius
    }

    impl<'a, F> CircularButton<'a, F> {
        /// Create a button context to be built upon.
        pub fn new() -> CircularButton<'a, F> {
            CircularButton {
                common: conrod::CommonBuilder::new(),
                maybe_react: None,
                maybe_label: None,
                style: Style::new(),
                enabled: true,
            }
        }

        /// Set the reaction for the Button. The reaction will be triggered upon release
        /// of the button. Like other Conrod configs, this returns self for chainability.
        pub fn react(mut self, reaction: F) -> Self {
            self.maybe_react = Some(reaction);
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
    impl<'a, F> Widget for CircularButton<'a, F>
        where F: FnMut()
    {
        /// The State struct that we defined above.
        type State = State;
        /// The Style struct that we defined using the `widget_style!` macro.
        type Style = Style;

        fn common(&self) -> &conrod::CommonBuilder {
            &self.common
        }

        fn common_mut(&mut self) -> &mut conrod::CommonBuilder {
            &mut self.common
        }

        fn init_state(&self) -> State {
            State {
                circle_idx: conrod::IndexSlot::new(),
                text_idx: conrod::IndexSlot::new(),
            }
        }

        fn style(&self) -> Style {
            self.style.clone()
        }

        /// Update the state of the button by handling any input that has occurred since the last
        /// update.
        fn update(self, args: conrod::UpdateArgs<Self>) {
            let conrod::UpdateArgs { idx, state, rect, mut ui, style, .. } = args;

            let color = {
                let input = ui.widget_input(idx);

                // If the button was clicked, call the user's `react` function.
                if input.clicks().left().next().is_some() {
                    if let Some(mut react) = self.maybe_react {
                        react();
                    }
                }

                let color = style.color(ui.theme());
                input.mouse()
                    .map(|mouse| {
                        if is_over_circ([0.0, 0.0], mouse.rel_xy(), rect.dim()) {
                            if mouse.buttons.left().is_down() {
                                color.clicked()
                            } else {
                                color.highlighted()
                            }
                        } else {
                            color
                        }
                    })
                    .unwrap_or(color)
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
            let circle_idx = state.circle_idx.get(&mut ui);
            conrod::Circle::fill(radius)
                .middle_of(idx)
                .graphics_for(idx)
                .color(color)
                .set(circle_idx, &mut ui);

            // Now we'll instantiate our label using the **Text** widget.
            let label_color = style.label_color(ui.theme());
            let font_size = style.label_font_size(ui.theme());
            let text_idx = state.text_idx.get(&mut ui);
            if let Some(ref label) = self.maybe_label {
                conrod::Text::new(label)
                    .middle_of(idx)
                    .font_size(font_size)
                    .graphics_for(idx)
                    .color(label_color)
                    .set(text_idx, &mut ui);
            }
        }

    }

    /// Provide the chainable color() configuration method.
    impl<'a, F> Colorable for CircularButton<'a, F> {
        fn color(mut self, color: conrod::Color) -> Self {
            self.style.color = Some(color);
            self
        }
    }

    /// Provide the chainable label(), label_color(), and label_font_size()
    /// configuration methods.
    impl<'a, F> Labelable<'a> for CircularButton<'a, F> {
        fn label(mut self, text: &'a str) -> Self {
            self.maybe_label = Some(text);
            self
        }
        fn label_color(mut self, color: conrod::Color) -> Self {
            self.style.label_color = Some(color);
            self
        }
        fn label_font_size(mut self, size: conrod::FontSize) -> Self {
            self.style.label_font_size = Some(size);
            self
        }
    }
}

pub fn main() {
    use conrod::{self, Colorable, Labelable, Positionable, Sizeable, Widget};
    use piston_window::{EventLoop, PistonWindow, OpenGL, UpdateEvent, WindowSettings};
    use self::circular_button::CircularButton;

    const WIDTH: u32 = 1200;
    const HEIGHT: u32 = 800;

    // Change this to OpenGL::V2_1 if not working.
    let opengl = OpenGL::V3_2;

    // PistonWindow has two type parameters, but the default type is
    // PistonWindow<T = (), W: Window = GlutinWindow>. To change the Piston backend,
    // specify a different type in the let binding, e.g.
    // let window: PistonWindow<(), Sdl2Window>.
    let mut window: PistonWindow = WindowSettings::new("Control Panel", [WIDTH, HEIGHT])
        .opengl(opengl)
        .exit_on_esc(true)
        .build().unwrap();
    window.set_ups(60);

    // construct our `Ui`.
    let mut ui = conrod::Ui::new(conrod::Theme::default());

    // Add a `Font` to the `Ui`'s `font::Map` from file.
    let assets = find_folder::Search::KidsThenParents(3, 5).for_folder("assets").unwrap();
    let font_path = assets.join("fonts/NotoSans/NotoSans-Regular.ttf");
    ui.fonts.insert_from_file(font_path).unwrap();

    // Create a texture to use for efficiently caching text on the GPU.
    let mut text_texture_cache =
        conrod::backend::piston_window::GlyphCache::new(&mut window, WIDTH, HEIGHT);

    // The image map describing each of our widget->image mappings (in our case, none).
    let image_map = conrod::image::Map::new();

    while let Some(event) = window.next() {

        // Convert the piston event to a conrod event.
        if let Some(e) = conrod::backend::piston_window::convert_event(event.clone(), &window) {
            ui.handle_event(e);
        }

        event.update(|_| ui.set_widgets(|ref mut ui| {

            // Sets a color to clear the background with before the Ui draws our widget.
            conrod::Canvas::new().color(conrod::color::DARK_RED).set(BACKGROUND, ui);

            // The `widget_ids` macro is a easy, safe way of generating unique `WidgetId`s.
            widget_ids! {
                // An ID for the background widget, upon which we'll place our custom button.
                BACKGROUND,
                // The WidgetId we'll use to plug our widget into the `Ui`.
                CIRCLE_BUTTON,
            }

            // Create an instance of our custom widget.
            CircularButton::new()
                .color(conrod::color::rgb(0.0, 0.3, 0.1))
                .middle_of(BACKGROUND)
                .w_h(256.0, 256.0)
                .label_color(conrod::color::WHITE)
                .label("Circular Button")
                // This is called when the user clicks the button.
                .react(|| println!("Click"))
                // Add the widget to the conrod::Ui. This schedules the widget it to be
                // drawn when we call Ui::draw.
                .set(CIRCLE_BUTTON, ui);
        }));

        // Draws the whole Ui (in this case, just our widget) whenever a change occurs.
        window.draw_2d(&event, |c, g| {
            if let Some(primitives) = ui.draw_if_changed(&image_map) {
                fn texture_from_image<T>(img: &T) -> &T { img };
                conrod::backend::piston_window::draw(c, g, primitives,
                                                     &mut text_texture_cache,
                                                     texture_from_image);
            }
        });
    }
}
