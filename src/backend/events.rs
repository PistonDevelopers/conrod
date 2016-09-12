//! A event loop for a typical UI application, adapted from the standard piston event loop.
//! Unlike the piston event loop, this event loop will block and wait for input rather than continuously poll, unless an animation is running.

#![deny(missing_docs)]
#![deny(missing_copy_implementations)]

extern crate glutin_window;
extern crate glutin;
extern crate window;
extern crate piston_window;
extern crate gfx;
extern crate gfx_core;
extern crate gfx_device_gl;

use std::thread;
use std::thread::sleep;
use std::time::{Duration, Instant};

use self::window::Window;
use piston_input::*;
use self::glutin_window::*;

use self::gfx_core::Device;
use self::gfx_core::factory::Typed;

use self::piston_window::PistonWindow;

enum State {
    Rendered,
    Updated,
    Waiting,
}

/// An event loop iterator
///
/// *Warning: Because the iterator polls events from the window back-end,
/// it must be used on the same thread as the window back-end (usually main thread),
/// unless the window back-end supports multi-thread event polling.*
//#[derive(Copy, Clone)]
pub struct WindowEvents {
    state: State,
    /// if true, an update should be triggered in time for the next frame,
    /// either because an input event happened, or the UI is animating
    idle: bool,
    last_frame_time: Instant,
    next_frame_time: Instant,
    dt_frame: Duration,
}

static BILLION: u64 = 1_000_000_000;
fn ns_to_duration(ns: u64) -> Duration {
    let secs = ns / BILLION;
    let nanos = (ns % BILLION) as u32;
    Duration::new(secs, nanos)
}
fn duration_to_f64(dur: Duration) -> f64 {
    dur.as_secs() as f64 + dur.subsec_nanos() as f64 / 1000_000_000.0
}
fn duration_to_seconds(dur: Duration) -> f64 {
    duration_to_f64(dur) / BILLION as f64
}

/// The default maximum frames per second.
pub const DEFAULT_MAX_FPS: u64 = 60;

fn render_args(window: &GlutinWindow, duration: f64) -> RenderArgs {
    RenderArgs {
        ext_dt: duration,
        width: window.size().width,
        height: window.size().height,
        draw_width: window.draw_size().width,
        draw_height: window.draw_size().height,
    }
}

impl WindowEvents
{
    /// Creates a new event iterator
    pub fn new_with_fps(max_fps: u64) -> WindowEvents {
        let start = Instant::now();
        let frame_length = ns_to_duration(BILLION / max_fps);
        WindowEvents {
            state: State::Waiting,
            idle: true,
            last_frame_time: start,
            next_frame_time: start + frame_length,
            dt_frame: frame_length,
        }
    }
    /// Creates a new event iterator with default FPS settings.
    pub fn new() -> WindowEvents {
        WindowEvents::new_with_fps(DEFAULT_MAX_FPS)
    }

    /// Returns the next event.
    pub fn next(&mut self, window: &mut GlutinWindow, animating: bool) -> Option<Event>
    {
        if window.should_close() { return None; }

        match self.state {
            State::Rendered => {
                window.swap_buffers();
                self.last_frame_time = Instant::now();
                self.next_frame_time = self.last_frame_time + self.dt_frame;
                self.state = State::Waiting;
                // never go idle as long as an animation is running
                self.idle = !animating;
                return Some(Event::AfterRender(AfterRenderArgs));
            },
            State::Updated => {
                let size = window.size();
                if size.width != 0 && size.height != 0 {
                    self.state = State::Rendered;
                    let duration = duration_to_seconds(Instant::now() - self.last_frame_time);
                    return Some(Event::Render(render_args(window, duration)));
                }
            }, _ => ()
        }
        loop {
            // handle any pending input before updating
            if let Some(e) = window.poll_event() {
                self.idle = false;
                return Some(Event::Input(e));
            }
            if !self.idle {
                let current_time = Instant::now();
                if current_time >= self.next_frame_time {
                    self.state = State::Updated;
                    let duration = duration_to_seconds(current_time - self.last_frame_time);
                    return Some(Event::Update(UpdateArgs{ dt: duration }));
                } else {
                    // schedule wake up from `wait_event` in time for the next frame
                    let window_proxy = window.window.create_window_proxy();
                    let sleep_time = self.next_frame_time - current_time;
                    thread::spawn(move || {
                        sleep(sleep_time);
                        window_proxy.wakeup_event_loop();
                    });
                }
            }
            // block and wait until an event is received, or it's time to update
            let event = window.wait_event();
            if let Some(e) = event {
                self.idle = false;
                return Some(Event::Input(e));
            }
        }
    }
}

// TODO: below code can be cleaned up/removed if piston_window is refactored
fn create_main_targets(dim: gfx::tex::Dimensions) ->
(gfx::handle::RenderTargetView<
    gfx_device_gl::Resources, gfx::format::Srgba8>,
 gfx::handle::DepthStencilView<
    gfx_device_gl::Resources, gfx::format::DepthStencil>) {
    use self::gfx_core::factory::Typed;
    use self::gfx::format::{DepthStencil, Format, Formatted, Srgba8};

    let color_format: Format = <Srgba8 as Formatted>::get_format();
    let depth_format: Format = <DepthStencil as Formatted>::get_format();
    let (output_color, output_stencil) =
        gfx_device_gl::create_main_targets_raw(dim,
                                               color_format.0,
                                               depth_format.0);
    let output_color = Typed::new(output_color);
    let output_stencil = Typed::new(output_stencil);
    (output_color, output_stencil)
}

/// Allows a window to use an external event loop
pub trait EventWindow {
    /// receive next event from event loop and handle it
    fn next_event(&mut self, events: &mut WindowEvents, animating: bool) -> Option<Event>;
    /// handle new event and update window
    fn handle_event(&mut self, event: &Event);
}

impl EventWindow for PistonWindow {
    fn next_event(&mut self, events: &mut WindowEvents, animating: bool) -> Option<Event> {
        let event = events.next(&mut self.window, animating);
        if let Some(e) = event {
            self.handle_event(&e);
            Some(e)
        } else { None }
    }
    fn handle_event(&mut self, event: &Event) {
        if let Some(_) = event.after_render_args() {
            // After swapping buffers.
            self.device.cleanup();
        }
        // Check whether window has resized and update the output.
        let dim = self.output_color.raw().get_dimensions();
        let (w, h) = (dim.0, dim.1);
        let draw_size = self.window.draw_size();
        if w != draw_size.width as u16 || h != draw_size.height as u16 {
            let dim = (draw_size.width as u16,
                       draw_size.height as u16,
                       dim.2, dim.3);
            let (output_color, output_stencil) = create_main_targets(dim);
            self.output_color = output_color;
            self.output_stencil = output_stencil;
        }
    }
}