//! An example demonstrating the use of `conrod_wgpu` alongside `winit`.

use conrod_example_shared::{WIN_H, WIN_W};
use winit::{
    event,
    event_loop::{ControlFlow, EventLoop}
};

// A wrapper around the winit window that allows us to implement the trait necessary for enabling
// the winit <-> conrod conversion functions.
struct WindowRef<'a>(&'a winit::window::Window);

// Implement the `WinitWindow` trait for `WindowRef` to allow for generating compatible conversion
// functions.
impl<'a> conrod_winit::WinitWindow for WindowRef<'a> {
    fn get_inner_size(&self) -> Option<(u32, u32)> {
        Some(winit::window::Window::inner_size(&self.0).into())
    }
    fn hidpi_factor(&self) -> f32 {
        winit::window::Window::scale_factor(&self.0) as _
    }
}

// Generate the winit <-> conrod_core type conversion fns.
conrod_winit::v021_conversion_fns!();

fn main() {
    let event_loop = EventLoop::new();

    // Create the window and surface.
    #[cfg(not(feature = "gl"))]
    let (window, size, surface) = {
        let window = winit::window::WindowBuilder::new()
            .with_inner_size(winit::dpi::LogicalSize { width: WIN_W, height: WIN_H })
            .build(&event_loop)
            .unwrap();
        let size = window.inner_size();
        let surface = wgpu::Surface::create(&window);
        (window, size, surface)
    };

    // Select an adapter and gpu device.
    let adapter_opts = wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::Default,
        backends: wgpu::BackendBit::PRIMARY,
    };
    let adapter = wgpu::Adapter::request(&adapter_opts).unwrap();
    let extensions = wgpu::Extensions { anisotropic_filtering: false };
    let limits = wgpu::Limits::default();
    let device_desc = wgpu::DeviceDescriptor { extensions, limits };
    let (device, mut queue) = adapter.request_device(&device_desc);

    // Create the renderer for rendering conrod primitives.
    let mut renderer = conrod_wgpu::Renderer::new(&device);

    // Create the swapchain.
    let swap_chain_desc = wgpu::SwapChainDescriptor {
        usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
        format: wgpu::TextureFormat::Bgra8UnormSrgb,
        width: size.width,
        height: size.height,
        present_mode: wgpu::PresentMode::Vsync,
    };
    let mut swap_chain = device.create_swap_chain(&surface, &swap_chain_desc);

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
    let logo_path = assets.join("images/rust.png");
    let rgba_logo_image = image::open(logo_path)
        .expect("Couldn't load logo")
        .to_rgba();
    let (logo_w, logo_h) = rgba_logo_image.dimensions();
    let logo_tex_extent = wgpu::Extent3d {
        width: logo_w,
        height: logo_h,
        depth: 1,
    };
    let logo_tex = device.create_texture(&wgpu::TextureDescriptor {
        size: logo_tex_extent,
        array_layer_count: 1,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsage::SAMPLED | wgpu::TextureUsage::COPY_DST,
    });
    let logo = conrod_wgpu::Image {
        texture: logo_tex,
        width: logo_w,
        height: logo_h,
    };
    let mut image_map = conrod_core::image::Map::new();
    let rust_logo = image_map.insert(logo);

    // Demonstration app state that we'll control with our conrod GUI.
    let mut app = conrod_example_shared::DemoApp::new(rust_logo);

    event_loop.run(move |event, _, control_flow| {
        if let Some(event) = convert_event(&event, &window) {
            ui.handle_event(event);
        }

        *control_flow = if cfg!(feature = "metal-auto-capture") {
            ControlFlow::Exit
        } else {
            ControlFlow::Poll
        };
        match event {
            event::Event::WindowEvent { event, .. } => match event {
                event::WindowEvent::KeyboardInput {
                    input:
                        event::KeyboardInput {
                            virtual_keycode: Some(event::VirtualKeyCode::Escape),
                            state: event::ElementState::Pressed,
                            ..
                        },
                    ..
                }
                | event::WindowEvent::CloseRequested => {
                    *control_flow = ControlFlow::Exit;
                }
                _ => {}
            },
            event::Event::MainEventsCleared => {
                // Update widgets if any event has happened
                if ui.global_input().events().next().is_some() {
                    let mut ui = ui.set_widgets();
                    conrod_example_shared::gui(&mut ui, &ids, &mut app);
                }

                // If the view has changed at all, it's time to draw.
                let primitives = match ui.draw_if_changed() {
                    None => return,
                    Some(ps) => ps,
                };

                // The window frame that we will draw to.
                let frame = swap_chain.get_next_texture();

                // Begin encoding commands.
                let cmd_encoder_desc = wgpu::CommandEncoderDescriptor { todo: 0 };
                let mut encoder = device.create_command_encoder(&cmd_encoder_desc);

                // Feed the renderer primitives and update glyph cache texture if necessary.
                let scale_factor = window.scale_factor();
                let [win_w, win_h]: [f32; 2] = window.inner_size().to_logical::<f32>(scale_factor).into();
                let viewport = [0.0, 0.0, win_w, win_h];
                if let Some(cmd) = renderer
                    .fill(&image_map, viewport, scale_factor, primitives)
                    .unwrap()
                {
                    cmd.load_buffer_and_encode(&device, &mut encoder);
                }

                // Begin the render pass and add the draw commands.
                {
                    let color_attachment_desc = wgpu::RenderPassColorAttachmentDescriptor {
                        attachment: &frame.view,
                        resolve_target: None,
                        load_op: wgpu::LoadOp::Clear,
                        store_op: wgpu::StoreOp::Store,
                        clear_color: wgpu::Color::BLACK,
                    };
                    let render_pass_desc = wgpu::RenderPassDescriptor {
                        color_attachments: &[color_attachment_desc],
                        depth_stencil_attachment: None,
                    };
                    let mut render_pass = encoder.begin_render_pass(&render_pass_desc);

                    let render = renderer.render(&device, &image_map, viewport);
                    render_pass.set_pipeline(render.pipeline);
                    render_pass.set_vertex_buffers(0, &[(&render.vertex_buffer, 0)]);
                    render_pass.set_bind_group(0, &render.bind_group, &[]);
                    let instance_range = 0..1;
                    for cmd in render.commands {
                        match cmd {
                            conrod_wgpu::RenderPassCommand::Scissor { top_left, dimensions } => {
                                let [x, y] = top_left;
                                let [w, h] = dimensions;
                                render_pass.set_scissor_rect(x, y, w, h);
                            }
                            conrod_wgpu::RenderPassCommand::Draw {
                                vertex_range,
                            } => {
                                render_pass.draw(vertex_range, instance_range.clone());
                            }
                        }
                    }
                }

                queue.submit(&[encoder.finish()]);
            }
            _ => (),
        }
    });
}
