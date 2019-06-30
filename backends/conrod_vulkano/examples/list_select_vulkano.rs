//! A demonstration of using `winit` to provide events and `vulkano` to draw the UI.

#![allow(unused_variables)]

#[macro_use]
extern crate conrod_core;
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

use std::collections::HashSet;
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

use conrod_vulkano::Renderer;

type DepthFormat = D16Unorm;
const DEPTH_FORMAT_TY: DepthFormat = D16Unorm;
const DEPTH_FORMAT: Format = Format::D16Unorm;
const CLEAR_COLOR: [f32; 4] = [0.2, 0.2, 0.2, 1.0];
const WIN_W: u32 = 600;
const WIN_H: u32 = 300;

widget_ids! {
    struct Ids { canvas, list_select }
}

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
    let mut ui = conrod_core::UiBuilder::new([WIN_W as f64, WIN_H as f64]).build();
    let ids = Ids::new(ui.widget_id_generator());

    // Load font from file
    let assets = find_folder::Search::KidsThenParents(3, 5)
        .for_folder("assets")
        .unwrap();
    let font_path = assets.join("fonts/NotoSans/NotoSans-Regular.ttf");
    ui.fonts.insert_from_file(font_path).unwrap();

    let image_map = conrod_core::image::Map::new();

    // Keep track of the previous frame so we can wait for it to complete before presenting a new
    // one. This should make sure the CPU never gets ahead of the presentation of frames, which can
    // cause high user-input latency and synchronisation strange bugs.
    let mut previous_frame_end: Option<FenceSignalFuture<_>> = None;

    // List of entries to display. They should implement the Display trait.
    let list_items = [
        "African Sideneck Turtle".to_string(),
        "Alligator Snapping Turtle".to_string(),
        "Common Snapping Turtle".to_string(),
        "Indian Peacock Softshelled Turtle".to_string(),
        "Eastern River Cooter".to_string(),
        "Eastern Snake Necked Turtle".to_string(),
        "Diamond Terrapin".to_string(),
        "Indian Peacock Softshelled Turtle".to_string(),
        "Musk Turtle".to_string(),
        "Reeves Turtle".to_string(),
        "Eastern Spiny Softshell Turtle".to_string(),
        "Red Ear Slider Turtle".to_string(),
        "Indian Tent Turtle".to_string(),
        "Mud Turtle".to_string(),
        "Painted Turtle".to_string(),
        "Spotted Turtle".to_string()
    ];

    // List of selections, should be same length as list of entries. Will be updated by the widget.
    let mut list_selected = std::collections::HashSet::new();

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
            let (w, h) = (win_w as conrod_core::Scalar, win_h as conrod_core::Scalar);
            //let dpi_factor = dpi_factor as conrod_core::Scalar;

            // Convert winit event to conrod event, requires conrod to be built with the `winit`
            // feature
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
            gui(&mut ui, &ids, &list_items[..], &mut list_selected);
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

fn gui(
    ui: &mut conrod_core::UiCell,
    ids: &Ids,
    list_items: &[String],
    list_selected: &mut HashSet<usize>,
) {
    use conrod_core::{widget, Borderable, Colorable, Labelable, Positionable, Sizeable, Widget};

    widget::Canvas::new().color(conrod_core::color::BLUE).set(ids.canvas, ui);

    // Instantiate the `ListSelect` widget.
    let num_items = list_items.len();
    let item_h = 30.0;
    let font_size = item_h as conrod_core::FontSize / 2;
    let (mut events, scrollbar) = widget::ListSelect::multiple(num_items)
        .flow_down()
        .item_size(item_h)
        .scrollbar_next_to()
        .w_h(400.0, 230.0)
        .top_left_with_margins_on(ids.canvas, 40.0, 40.0)
        .set(ids.list_select, ui);

    // Handle the `ListSelect`s events.
    while let Some(event) = events.next(ui, |i| list_selected.contains(&i)) {
        use conrod_core::widget::list_select::Event;
        match event {

            // For the `Item` events we instantiate the `List`'s items.
            Event::Item(item) => {
                let label = &list_items[item.i];
                let (color, label_color) = match list_selected.contains(&item.i) {
                    true => (conrod_core::color::LIGHT_BLUE, conrod_core::color::YELLOW),
                    false => (conrod_core::color::LIGHT_GREY, conrod_core::color::BLACK),
                };
                let button = widget::Button::new()
                    .border(0.0)
                    .color(color)
                    .label(label)
                    .label_font_size(font_size)
                    .label_color(label_color);
                item.set(button, ui);
            }

            // The selection has changed.
            Event::Selection(selection) => {
                selection.update_index_set(list_selected);
                println!("selected indices: {:?}", list_selected);
            }

            // The remaining events indicate interactions with the `ListSelect` widget.
            event => println!("{:?}", &event),
        }
    }

    // Instantiate the scrollbar for the list.
    if let Some(s) = scrollbar { s.set(ui); }
}
