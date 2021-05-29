//! A demonstration of using `winit` to provide events and `vulkano` to draw the UI.
use conrod_example_shared::{WIN_H, WIN_W};

use vulkano::image::{AttachmentImage, MipmapsCount};

use vulkano::swapchain::{AcquireError, SwapchainAcquireFuture};

use conrod_vulkano::Image as VulkanoGuiImage;
use conrod_vulkano::Renderer;
use conrod_winit::WinitWindow;
use std::sync::Arc;
use vulkano::command_buffer::{
    AutoCommandBufferBuilder, CommandBufferExecError, CommandBufferExecFuture, CommandBufferUsage,
    PrimaryAutoCommandBuffer, SubpassContents,
};
use vulkano::format::Format;
use vulkano::image::view::ImageView;

use vulkano::render_pass::Framebuffer;
use vulkano::render_pass::{FramebufferAbstract, RenderPass};
use vulkano::single_pass_renderpass;
use vulkano::sync::{FenceSignalFuture, GpuFuture};
use winit::event;

use vulkano::command_buffer::pool::standard::{
    StandardCommandPoolAlloc, StandardCommandPoolBuilder,
};
use vulkano::descriptor::pipeline_layout::PipelineLayout;
use winit::event::{ElementState, KeyboardInput, VirtualKeyCode};
use winit::event_loop::ControlFlow;
use winit::window::Window;

conrod_winit::v023_conversion_fns!();
mod support;
const DEPTH_FORMAT: Format = Format::D16Unorm;
const CLEAR_COLOR: [f32; 4] = [0.2, 0.2, 0.2, 1.0];
fn main() {
    let event_loop = winit::event_loop::EventLoop::new();
    let mut window = support::Window::new(WIN_W, WIN_H, "Conrod with vulkno", &event_loop);
    let mut render_target = RenderTarget::new(&window);
    let subpass = vulkano::render_pass::Subpass::from(render_target.render_pass.clone(), 0)
        .expect("Couldn't create subpass for gui!");
    let queue = window.queue.clone();
    let mut renderer = Renderer::new(
        window.device.clone(),
        subpass,
        queue.family(),
        [WIN_W, WIN_H],
        window.surface.window().scale_factor(),
    )
    .unwrap();
    let mut ui = conrod_core::UiBuilder::new([WIN_W as f64, WIN_H as f64])
        .theme(conrod_example_shared::theme())
        .build();
    let ids = conrod_example_shared::Ids::new(ui.widget_id_generator());
    let assets = find_folder::Search::KidsThenParents(3, 5)
        .for_folder("assets")
        .unwrap();
    let font_path = assets.join("fonts/NotoSans/NotoSans-Regular.ttf");
    ui.fonts.insert_from_file(font_path).unwrap();
    let logo_path = assets.join("images/rust.png");
    let rgba_logo_image = image::open(logo_path)
        .expect("Couldn't load logo")
        .to_rgba8();
    let logo_dimensions = rgba_logo_image.dimensions();
    let (logo_texture, logo_texture_future) = vulkano::image::immutable::ImmutableImage::from_iter(
        rgba_logo_image.into_raw().clone().iter().cloned(),
        vulkano::image::ImageDimensions::Dim2d {
            width: logo_dimensions.0,
            height: logo_dimensions.1,
            array_layers: 1,
        },
        MipmapsCount::One,
        vulkano::format::Format::R8G8B8A8Srgb,
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

    logo_texture_future
        .then_signal_fence_and_flush()
        .expect("failed to signal fence and flush logo future")
        .wait(None)
        .expect("failed to wait for logo texture to load");

    event_loop.run(move |event, _, control_flow| {
        if let Some(event) = convert_event(&event, window.surface.window()) {
            ui.handle_event(event);
        }
        match &event {
            // Recreate swapchain when window is resized.
            event::Event::WindowEvent { event, .. } => match event {
                event::WindowEvent::Resized(_new_size) => {
                    window.handle_resize();
                    render_target.handle_resize(&window);
                }
                event::WindowEvent::CloseRequested
                | event::WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            state: ElementState::Pressed,
                            virtual_keycode: Some(VirtualKeyCode::Escape),
                            ..
                        },
                    ..
                } => {
                    *control_flow = ControlFlow::Exit;
                    return;
                }
                _ => {}
            },
            _ => {}
        }

        let (win_w, win_h) = match window.get_dimensions() {
            Some(s) => s,
            None => return,
        };
        // Update widgets if any event has happened
        if ui.global_input().events().next().is_some() {
            let mut ui = ui.set_widgets();
            conrod_example_shared::gui(&mut ui, &ids, &mut app);
        }
        match &event {
            event::Event::RedrawRequested(_) => {
                let primitives = ui.draw();
                let (image_num, sub_optimal, acquire_future) =
                    match vulkano::swapchain::acquire_next_image(window.swapchain.clone(), None) {
                        Ok(r) => r,
                        Err(AcquireError::OutOfDate) => {
                            window.handle_resize();
                            render_target.handle_resize(&mut window);
                            return;
                        }
                        Err(err) => panic!("{:?}", err),
                    };
                if sub_optimal {
                    return;
                }
                println!("bake to image_num {}", image_num);

                {
                    let mut command_buffer_builder = AutoCommandBufferBuilder::primary(
                        window.device.clone(),
                        window.queue.family(),
                        CommandBufferUsage::OneTimeSubmit,
                    )
                    .expect("Failed to create AutoCommandBufferBuilder");
                    let viewport = [0.0, 0.0, win_w as f32, win_h as f32];

                    let dpi_factor = window.hidpi_factor() as f64;
                    if let Some(cmd) = renderer
                        .fill(&image_map, viewport, dpi_factor, primitives)
                        .unwrap()
                    {
                        let buffer = cmd
                            .glyph_cpu_buffer_pool
                            .chunk(cmd.glyph_cache_pixel_buffer.iter().cloned())
                            .unwrap();
                        command_buffer_builder
                            .copy_buffer_to_image(buffer, cmd.glyph_cache_texture)
                            .expect("failed to submit command for caching glyph");
                    }
                    command_buffer_builder
                        .begin_render_pass(
                            render_target.framebuffers[image_num].clone(),
                            SubpassContents::Inline,
                            vec![CLEAR_COLOR.into(), 1f32.into()],
                        )
                        .unwrap();
                    let draw_cmds = renderer
                        .draw(window.queue.clone(), &image_map, viewport)
                        .unwrap();
                    for cmd in draw_cmds {
                        let conrod_vulkano::DrawCommand {
                            graphics_pipeline,
                            dynamic_state,
                            vertex_buffer,
                            descriptor_set,
                        } = cmd;
                        command_buffer_builder
                            .draw(
                                graphics_pipeline,
                                &dynamic_state,
                                vec![vertex_buffer],
                                descriptor_set,
                                (),
                                vec![],
                            )
                            .expect("failed to submit draw command");
                    }
                    command_buffer_builder.end_render_pass().unwrap();
                    let command_buffer = command_buffer_builder.build().unwrap();
                    if let Ok(future) =
                        acquire_future.then_execute(window.queue.clone(), command_buffer)
                    {
                        let _ = future
                            .then_swapchain_present(
                                window.queue.clone(),
                                window.swapchain.clone(),
                                image_num,
                            )
                            .then_signal_fence_and_flush()
                            .and_then(|future| future.wait(None));
                    }
                }
            }
            event::Event::RedrawEventsCleared => {
                //    window.surface.window().request_redraw();
            }
            _ => {}
        }
    });
}
pub struct RenderTarget {
    depth_buffer: Arc<AttachmentImage>,
    render_pass: Arc<RenderPass>,
    framebuffers: Vec<Arc<dyn FramebufferAbstract + Send + Sync>>,
}

