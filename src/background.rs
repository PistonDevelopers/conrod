
use color::Color;
use draw::Drawable;
use graphics;
use graphics::BackEnd;
use graphics::character::CharacterCache;
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

quack! {
    bg: Background[]
    get:
    set:
        fn (val: Color) [] { bg.maybe_color = Some(val) }
    action:
}

impl Drawable for Background {
    fn draw<B, C>(&mut self, uic: &mut UiContext<C>, graphics: &mut B)
        where
            B: BackEnd<Texture = <C as CharacterCache>::Texture>,
            C: CharacterCache
    {
        let Color(col) = self.maybe_color
            .unwrap_or(uic.theme.background_color);
        graphics::clear(col, graphics);
    }
}
