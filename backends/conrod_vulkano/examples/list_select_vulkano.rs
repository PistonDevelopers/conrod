//! A demonstration of using `winit` to provide events and `vulkano` to draw the UI.

#![allow(unused_variables)]

#[macro_use]
extern crate conrod_core;
extern crate conrod_vulkano;
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
    format::Format,
    image::AttachmentImage,
    render_pass::{Framebuffer, FramebufferAbstract, RenderPass},
    swapchain::AcquireError,
    sync::GpuFuture,
};

use conrod_vulkano::Renderer;
use vulkano::command_buffer::{CommandBufferUsage, SubpassContents};
use vulkano::image::view::ImageView;

use conrod_winit::WinitWindow;
use winit::event::{ElementState, KeyboardInput, VirtualKeyCode};
use winit::event_loop::ControlFlow;

const DEPTH_FORMAT: Format = Format::D16Unorm;
const CLEAR_COLOR: [f32; 4] = [0.2, 0.2, 0.2, 1.0];
const WIN_W: u32 = 600;
const WIN_H: u32 = 300;
use winit::event;

widget_ids! {
    struct Ids { canvas, list_select }
}

fn main() {
    let event_loop = winit::event_loop::EventLoop::new();
    let mut window = support::Window::new(WIN_W, WIN_H, "Conrod with vulkano", &event_loop);

    let mut render_target = RenderTarget::new(&window);

    let subpass = vulkano::render_pass::Subpass::from(render_target.render_pass.clone(), 0)
        .expect("Couldn't create subpass for gui!");
    let queue = window.queue.clone();
    let mut renderer = Renderer::new(
        window.device.clone(),
        subpass,
        queue.family(),
        [WIN_W, WIN_H],
        window.surface.window().scale_factor() as f64,
    )
    .unwrap();

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
        "Spotted Turtle".to_string(),
    ];

    // List of selections, should be same length as list of entries. Will be updated by the widget.
    let mut list_selected = std::collections::HashSet::new();
    let sixteen_ms = std::time::Duration::from_millis(16);
    let mut next_update = None;
    let mut ui_update_needed = false;
    event_loop.run(move |event, _, control_flow| {
        if let Some(event) = support::convert_event(&event, window.surface.window()) {
            ui.handle_event(event);
            ui_update_needed = true;
        }
        match &event {
            // Recreate swapchain when window is resized.
            event::Event::WindowEvent { event, .. } => match event {
                event::WindowEvent::Resized(_new_size) => {
                    window.handle_resize();
                    render_target.handle_resize(&window);
                    return;
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
        let should_set_ui_on_main_events_cleared = next_update.is_none() && ui_update_needed;
        match (&event, should_set_ui_on_main_events_cleared) {
            (event::Event::NewEvents(event::StartCause::Init { .. }), _)
            | (event::Event::NewEvents(event::StartCause::ResumeTimeReached { .. }), _)
            | (event::Event::MainEventsCleared, true) => {
                next_update = Some(std::time::Instant::now() + sixteen_ms);
                ui_update_needed = false;

                gui(&mut ui.set_widgets(), &ids, &list_items, &mut list_selected);
                if ui.has_changed() {
                    // If the view has changed at all, request a redraw.
                    window.surface.window().request_redraw();
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
                //begin the render pass and add the draw command
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
    let depth = ImageView::new(depth_buffer).unwrap();
    window
        .images
        .iter()
        .map(|image| {
            Arc::new(
                Framebuffer::start(render_pass.clone())
                    .add(ImageView::new(image.clone()).unwrap())
                    .unwrap()
                    .add(depth.clone())
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

    widget::Canvas::new()
        .color(conrod_core::color::BLUE)
        .set(ids.canvas, ui);

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
    if let Some(s) = scrollbar {
        s.set(ui);
    }
}