impl RenderTarget {
    pub fn new(window: &support::Window) -> Self {
        let (win_w, win_h) = window
            .get_dimensions()
            .expect("couldn't get window dimensions");
        let win_dims = [win_w, win_h];
        let device = window.device.clone();
        let depth_buffer = AttachmentImage::transient(device, win_dims, DEPTH_FORMAT).unwrap();

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
        let (win_w, win_h) = window
            .get_dimensions()
            .expect("couldn't get window dimensions");
        let win_dims = [win_w, win_h];
        let device = window.device.clone();
        if fb_w != win_w || fb_h != win_h {
            self.depth_buffer = AttachmentImage::transient(device, win_dims, DEPTH_FORMAT).unwrap();
            self.framebuffers =
                create_framebuffers(window, self.render_pass.clone(), self.depth_buffer.clone());
        }
    }
}

fn create_framebuffers(
    window: &support::Window,
    render_pass: Arc<RenderPass>,
    depth_buffer: Arc<AttachmentImage>,
) -> Vec<Arc<dyn FramebufferAbstract + Send + Sync>> {
    let depth_buffer = ImageView::new(depth_buffer.clone()).unwrap();
    window
        .images
        .iter()
        .map(|image| {
            Arc::new(
                Framebuffer::start(render_pass.clone())
                    .add(ImageView::new(image.clone()).unwrap())
                    .unwrap()
                    .add(depth_buffer.clone())
                    .unwrap()
                    .build()
                    .unwrap(),
            ) as Arc<_>
        })
        .collect()
}
