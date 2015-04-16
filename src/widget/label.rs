
use color::{black, Color, Colorable};
use graphics::Graphics;
use graphics::character::CharacterCache;
use label::FontSize;
use point::Point;
use position::Positionable;
use ui::Ui;

/// Displays some given text centred within a rectangle.
pub struct Label<'a> {
    text: &'a str,
    pos: Point,
    size: FontSize,
    maybe_color: Option<Color>,
}

impl<'a> Label<'a> {

    /// Set the font size for the label.
    pub fn size(self, size: FontSize) -> Label<'a> {
        Label { size: size, ..self }
    }

}

impl<'a> Label<'a> {

    /// Construct a new Label widget.
    pub fn new(text: &'a str) -> Label<'a> {
        Label {
            text: text,
            pos: [0.0, 0.0],
            size: 24u32,
            maybe_color: None,
        }
    }

}

impl<'a> Colorable for Label<'a> {
    fn color(mut self, color: Color) -> Self {
        self.maybe_color = Some(color);
        self
    }
}

impl<'a> Positionable for Label<'a> {
    fn point(mut self, pos: Point) -> Self {
        self.pos = pos;
        self
    }
}

impl<'a> ::draw::Drawable for Label<'a> {
    fn draw<B, C>(&mut self, ui: &mut Ui<C>, graphics: &mut B)
        where
            B: Graphics<Texture = <C as CharacterCache>::Texture>,
            C: CharacterCache
    {
        let color = self.maybe_color.unwrap_or(black());
        ui.draw_text(graphics, self.pos, self.size, color, self.text);
    }
}

