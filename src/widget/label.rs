
use color::{Color, Colorable};
use graphics::character::CharacterCache;
use label::{self, FontSize};
use position::{Depth, HorizontalAlign, Position, Positionable, VerticalAlign};
use ui::{Ui, UiId};

/// Displays some given text centred within a rectangle.
pub struct Label<'a> {
    text: &'a str,
    pos: Position,
    maybe_h_align: Option<HorizontalAlign>,
    maybe_v_align: Option<VerticalAlign>,
    depth: Depth,
    size: FontSize,
    maybe_color: Option<Color>,
}

impl<'a> Label<'a> {

    /// Construct a new Label widget.
    pub fn new(text: &'a str) -> Label<'a> {
        Label {
            text: text,
            pos: Position::default(),
            maybe_h_align: None,
            maybe_v_align: None,
            depth: 0.0,
            size: 24u32,
            maybe_color: None,
        }
    }

    /// Set the font size for the label.
    #[inline]
    pub fn size(self, size: FontSize) -> Label<'a> {
        Label { size: size, ..self }
    }

    /// After building the Label, use this method to set its current state into the given `Ui`. It
    /// will use this state for rendering the next time `ui.draw(graphics)` is called.
    pub fn set<C>(self, ui_id: UiId, ui: &mut Ui<C>)
        where
            C: CharacterCache,
    {
        use elmesque::form::{text, collage};
        use elmesque::text::Text;
        let dim = [label::width(ui, self.size, self.text), self.size as f64];
        let h_align = self.maybe_h_align.unwrap_or(ui.theme.h_align);
        let v_align = self.maybe_v_align.unwrap_or(ui.theme.v_align);
        let xy = ui.get_xy(self.pos, dim, h_align, v_align);
        let color = self.maybe_color.unwrap_or(ui.theme.label_color);
        let form = text(Text::from_string(self.text.to_string())
                            .color(color)
                            .height(self.size as f64)).shift(xy[0].floor(), xy[1].floor());
        let element = collage(dim[0] as i32, dim[1] as i32, vec![form]);
        // Store the label's new state in the Ui.
        ui.set_widget(ui_id, ::widget::Widget {
            kind: ::widget::Kind::Label,
            xy: xy,
            depth: self.depth,
            element: Some(element),
        });
    }

}

impl<'a> Colorable for Label<'a> {
    fn color(mut self, color: Color) -> Self {
        self.maybe_color = Some(color);
        self
    }
}

impl<'a> Positionable for Label<'a> {
    fn position(mut self, pos: Position) -> Self {
        self.pos = pos;
        self
    }
    #[inline]
    fn horizontal_align(self, h_align: HorizontalAlign) -> Self {
        Label { maybe_h_align: Some(h_align), ..self }
    }
    #[inline]
    fn vertical_align(self, v_align: VerticalAlign) -> Self {
        Label { maybe_v_align: Some(v_align), ..self }
    }
}

