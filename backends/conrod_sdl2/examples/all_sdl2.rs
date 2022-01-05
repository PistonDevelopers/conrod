use conrod_example_shared::{DemoApp, WIN_H, WIN_W};
use sdl2::{video::WindowBuildError, IntegerOrSdlError, image::LoadTexture};
use thiserror::Error;

fn main() -> Result<(), SdlError> {
    let sdl = sdl2::init().map_err(SdlError::String)?;

    let video_subsystem = sdl.video().map_err(SdlError::String)?;
    let window = video_subsystem
        .window("Conrod with SDL2!", WIN_W, WIN_H)
        .build()?;
    let mut canvas = window.into_canvas().present_vsync().build()?;
    let texture_creator = canvas.texture_creator();

    let mut event_pump = sdl.event_pump().map_err(SdlError::String)?;

    // Load Rust logo as a texture.
    let rust_logo_path = find_folder::Search::KidsThenParents(3, 5)
        .for_folder("assets")
        .expect("Assets directory not found")
        .join("images/rust.png");
    let mut image_map = conrod_core::image::Map::new();
    let rust_logo = image_map.insert(texture_creator.load_texture(rust_logo_path));

    // Demonstration app state that we'll control with our conrod GUI.
    let app = DemoApp::new(rust_logo);

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
