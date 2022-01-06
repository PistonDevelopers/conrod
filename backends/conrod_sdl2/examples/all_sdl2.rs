use conrod_example_shared::{DemoApp, WIN_H, WIN_W};
use conrod_sdl2::convert_event;
use sdl2::{image::LoadTexture, rect::Rect, video::WindowBuildError, IntegerOrSdlError};
use thiserror::Error;

fn main() -> Result<(), SdlError> {
    let sdl = sdl2::init().map_err(SdlError::String)?;

    let video_subsystem = sdl.video().map_err(SdlError::String)?;
    let window = video_subsystem
        .window("Conrod with SDL2!", WIN_W, WIN_H)
        .build()?;
    let window_size = window.size();
    let mut canvas = window.into_canvas().present_vsync().build()?;
    let texture_creator = canvas.texture_creator();

    let mut event_pump = sdl.event_pump().map_err(SdlError::String)?;

    // Load Rust logo as a texture.
    let rust_logo_path = find_folder::Search::KidsThenParents(3, 5)
        .for_folder("assets")
        .expect("Assets directory not found")
        .join("images/rust.png");
    let mut image_map = conrod_core::image::Map::new();
    let rust_logo = texture_creator
        .load_texture(rust_logo_path)
        .map_err(SdlError::String)?;
    let rust_logo = image_map.insert(rust_logo);

    // Create Ui and Ids of widgets to instantiate
    let mut ui = conrod_core::UiBuilder::new([WIN_W as f64, WIN_H as f64])
        .theme(conrod_example_shared::theme())
        .build();
    let ids = conrod_example_shared::Ids::new(ui.widget_id_generator());
    // Demonstration app state that we'll control with our conrod GUI.
    let mut app = DemoApp::new(rust_logo);

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
            for event in convert_event(event, window_size).into_iter().flatten() {
                ui.handle_event(event);
            }
        }

        conrod_example_shared::gui(&mut ui.set_widgets(), &ids, &mut app);
        if let Some(primitives) = ui.draw_if_changed() {
            conrod_sdl2::draw(&mut canvas, &mut image_map, window_size, primitives)
                .map_err(SdlError::String)?;
        } else {
            // If `canvas.present()` is called without any actual drawing command, it clears the screen.
            // We want to preserve what is drawn, so we draw a dummy rectangle outside the canvas.
            canvas
                .draw_rect(Rect::new(-1, -1, 1, 1))
                .map_err(SdlError::String)?;
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
