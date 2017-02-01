#[macro_use] extern crate conrod;

use conrod::backend::glium::glium;
use conrod::backend::glium::glium::{DisplayBuild, Surface};

use std::time;

const FONT: &'static str = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/fonts/NotoSans/NotoSans-Bold.ttf");

pub struct EventLoop {
    ui_needs_update: bool,
    last_update: time::Instant,
}

impl EventLoop {
    pub fn new() -> Self {
        EventLoop {
            last_update: time::Instant::now(),
            ui_needs_update: false,
        }
    }

    /// Produce an iterator yielding all available events.
    pub fn next(&mut self, display: &glium::Display) -> Vec<glium::glutin::Event> {
        // We don't want to loop any faster than 60 FPS, so wait until it has been at least 16ms
        // since the last yield.
        let last_update = self.last_update;
        let frame = time::Duration::from_millis(16);
        let dt = time::Instant::now().duration_since(last_update);

        if dt < frame {
            std::thread::sleep(frame - dt);
        }

        // Collect all pending events.
        let mut events = Vec::new();
        events.extend(display.poll_events());

        // If there are no events and the `Ui` does not need updating, wait for the next event.
        if events.is_empty() && !self.ui_needs_update {
            events.extend(display.wait_events().next());
        }

        self.ui_needs_update = false;
        self.last_update = time::Instant::now();

        events
    }

    /// Notifies the event loop that the `Ui` requires another update whether or not there are any
    /// pending events.
    ///
    /// This is primarily used on the occasion that some part of the `Ui` is still animating and
    /// requires further updates to do so.
    pub fn needs_update(&mut self) {
        self.ui_needs_update = true;
    }

}

widget_ids! {
    struct Ids {
        hello,
    }
}

fn set_ui(ui: &mut conrod::UiCell, ids: &Ids) {
    use conrod::{color, widget, Widget, Scalar, Colorable, Positionable};

    const PAD: Scalar = 20.0;
    widget::Text::new("Hello world!")
    .color(color::WHITE)
    .center_justify()
    .set(ids.hello, ui)
}

fn main() {
    let display = glium::glutin::WindowBuilder::new()
        .with_vsync()
        .with_dimensions(640, 480)
        .with_title("Conrod with glium!")
        .build_glium()
        .unwrap();
    let mut ui = conrod::UiBuilder::new([640.0, 480.0 as f64]).build();
    let ids = Ids::new(ui.widget_id_generator());

    let font = ui.fonts.insert_from_file(FONT).unwrap();
    ui.theme.font_id = Some(font);

    let image_map = conrod::image::Map::<glium::texture::Texture2d>::new();
    let mut renderer = conrod::backend::glium::Renderer::new(&display).unwrap();
    let mut ev_loop = EventLoop::new();

    loop {
        for event in ev_loop.next(&display) {
            if let Some(e) = conrod::backend::winit::convert(event.clone(), &display) {
                ui.handle_event(e);
                ev_loop.needs_update();
            }

            match event {
                // Break from the loop upon `Escape`.
                glium::glutin::Event::KeyboardInput(_, _, Some(glium::glutin::VirtualKeyCode::Escape)) |
                glium::glutin::Event::Closed =>
                    return,
                _ => {},
            }
        }

        set_ui(&mut ui.set_widgets(), &ids);

        if let Some(prims) = ui.draw_if_changed() {
            renderer.fill(&display, prims, &image_map);
            
            let mut target = display.draw();
            target.clear_color(0.0, 0.0, 0.0, 1.0);
            renderer.draw(&display, &mut target, &image_map).unwrap();
            target.finish().unwrap();
        }
    }
}
