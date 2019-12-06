use conrod_core::Ui;
use conrod_example_shared::{WIN_H, WIN_W};
use conrod_rendy::ConrodPipeline;
use env_logger::Env;
use rendy::command::Families;
use rendy::factory::{Config, Factory};
use rendy::graph::{present::PresentNode, render::*, Graph, GraphBuilder};
use rendy::hal::{
    command::{ClearColor, ClearValue},
    image::Kind,
    Backend,
};
use rendy::init::{
    winit::{
        event::{Event, WindowEvent},
        event_loop::{ControlFlow, EventLoop},
        window::WindowBuilder,
    },
    AnyWindowedRendy,
};

const CLEAR_COLOR: [f32; 4] = [0.2, 0.2, 0.2, 1.0];

fn main() {
    env_logger::Builder::from_env(Env::default().filter("trace")).init();

    // Create the window manager (winit)
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_inner_size((WIN_W, WIN_H).into())
        .with_title("Conrod with Rendy and winit");

    // Create the UI
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

    // Initialize Rendy
    let config: Config = Default::default();
    let rendy = AnyWindowedRendy::init_auto(&config, window, &event_loop).unwrap();

    rendy::with_any_windowed_rendy!((rendy)
        (mut factory, mut families, surface, window) => {
            let mut graph_builder = GraphBuilder::<_, Ui>::new();
            let size = window.inner_size().to_physical(window.hidpi_factor());

            let color = graph_builder.create_image(
                Kind::D2(size.width as u32, size.height as u32, 1, 1),
                1,
                factory.get_surface_format(&surface),
                Some(ClearValue {
                    color: ClearColor {
                        float32: CLEAR_COLOR,
                    },
                }),
            );

            let pass = graph_builder.add_node(
                ConrodPipeline::builder()
                    .into_subpass()
                    .with_color(color)
                    .into_pass(),
            );

            graph_builder.add_node(PresentNode::builder(&factory, surface, color).with_dependency(pass));

            let graph = graph_builder
                .build(&mut factory, &mut families, &ui)
                .unwrap();

            run(event_loop, factory, families, graph, ui, ids, app);
        }
    );
}

pub fn run<B: Backend>(
    event_loop: EventLoop<()>,
    mut factory: Factory<B>,
    mut families: Families<B>,
    graph: Graph<B, Ui>,
    mut ui: Ui,
    ids: conrod_example_shared::Ids,
    mut app: conrod_example_shared::DemoApp,
) {
    let mut graph = Some(graph);

    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::EventsCleared => {
                if let Some(ref mut graph) = graph {
                    graph.run(&mut factory, &mut families, &ui);
                }
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            _ => *control_flow = ControlFlow::Poll,
        }

        if *control_flow == ControlFlow::Exit && graph.is_some() {
            graph.take().unwrap().dispose(&mut factory, &ui);
        }

        // Update widgets if any event has happened
        if ui.global_input().events().next().is_some() {
            let mut ui = ui.set_widgets();
            conrod_example_shared::gui(&mut ui, &ids, &mut app);
        }
    });
}
