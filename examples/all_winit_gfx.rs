//! A demonstration of using `winit` to provide events and GFX to draw the UI.
//!
//! `winit` is used via the `glutin` crate which also provides an OpenGL context for drawing
//! `conrod::render::Primitives` to the screen.

#![allow(unused_variables)]

#[cfg(feature="winit")] #[macro_use] extern crate conrod;
#[cfg(feature="winit")] extern crate glutin;
#[cfg(feature="winit")] extern crate winit;
#[macro_use] extern crate gfx;
extern crate gfx_core;

#[cfg(feature="winit")]
mod support;


fn main() {
    feature::main();
}

#[cfg(feature="winit")]
mod feature {
    extern crate gfx_window_glutin;
    extern crate find_folder;

    use conrod;
    use glutin;
    use gfx;
    use support;
    use winit;

    use glutin::GlContext;
    use gfx::{Factory, Device, texture};
    use gfx::traits::FactoryExt;
    use conrod::render;
    use conrod::text::rt;

    const FRAGMENT_SHADER: &'static [u8] = b"
        #version 140

        uniform sampler2D t_Color;

        in vec2 v_Uv;
        in vec4 v_Color;

        out vec4 f_Color;

        void main() {
            vec4 tex = texture(t_Color, v_Uv);
            f_Color = v_Color * tex;
        }
    ";

    const VERTEX_SHADER: &'static [u8] = b"
        #version 140

        in vec2 a_Pos;
        in vec2 a_Uv;
        in vec4 a_Color;

        out vec2 v_Uv;
        out vec4 v_Color;

        void main() {
            v_Uv = a_Uv;
            v_Color = a_Color;
            gl_Position = vec4(a_Pos, 0.0, 1.0);
        }
    ";

    // Format definitions (must be pub for  gfx_defines to use them)
    pub type ColorFormat = gfx::format::Srgba8;
    type DepthFormat = gfx::format::DepthStencil;
    type SurfaceFormat = gfx::format::R8_G8_B8_A8;
    type FullFormat = (SurfaceFormat, gfx::format::Unorm);

    // Vertex and pipeline declarations
    gfx_defines! {
        vertex Vertex {
            pos: [f32; 2] = "a_Pos",
            uv: [f32; 2] = "a_Uv",
            color: [f32; 4] = "a_Color",
        }

        pipeline pipe {
            vbuf: gfx::VertexBuffer<Vertex> = (),
            color: gfx::TextureSampler<[f32; 4]> = "t_Color",
            out: gfx::BlendTarget<ColorFormat> = ("f_Color", ::gfx::state::MASK_ALL, ::gfx::preset::blend::ALPHA),
        }
    }

    // Convenience constructor
    impl Vertex {
        fn new(pos: [f32; 2], uv: [f32; 2], color: [f32; 4]) -> Vertex {
            Vertex {
                pos: pos,
                uv: uv,
                color: color,
            }
        }
    }

    const WIN_W: u32 = support::WIN_W;
    const WIN_H: u32 = support::WIN_H;
    const CLEAR_COLOR: [f32; 4] = [0.2, 0.2, 0.2, 1.0];

    fn gamma_srgb_to_linear(c: [f32; 4]) -> [f32; 4] {
        fn component(f: f32) -> f32 {
            // Taken from https://github.com/PistonDevelopers/graphics/src/color.rs#L42
            if f <= 0.04045 {
                f / 12.92
            } else {
                ((f + 0.055) / 1.055).powf(2.4)
            }
        }
        [component(c[0]), component(c[1]), component(c[2]), c[3]]
    }

    // Creates a gfx texture with the given data
    fn create_texture<F, R>(factory: &mut F, width: u32, height: u32, data: &[u8])
        -> (gfx::handle::Texture<R, SurfaceFormat>, gfx::handle::ShaderResourceView<R, [f32; 4]>)

        where R: gfx::Resources, F: gfx::Factory<R>
    {
        // Modified `Factory::create_texture_immutable_u8` for dynamic texture.
        fn create_texture<T, F, R>(
            factory: &mut F,
            kind: gfx::texture::Kind,
            data: &[&[u8]]
        ) -> Result<(
            gfx::handle::Texture<R, T::Surface>,
            gfx::handle::ShaderResourceView<R, T::View>
        ), gfx::CombinedError>
            where F: gfx::Factory<R>,
                  R: gfx::Resources,
                  T: gfx::format::TextureFormat
        {
            use gfx::{format, texture};
            use gfx::memory::{Usage, SHADER_RESOURCE};
            use gfx_core::memory::Typed;

            let surface = <T::Surface as format::SurfaceTyped>::get_surface_type();
            let num_slices = kind.get_num_slices().unwrap_or(1) as usize;
            let num_faces = if kind.is_cube() {6} else {1};
            let desc = texture::Info {
                kind: kind,
                levels: (data.len() / (num_slices * num_faces)) as texture::Level,
                format: surface,
                bind: SHADER_RESOURCE,
                usage: Usage::Dynamic,
            };
            let cty = <T::Channel as format::ChannelTyped>::get_channel_type();
            let raw = try!(factory.create_texture_raw(desc, Some(cty), Some(data)));
            let levels = (0, raw.get_info().levels - 1);
            let tex = Typed::new(raw);
            let view = try!(factory.view_texture_as_shader_resource::<T>(
                &tex, levels, format::Swizzle::new()
            ));
            Ok((tex, view))
        }

        let kind = texture::Kind::D2(
            width as texture::Size,
            height as texture::Size,
            texture::AaMode::Single
        );
        create_texture::<ColorFormat, F, R>(factory, kind, &[data]).unwrap()
    }

