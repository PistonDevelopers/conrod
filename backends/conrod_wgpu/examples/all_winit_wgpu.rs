//! An example demonstrating the use of `conrod_wgpu` alongside `winit`.

use conrod_example_shared::{WIN_H, WIN_W};
use winit::{
    event,
    event_loop::{ControlFlow, EventLoop},
};

// Generate the winit <-> conrod_core type conversion fns.
conrod_winit::v021_conversion_fns!();

const LOGO_TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Bgra8UnormSrgb;
const MSAA_SAMPLES: u32 = 8;

fn main() {
    let event_loop = EventLoop::new();

    // Create the window and surface.
    #[cfg(not(feature = "gl"))]
    let (window, mut size, surface) = {
        let window = winit::window::WindowBuilder::new()
            .with_title("Conrod with wgpu")
            .with_inner_size(winit::dpi::LogicalSize {
                width: WIN_W,
                height: WIN_H,
            })
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
    let extensions = wgpu::Extensions {
        anisotropic_filtering: false,
    };
    let limits = wgpu::Limits::default();
    let device_desc = wgpu::DeviceDescriptor { extensions, limits };
    let (device, mut queue) = adapter.request_device(&device_desc);

    // Create the swapchain.
    let format = wgpu::TextureFormat::Bgra8UnormSrgb;
    let mut swap_chain_desc = wgpu::SwapChainDescriptor {
        usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
        format,
        width: size.width,
        height: size.height,
        present_mode: wgpu::PresentMode::Vsync,
    };
    let mut swap_chain = device.create_swap_chain(&surface, &swap_chain_desc);

    // Create the renderer for rendering conrod primitives.
    let mut renderer = conrod_wgpu::Renderer::new(&device, MSAA_SAMPLES, format);

    // The intermediary multisampled texture that will be resolved (MSAA).
    let mut multisampled_framebuffer =
        create_multisampled_framebuffer(&device, &swap_chain_desc, MSAA_SAMPLES);

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

    // Create the GPU texture and upload the image data.
    let (logo_w, logo_h) = rgba_logo_image.dimensions();
    let logo_tex = create_logo_texture(&device, &mut queue, rgba_logo_image);
    let logo = conrod_wgpu::Image {
        texture: logo_tex,
        texture_format: LOGO_TEXTURE_FORMAT,
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
            ControlFlow::Wait
        };
        match event {
            event::Event::WindowEvent { event, .. } => match event {
                // Recreate swapchain when window is resized.
                event::WindowEvent::Resized(new_size) => {
                    size = new_size;
                    swap_chain_desc.width = new_size.width;
                    swap_chain_desc.height = new_size.height;
                    swap_chain = device.create_swap_chain(&surface, &swap_chain_desc);
                    multisampled_framebuffer =
                        create_multisampled_framebuffer(&device, &swap_chain_desc, MSAA_SAMPLES);
                }

                // Close on request or on Escape.
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
                    window.request_redraw();
                }
            }

            event::Event::RedrawRequested(_) => {
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
                let [win_w, win_h]: [f32; 2] = [size.width as f32, size.height as f32];
                let viewport = [0.0, 0.0, win_w, win_h];
                if let Some(cmd) = renderer
                    .fill(&image_map, viewport, scale_factor, primitives)
                    .unwrap()
                {
                    cmd.load_buffer_and_encode(&device, &mut encoder);
                }

                // Begin the render pass and add the draw commands.
                {
                    // This condition allows to more easily tweak the MSAA_SAMPLES constant.
                    let (attachment, resolve_target) = match MSAA_SAMPLES {
                        1 => (&frame.view, None),
                        _ => (&multisampled_framebuffer, Some(&frame.view)),
                    };
                    let color_attachment_desc = wgpu::RenderPassColorAttachmentDescriptor {
                        attachment,
                        resolve_target,
                        load_op: wgpu::LoadOp::Clear,
                        store_op: wgpu::StoreOp::Store,
                        clear_color: wgpu::Color::BLACK,
                    };

                    let render_pass_desc = wgpu::RenderPassDescriptor {
                        color_attachments: &[color_attachment_desc],
                        depth_stencil_attachment: None,
                    };
                    let mut render_pass = encoder.begin_render_pass(&render_pass_desc);

                    let render = renderer.render(&device, &image_map);
                    render_pass.set_pipeline(render.pipeline);
                    render_pass.set_vertex_buffers(0, &[(&render.vertex_buffer, 0)]);
                    let instance_range = 0..1;
                    for cmd in render.commands {
                        match cmd {
                            conrod_wgpu::RenderPassCommand::SetBindGroup { bind_group } => {
                                render_pass.set_bind_group(0, bind_group, &[]);
                            }
                            conrod_wgpu::RenderPassCommand::SetScissor {
                                top_left,
                                dimensions,
                            } => {
                                let [x, y] = top_left;
                                let [w, h] = dimensions;
                                render_pass.set_scissor_rect(x, y, w, h);
                            }
                            conrod_wgpu::RenderPassCommand::Draw { vertex_range } => {
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

fn create_multisampled_framebuffer(
    device: &wgpu::Device,
    sc_desc: &wgpu::SwapChainDescriptor,
    sample_count: u32,
) -> wgpu::TextureView {
    let multisampled_texture_extent = wgpu::Extent3d {
        width: sc_desc.width,
        height: sc_desc.height,
        depth: 1,
    };
    let multisampled_frame_descriptor = &wgpu::TextureDescriptor {
        size: multisampled_texture_extent,
        array_layer_count: 1,
        mip_level_count: 1,
        sample_count: sample_count,
        dimension: wgpu::TextureDimension::D2,
        format: sc_desc.format,
        usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
    };
    device
        .create_texture(multisampled_frame_descriptor)
        .create_default_view()
}

fn create_logo_texture(
    device: &wgpu::Device,
    queue: &mut wgpu::Queue,
    image: image::RgbaImage,
) -> wgpu::Texture {
    // Initialise the texture.
    let (width, height) = image.dimensions();
    let logo_tex_extent = wgpu::Extent3d {
        width,
        height,
        depth: 1,
    };
    let logo_tex = device.create_texture(&wgpu::TextureDescriptor {
        size: logo_tex_extent,
        array_layer_count: 1,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: LOGO_TEXTURE_FORMAT,
        usage: wgpu::TextureUsage::SAMPLED | wgpu::TextureUsage::COPY_DST,
    });

    // Upload the pixel data.
    let data = &image.into_raw()[..];
    let buffer = device
        .create_buffer_mapped(data.len(), wgpu::BufferUsage::COPY_SRC)
        .fill_from_slice(data);

    // Submit command for copying pixel data to the texture.
    let pixel_size_bytes = 4; // Rgba8, as above.
    let buffer_copy_view = wgpu::BufferCopyView {
        buffer: &buffer,
        offset: 0,
        row_pitch: width * pixel_size_bytes,
        image_height: height,
    };
    let texture_copy_view = wgpu::TextureCopyView {
        texture: &logo_tex,
        mip_level: 0,
        array_layer: 0,
        origin: wgpu::Origin3d::ZERO,
    };
    let extent = wgpu::Extent3d {
        width: width,
        height: height,
        depth: 1,
    };
    let cmd_encoder_desc = wgpu::CommandEncoderDescriptor { todo: 0 };
    let mut encoder = device.create_command_encoder(&cmd_encoder_desc);
    encoder.copy_buffer_to_texture(buffer_copy_view, texture_copy_view, extent);
    queue.submit(&[encoder.finish()]);

    logo_tex
}
