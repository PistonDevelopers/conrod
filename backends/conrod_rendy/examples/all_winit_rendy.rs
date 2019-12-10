use conrod_core::Ui;
use conrod_example_shared::{WIN_H, WIN_W};
use conrod_rendy::Renderer;
use rendy::init::winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

const CLEAR_COLOR: [f32; 4] = [0.2, 0.2, 0.2, 1.0];

fn main() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_inner_size((WIN_W, WIN_H).into())
        .with_title("Conrod with Rendy and winit");

    let mut ui = conrod_core::UiBuilder::new([WIN_W as f64, WIN_H as f64])
        .theme(conrod_example_shared::theme())
        .build();
    let ids = conrod_example_shared::Ids::new(ui.widget_id_generator());
    let mut image_map = conrod_core::image::Map::new();
    struct FakeImage;
    let rust_logo = image_map.insert(FakeImage);
    let mut app = conrod_example_shared::DemoApp::new(rust_logo);
    {
        let mut ui_cell = ui.set_widgets();
        conrod_example_shared::gui(&mut ui_cell, &ids, &mut app);
    }

    let renderer = Renderer::new(window, &event_loop, &ui, CLEAR_COLOR);
    run(event_loop, renderer, ui, ids, app);
}

pub fn run(
    event_loop: EventLoop<()>,
    mut renderer: Renderer,
    mut ui: Ui,
    ids: conrod_example_shared::Ids,
    mut app: conrod_example_shared::DemoApp,
) {
    event_loop.run(move |event, _, control_flow| {
        if let Some(event) =
            conrod_rendy::winit_convert::convert_event(event.clone(), renderer.get_window())
        {
            ui.handle_event(event);
        }

        match event {
            Event::EventsCleared => {
                renderer.draw(&ui);
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            _ => *control_flow = ControlFlow::Poll,
        }

        if *control_flow == ControlFlow::Exit {
            renderer.dispose(&ui);
        }

        // Update widgets if any event has happened
        if ui.global_input().events().next().is_some() {
            let mut ui = ui.set_widgets();
            conrod_example_shared::gui(&mut ui, &ids, &mut app);
        }
    });
}
