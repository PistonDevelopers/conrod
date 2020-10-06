use conrod_example_shared::{WIN_H, WIN_W};
use conrod_rendy::{SimpleUiAux, UiPipeline, UiTexture};
use image;
use rendy::{
    command::{Families, QueueId, QueueType},
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
            self,
            event::{Event, WindowEvent},
            event_loop::{ControlFlow, EventLoop},
            window::{Window, WindowBuilder},
        },
        AnyWindowedRendy,
    },
    wsi::Surface,
};
use std::path::Path;

const CLEAR_COLOR: [f32; 4] = [0.2, 0.2, 0.2, 1.0];

// A wrapper around the winit window that allows us to implement the trait necessary for enabling
// the winit <-> conrod conversion functions.
struct WindowRef<'a>(&'a Window);

// Implement the `WinitWindow` trait for `WindowRef` to allow for generating compatible conversion
// functions.
impl<'a> conrod_winit::WinitWindow for WindowRef<'a> {
    fn get_inner_size(&self) -> Option<(u32, u32)> {
        Some(winit::window::Window::inner_size(&self.0).into())
    }
    fn hidpi_factor(&self) -> f32 {
        winit::window::Window::hidpi_factor(&self.0) as _
    }
}

// Generate the winit <-> conrod_core type conversion fns.
conrod_winit::v020_conversion_fns!();

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
            let family_id = families
                .find(|f| match f.capability() {
                    QueueType::General | QueueType::Graphics => true,
                    _ => false,
                })
                .expect("no queue to load image");
            let family = families.family(family_id);
            let queue_id = family.queue(0).id();
            let image = ui_texture_from_path(&logo_path, &mut factory, queue_id).unwrap();
            let mut image_map = conrod_core::image::Map::new();
            let rust_logo = image_map.insert(image);
            let app = conrod_example_shared::DemoApp::new(rust_logo);
            let dpi_factor = window.hidpi_factor();
            let aux = SimpleUiAux { ui, image_map, dpi_factor };
            let size = window.inner_size().to_physical(dpi_factor);
            let win_size = [size.width as u32, size.height as u32];
            let graph = create_graph(win_size, &mut factory, &mut families, surface, &aux);
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
        if let Some(event) = convert_event(event.clone(), &window) {
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
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested | WindowEvent::Destroyed => {
                    *control_flow = ControlFlow::Exit
                }
                WindowEvent::Resized(size) => {
                    let size = size.to_physical(window.hidpi_factor());
                    let win_size = [size.width as u32, size.height as u32];
                    recreate_graph(
                        win_size,
                        &mut factory,
                        &mut families,
                        &window,
                        &aux,
                        &mut graph,
                    );
                }
                WindowEvent::HiDpiFactorChanged(dpi_factor) => {
                    aux.dpi_factor = dpi_factor;
                    let size = window.inner_size().to_physical(dpi_factor);
                    let win_size = [size.width as u32, size.height as u32];
                    recreate_graph(
                        win_size,
                        &mut factory,
                        &mut families,
                        &window,
                        &aux,
                        &mut graph,
                    );
                }
                _ => (),
            },
            _ => (),
        }

        if *control_flow == ControlFlow::Exit {
            if let Some(graph) = graph.take() {
                graph.dispose(&mut factory, &aux);
            }
        }
    });
}

fn create_graph<B>(
    [win_w, win_h]: [u32; 2],
    factory: &mut Factory<B>,
    families: &mut Families<B>,
    surface: Surface<B>,
    aux: &SimpleUiAux<B>,
) -> Graph<B, SimpleUiAux<B>>
where
    B: Backend,
{
    let mut graph_builder = GraphBuilder::<_, SimpleUiAux<_>>::new();

    // Create the target, color image.
    let kind = Kind::D2(win_w, win_h, 1, 1);
    let levels = 1;
    let format = factory.get_surface_format(&surface);
    let clear = Some(ClearValue {
        color: ClearColor {
            float32: CLEAR_COLOR,
        },
    });
    let color = graph_builder.create_image(kind, levels, format, clear);

    // Create the UI graphics pipeline node.
    let pass = graph_builder.add_node(
        UiPipeline::builder()
            .into_subpass()
            .with_color(color)
            .into_pass(),
    );

    // The pass for presenting the colour image to the surface.
    graph_builder.add_node(PresentNode::builder(factory, surface, color).with_dependency(pass));

    graph_builder
        .build(factory, families, &aux)
        .expect("failed to build the graph")
}

fn recreate_graph<B>(
    win_size: [u32; 2],
    factory: &mut Factory<B>,
    families: &mut Families<B>,
    window: &Window,
    aux: &SimpleUiAux<B>,
    graph: &mut Option<Graph<B, SimpleUiAux<B>>>,
) where
    B: Backend,
{
    if let Some(graph) = graph.take() {
        graph.dispose(factory, aux);
    }
    let surface = factory
        .create_surface(window)
        .expect("failed to create surface");
    *graph = Some(create_graph(win_size, factory, families, surface, aux));
}

fn ui_texture_from_path<B>(
    path: &Path,
    factory: &mut Factory<B>,
    queue_id: QueueId,
) -> Result<UiTexture<B>, image::ImageError>
where
    B: Backend,
{
    let image = image::open(path)?.to_rgba();
    let (width, height) = image.dimensions();
    let dimensions = [width, height];
    let texture = UiTexture::from_rgba_bytes(&image, dimensions, factory, queue_id)
        .expect("failed to create `UiTexture`");
    Ok(texture)
}
