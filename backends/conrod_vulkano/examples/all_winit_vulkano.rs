//! A demonstration of using `winit` to provide events and `vulkano` to draw the UI.

#![allow(unused_variables)]

extern crate conrod_core;
extern crate conrod_example_shared;
extern crate conrod_vulkano;
#[macro_use]
extern crate conrod_winit;
extern crate find_folder;
extern crate image;
#[macro_use]
extern crate vulkano;
extern crate vulkano_win;
extern crate winit;

mod support;

use conrod_example_shared::{WIN_H, WIN_W};
use std::sync::Arc;
use vulkano::{
    command_buffer::AutoCommandBufferBuilder,
    format::{D16Unorm, Format},
    framebuffer::{Framebuffer, FramebufferAbstract, RenderPassAbstract},
    image::AttachmentImage,
    swapchain,
    swapchain::AcquireError,
    sync::{FenceSignalFuture, GpuFuture},
};

use conrod_vulkano::{Image as VulkanoGuiImage, Renderer};

type DepthFormat = D16Unorm;
const DEPTH_FORMAT_TY: DepthFormat = D16Unorm;
const DEPTH_FORMAT: Format = Format::D16Unorm;
const CLEAR_COLOR: [f32; 4] = [0.2, 0.2, 0.2, 1.0];

fn main() {
    let mut events_loop = winit::EventsLoop::new();
    let mut window = support::Window::new(WIN_W, WIN_H, "Conrod with vulkano", &events_loop);

    let mut render_target = RenderTarget::new(&window);

    let subpass = vulkano::framebuffer::Subpass::from(render_target.render_pass.clone(), 0)
        .expect("Couldn't create subpass for gui!");
    let queue = window.queue.clone();
    let mut renderer = Renderer::new(
        window.device.clone(),
        subpass,
        queue.family(),
        [WIN_W, WIN_H],
        window.surface.window().get_hidpi_factor() as f64,
    ).unwrap();

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
    let logo_dimensions = rgba_logo_image.dimensions();

    let (logo_texture, logo_texture_future) = vulkano::image::immutable::ImmutableImage::from_iter(
        rgba_logo_image.into_raw().clone().iter().cloned(),
        vulkano::image::Dimensions::Dim2d {
            width: logo_dimensions.0,
            height: logo_dimensions.1,
        },
        vulkano::format::R8G8B8A8Unorm,
        window.queue.clone(),
    )
    .expect("Couldn't create vulkan texture for logo");

    let logo = VulkanoGuiImage {
        image_access: logo_texture,
        width: logo_dimensions.0,
        height: logo_dimensions.1,
    };
    let mut image_map = conrod_core::image::Map::new();
    let rust_logo = image_map.insert(logo);

    // Demonstration app state that we'll control with our conrod GUI.
    let mut app = conrod_example_shared::DemoApp::new(rust_logo);

    // Keep track of the previous frame so we can wait for it to complete before presenting a new
    // one. This should make sure the CPU never gets ahead of the presentation of frames, which can
    // cause high user-input latency and synchronisation strange bugs.
    let mut previous_frame_end: Option<FenceSignalFuture<_>> = None;

    // Wait for the logo to load onto the GPU before we begin our main loop.
    logo_texture_future
        .then_signal_fence_and_flush()
        .expect("failed to signal fence and flush logo future")
        .wait(None)
        .expect("failed to wait for logo texture to load");

    'main: loop {
        // If the window is closed, this will be None for one tick, so to avoid panicking with
        // unwrap, instead break the loop
        let (win_w, win_h) = match window.get_dimensions() {
            Some(s) => s,
            None => break 'main,
        };

        if let Some(primitives) = ui.draw_if_changed() {
            let (image_num, acquire_future) =
                match swapchain::acquire_next_image(window.swapchain.clone(), None) {
                    Ok(r) => r,
                    Err(AcquireError::OutOfDate) => {
                        window.handle_resize();
                        render_target.handle_resize(&mut window);
                        continue;
                    }
                    Err(err) => panic!("{:?}", err),
                };

            let mut command_buffer_builder = AutoCommandBufferBuilder::primary_one_time_submit(
                window.device.clone(),
                window.queue.family(),
            )
            .expect("Failed to create AutoCommandBufferBuilder");

            let viewport = [0.0, 0.0, win_w as f32, win_h as f32];
            let dpi_factor = window.surface.window().get_hidpi_factor() as f64;
            if let Some(cmd) = renderer.fill(&image_map, viewport, dpi_factor, primitives).unwrap() {
                let buffer = cmd.glyph_cpu_buffer_pool
                    .chunk(cmd.glyph_cache_pixel_buffer.iter().cloned())
                    .unwrap();
                command_buffer_builder = command_buffer_builder
                    .copy_buffer_to_image(buffer, cmd.glyph_cache_texture)
                    .expect("failed to submit command for caching glyph");
            }

            let mut command_buffer_builder = command_buffer_builder
                .begin_render_pass(
                    render_target.framebuffers[image_num].clone(),
                    false,
                    vec![CLEAR_COLOR.into(), 1f32.into()],
                ) // Info: We need to clear background AND depth buffer here!
                .expect("Failed to begin render pass!");

            let draw_cmds = renderer.draw(
                window.queue.clone(),
                &image_map,
                [0.0, 0.0, win_w as f32, win_h as f32],
            ).unwrap();
            for cmd in draw_cmds {
                let conrod_vulkano::DrawCommand {
                    graphics_pipeline,
                    dynamic_state,
                    vertex_buffer,
                    descriptor_set,
                } = cmd;
                command_buffer_builder = command_buffer_builder
                    .draw(
                        graphics_pipeline,
                        &dynamic_state,
                        vec![vertex_buffer],
                        descriptor_set,
                        (),
                    )
                    .expect("failed to submit draw command");
            }

            let command_buffer = command_buffer_builder
                .end_render_pass()
                .unwrap()
                .build()
                .unwrap();

            // Wait for the previous frame to finish presentation.
            if let Some(prev_frame) = previous_frame_end.take() {
                prev_frame
                    .wait(None)
                    .expect("failed to wait for presentation of previous frame");
            }

            let future_result = acquire_future
                .then_execute(window.queue.clone(), command_buffer)
                .expect("failed to join previous frame with new one")
                .then_swapchain_present(window.queue.clone(), window.swapchain.clone(), image_num)
                .then_signal_fence_and_flush();

            // Hold onto the future representing the presentation of this frame.
            // We'll wait for it before we present the next one.
            if let Ok(future) = future_result {
                previous_frame_end = Some(future);
            }
        }

        let mut should_quit = false;

        events_loop.poll_events(|event| {
            // Convert winit event to conrod event.
            // Function generated by `conrod_winit::conversion_fns` in `support` module.
            if let Some(event) = support::convert_event(event.clone(), &window) {
                ui.handle_event(event);
            }

            // Close window if the escape key or the exit button is pressed
            match event {
                winit::Event::WindowEvent {
                    event:
                        winit::WindowEvent::KeyboardInput {
                            input:
                                winit::KeyboardInput {
                                    virtual_keycode: Some(winit::VirtualKeyCode::Escape),
                                    ..
                                },
                            ..
                        },
                    ..
                }
                | winit::Event::WindowEvent {
                    event: winit::WindowEvent::CloseRequested,
                    ..
                } => should_quit = true,
                _ => {}
            }
        });
        if should_quit {
            break 'main;
        }

        // Update widgets if any event has happened
        if ui.global_input().events().next().is_some() {
            let mut ui = ui.set_widgets();
            conrod_example_shared::gui(&mut ui, &ids, &mut app);
        }
    }
}

