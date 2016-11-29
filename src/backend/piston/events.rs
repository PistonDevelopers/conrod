//! A event loop for a typical UI application, which blocks and waits for user input when idle.
//! Unlike `pistoncore_event_loop` it saves CPU cycles by not polling the window for new events
//! continuously.
//!
//! To schedule the event loop to send a new `Update` event in time for the next frame, call `update`
//! in between calls to `next`, otherwise the event loop will go idle until the next input event.
//!
//! `update` can be used to emulate the behaviour of the piston event loop if called every frame, or it 
//! can be called only when an animation is currently running, to post updates in the absence of 
//! user input.


#![deny(missing_docs)]
#![deny(missing_copy_implementations)]

extern crate window as pistoncore_window;

use std::time::{Duration, Instant};

use self::pistoncore_window::Window as BasicWindow;
use piston_input::{Event, UpdateArgs, RenderArgs, AfterRenderArgs, IdleArgs};

use super::window::Window;

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
    /// set externally to prevent the event loop from setting `idle` to
    /// true after the current frame, in case the UI needs to update or animate
    updating: bool,
    last_frame_time: Instant,
    next_frame_time: Instant,
    dt_frame: Duration,
}

static BILLION: u64 = 1_000_000_000;
trait UpdateDuration {
    fn from_freq(hz: u64) -> Duration;
    fn as_secs_f64(self) -> f64;
}
impl UpdateDuration for Duration {
    fn from_freq(hz: u64) -> Duration {
        let secs = (1.0 / hz as f64).floor() as u64;
        let nanos = ((BILLION / hz) % BILLION) as u32;
        Duration::new(secs, nanos)
    }
    fn as_secs_f64(self) -> f64 {
        self.as_secs() as f64 + self.subsec_nanos() as f64 / BILLION as f64
    }
}

/// The default maximum frames per second.
pub const DEFAULT_MAX_FPS: u64 = 60;

fn render_args(window: &Window, duration: f64) -> RenderArgs {
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
        let frame_length = Duration::from_freq(max_fps);
        WindowEvents {
            state: State::Waiting,
            idle: false,
            updating: false,
            last_frame_time: start,
            next_frame_time: start,
            dt_frame: frame_length,
        }
    }
    /// Creates a new event iterator with default FPS settings.
    pub fn new() -> WindowEvents {
        WindowEvents::new_with_fps(DEFAULT_MAX_FPS)
    }

    /// Use to trigger an update event by preventing the event loop from going idle.
    /// Call once per update loop for continuous animation, or call once to refresh the UI.
    pub fn update(&mut self) {
        self.updating = true;
    }

    /// Returns the next event.
    ///
    /// While in the `Waiting` state, returns `Input` events up until `dt_frame` has passed, or if idle, waits indefinitely.
    /// Once `dt_frame` has elapsed, or no longer idle, returns in order, `Update`, `Render` and `AfterRender` then resumes `Waiting` state.
    pub fn next(&mut self, window: &mut Window) -> Option<Event>
    {
        if window.should_close() { return None; }

        match self.state {
            State::Waiting => {
                if self.idle {
                    // Block and wait until an event is received.
                    let event = window.wait_event();
                    self.idle = false;
                    Some(Event::Input(event))
                } else {
                    let current_time = Instant::now();
                    if current_time < self.next_frame_time {
                        // Wait for events until ready for next frame.
                        let event = window.wait_event_timeout(self.next_frame_time - current_time);
                        if let Some(e) = event {
                            return Some(Event::Input(e));
                        }
                    }
                    // Handle any pending input before updating.
                    if let Some(e) = window.poll_event() {
                        return Some(Event::Input(e));
                    }
                    self.state = State::Updated;
                    let duration = (current_time - self.last_frame_time).as_secs_f64();
                    Some(Event::Update(UpdateArgs{ dt: duration }))
                }
            },
            State::Updated => {
                // Update event just posted, send `Render` event (if window can be drawn to).
                let size = window.size();
                let duration = (Instant::now() - self.last_frame_time).as_secs_f64();
                if size.width != 0 && size.height != 0 {
                    self.state = State::Rendered;
                    Some(Event::Render(render_args(window, duration)))
                } else {
                    self.state = State::Waiting;
                    Some(Event::Idle(IdleArgs{ dt: duration }))
                }
            },
            State::Rendered => {
                // Just rendered, send `AfterRender`, initialize for next frame
                // and resume `Waiting`
                self.last_frame_time = Instant::now();
                self.next_frame_time = self.last_frame_time + self.dt_frame;
                self.state = State::Waiting;
                self.idle = !self.updating;
                self.updating = false;
                Some(Event::AfterRender(AfterRenderArgs))
            },
        }
    }
}