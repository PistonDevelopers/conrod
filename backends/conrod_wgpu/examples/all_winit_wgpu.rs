//! An example demonstrating the use of `conrod_wgpu` alongside `winit`.

use conrod_example_shared::{WIN_H, WIN_W};
use winit::{
    event,
    event_loop::{ControlFlow, EventLoop},
};

// Generate the winit <-> conrod_core type conversion fns.
conrod_winit::v023_conversion_fns!();

const LOGO_TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Bgra8UnormSrgb;
const MSAA_SAMPLES: u32 = 4;

fn main() {
    let event_loop = EventLoop::new();

    let backends = wgpu::BackendBit::PRIMARY;
    let instance = wgpu::Instance::new(backends);

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
        let surface = unsafe { instance.create_surface(&window) };
        (window, size, surface)
    };

    // Select an adapter and gpu device.
    let adapter_opts = wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::Default,
        compatible_surface: Some(&surface),
    };
    let adapter = futures::executor::block_on(instance.request_adapter(&adapter_opts)).unwrap();
    let limits = wgpu::Limits::default();
    let device_desc = wgpu::DeviceDescriptor {
        features: wgpu::Features::empty(),
        limits,
        shader_validation: true,
    };
    let device_request = adapter.request_device(&device_desc, None);
    let (device, mut queue) = futures::executor::block_on(device_request).unwrap();

    // Create the swapchain.
    let format = wgpu::TextureFormat::Bgra8UnormSrgb;
    let mut swap_chain_desc = wgpu::SwapChainDescriptor {
        usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
        format,
        width: size.width,
        height: size.height,
        present_mode: wgpu::PresentMode::Fifo,
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

    let sixteen_ms = std::time::Duration::from_millis(16);
    let mut next_update = None;
    let mut ui_update_needed = false;
    event_loop.run(move |event, _, control_flow| {
        if let Some(event) = convert_event(&event, &window) {
            ui.handle_event(event);
            ui_update_needed = true;
        }

        match &event {
            event::Event::WindowEvent { event, .. } => match event {
                // Recreate swapchain when window is resized.
                event::WindowEvent::Resized(new_size) => {
                    size = *new_size;
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
                    return;
                }
                _ => {}
            },
            _ => {}
        }

        // We don't want to draw any faster than 60 FPS, so set the UI only on every 16ms, unless:
        // - this is the very first event, or
        // - we didn't request update on the last event and new events have arrived since then.
        let should_set_ui_on_main_events_cleared = next_update.is_none() && ui_update_needed;
        match (&event, should_set_ui_on_main_events_cleared) {
            (event::Event::NewEvents(event::StartCause::Init { .. }), _)
            | (event::Event::NewEvents(event::StartCause::ResumeTimeReached { .. }), _)
            | (event::Event::MainEventsCleared, true) => {
                next_update = Some(std::time::Instant::now() + sixteen_ms);
                ui_update_needed = false;

                // Instantiate a GUI demonstrating every widget type provided by conrod.
                conrod_example_shared::gui(&mut ui.set_widgets(), &ids, &mut app);

                if ui.has_changed() {
                    // If the view has changed at all, request a redraw.
                    window.request_redraw();
                } else {
                    // We don't need to update the UI anymore until more events arrives.
                    next_update = None;
                }
            }
            _ => (),
        }
        if let Some(next_update) = next_update {
            *control_flow = ControlFlow::WaitUntil(next_update);
        } else {
            *control_flow = ControlFlow::Wait;
        }

        match &event {
            event::Event::RedrawRequested(_) => {
                let primitives = ui.draw();

                // The window frame that we will draw to.
                let frame = swap_chain.get_current_frame().unwrap();

                // Begin encoding commands.
                let cmd_encoder_desc = wgpu::CommandEncoderDescriptor {
                    label: Some("conrod_command_encoder"),
                };
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
                        1 => (&frame.output.view, None),
                        _ => (&multisampled_framebuffer, Some(&frame.output.view)),
                    };
                    let color_attachment_desc = wgpu::RenderPassColorAttachmentDescriptor {
                        attachment,
                        resolve_target,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                            store: true,
                        },
                    };

                    let render_pass_desc = wgpu::RenderPassDescriptor {
                        color_attachments: &[color_attachment_desc],
                        depth_stencil_attachment: None,
                    };
                    let render = renderer.render(&device, &image_map);

                    {
                        let mut render_pass = encoder.begin_render_pass(&render_pass_desc);
                        let slot = 0;
                        render_pass.set_vertex_buffer(slot, render.vertex_buffer.slice(..));
                        let instance_range = 0..1;
                        for cmd in render.commands {
                            match cmd {
                                conrod_wgpu::RenderPassCommand::SetPipeline { pipeline } => {
                                    render_pass.set_pipeline(pipeline);
                                }
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
                }

                queue.submit(Some(encoder.finish()));
            }
            _ => {}
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
        label: Some("conrod_msaa_texture"),
        size: multisampled_texture_extent,
        mip_level_count: 1,
        sample_count: sample_count,
        dimension: wgpu::TextureDimension::D2,
        format: sc_desc.format,
        usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
    };
    device
        .create_texture(multisampled_frame_descriptor)
        .create_view(&wgpu::TextureViewDescriptor::default())
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
        label: Some("conrod_rust_logo_texture"),
        size: logo_tex_extent,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: LOGO_TEXTURE_FORMAT,
        usage: wgpu::TextureUsage::SAMPLED | wgpu::TextureUsage::COPY_DST,
    });

    // Upload the pixel data.
    let data = &image.into_raw()[..];

    // Submit command for copying pixel data to the texture.
    let pixel_size_bytes = 4; // Rgba8, as above.
    let data_layout = wgpu::TextureDataLayout {
        offset: 0,
        bytes_per_row: width * pixel_size_bytes,
        rows_per_image: height,
    };
    let texture_copy_view = wgpu::TextureCopyView {
        texture: &logo_tex,
        mip_level: 0,
        origin: wgpu::Origin3d::ZERO,
    };
    let extent = wgpu::Extent3d {
        width: width,
        height: height,
        depth: 1,
    };
    let cmd_encoder_desc = wgpu::CommandEncoderDescriptor {
        label: Some("conrod_upload_image_command_encoder"),
    };
    let encoder = device.create_command_encoder(&cmd_encoder_desc);
    queue.write_texture(texture_copy_view, data, data_layout, extent);
    queue.submit(Some(encoder.finish()));

    logo_tex
}
