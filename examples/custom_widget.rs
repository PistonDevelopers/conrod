//! A demonstration of designing a custom, third-party widget.
//!
//! In this case, we'll design a simple circular button.
//!
//! All of the custom widget design will occur within the `circular_button` module.
//!
//! We'll *use* our fancy circular button in the `main` function (below the circular_button module).
//!
//! Note that in this case, we use `backend::piston` to draw our widget, however in practise you may
//! use any backend you wish.
//!
//! For more information, please see the `Widget` trait documentation.

#[macro_use] extern crate conrod;
extern crate find_folder;


/// The module in which we'll implement our own custom circular button.
mod circular_button {
    use conrod::{self, widget, Colorable, Dimensions, Labelable, Point, Positionable, Widget};

    /// The type upon which we'll implement the `Widget` trait.
    pub struct CircularButton<'a> {
        /// An object that handles some of the dirty work of rendering a GUI. We don't
        /// really have to worry about it.
        common: widget::CommonBuilder,
        /// Optional label string for the button.
        maybe_label: Option<&'a str>,
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
            /// Specify a unique font for the label.
            - label_font_id: Option<conrod::text::font::Id> { theme.font_id }
        }
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

    impl<'a> CircularButton<'a> {

        /// Create a button context to be built upon.
        pub fn new() -> Self {
            CircularButton {
                common: widget::CommonBuilder::new(),
                maybe_label: None,
                style: Style::new(),
                enabled: true,
            }
        }

        /// Specify the font used for displaying the label.
        pub fn label_font_id(mut self, font_id: conrod::text::font::Id) -> Self {
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

        fn common(&self) -> &widget::CommonBuilder {
            &self.common
        }

        fn common_mut(&mut self) -> &mut widget::CommonBuilder {
            &mut self.common
        }

        fn init_state(&self, id_gen: widget::id::Generator) -> Self::State {
            State { ids: Ids::new(id_gen) }
        }

        fn style(&self) -> Self::Style {
            self.style.clone()
        }

        /// Update the state of the button by handling any input that has occurred since the last
        /// update.
        fn update(self, args: widget::UpdateArgs<Self>) -> Self::Event {
            let widget::UpdateArgs { id, state, rect, mut ui, style, .. } = args;

            let (color, event) = {
                let input = ui.widget_input(id);

                // If the button was clicked, produce `Some` event.
                let event = input.clicks().left().next().map(|_| ());

                let color = style.color(&ui.theme);
                let color = input.mouse().map_or(color, |mouse| {
                    if is_over_circ([0.0, 0.0], mouse.rel_xy(), rect.dim()) {
                        if mouse.buttons.left().is_down() {
                            color.clicked()
                        } else {
                            color.highlighted()
                        }
                    } else {
                        color
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
        fn color(mut self, color: conrod::Color) -> Self {
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
    use conrod::{self, widget, Colorable, Labelable, Positionable, Sizeable, Widget};
    use conrod::backend::piston::{self, Window, WindowEvents, OpenGL};
    use conrod::backend::piston::event::UpdateEvent;

    use self::circular_button::CircularButton;

    const WIDTH: u32 = 1200;
    const HEIGHT: u32 = 800;

    // Change this to OpenGL::V2_1 if not working.
    let opengl = OpenGL::V3_2;

    // PistonWindow has two type parameters, but the default type is
    // PistonWindow<T = (), W: Window = GlutinWindow>. To change the Piston backend,
    // specify a different type in the let binding, e.g.
    // let window: PistonWindow<(), Sdl2Window>.
    let mut window: Window =
        piston::window::WindowSettings::new("Control Panel", [WIDTH, HEIGHT])
            .opengl(opengl)
            .exit_on_esc(true)
            .build().unwrap();

    // Create the event loop.
    let mut events = WindowEvents::new();

    // construct our `Ui`.
    let mut ui = conrod::UiBuilder::new([WIDTH as f64, HEIGHT as f64]).build();

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
    let assets = find_folder::Search::KidsThenParents(3, 5).for_folder("assets").unwrap();
    let font_path = assets.join("fonts/NotoSans/NotoSans-Regular.ttf");
    let regular = ui.fonts.insert_from_file(font_path).unwrap();

    // Create a texture to use for efficiently caching text on the GPU.
    let mut text_texture_cache = piston::window::GlyphCache::new(&mut window, WIDTH, HEIGHT);

    // The image map describing each of our widget->image mappings (in our case, none).
    let image_map = conrod::image::Map::new();

    while let Some(event) = window.next_event(&mut events) {

        // Convert the piston event to a conrod event.
        if let Some(e) = piston::window::convert_event(event.clone(), &window) {
            ui.handle_event(e);
        }

        event.update(|_| {
            let ui = &mut ui.set_widgets();

            // Sets a color to clear the background with before the Ui draws our widget.
            widget::Canvas::new().color(conrod::color::DARK_RED).set(ids.background, ui);

            // Instantiate of our custom widget.
            for _click in CircularButton::new()
                .color(conrod::color::rgb(0.0, 0.3, 0.1))
                .middle_of(ids.background)
                .w_h(256.0, 256.0)
                .label_font_id(regular)
                .label_color(conrod::color::WHITE)
                .label("Circular Button")
                // Add the widget to the conrod::Ui. This schedules the widget it to be
                // drawn when we call Ui::draw.
                .set(ids.circle_button, ui)
            {
                println!("Click!");
            }
        });

        // Draws the whole Ui (in this case, just our widget) whenever a change occurs.
        window.draw_2d(&event, |c, g| {
            if let Some(primitives) = ui.draw_if_changed() {
                fn texture_from_image<T>(img: &T) -> &T { img };
                piston::window::draw(c, g, primitives,
                                     &mut text_texture_cache,
                                     &image_map,
                                     texture_from_image);
            }
        });
    }
}
