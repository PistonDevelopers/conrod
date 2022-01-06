use conrod_core::{
    event::Input,
    input,
    render::{Primitive, PrimitiveKind, PrimitiveWalker},
    text::{
        rt::gpu_cache::{CacheReadErr, CacheWriteErr},
        GlyphCache,
    },
    Scalar,
};
use sdl2::{
    event::Event,
    gfx::primitives::DrawRenderer,
    render::{Canvas, RenderTarget, Texture},
};
use thiserror::Error;

type SdlWindowSize = (u32, u32);

/// A function for converting a `sdl2::event::Event` to a `conrod::event::Input`.
///
/// This can be useful for single-window applications. Note that win_w and win_h should be the dpi
/// agnostic values (i.e. pass window.size() values and NOT window.drawable_size())
///
/// NOTE: The sdl2 MouseMotion event is a combination of a MouseCursor and MouseRelative conrod
/// events. Thus we may sometimes return two events in place of one, hence the tuple return type
pub fn convert_event(e: Event, window_size: SdlWindowSize) -> [Option<Input>; 2] {
    use sdl2::event::WindowEvent;

    // Translate the coordinates from top-left-origin-with-y-down to centre-origin-with-y-up.
    //
    // winit produces input events in pixels, so these positions need to be divided by the width
    // and height of the window in order to be DPI agnostic.
    let tx = |x: f32| x as Scalar - window_size.0 as Scalar / 2.0;
    let ty = |y: f32| -(y as Scalar - window_size.1 as Scalar / 2.0);

    // If the event is converted to a single input, it will stored to this variable `event`
    // and returned later.
    // If zero or two events is returned, it is early-returned.
    // We do so to simplify the code as much as possible.
    let event = match e {
        Event::Window { win_event, .. } => match win_event {
            WindowEvent::Resized(w, h) => Input::Resize(w as Scalar, h as Scalar),
            WindowEvent::FocusGained => Input::Focus(true),
            WindowEvent::FocusLost => Input::Focus(false),
            _ => return [None, None],
        },
        Event::TextInput { text, .. } => Input::Text(text),
        Event::KeyDown {
            keycode: Some(key), ..
        } => Input::Press(input::Button::Keyboard(convert_key(key))),
        Event::KeyUp {
            keycode: Some(key), ..
        } => Input::Release(input::Button::Keyboard(convert_key(key))),
        Event::FingerDown { touch_id, x, y, .. } => {
            let xy = [x as f64, y as f64];
            let id = input::touch::Id::new(touch_id as u64);
            let phase = input::touch::Phase::Start;
            let touch = input::Touch { phase, id, xy };
            Input::Touch(touch)
        }
        Event::FingerMotion { touch_id, x, y, .. } => {
            let xy = [x as f64, y as f64];
            let id = input::touch::Id::new(touch_id as u64);
            let phase = input::touch::Phase::Move;
            let touch = input::Touch { phase, id, xy };
            Input::Touch(touch)
        }

        Event::FingerUp { touch_id, x, y, .. } => {
            let xy = [x as f64, y as f64];
            let id = input::touch::Id::new(touch_id as u64);
            let phase = input::touch::Phase::End;
            let touch = input::Touch { phase, id, xy };
            Input::Touch(touch)
        }
        Event::MouseMotion {
            x, y, xrel, yrel, ..
        } => {
            let cursor = input::Motion::MouseCursor {
                x: tx(x as f32),
                y: ty(y as f32),
            };
            let relative = input::Motion::MouseRelative {
                x: tx(xrel as f32),
                y: ty(yrel as f32),
            };
            return [Some(Input::Motion(cursor)), Some(Input::Motion(relative))];
        }
        Event::MouseWheel { x, y, .. } => {
            // Invert the scrolling of the *y* axis as *y* is up in conrod.
            const ARBITRARY_POINTS_PER_LINE_FACTOR: Scalar = 10.0;
            let x = ARBITRARY_POINTS_PER_LINE_FACTOR * x as Scalar;
            let y = ARBITRARY_POINTS_PER_LINE_FACTOR * (-y) as Scalar;
            let motion = input::Motion::Scroll { x, y };
            Input::Motion(motion)
        }
        Event::MouseButtonDown { mouse_btn, .. } => {
            Input::Press(input::Button::Mouse(convert_mouse_button(mouse_btn)))
        }
        Event::MouseButtonUp { mouse_btn, .. } => {
            Input::Release(input::Button::Mouse(convert_mouse_button(mouse_btn)))
        }
        Event::JoyAxisMotion {
            which,
            axis_idx,
            value,
            ..
        } => {
            // Axis motion is an absolute value in the range
            // [-32768, 32767]. Normalize it down to a float.
            use std::i16::MAX;
            let normalized_value = value as f64 / MAX as f64;
            Input::Motion(input::Motion::ControllerAxis(
                input::ControllerAxisArgs::new(which, axis_idx, normalized_value),
            ))
        }
        _ => return [None, None],
    };
    [Some(event), None]
}