    // Updates a texture with the given data (used for updating the GlyphCache texture)
    fn update_texture<R, C>(encoder: &mut gfx::Encoder<R, C>,
                            texture: &gfx::handle::Texture<R, SurfaceFormat>,
                            offset: [u16; 2],
                            size: [u16; 2],
                            data: &[[u8; 4]])

        where R: gfx::Resources, C: gfx::CommandBuffer<R>
    {
        let info = texture::ImageInfoCommon {
                xoffset: offset[0],
                yoffset: offset[1],
                zoffset: 0,
                width: size[0],
                height: size[1],
                depth: 0,
                format: (),
                mipmap: 0,
        };

        encoder.update_texture::<SurfaceFormat, FullFormat>(texture, None, info, data).unwrap();
    }

    pub fn main() {
        // Builder for window
        let builder = glutin::WindowBuilder::new()
            .with_title("Conrod with GFX and Glutin")
            .with_dimensions(WIN_W, WIN_H);

        let context = glutin::ContextBuilder::new()
            .with_multisampling(4);

        let mut events_loop = winit::EventsLoop::new();

        // Initialize gfx things
        let (window, mut device, mut factory, rtv, _) =
            gfx_window_glutin::init::<ColorFormat, DepthFormat>(builder, context, &events_loop );
        let mut encoder: gfx::Encoder<_, _> = factory.create_command_buffer().into();

        let mut renderer = conrod::backend::gfx::Renderer::new(&mut factory, &rtv, window.hidpi_factor()).unwrap();

        // Create Ui and Ids of widgets to instantiate
        let mut ui = conrod::UiBuilder::new([WIN_W as f64, WIN_H as f64]).theme(support::theme()).build();
        let ids = support::Ids::new(ui.widget_id_generator());

        // Load font from file
        let assets = find_folder::Search::KidsThenParents(3, 5).for_folder("assets").unwrap();
        let font_path = assets.join("fonts/NotoSans/NotoSans-Regular.ttf");
        ui.fonts.insert_from_file(font_path).unwrap();

        // FIXME: We don't yet load the rust logo, so just insert nothing for now so we can get an
        // identifier used to construct the DemoApp. This should be changed to *actually* load a
        // gfx texture for the rust logo and insert it into the map.
        let mut image_map = conrod::image::Map::new();
        let rust_logo = image_map.insert(());

        // Demonstration app state that we'll control with our conrod GUI.
        let mut app = support::DemoApp::new(rust_logo);

        'main: loop {
            // If the window is closed, this will be None for one tick, so to avoid panicking with
            // unwrap, instead break the loop
            let (win_w, win_h) = match window.get_inner_size() {
                Some(s) => s,
                None => break 'main,
            };

            let dpi_factor = window.hidpi_factor();

            if let Some(primitives) = ui.draw_if_changed() {
                let dims = (win_w as f32 * dpi_factor, win_h as f32 * dpi_factor);

                //Clear the window
                encoder.clear(&self.rtv, CLEAR_COLOR);

                renderer.draw(&mut factory,&mut encoder,&mut device,primitives,dims);

                encoder.flush(&mut device);
                window.swap_buffers().unwrap();
                device.cleanup();
            }

            let mut should_quit = false;
            events_loop.poll_events(|event|{
                let (w, h) = (win_w as conrod::Scalar, win_h as conrod::Scalar);
                let dpi_factor = dpi_factor as conrod::Scalar;

                // Convert winit event to conrod event, requires conrod to be built with the `winit` feature
                if let Some(event) = conrod::backend::winit::convert_event(event.clone(), window.window()) {
                    ui.handle_event(event);
                }

                // Close window if the escape key or the exit button is pressed
                match event {
                    winit::Event::WindowEvent{event: winit::WindowEvent::KeyboardInput{input: winit::KeyboardInput{virtual_keycode: Some(winit::VirtualKeyCode::Escape),..}, ..}, .. } |
                    winit::Event::WindowEvent{event: winit::WindowEvent::Closed, ..} =>
                        should_quit = true,
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

#[cfg(not(feature="winit"))]
mod feature {
    pub fn main() {
        println!("This example requires the `winit` feature. \
                 Try running `cargo run --release --no-default-features --features=\"winit\" --example <example_name>`");
   }
}