pub struct RenderTarget {
    depth_buffer: Arc<AttachmentImage<D16Unorm>>,
    render_pass: Arc<RenderPassAbstract + Send + Sync>,
    framebuffers: Vec<Arc<FramebufferAbstract + Send + Sync>>,
}

impl RenderTarget {
    pub fn new(window: &support::Window) -> Self {
        let (win_w, win_h) = window.get_dimensions().expect("couldn't get window dimensions");
        let win_dims = [win_w, win_h];
        let device = window.device.clone();
        let depth_buffer = AttachmentImage::transient(device, win_dims, DEPTH_FORMAT_TY).unwrap();

        let render_pass = Arc::new(
            single_pass_renderpass!(window.device.clone(),
                attachments: {
                    color: {
                        load: Clear,
                        store: Store,
                        format: window.swapchain.format(),
                        samples: 1,
                    },
                    depth: {
                        load: Clear,
                        store: DontCare,
                        format: DEPTH_FORMAT,
                        samples: 1,
                    }
                },
                pass: {
                    color: [color],
                    depth_stencil: {depth}
                }
            )
            .unwrap(),
        );

        let framebuffers = create_framebuffers(window, render_pass.clone(), depth_buffer.clone());

        RenderTarget {
            depth_buffer,
            framebuffers,
            render_pass,
        }
    }

    pub fn handle_resize(&mut self, window: &support::Window) {
        let [fb_w, fb_h, _] = self.framebuffers[0].dimensions();
        let (win_w, win_h) = window.get_dimensions().expect("couldn't get window dimensions");
        let win_dims = [win_w, win_h];
        let device = window.device.clone();
        if fb_w != win_w || fb_h != win_h {
            self.depth_buffer = AttachmentImage::transient(device, win_dims, DEPTH_FORMAT_TY)
                .unwrap();
            self.framebuffers = create_framebuffers(
                window,
                self.render_pass.clone(),
                self.depth_buffer.clone(),
            );
        }
    }
}

fn create_framebuffers(
    window: &support::Window,
    render_pass: Arc<RenderPassAbstract + Send + Sync>,
    depth_buffer: Arc<AttachmentImage<D16Unorm>>,
) -> Vec<Arc<FramebufferAbstract + Send + Sync>> {
    window
        .images
        .iter()
        .map(|image| {
            Arc::new(
                Framebuffer::start(render_pass.clone())
                    .add(image.clone())
                    .unwrap()
                    .add(depth_buffer.clone())
                    .unwrap()
                    .build()
                    .unwrap(),
            ) as Arc<_>
        })
        .collect()
}
