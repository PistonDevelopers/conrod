#[macro_use]
extern crate conrod_core;
extern crate glium;

extern crate conrod_glium;
extern crate find_folder;
#[macro_use]
extern crate conrod_winit;

mod support;

use glium::Surface;

const WIDTH: u32 = 720;
const HEIGHT: u32 = 360;

widget_ids! {
    struct Ids { canvas, grid, plot }
}

fn main() {
    // Build the window.
    let event_loop = glium::glutin::event_loop::EventLoop::new();
    let window = glium::glutin::window::WindowBuilder::new()
        .with_title("PlotPath Demo")
        .with_inner_size(glium::glutin::dpi::LogicalSize::new(WIDTH, HEIGHT));
    let context = glium::glutin::ContextBuilder::new()
        .with_vsync(true)
        .with_multisampling(4);
    let display = glium::Display::new(window, context, &event_loop).unwrap();

    // Construct our `Ui`.
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
                // Instantiate the widgets.
                use conrod_core::{color, widget, Colorable, Positionable, Sizeable, Widget};

                let ui = &mut ui.set_widgets();

                widget::Canvas::new()
                    .color(color::DARK_CHARCOAL)
                    .set(ids.canvas, ui);

                let min_x = 0.0;
                let max_x = std::f64::consts::PI * 2.0;
                let min_y = -1.0;
                let max_y = 1.0;

                let quarter_lines = widget::grid::Lines::step(0.5_f64).thickness(2.0);
                let sixteenth_lines = widget::grid::Lines::step(0.125_f64).thickness(1.0);
                let lines = &[
                    quarter_lines.x(),
                    quarter_lines.y(),
                    sixteenth_lines.x(),
                    sixteenth_lines.y(),
                ];

                widget::Grid::new(min_x, max_x, min_y, max_y, lines.iter().cloned())
                    .color(color::rgb(0.1, 0.12, 0.15))
                    .wh_of(ids.canvas)
                    .middle_of(ids.canvas)
                    .set(ids.grid, ui);
                widget::PlotPath::new(min_x, max_x, min_y, max_y, f64::sin)
                    .color(color::LIGHT_BLUE)
                    .thickness(2.0)
                    .wh_of(ids.canvas)
                    .middle_of(ids.canvas)
                    .set(ids.plot, ui);
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
