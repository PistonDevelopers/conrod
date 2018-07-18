#[cfg(all(feature="winit", feature="glium"))] #[macro_use] extern crate conrod;
#[cfg(all(feature="winit", feature="glium"))] mod support;

fn main() {
    feature::main();
}

#[cfg(all(feature="winit", feature="glium"))]
mod feature {
    extern crate find_folder;
    use conrod;
    use conrod::backend::glium::glium;
    use conrod::backend::glium::glium::Surface;
    use support;

    struct Fonts {
        regular: conrod::text::font::Id,
        italic: conrod::text::font::Id,
        bold: conrod::text::font::Id,
    }

    pub fn main() {
        const WIDTH: u32 = 1080;
        const HEIGHT: u32 = 720;

        // Build the window.
        let mut events_loop = glium::glutin::EventsLoop::new();
        let window = glium::glutin::WindowBuilder::new()
            .with_title("Text Demo")
            .with_dimensions((WIDTH, HEIGHT).into());
        let context = glium::glutin::ContextBuilder::new()
            .with_vsync(true)
            .with_multisampling(4);
        let display = glium::Display::new(window, context, &events_loop).unwrap();

        // Construct our `Ui`.
        let mut ui = conrod::UiBuilder::new([WIDTH as f64, HEIGHT as f64]).build();

        // A unique identifier for each widget.
        let ids = Ids::new(ui.widget_id_generator());

        // Add a `Font` to the `Ui`'s `font::Map` from file.
        let assets = find_folder::Search::KidsThenParents(3, 5).for_folder("assets").unwrap();
        let noto_sans = assets.join("fonts/NotoSans");
        // Store our `font::Id`s in a struct for easy access in the `set_ui` function.
        let fonts = Fonts {
            regular: ui.fonts.insert_from_file(noto_sans.join("NotoSans-Regular.ttf")).unwrap(),
            italic: ui.fonts.insert_from_file(noto_sans.join("NotoSans-Italic.ttf")).unwrap(),
            bold: ui.fonts.insert_from_file(noto_sans.join("NotoSans-Bold.ttf")).unwrap(),
        };

        // Specify the default font to use when none is specified by the widget.
        //
        // By default, the theme's font_id field is `None`. In this case, the first font that is found
        // within the `Ui`'s `font::Map` will be used.
        ui.theme.font_id = Some(fonts.regular);

        // A type used for converting `conrod::render::Primitives` into `Command`s that can be used
        // for drawing to the glium `Surface`.
        let mut renderer = conrod::backend::glium::Renderer::new(&display).unwrap();

        // The image map describing each of our widget->image mappings (in our case, none).
        let image_map = conrod::image::Map::<glium::texture::Texture2d>::new();

        // Poll events from the window.
        let mut event_loop = support::EventLoop::new();
        'main: loop {

            // Handle all events.
            for event in event_loop.next(&mut events_loop) {

                // Use the `winit` backend feature to convert the winit event to a conrod one.
                if let Some(event) = conrod::backend::winit::convert_event(event.clone(), &display) {
                    ui.handle_event(event);
                }

                match event {
                    glium::glutin::Event::WindowEvent { event, .. } => match event {
                        // Break from the loop upon `Escape`.
                        glium::glutin::WindowEvent::CloseRequested |
                        glium::glutin::WindowEvent::KeyboardInput {
                            input: glium::glutin::KeyboardInput {
                                virtual_keycode: Some(glium::glutin::VirtualKeyCode::Escape),
                                ..
                            },
                            ..
                        } => break 'main,
                        _ => (),
                    },
                    _ => (),
                }
            }

            set_ui(ui.set_widgets(), &ids, &fonts);

            // Render the `Ui` and then display it on the screen.
            if let Some(primitives) = ui.draw_if_changed() {
                renderer.fill(&display, primitives, &image_map);
                let mut target = display.draw();
                target.clear_color(0.0, 0.0, 0.0, 1.0);
                renderer.draw(&display, &mut target, &image_map).unwrap();
                target.finish().unwrap();
            }
        }
    }

    // Generate a unique const `WidgetId` for each widget.
    widget_ids!{
        struct Ids {
            master,
            left_col,
            middle_col,
            right_col,
            left_text,
            middle_text,
            right_text,
        }
    }

    fn set_ui(ref mut ui: conrod::UiCell, ids: &Ids, fonts: &Fonts) {
        use conrod::{color, widget, Colorable, Positionable, Scalar, Sizeable, Widget};

        // Our `Canvas` tree, upon which we will place our text widgets.
        widget::Canvas::new().flow_right(&[
            (ids.left_col, widget::Canvas::new().color(color::BLACK)),
            (ids.middle_col, widget::Canvas::new().color(color::DARK_CHARCOAL)),
            (ids.right_col, widget::Canvas::new().color(color::CHARCOAL)),
        ]).set(ids.master, ui);

        const DEMO_TEXT: &'static str = "Lorem ipsum dolor sit amet, consectetur adipiscing elit. \
            Mauris aliquet porttitor tellus vel euismod. Integer lobortis volutpat bibendum. Nulla \
            finibus odio nec elit condimentum, rhoncus fermentum purus lacinia. Interdum et malesuada \
            fames ac ante ipsum primis in faucibus. Cras rhoncus nisi nec dolor bibendum pellentesque. \
            Cum sociis natoque penatibus et magnis dis parturient montes, nascetur ridiculus mus. \
            Quisque commodo nibh hendrerit nunc sollicitudin sodales. Cras vitae tempus ipsum. Nam \
            magna est, efficitur suscipit dolor eu, consectetur consectetur urna.";

        const PAD: Scalar = 20.0;

        widget::Text::new(DEMO_TEXT)
            .font_id(fonts.regular)
            .color(color::LIGHT_RED)
            .padded_w_of(ids.left_col, PAD)
            .mid_top_with_margin_on(ids.left_col, PAD)
            .left_justify()
            .line_spacing(10.0)
            .set(ids.left_text, ui);

        widget::Text::new(DEMO_TEXT)
            .font_id(fonts.italic)
            .color(color::LIGHT_GREEN)
            .padded_w_of(ids.middle_col, PAD)
            .middle_of(ids.middle_col)
            .center_justify()
            .line_spacing(2.5)
            .set(ids.middle_text, ui);

        widget::Text::new(DEMO_TEXT)
            .font_id(fonts.bold)
            .color(color::LIGHT_BLUE)
            .padded_w_of(ids.right_col, PAD)
            .mid_bottom_with_margin_on(ids.right_col, PAD)
            .right_justify()
            .line_spacing(5.0)
            .set(ids.right_text, ui);
    }
}

#[cfg(not(all(feature="winit", feature="glium")))]
mod feature {
    pub fn main() {
        println!("This example requires the `winit` and `glium` features. \
                 Try running `cargo run --release --features=\"winit glium\" --example <example_name>`");
    }
}
