use conrod_example_shared::{WIN_H, WIN_W};
use sdl2::{video::WindowBuildError, IntegerOrSdlError};
use thiserror::Error;

fn main() -> Result<(), SdlError> {
    let sdl = sdl2::init().map_err(SdlError::String)?;

    let video_subsystem = sdl.video().map_err(SdlError::String)?;
    let window = video_subsystem
        .window("Conrod with SDL2!", WIN_W, WIN_H)
        .build()?;
    let mut canvas = window.into_canvas().present_vsync().build()?;

    let mut event_pump = sdl.event_pump().map_err(SdlError::String)?;

    'main: loop {
        for event in event_pump.poll_iter() {
            #[allow(clippy::single_match)]
            match event {
                sdl2::event::Event::Quit { .. } => break 'main,
                _ => {}
            }
        }

        canvas.present();
    }

    Ok(())
}

#[derive(Debug, Error)]
pub enum SdlError {
    #[error("SDL returned an error: {0}")]
    String(String),
    #[error("SDL failed to build a window: {0}")]
    WindowBuildError(#[from] WindowBuildError),
    #[error("Integer overflowed or SDL returned an error: {0}")]
    IntegerOrSdlError(#[from] IntegerOrSdlError),
}
