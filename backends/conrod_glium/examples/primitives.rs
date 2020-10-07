#[macro_use]
extern crate conrod_core;
extern crate conrod_glium;
#[macro_use]
extern crate conrod_winit;
extern crate find_folder;
extern crate glium;

mod support;

use glium::Surface;

// Generate a type that will produce a unique `widget::Id` for each widget.
widget_ids! {
    struct Ids {
        canvas,
        line,
        point_path,
        rectangle_fill,
        rectangle_outline,
        trapezoid,
        oval_fill,
        oval_outline,
        circle,
    }
}

fn main() {
    const WIDTH: u32 = 400;
    const HEIGHT: u32 = 720;

    // Build the window.
    let event_loop = glium::glutin::event_loop::EventLoop::new();
    let window = glium::glutin::window::WindowBuilder::new()
        .with_title("Primitive Widgets Demo")
        .with_inner_size(glium::glutin::dpi::LogicalSize::new(WIDTH, HEIGHT));
    let context = glium::glutin::ContextBuilder::new()
        .with_vsync(true)
        .with_multisampling(4);
    let display = glium::Display::new(window, context, &event_loop).unwrap();

    // construct our `Ui`.
    let mut ui = conrod_core::UiBuilder::new([WIDTH as f64, HEIGHT as f64]).build();

    // A unique identifier for each widget.
    let ids = Ids::new(ui.widget_id_generator());

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
                set_ui(ui.set_widgets(), &ids);

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

fn set_ui(ref mut ui: conrod_core::UiCell, ids: &Ids) {
    use conrod_core::widget::{Canvas, Circle, Line, Oval, PointPath, Polygon, Rectangle};
    use conrod_core::{Positionable, Widget};
    use std::iter::once;

    // The background canvas upon which we'll place our widgets.
    Canvas::new().pad(80.0).set(ids.canvas, ui);

    Line::centred([-40.0, -40.0], [40.0, 40.0])
        .top_left_of(ids.canvas)
        .set(ids.line, ui);

    let left = [-40.0, -40.0];
    let top = [0.0, 40.0];
    let right = [40.0, -40.0];
    let points = once(left).chain(once(top)).chain(once(right));
    PointPath::centred(points)
        .down(80.0)
        .set(ids.point_path, ui);

    Rectangle::fill([80.0, 80.0])
        .down(80.0)
        .set(ids.rectangle_fill, ui);

    Rectangle::outline([80.0, 80.0])
        .down(80.0)
        .set(ids.rectangle_outline, ui);

    let bl = [-40.0, -40.0];
    let tl = [-20.0, 40.0];
    let tr = [20.0, 40.0];
    let br = [40.0, -40.0];
    let points = once(bl).chain(once(tl)).chain(once(tr)).chain(once(br));
    Polygon::centred_fill(points)
        .right_from(ids.line, 80.0)
        .set(ids.trapezoid, ui);

    Oval::fill([40.0, 80.0])
        .down(80.0)
        .align_middle_x()
        .set(ids.oval_fill, ui);

    Oval::outline([80.0, 40.0])
        .down(100.0)
        .align_middle_x()
        .set(ids.oval_outline, ui);

    Circle::fill(40.0)
        .down(100.0)
        .align_middle_x()
        .set(ids.circle, ui);
}
