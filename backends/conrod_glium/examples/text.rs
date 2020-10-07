#[macro_use]
extern crate conrod_core;
extern crate conrod_glium;
extern crate conrod_winit;
extern crate find_folder;
extern crate glium;

mod support;

use glium::Surface;

struct Fonts {
    regular: conrod_core::text::font::Id,
    italic: conrod_core::text::font::Id,
    bold: conrod_core::text::font::Id,
}

fn main() {
    const WIDTH: u32 = 1080;
    const HEIGHT: u32 = 720;

    // Build the window.
    let event_loop = glium::glutin::event_loop::EventLoop::new();
    let window = glium::glutin::window::WindowBuilder::new()
        .with_title("Text Demo")
        .with_inner_size(glium::glutin::dpi::LogicalSize::new(WIDTH, HEIGHT));
    let context = glium::glutin::ContextBuilder::new()
        .with_vsync(true)
        .with_multisampling(4);
    let display = glium::Display::new(window, context, &event_loop).unwrap();

    // Construct our `Ui`.
    let mut ui = conrod_core::UiBuilder::new([WIDTH as f64, HEIGHT as f64]).build();

    // A unique identifier for each widget.
    let ids = Ids::new(ui.widget_id_generator());

    // Add a `Font` to the `Ui`'s `font::Map` from file.
    let assets = find_folder::Search::KidsThenParents(3, 5)
        .for_folder("assets")
        .unwrap();
    let noto_sans = assets.join("fonts/NotoSans");
    // Store our `font::Id`s in a struct for easy access in the `set_ui` function.
    let fonts = Fonts {
        regular: ui
            .fonts
            .insert_from_file(noto_sans.join("NotoSans-Regular.ttf"))
            .unwrap(),
        italic: ui
            .fonts
            .insert_from_file(noto_sans.join("NotoSans-Italic.ttf"))
            .unwrap(),
        bold: ui
            .fonts
            .insert_from_file(noto_sans.join("NotoSans-Bold.ttf"))
            .unwrap(),
    };

    // Specify the default font to use when none is specified by the widget.
    //
    // By default, the theme's font_id field is `None`. In this case, the first font that is found
    // within the `Ui`'s `font::Map` will be used.
    ui.theme.font_id = Some(fonts.regular);

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
                set_ui(ui.set_widgets(), &ids, &fonts);

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

// Generate a unique const `WidgetId` for each widget.
widget_ids! {
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

fn set_ui(ref mut ui: conrod_core::UiCell, ids: &Ids, fonts: &Fonts) {
    use conrod_core::{color, widget, Colorable, Positionable, Scalar, Sizeable, Widget};

    // Our `Canvas` tree, upon which we will place our text widgets.
    widget::Canvas::new()
        .flow_right(&[
            (ids.left_col, widget::Canvas::new().color(color::BLACK)),
            (
                ids.middle_col,
                widget::Canvas::new().color(color::DARK_CHARCOAL),
            ),
            (ids.right_col, widget::Canvas::new().color(color::CHARCOAL)),
        ])
        .set(ids.master, ui);

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