/// Maps sdl2's key to a conrod `Key`.
pub fn convert_key(keycode: sdl2::keyboard::Keycode) -> input::keyboard::Key {
    (keycode as u32).into()
}

/// Maps sdl2's mouse button to conrod's mouse button.
pub fn convert_mouse_button(mouse_button: sdl2::mouse::MouseButton) -> input::MouseButton {
    use conrod_core::input::MouseButton;
    match mouse_button {
        sdl2::mouse::MouseButton::Left => MouseButton::Left,
        sdl2::mouse::MouseButton::Right => MouseButton::Right,
        sdl2::mouse::MouseButton::Middle => MouseButton::Middle,
        sdl2::mouse::MouseButton::X1 => MouseButton::X1,
        sdl2::mouse::MouseButton::X2 => MouseButton::X2,
        _ => MouseButton::Unknown,
    }
}

pub fn draw(
    canvas: &mut Canvas<impl RenderTarget>,
    image_map: &mut conrod_core::image::Map<sdl2::render::Texture>,
    glyph_cache: &mut GlyphCache,
    glyph_texture: &mut Texture,
    window_size: SdlWindowSize,
    mut primitives: impl PrimitiveWalker,
) -> Result<(), DrawPrimitiveError> {
    while let Some(primitive) = primitives.next_primitive() {
        // TODO: this function returns as soon as an error occurs.
        // shuold we just ignore the error instead?
        draw_primitive(
            canvas,
            image_map,
            glyph_cache,
            glyph_texture,
            window_size,
            primitive,
        )?;
    }
    Ok(())
}

