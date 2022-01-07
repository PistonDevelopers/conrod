use std::{thread::sleep, time::Duration};

use conrod_core::UiBuilder;
use conrod_example_shared::{DemoApp, WIN_H, WIN_W};
use conrod_sdl2::{convert_event, DrawPrimitiveError};
use sdl2::{
    image::LoadTexture, render::TextureValueError, video::WindowBuildError, IntegerOrSdlError,
};
use thiserror::Error;

fn main() -> Result<(), SdlError> {
    let sdl = sdl2::init().map_err(SdlError::String)?;

    let video_subsystem = sdl.video().map_err(SdlError::String)?;
    let window = video_subsystem
        .window("Conrod with SDL2!", WIN_W, WIN_H)
        .resizable()
        .allow_highdpi()
        .build()?;
    let mut canvas = window.into_canvas().present_vsync().build()?;
    let texture_creator = canvas.texture_creator();

    let mut event_pump = sdl.event_pump().map_err(SdlError::String)?;

    // The assets directory, where the Rust logo and the font file belong.
    let assets = find_folder::Search::KidsThenParents(3, 5)
        .for_folder("assets")
        .expect("Assets directory not found");

    // Load Rust logo as a texture.
    let rust_logo_path = assets.join("images/rust.png");
    let mut image_map = conrod_core::image::Map::new();
    let rust_logo = texture_creator
        .load_texture(rust_logo_path)
        .map_err(SdlError::String)?;
    let rust_logo = image_map.insert(rust_logo);

    // Create Ui and Ids of widgets to instantiate
    let mut ui = UiBuilder::new([WIN_W as f64, WIN_H as f64])
        .theme(conrod_example_shared::theme())
        .build();
    let ids = conrod_example_shared::Ids::new(ui.widget_id_generator());
    // Demonstration app state that we'll control with our conrod GUI.
    let mut app = DemoApp::new(rust_logo);

    // Load font file
    let font_path = assets.join("fonts/NotoSans/NotoSans-Regular.ttf");
    ui.fonts.insert_from_file(font_path).unwrap();

    // Renderer
    let mut renderer = conrod_sdl2::Renderer::new(&texture_creator)?;

    'main: loop {
        for event in event_pump.poll_iter() {
            match event {
                sdl2::event::Event::Quit { .. }
                | sdl2::event::Event::KeyDown {
                    keycode: Some(sdl2::keyboard::Keycode::Escape),
                    ..
                } => break 'main,
                _ => {}
            }
            for event in convert_event(event, canvas.window().size())
                .into_iter()
                .flatten()
            {
                ui.handle_event(event);
            }
        }

        conrod_example_shared::gui(&mut ui.set_widgets(), &ids, &mut app);
        if let Some(primitives) = ui.draw_if_changed() {
            renderer.draw(&mut canvas, &mut image_map, primitives)?;
            canvas.present();
        } else {
            // We should sleep for a reasonable duration before polloing for new events
            // in order to avoid polloing new events very frequently.
            sleep(Duration::from_secs_f64(1. / 60.));
        }
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
    #[error("Failed to create a texture: {0}")]
    TextureValueError(#[from] TextureValueError),
    #[error("Failed to draw a primitive: {0}")]
    DrawPrimitiveError(#[from] DrawPrimitiveError),
}
