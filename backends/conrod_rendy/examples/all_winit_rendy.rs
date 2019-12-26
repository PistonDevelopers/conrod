use conrod_example_shared::{WIN_H, WIN_W};
use conrod_rendy::{UiTexture, UiPipeline, SimpleUiAux};
use rendy::{
    command::{Families, QueueType},
    factory::{self, Factory},
    graph::{
        present::PresentNode,
        render::{RenderGroupBuilder, SimpleGraphicsPipeline},
        Graph, GraphBuilder,
    },
    hal::{
        command::{ClearColor, ClearValue},
        image::Kind,
        Backend,
    },
    init::{
        winit::{
            event::{Event, WindowEvent},
            event_loop::{ControlFlow, EventLoop},
            window::{Window, WindowBuilder},
        },
        AnyWindowedRendy,
    },
};

const CLEAR_COLOR: [f32; 4] = [0.2, 0.2, 0.2, 1.0];

fn main() {
    // Create the window manager
    let event_loop = EventLoop::new();
    let window_builder = WindowBuilder::new()
        .with_inner_size((WIN_W, WIN_H).into())
        .with_title("Conrod with Rendy and winit");

    // Create Ui and Ids of widgets to instantiate
    let mut ui = conrod_core::UiBuilder::new([WIN_W as f64, WIN_H as f64])
        .theme(conrod_example_shared::theme())
        .build();
    let ids = conrod_example_shared::Ids::new(ui.widget_id_generator());

    // Load font from file
    let assets = find_folder::Search::KidsThenParents(3, 5)
        .for_folder("assets")
        .unwrap();
    let font_path = assets.join("fonts/NotoSans/NotoSans-Regular.ttf");
    ui.fonts.insert_from_file(font_path).unwrap();

    // We'll load the Rust logo from our assets folder to use as an example image.
    let logo_path = assets.join("images/rust.png");

    let config: factory::Config = Default::default();
    let rendy = AnyWindowedRendy::init_auto(&config, window_builder, &event_loop).unwrap();
    rendy::with_any_windowed_rendy!((rendy)
        (mut factory, mut families, surface, window) => {
            // Load the image onto the GPU.
            let family_id = families
                .find(|f| match f.capability() {
                    QueueType::General | QueueType::Graphics => true,
                    _ => false,
                })
                .expect("no queue to load image");
            let family = families.family(family_id);
            let queue_id = family.queue(0).id();
            let image = UiTexture::from_path(&logo_path, &mut factory, queue_id).unwrap();
            let mut image_map = conrod_core::image::Map::new();
            let rust_logo = image_map.insert(image);

            // Create the demo application.
            let mut app = conrod_example_shared::DemoApp::new(rust_logo);

            let mut graph_builder = GraphBuilder::<_, SimpleUiAux<_>>::new();
            let size = window
                .inner_size()
                .to_physical(window.hidpi_factor());

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
                UiPipeline::builder()
                    .into_subpass()
                    .with_color(color)
                    .into_pass(),
            );

            graph_builder.add_node(
                PresentNode::builder(&factory, surface, color).with_dependency(pass),
            );

            let aux = SimpleUiAux {
                ui,
                image_map,
                dpi_factor: window.hidpi_factor(),
            };

            let graph = graph_builder
                .build(&mut factory, &mut families, &aux)
                .unwrap();

            run(event_loop, aux, ids, app, factory, families, window, Some(graph));
        }
    );
}

pub fn run<B: Backend>(
    event_loop: EventLoop<()>,
    mut aux: SimpleUiAux<B>,
    ids: conrod_example_shared::Ids,
    mut app: conrod_example_shared::DemoApp,
    mut factory: Factory<B>,
    mut families: Families<B>,
    window: Window,
    mut graph: Option<Graph<B, SimpleUiAux<B>>>,
) {
    event_loop.run(move |event, _, control_flow| {
        if let Some(event) = conrod_rendy::winit_convert::convert_event(event.clone(), &window) {
            aux.ui.handle_event(event);
        }

        // Update widgets if any event has happened
        if aux.ui.global_input().events().next().is_some() {
            let mut ui = aux.ui.set_widgets();
            conrod_example_shared::gui(&mut ui, &ids, &mut app);
        }

        match event {
            Event::EventsCleared => {
                factory.maintain(&mut families);
                if let Some(graph) = graph.as_mut() {
                    graph.run(&mut factory, &mut families, &aux);
                }
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            _ => *control_flow = ControlFlow::Poll,
        }

        if *control_flow == ControlFlow::Exit {
            if let Some(graph) = graph.take() {
                graph.dispose(&mut factory, &aux);
            }
        }
    });
}
