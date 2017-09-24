//! A simple demonstration of how to construct and use Canvasses by splitting up the window.

#[cfg(all(feature="winit", feature="glium"))] #[macro_use] extern crate conrod;
#[cfg(all(feature="winit", feature="glium"))] mod support;

fn main() {
    feature::main();
}

#[cfg(all(feature="winit", feature="glium"))]
mod feature {
    extern crate find_folder;
    extern crate winit;
    use conrod;
    use conrod::backend::glium::glium;
    use conrod::backend::glium::glium::Surface;
    use conrod::backend::winit::WinitWindow;
    use support;

    pub fn theme() -> conrod::Theme {
        use conrod::position::{Align, Direction, Padding, Position, Relative};
        use conrod::color::{BLACK, WHITE};
        conrod::Theme {
            name: "Demo Theme".to_string(),
            padding: Padding::none(),
            x_position: Position::Relative(Relative::Align(Align::Start), None),
            y_position: Position::Relative(Relative::Direction(Direction::Backwards, 20.0), None),
            background_color: BLACK,
            shape_color: WHITE,
            border_color: BLACK,
            border_width: 1.0,
            label_color: BLACK,
            font_id: None,
            font_size_large: 26,
            font_size_medium: 18,
            font_size_small: 12,
            widget_styling: conrod::theme::StyleMap::default(),
            mouse_drag_threshold: 0.0,
            double_click_threshold: ::std::time::Duration::from_millis(500),
        }
    }

