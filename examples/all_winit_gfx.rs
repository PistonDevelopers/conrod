//! A demonstration of using `winit` to provide events and GFX to draw the UI.
//!
//! `winit` is used via the `glutin` crate which also provides an OpenGL context for drawing
//! `conrod::render::Primitives` to the screen.

#![allow(unused_variables)]

#[cfg(feature="winit")] #[macro_use] extern crate conrod;
#[cfg(feature="winit")] extern crate glutin;
#[cfg(feature="winit")] extern crate winit;
#[cfg(feature="gfx_rs")] extern crate gfx;
#[cfg(feature="gfx_rs")] extern crate gfx_core;

#[cfg(feature="winit")]
mod support;


fn main() {
    feature::main();
}

#[cfg(all(feature="winit",feature="gfx_rs"))]
mod feature {
    extern crate gfx_window_glutin;
    extern crate find_folder;
    extern crate image;

    use std;

    use conrod;
    use glutin;
    use gfx;
    use gfx_core;
    use support;
    use winit;

    use glutin::GlContext;
    use gfx::Device;


    const WIN_W: u32 = support::WIN_W;
    const WIN_H: u32 = support::WIN_H;
    const CLEAR_COLOR: [f32; 4] = [0.2, 0.2, 0.2, 1.0];

    type DepthFormat = gfx::format::DepthStencil;

    pub fn main() {
        // Builder for window
        let builder = glutin::WindowBuilder::new()
            .with_title("Conrod with GFX and Glutin")
            .with_dimensions((WIN_W, WIN_H).into());

        let context = glutin::ContextBuilder::new()
            .with_multisampling(4);

        let mut events_loop = winit::EventsLoop::new();

        // Initialize gfx things
        let (window, mut device, mut factory, rtv, _) =
            gfx_window_glutin::init::<conrod::backend::gfx::ColorFormat, DepthFormat>(builder, context, &events_loop );
        let mut encoder: gfx::Encoder<_, _> = factory.create_command_buffer().into();

        let mut renderer = conrod::backend::gfx::Renderer::new(&mut factory, &rtv, window.get_hidpi_factor() as f64).unwrap();

        // Create Ui and Ids of widgets to instantiate
        let mut ui = conrod::UiBuilder::new([WIN_W as f64, WIN_H as f64]).theme(support::theme()).build();
        let ids = support::Ids::new(ui.widget_id_generator());

        // Load font from file
        let assets = find_folder::Search::KidsThenParents(3, 5).for_folder("assets").unwrap();
        let font_path = assets.join("fonts/NotoSans/NotoSans-Regular.ttf");
        ui.fonts.insert_from_file(font_path).unwrap();

        // Load the Rust logo from our assets folder to use as an example image.
        fn load_rust_logo<T: gfx::format::TextureFormat,R: gfx_core::Resources, F: gfx::Factory<R>>(factory: &mut F) -> (gfx::handle::ShaderResourceView<R, <T as gfx::format::Formatted>::View>,(u32,u32)) {
            use gfx::{format, texture};
            use gfx::memory::Bind;
            use gfx::memory::Usage;
            let assets = find_folder::Search::ParentsThenKids(3, 3).for_folder("assets").unwrap();
            let path = assets.join("images/rust.png");
            let rgba_image = image::open(&std::path::Path::new(&path)).unwrap().to_rgba();
            let image_dimensions = rgba_image.dimensions();
            let kind = texture::Kind::D2(
                image_dimensions.0 as texture::Size,
                image_dimensions.1 as texture::Size,
                texture::AaMode::Single
            );
            let info = texture::Info {
                kind: kind,
                levels: 1,
                format: <T::Surface as format::SurfaceTyped>::get_surface_type(),
                bind: Bind::SHADER_RESOURCE,
                usage: Usage::Dynamic,
            };
            let raw = factory.create_texture_raw(
                info,
                Some(<T::Channel as format::ChannelTyped>::get_channel_type()),
                Some((&[rgba_image.into_raw().as_slice()], texture::Mipmap::Provided))).unwrap();
            let tex = gfx_core::memory::Typed::new(raw);
            let view = factory.view_texture_as_shader_resource::<T>(
                &tex, (0,0), format::Swizzle::new()
            ).unwrap();
            (view,image_dimensions)
        }

        let mut image_map = conrod::image::Map::new();
        let rust_logo = image_map.insert(load_rust_logo::<conrod::backend::gfx::ColorFormat,_,_>(&mut factory));

        // Demonstration app state that we'll control with our conrod GUI.
        let mut app = support::DemoApp::new(rust_logo);

        'main: loop {
            // If the window is closed, this will be None for one tick, so to avoid panicking with
            // unwrap, instead break the loop
            let (win_w, win_h): (u32, u32) = match window.get_inner_size() {
                Some(s) => s.into(),
                None => break 'main,
            };

            let dpi_factor = window.get_hidpi_factor() as f32;

            if let Some(primitives) = ui.draw_if_changed() {
                let dims = (win_w as f32 * dpi_factor, win_h as f32 * dpi_factor);

                //Clear the window
                renderer.clear(&mut encoder, CLEAR_COLOR);

                renderer.fill(&mut encoder,dims,dpi_factor as f64,primitives,&image_map);

                renderer.draw(&mut factory,&mut encoder,&image_map);

                encoder.flush(&mut device);
                window.swap_buffers().unwrap();
                device.cleanup();
            }

            let mut should_quit = false;
            events_loop.poll_events(|event|{

                // Convert winit event to conrod event, requires conrod to be built with the `winit` feature
                if let Some(event) = conrod::backend::winit::convert_event(event.clone(), window.window()) {
                    ui.handle_event(event);
                }

                // Close window if the escape key or the exit button is pressed
                match event {
                    winit::Event::WindowEvent{event, .. } =>
                        match event {
                            winit::WindowEvent::KeyboardInput{ input: winit::KeyboardInput{ virtual_keycode: Some(winit::VirtualKeyCode::Escape),..}, ..} |
                            winit::WindowEvent::CloseRequested => should_quit = true,
                            winit::WindowEvent::Resized(logical_size) => {
                                let hidpi_factor = window.get_hidpi_factor();
                                let physical_size = logical_size.to_physical(hidpi_factor);
                                window.resize(physical_size);
                                let (new_color, _) = gfx_window_glutin::new_views::<conrod::backend::gfx::ColorFormat, DepthFormat>(&window);
                                renderer.on_resize(new_color);
                            }
                            _ => {},
                        },
                    _ => {},
                }
            });
            if should_quit {
                break 'main;
            }

            // Update widgets if any event has happened
            if ui.global_input().events().next().is_some() {
                let mut ui = ui.set_widgets();
                support::gui(&mut ui, &ids, &mut app);
            }
        }
    }
}

#[cfg(not(all(feature="winit",feature="gfx_rs")))]
mod feature {
    pub fn main() {
        println!("This example requires the `winit` feature and the `gfx_rs` feature. \
                 Try running `cargo run --release --no-default-features --features=\"winit gfx_rs\" --example <example_name>`");
   }
}
