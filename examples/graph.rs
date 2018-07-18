//! A simple example that demonstrates the **Graph** widget functionality.

#[cfg(all(feature="winit", feature="glium"))] #[macro_use] extern crate conrod;
#[cfg(all(feature="winit", feature="glium"))] mod support;

fn main() {
    feature::main();
}

#[cfg(all(feature="winit", feature="glium"))]
mod feature {
    extern crate petgraph;
    use conrod::{self, widget, Borderable, Colorable, Labelable, Positionable, Sizeable, Widget};
    use conrod::backend::glium::glium::{self, Surface};
    use conrod::widget::graph::{node, Event, EdgeEvent, Node, NodeEvent, NodeSocket};
    use std::collections::HashMap;
    use support;

    widget_ids! {
        struct Ids {
            graph,
        }
    }

    type MyGraph = petgraph::Graph<&'static str, (usize, usize)>;
    type Layout = widget::graph::Layout<petgraph::graph::NodeIndex>;

    pub fn main() {
        const WIDTH: u32 = 900;
        const HEIGHT: u32 = 500;

        // Demo Graph.
        let mut graph = MyGraph::new();
        let a = graph.add_node("A");
        let b = graph.add_node("B");
        let c = graph.add_node("C");
        let d = graph.add_node("D");
        let e = graph.add_node("E");
        graph.extend_with_edges(&[
            (a, c, (1, 0)),
            (a, d, (0, 1)),
            (b, d, (0, 0)),
            (c, d, (0, 2)),
            (d, e, (0, 0)),
        ]);

        // Construct a starting layout for the nodes.
        let mut layout_map = HashMap::new();
        layout_map.insert(b, [-100.0, 100.0]);
        layout_map.insert(a, [-300.0, 0.0]);
        layout_map.insert(c, [-100.0, -100.0]);
        layout_map.insert(d, [100.0, 0.0]);
        layout_map.insert(e, [300.0, 0.0]);
        let mut layout = Layout::from(layout_map);

        // Build the window.
        let mut events_loop = glium::glutin::EventsLoop::new();
        let window = glium::glutin::WindowBuilder::new()
            .with_title("Conrod Graph Widget")
            .with_dimensions((WIDTH, HEIGHT).into());
        let context = glium::glutin::ContextBuilder::new()
            .with_multisampling(4)
            .with_vsync(true);
        let display = glium::Display::new(window, context, &events_loop).unwrap();

        // construct our `Ui`.
        let mut ui = conrod::UiBuilder::new([WIDTH as f64, HEIGHT as f64]).build();

        // Generate the widget identifiers.
        let ids = Ids::new(ui.widget_id_generator());

        // Add a `Font` to the `Ui`'s `font::Map` from file.
        const FONT_PATH: &'static str =
            concat!(env!("CARGO_MANIFEST_DIR"), "/assets/fonts/NotoSans/NotoSans-Regular.ttf");
        ui.fonts.insert_from_file(FONT_PATH).unwrap();

        // A type used for converting `conrod::render::Primitives` into `Command`s that can be used
        // for drawing to the glium `Surface`.
        let mut renderer = conrod::backend::glium::Renderer::new(&display).unwrap();

        // The image map describing each of our widget->image mappings (in our case, none).
        let image_map = conrod::image::Map::<glium::texture::Texture2d>::new();

        // Begin the event loop.
        let mut event_loop = support::EventLoop::new();
        'main: loop {
            // Handle all events.
            for event in event_loop.next(&mut events_loop) {

                // Use the `winit` backend feature to convert the winit event to a conrod one.
                if let Some(event) = conrod::backend::winit::convert_event(event.clone(), &display) {
                    ui.handle_event(event);
                    event_loop.needs_update();
                }

                // Break from the loop upon `Escape` or closed window.
                match event.clone() {
                    glium::glutin::Event::WindowEvent { event, .. } => {
                        match event {
                            glium::glutin::WindowEvent::CloseRequested |
                            glium::glutin::WindowEvent::KeyboardInput {
                                input: glium::glutin::KeyboardInput {
                                    virtual_keycode: Some(glium::glutin::VirtualKeyCode::Escape),
                                    ..
                                },
                                ..
                            } => break 'main,
                            _ => (),
                        }
                    }
                    _ => (),
                }
            }

            // Set the widgets.
            set_widgets(&mut ui.set_widgets(), &ids, &mut graph, &mut layout);

            // Draw the `Ui` if it has changed.
            if let Some(primitives) = ui.draw_if_changed() {
                renderer.fill(&display, primitives, &image_map);
                let mut target = display.draw();
                target.clear_color(0.0, 0.0, 0.0, 1.0);
                renderer.draw(&display, &mut target, &image_map).unwrap();
                target.finish().unwrap();
            }
        }
    }

    fn set_widgets(ui: &mut conrod::UiCell, ids: &Ids, graph: &mut MyGraph, layout: &mut Layout) {

        /////////////////
        ///// GRAPH /////
        /////////////////
        //
        // Set the `Graph` widget.
        //
        // This returns a session on which we can begin setting nodes and edges.
        //
        // The session is used in multiple stages:
        //
        // 1. `Nodes` for setting a node widget for each node.
        // 2. `Edges` for setting an edge widget for each edge.
        // 3. `Final` for optionally displaying zoom percentage and cam position.

        let session = {
            // An identifier for each node in the graph.
            let node_indices = graph.node_indices();
            // Describe each edge in the graph as NodeSocket -> NodeSocket.
            let edges = graph.raw_edges()
                .iter()
                .map(|e| {
                    let start = NodeSocket { id: e.source(), socket_index: e.weight.0 };
                    let end = NodeSocket { id: e.target(), socket_index: e.weight.1 };
                    (start, end)
                });
            widget::Graph::new(node_indices, edges, layout)
                .background_color(conrod::color::rgb(0.31, 0.33, 0.35))
                .wh_of(ui.window)
                .middle_of(ui.window)
                .set(ids.graph, ui)
        };

        //////////////////
        ///// EVENTS /////
        //////////////////
        //
        // Graph events that have occurred since the last time the graph was instantiated.

        for event in session.events() {
            match event {
                Event::Node(event) => match event {
                    // NodeEvent::Add(node_kind) => {
                    // },
                    NodeEvent::Remove(node_id) => {
                    },
                    NodeEvent::Dragged { node_id, to, .. } => {
                        *layout.get_mut(&node_id).unwrap() = to;
                    },
                },
                Event::Edge(event) => match event {
                    EdgeEvent::AddStart(node_socket) => {
                    },
                    EdgeEvent::Add { start, end } => {
                    },
                    EdgeEvent::Cancelled(node_socket) => {
                    },
                    EdgeEvent::Remove { start, end } => {
                    },
                },
            }
        }

        /////////////////
        ///// NODES /////
        /////////////////
        //
        // Instantiate a widget for each node within the graph.

        let mut session = session.next();
        for node in session.nodes() {
            // Each `Node` contains:
            //
            // `id` - The unique node identifier for this node.
            // `point` - The position at which this node will be set.
            // `inputs`
            // `outputs`
            //
            // Calling `node.widget(some_widget)` returns a `NodeWidget`, which contains:
            //
            // `wiget_id` - The widget identifier for the widget that will represent this node.
            let node_id = node.node_id();
            let inputs = graph.neighbors_directed(node_id, petgraph::Incoming).count();
            let outputs = graph.neighbors_directed(node_id, petgraph::Outgoing).count();
            let button = widget::Button::new()
                .label(&graph[node_id])
                .border(0.0);
            let widget = Node::new(button)
                .inputs(inputs)
                .outputs(outputs)
                //.socket_color(conrod::color::LIGHT_RED)
                .w_h(100.0, 60.0);
            for _click in node.widget(widget).set(ui).widget_event {
                println!("{} was clicked!", &graph[node_id]);
            }
        }

        /////////////////
        ///// EDGES /////
        /////////////////
        //
        // Instantiate a widget for each edge within the graph.

        let mut session = session.next();
        for edge in session.edges() {
            let (a, b) = node::edge_socket_rects(&edge, ui);
            let line = widget::Line::abs(a.xy(), b.xy())
                .color(conrod::color::DARK_CHARCOAL)
                .thickness(3.0);

            // Each edge contains:
            //
            // `start` - The unique node identifier for the node at the start of the edge with point.
            // `end` - The unique node identifier for the node at the end of the edge with point.
            // `widget_id` - The wiget identifier for this edge.
            edge.widget(line).set(ui);
        }
    }
}

#[cfg(not(all(feature="winit", feature="glium")))]
mod feature {
    pub fn main() {
        println!("This example requires the `winit` and `glium` features. \
                 Try running `cargo run --release --features=\"winit glium\" --example <example_name>`");
    }
}