    pub fn main() {
        const WIDTH: u32 = 800;
        const HEIGHT: u32 = 600;

        // Build the window.
        let mut events_loop = glium::glutin::EventsLoop::new();
        let window = glium::glutin::WindowBuilder::new()
            .with_title("PaneCanvas")
            .with_dimensions(WIDTH, HEIGHT);
        let context = glium::glutin::ContextBuilder::new()
            .with_vsync(true)
            .with_multisampling(4);
        let display = glium::Display::new(window, context, &events_loop).unwrap();

        // construct our `Ui`.
        let mut ui = conrod::UiBuilder::new([WIDTH as f64, HEIGHT as f64]).theme(theme()).build();

        // Add a `Font` to the `Ui`'s `font::Map` from file.
        let assets = find_folder::Search::KidsThenParents(3, 5).for_folder("assets").unwrap();
        let font_path = assets.join("fonts/NotoSans/NotoSans-Regular.ttf");
        ui.fonts.insert_from_file(font_path).unwrap();

        // A type used for converting `conrod::render::Primitives` into `Command`s that can be used
        // for drawing to the glium `Surface`.
        let mut renderer = conrod::backend::glium::Renderer::new(&display).unwrap();

        // The image map describing each of our widget->image mappings (in our case, none).
        let image_map = conrod::image::Map::<glium::texture::Texture2d>::new();

        // Instantiate the generated list of widget identifiers.
        let ids = &mut Ids::new(ui.widget_id_generator());

        // Poll events from the window.
        let mut event_loop = support::EventLoop::new();
        'main: loop {

            // Handle all events.
            for event in event_loop.next(&mut events_loop) {

                // Use the `winit` backend feature to convert the winit event to a conrod one.
                if let Some(event) = conrod::backend::winit::convert_event(event.clone(), &display) {
                    ui.handle_event(event);
                    event_loop.needs_update();
                }

                match event {
                    glium::glutin::Event::WindowEvent { event, .. } => match event {
                        // Break from the loop upon `Escape`.
                        glium::glutin::WindowEvent::Closed |
                        glium::glutin::WindowEvent::KeyboardInput {
                            input: glium::glutin::KeyboardInput {
                                virtual_keycode: Some(glium::glutin::VirtualKeyCode::Escape),
                                ..
                            },
                            ..
                        } => break 'main,
                        _ => (),
                    },
                    _ => (),
                }
            }

            // Instnatiate all widgets in the GUI.
            set_widgets(ui.set_widgets(), ids);

            display.set_mouse_cursor(ui.mouse_cursor());
            
            // Render the `Ui` and then display it on the screen.
            if let Some(primitives) = ui.draw_if_changed() {
                renderer.fill(&display, primitives, &image_map);
                let mut target = display.draw();
                target.clear_color(0.0, 0.0, 0.0, 1.0);
                renderer.draw(&display, &mut target, &image_map).unwrap();
                target.finish().unwrap();
            }
        }
    }

    // Draw the Ui.
    fn set_widgets(ref mut ui: conrod::UiCell, ids: &mut Ids) {
        use conrod::{color, widget, Colorable, Labelable, Positionable, Sizeable, Widget};

        // Construct our main `PaneCanvas` tree.
        widget::PaneCanvas::new().flow_down(&[
            (ids.header, widget::PaneCanvas::new().color(color::BLUE).pad_bottom(20.0)),
            (ids.body, widget::PaneCanvas::new().length(300.0).divider_width(10.0).divider_color(color::LIGHT_ORANGE)
             .flow_right(&[
                (ids.left_column, widget::PaneCanvas::new().color(color::LIGHT_ORANGE).pad(20.0)),
                (ids.middle_column, widget::PaneCanvas::new().color(color::ORANGE)),
                (ids.right_column, widget::PaneCanvas::new().color(color::DARK_ORANGE).pad(20.0)),
            ])),
            (ids.footer, widget::PaneCanvas::new().color(color::BLUE).scroll_kids_vertically()),
        ]).set(ids.master, ui);

        // A scrollbar for the `FOOTER` canvas.
        widget::Scrollbar::y_axis(ids.footer).auto_hide(true).set(ids.footer_scrollbar, ui);

        // Now we'll make a couple floating `PaneCanvas`ses.
        let floating = widget::PaneCanvas::new().floating(true).w_h(110.0, 150.0).label_color(color::WHITE);
        floating.clone().middle_of(ids.left_column).title_bar("Blue").color(color::BLUE).set(ids.floating_a, ui);
        floating.clone().middle_of(ids.right_column).title_bar("Orange").color(color::LIGHT_ORANGE).set(ids.floating_b, ui);

        // Here we make some canvas `Tabs` in the middle column.
        widget::Tabs::new(&[(ids.tab_foo, "FOO"), (ids.tab_bar, "BAR"), (ids.tab_baz, "BAZ")])
            .wh_of(ids.middle_column)
            .color(color::BLUE)
            .label_color(color::WHITE)
            .middle_of(ids.middle_column)
            .set(ids.tabs, ui);

        widget::Text::new("Fancy Title")
            .color(color::LIGHT_ORANGE)
            .font_size(48)
            .middle_of(ids.header)
            .set(ids.title, ui);
        widget::Text::new("Subtitle")
            .color(color::BLUE.complement())
            .mid_bottom_of(ids.header)
            .set(ids.subtitle, ui);

        widget::Text::new("Top Left")
            .color(color::LIGHT_ORANGE.complement())
            .top_left_of(ids.left_column)
            .set(ids.top_left, ui);

        widget::Text::new("Bottom Right")
            .color(color::DARK_ORANGE.complement())
            .bottom_right_of(ids.right_column)
            .set(ids.bottom_right, ui);

        fn text (text: widget::Text) -> widget::Text { text.color(color::WHITE).font_size(36) }
        text(widget::Text::new("Foo!")).middle_of(ids.tab_foo).set(ids.foo_label, ui);
        text(widget::Text::new("Bar!")).middle_of(ids.tab_bar).set(ids.bar_label, ui);
        text(widget::Text::new("BAZ!")).middle_of(ids.tab_baz).set(ids.baz_label, ui);

        let footer_wh = ui.wh_of(ids.footer).unwrap();
        let mut elements = widget::Matrix::new(COLS, ROWS)
            .w_h(footer_wh[0], footer_wh[1] * 2.0)
            .mid_top_of(ids.footer)
            .set(ids.button_matrix, ui);
        while let Some(elem) = elements.next(ui) {
            let (r, c) = (elem.row, elem.col);
            let n = c + r * c;
            let luminance = n as f32 / (COLS * ROWS) as f32;
            let button = widget::Button::new().color(color::BLUE.with_luminance(luminance));
            for _click in elem.set(button, ui) {
                println!("Hey! {:?}", (r, c));
            }
        }

        let button = widget::Button::new().color(color::RED).w_h(30.0, 30.0);
        for _click in button.clone().middle_of(ids.floating_a).set(ids.bing, ui) {
            println!("Bing!");
        }
        for _click in button.middle_of(ids.floating_b).set(ids.bong, ui) {
            println!("Bong!");
        }
    }


    // Button matrix dimensions.
    const ROWS: usize = 10;
    const COLS: usize = 24;

    // Generate a unique `WidgetId` for each widget.
    widget_ids! {
        struct Ids {
            master,
            header,
            body,
            left_column,
            middle_column,
            right_column,
            footer,
            footer_scrollbar,
            floating_a,
            floating_b,
            tabs,
            tab_foo,
            tab_bar,
            tab_baz,

            title,
            subtitle,
            top_left,
            bottom_right,
            foo_label,
            bar_label,
            baz_label,
            button_matrix,
            bing,
            bong,
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
