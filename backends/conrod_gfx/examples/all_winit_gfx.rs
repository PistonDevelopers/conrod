//! A demonstration of using `winit` to provide events and GFX to draw the UI.
//!
//! `winit` is used via the `glutin` crate which also provides an OpenGL context for drawing
//! `conrod_core::render::Primitives` to the screen.

#![allow(unused_variables)]

extern crate conrod_core;
extern crate conrod_example_shared;
extern crate conrod_gfx;
extern crate conrod_winit;
extern crate find_folder;
extern crate gfx;
extern crate gfx_core;
extern crate gfx_device_gl;
extern crate glutin;
extern crate image;
extern crate old_school_gfx_glutin_ext;
extern crate winit;

use conrod_example_shared::{WIN_H, WIN_W};
use gfx::format::Formatted;
use gfx::Device;
use old_school_gfx_glutin_ext::*;

const CLEAR_COLOR: [f32; 4] = [0.2, 0.2, 0.2, 1.0];

type DepthFormat = gfx::format::DepthStencil;

fn get_window_dimensions(
    ctx: &glutin::WindowedContext<glutin::PossiblyCurrent>,
) -> gfx::texture::Dimensions {
    let window = ctx.window();
    let (width, height) = {
        let size = window.inner_size();
        (size.width as _, size.height as _)
    };
    let aa = ctx.get_pixel_format().multisampling.unwrap_or(0) as gfx::texture::NumSamples;

    (width, height, 1, aa.into())
}

// Generate the winit <-> conrod_core type conversion fns.
conrod_winit::v023_conversion_fns!();

fn main() {
    // Builder for window
    let builder = glutin::window::WindowBuilder::new()
        .with_title("Conrod with GFX and Glutin")
        .with_inner_size(glutin::dpi::Size::new(glutin::dpi::PhysicalSize::new(
            WIN_W, WIN_H,
        )));
    let mut user_input =("Editable text!".to_string(),"Multiple lines of \neditable text!".to_string());

    let context = glutin::ContextBuilder::new().with_multisampling(4);

    let event_loop = winit::event_loop::EventLoop::new();

    // Initialize gfx things
    let (context, mut device, mut factory, rtv, _) = context
        .build_windowed(builder, &event_loop)
        .unwrap()
        .init_gfx::<conrod_gfx::ColorFormat, DepthFormat>();

    let mut encoder: gfx::Encoder<_, _> = factory.create_command_buffer().into();

    let mut renderer =
        conrod_gfx::Renderer::new(&mut factory, &rtv, context.window().scale_factor()).unwrap();

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

    // Load the Rust logo from our assets folder to use as an example image.
    fn load_rust_logo<T: gfx::format::TextureFormat, R: gfx_core::Resources, F: gfx::Factory<R>>(
        factory: &mut F,
    ) -> (
        gfx::handle::ShaderResourceView<R, <T as gfx::format::Formatted>::View>,
        (u32, u32),
    ) {
        use gfx::memory::Bind;
        use gfx::memory::Usage;
        use gfx::{format, texture};
        let assets = find_folder::Search::ParentsThenKids(5, 3)
            .for_folder("assets")
            .unwrap();
        let path = assets.join("images/rust.png");
        let rgba_image = image::open(&std::path::Path::new(&path)).unwrap().to_rgba();
        let image_dimensions = rgba_image.dimensions();
        let kind = texture::Kind::D2(
            image_dimensions.0 as texture::Size,
            image_dimensions.1 as texture::Size,
            texture::AaMode::Single,
        );
        let info = texture::Info {
            kind: kind,
            levels: 1,
            format: <T::Surface as format::SurfaceTyped>::get_surface_type(),
            bind: Bind::SHADER_RESOURCE,
            usage: Usage::Dynamic,
        };
        let raw = factory
            .create_texture_raw(
                info,
                Some(<T::Channel as format::ChannelTyped>::get_channel_type()),
                Some((
                    &[rgba_image.into_raw().as_slice()],
                    texture::Mipmap::Provided,
                )),
            )
            .unwrap();
        let tex = gfx_core::memory::Typed::new(raw);
        let view = factory
            .view_texture_as_shader_resource::<T>(&tex, (0, 0), format::Swizzle::new())
            .unwrap();
        (view, image_dimensions)
    }

    let mut image_map = conrod_core::image::Map::new();
    let rust_logo = image_map.insert(load_rust_logo::<conrod_gfx::ColorFormat, _, _>(
        &mut factory,
    ));

    // Demonstration app state that we'll control with our conrod GUI.
    let mut app = conrod_example_shared::DemoApp::new(rust_logo);

    // Gfx dimensions. Tracked for updated_views_raw() call.
    let mut dimensions = get_window_dimensions(&context);

    event_loop.run(move |event, _, control_flow| {
        let window_size = context.window().inner_size();
        let dpi_factor = context.window().scale_factor();

        if let Some(primitives) = ui.draw_if_changed() {
            let dims = window_size.into();

            //Clear the window
            renderer.clear(&mut encoder, CLEAR_COLOR);

            renderer.fill(&mut encoder, dims, dpi_factor, primitives, &image_map);

            renderer.draw(&mut factory, &mut encoder, &image_map);

            encoder.flush(&mut device);
            context.swap_buffers().unwrap();
            device.cleanup();
        }

        // Convert winit event to conrod event, requires conrod to be built with the `winit` feature
        if let Some(event) = convert_event(&event, &context.window()) {
            ui.handle_event(event);
        }

        // Close window if the escape key or the exit button is pressed
        match event {
            winit::event::Event::WindowEvent { event, .. } => match event {
                winit::event::WindowEvent::KeyboardInput {
                    input:
                        winit::event::KeyboardInput {
                            virtual_keycode: Some(winit::event::VirtualKeyCode::Escape),
                            ..
                        },
                    ..
                }
                | winit::event::WindowEvent::CloseRequested => {
                    *control_flow = glutin::event_loop::ControlFlow::Exit
                }
                winit::event::WindowEvent::Resized(physical_size) => {
                    context.resize(physical_size);
                    if let Some((new_color, _)) = context.updated_views_raw(
                        dimensions,
                        conrod_gfx::ColorFormat::get_format(),
                        DepthFormat::get_format(),
                    ) {
                        renderer.on_resize(gfx::memory::Typed::new(new_color));
                        dimensions = get_window_dimensions(&context);
                    }
                }
                _ => {}
            },
            _ => {}
        }

        // Update widgets if any event has happened
        if ui.global_input().events().next().is_some() {
            let mut ui = ui.set_widgets();
            user_input = conrod_example_shared::gui(&mut ui, &ids, &mut app, &mut user_input.0, &mut user_input.1);
        }
    });
}
