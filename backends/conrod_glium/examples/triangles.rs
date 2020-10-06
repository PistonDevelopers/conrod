//! A simple example demonstrating the `Triangles` widget.

#[macro_use]
extern crate conrod_core;
extern crate conrod_glium;
extern crate glium;
#[macro_use]
extern crate conrod_winit;
extern crate find_folder;

mod support;

use conrod_core::widget::triangles::Triangle;
use conrod_core::{color, widget, Widget};
use glium::Surface;

fn main() {
    const WIDTH: u32 = 700;
    const HEIGHT: u32 = 400;

    // Build the window.
    let event_loop = glium::glutin::event_loop::EventLoop::new();
    let window = glium::glutin::window::WindowBuilder::new()
        .with_title("Triangles!")
        .with_inner_size(glium::glutin::dpi::LogicalSize::new(WIDTH, HEIGHT));
    let context = glium::glutin::ContextBuilder::new()
        .with_vsync(true)
        .with_multisampling(4);
    let display = glium::Display::new(window, context, &event_loop).unwrap();

    // construct our `Ui`.
    let mut ui = conrod_core::UiBuilder::new([WIDTH as f64, HEIGHT as f64]).build();

    // Generate the widget identifiers.
    widget_ids!(struct Ids { triangles });
    let ids = Ids::new(ui.widget_id_generator());

    // Add a `Font` to the `Ui`'s `font::Map` from file.
    let assets = find_folder::Search::KidsThenParents(3, 5)
        .for_folder("assets")
        .unwrap();
    let font_path = assets.join("fonts/NotoSans/NotoSans-Regular.ttf");
    ui.fonts.insert_from_file(font_path).unwrap();

    // A type used for converting `conrod_core::render::Primitives` into `Command`s that can be used
    // for drawing to the glium `Surface`.
    let mut renderer = conrod_glium::Renderer::new(&display).unwrap();

    // The image map describing each of our widget->image mappings (in our case, none).
    let image_map = conrod_core::image::Map::<glium::texture::Texture2d>::new();

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
                // Set the triangle widget.
                let ui = &mut ui.set_widgets();
                let rect = ui.rect_of(ui.window).unwrap();
                let (l, r, b, t) = rect.l_r_b_t();
                let (c1, c2, c3) = (
                    color::RED.to_rgb(),
                    color::GREEN.to_rgb(),
                    color::BLUE.to_rgb(),
                );

                let triangles = [
                    Triangle([([l, b], c1), ([l, t], c2), ([r, t], c3)]),
                    Triangle([([r, t], c1), ([r, b], c2), ([l, b], c3)]),
                ];

                widget::Triangles::multi_color(triangles.iter().cloned())
                    .with_bounding_rect(rect)
                    .set(ids.triangles, ui);

                *needs_redraw = ui.has_changed();
            }
            support::Request::Redraw => {
                // Draw the `Ui` if it has changed.
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