pub fn draw_primitive(
    canvas: &mut Canvas<impl RenderTarget>,
    image_map: &mut conrod_core::image::Map<sdl2::render::Texture>,
    glyph_cache: &mut GlyphCache,
    glyph_texture: &mut Texture,
    window_size: SdlWindowSize,
    primitive: Primitive,
) -> Result<(), DrawPrimitiveError> {
    let convert_point = |r| convert_point(window_size, r);
    let convert_rect = |r| convert_rect(window_size, r);
    canvas.set_clip_rect(convert_rect(primitive.scizzor));
    match primitive.kind {
        PrimitiveKind::Rectangle { color } => {
            // println!("{:?} {:?} {:?}", primitive.rect, convert_rect(primitive.rect), color);
            canvas.set_draw_color(convert_color(color));
            canvas
                .fill_rect(convert_rect(primitive.rect))
                .map_err(DrawPrimitiveError::Sdl)?;
        }
        PrimitiveKind::TrianglesSingleColor { color, triangles } => {
            canvas.set_draw_color(sdl2::pixels::Color::RGBA(255, 0, 0, 128));
            for t in triangles {
                let [(ax, ay), (bx, by), (cx, cy)] = t.0.map(convert_point);
                canvas
                    .filled_trigon(
                        ax as _,
                        ay as _,
                        bx as _,
                        by as _,
                        cx as _,
                        cy as _,
                        convert_color(color),
                    )
                    .map_err(DrawPrimitiveError::Sdl)?;
            }
        }
        PrimitiveKind::TrianglesMultiColor { triangles } => {
            // Multicolor triangles are not supported in pure SDL,
            // so we workaround by using only the first color
            for t in triangles {
                let [((ax, ay), color), ((bx, by), _), ((cx, cy), _)] =
                    t.0.map(|(p, c)| (convert_point(p), c));
                canvas
                    .filled_trigon(
                        ax as _,
                        ay as _,
                        bx as _,
                        by as _,
                        cx as _,
                        cy as _,
                        convert_color(color),
                    )
                    .map_err(DrawPrimitiveError::Sdl)?;
            }
        }
        PrimitiveKind::Image {
            image_id,
            color,
            source_rect,
        } => {
            // TODO: should we really panic if no image was found?
            let texture = image_map
                .get_mut(image_id)
                .expect("No image found for the given image id");
            if let Some(color) = color {
                let color = convert_color(color);
                texture.set_color_mod(color.r, color.g, color.b);
                texture.set_alpha_mod(color.a);
            }
            canvas
                .copy(
                    texture,
                    source_rect.map(convert_rect),
                    convert_rect(primitive.rect),
                )
                .map_err(DrawPrimitiveError::Sdl)?;
        }
        PrimitiveKind::Text {
            color,
            text,
            font_id,
        } => {
            let font_id = font_id.index();

            // FIXME: We can calculate the DPI factor by accessing the video subsystem
            let dpi_factor = 1.0;
            let positioned_glyphs = text.positioned_glyphs(dpi_factor);

            // Queue the glyphs to be cached.
            for glyph in positioned_glyphs {
                glyph_cache.queue_glyph(font_id, glyph.clone());
            }

            // Cache the glyphs within the GPU cache.
            glyph_cache.cache_queued(|rect, data| {
                let rect = sdl2::rect::Rect::new(
                    rect.min.x as i32,
                    rect.min.y as i32,
                    rect.width(),
                    rect.height(),
                );
                // TODO: propagate errors?
                let _ = glyph_texture.with_lock(rect, |out, pitch| {
                    let out = out.chunks_mut(pitch);
                    let data = data.chunks(rect.width() as _);
                    for (out, data) in out.zip(data) {
                        for (out, &data) in out.chunks_mut(4).zip(data) {
                            out[3] = data;
                            out[0..3].fill(255);
                        }
                    }
                });
            })?;

            let color = convert_color(color);
            glyph_texture.set_color_mod(color.r, color.g, color.b);
            glyph_texture.set_alpha_mod(color.a);

            let (glyph_texture_w, glyph_texture_h) = {
                let q = glyph_texture.query();
                (q.width as f32, q.height as f32)
            };

            for glyph in positioned_glyphs {
                if let Some((src, dst)) = glyph_cache.rect_for(font_id, glyph)? {
                    let src = {
                        let x = src.min.x * glyph_texture_w;
                        let y = src.min.y * glyph_texture_h;
                        let w = src.width() * glyph_texture_w;
                        let h = src.height() * glyph_texture_h;
                        (x as _, y as _, w as _, h as _)
                    };
                    let dst = {
                        let x = dst.min.x as f32 / dpi_factor;
                        let y = dst.min.y as f32 / dpi_factor;
                        let w = dst.width() as f32 / dpi_factor;
                        let h = dst.height() as f32 / dpi_factor;
                        (x as _, y as _, w as _, h as _)
                    };
                    canvas
                        .copy(glyph_texture, Some(src.into()), Some(dst.into()))
                        .map_err(DrawPrimitiveError::Sdl)?;
                }
            }
        }
        PrimitiveKind::Other(_) => {}
    }
    Ok(())
}

#[derive(Debug, Error)]
pub enum DrawPrimitiveError {
    #[error("SDL returned an error: {0}")]
    Sdl(String),
    #[error("GlyphCache::cache_queued failed: {0}")]
    GlyphCacheWriteErr(#[from] CacheWriteErr),
    #[error("GlyphCache:rect_for failed: {0}")]
    GlyphCacheReadErr(#[from] CacheReadErr),
}

pub fn convert_rect(window_size: SdlWindowSize, rect: conrod_core::Rect) -> sdl2::rect::Rect {
    let (x, y) = convert_point(window_size, rect.top_left());
    sdl2::rect::Rect::new(x, y, rect.w() as _, rect.h() as _)
}

pub fn convert_point(window_size: SdlWindowSize, [x, y]: conrod_core::Point) -> (i32, i32) {
    let x = window_size.0 as f64 / 2. + x;
    let y = window_size.1 as f64 / 2. - y;
    (x as _, y as _)
}

pub fn convert_color(color: impl Into<conrod_core::Color>) -> sdl2::pixels::Color {
    let [r, g, b, a] = color.into().to_byte_fsa();
    sdl2::pixels::Color::RGBA(r, g, b, a)
}
