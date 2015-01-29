
use color::Color;
use draw::Drawable;
use graphics;
use opengl_graphics::Gl;
use ui_context::UiContext;

/// The context from which we'll draw the background.
#[derive(Copy)]
pub struct Background {
    maybe_color: Option<Color>,
}

impl Background {
    pub fn new() -> Background {
        Background {
            maybe_color: None,
        }
    }
}

// impl_colorable!(Background,);

impl ::color::Colorable for Background {
    #[inline]
    fn color(self, color: Color) -> Self {
        Background { maybe_color: Some(color), ..self }
    }
    #[inline]
    fn rgba(self, r: f32, g: f32, b: f32, a: f32) -> Self {
        Background { maybe_color: Some(Color::new(r, g, b, a)), ..self }
    }
}

impl Drawable for Background {
    fn draw(&mut self, uic: &mut UiContext, graphics: &mut Gl) {
        let Color(col) = self.maybe_color
            .unwrap_or(uic.theme.background_color);
        graphics::clear(col, graphics);
    }
}
