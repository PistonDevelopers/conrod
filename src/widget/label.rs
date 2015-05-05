
use color::{Color, Colorable};
use elmesque::Element;
use graphics::character::CharacterCache;
use label::{self, FontSize};
use position::{Depth, HorizontalAlign, Position, Positionable, VerticalAlign};
use theme::Theme;
use ui::{Ui, UiId};
use widget::{self, Widget};


/// Displays some given text centred within a rectangle.
#[derive(Clone, Debug)]
pub struct Label<'a> {
    text: &'a str,
    pos: Position,
    maybe_h_align: Option<HorizontalAlign>,
    maybe_v_align: Option<VerticalAlign>,
    depth: Depth,
    style: Style,
}

/// The styling for a Label's renderable Element.
#[derive(Clone, Debug, PartialEq, RustcEncodable, RustcDecodable)]
pub struct Style {
    maybe_font_size: Option<FontSize>,
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
            style: Style::new(),
        }
    }

    /// Set the font size for the label.
    #[inline]
    pub fn font_size(mut self, size: FontSize) -> Label<'a> {
        self.style.maybe_font_size = Some(size);
        self
    }

}


impl<'a> Widget for Label<'a> {
    type State = ();
    type Style = Style;
    fn unique_kind(&self) -> &'static str { "Label" }
    fn init_state(&self) -> () { () }
    fn style(&self) -> Style { self.style.clone() }

    /// Update the state of the Label.
    fn update<C>(&mut self,
                 _prev_state: &widget::State<()>,
                 style: &Style,
                 _ui_id: UiId,
                 ui: &mut Ui<C>) -> widget::State<()>
        where
            C: CharacterCache,
    {
        let size = style.font_size(&ui.theme);
        let dim = [label::width(ui, size, self.text), size as f64];
        let h_align = self.maybe_h_align.unwrap_or(ui.theme.align.horizontal);
        let v_align = self.maybe_v_align.unwrap_or(ui.theme.align.vertical);
        let xy = ui.get_xy(self.pos, dim, h_align, v_align);
        widget::State { state: (), xy: xy, depth: self.depth }
    }

    /// Construct an Element for the Label.
    fn draw<C>(&mut self,
               new_state: &widget::State<()>,
               style: &Style,
               _ui_id: UiId,
               ui: &mut Ui<C>) -> Element
        where
            C: CharacterCache,
    {
        use elmesque::form::{text, collage};
        use elmesque::text::Text;
        let widget::State { xy, .. } = *new_state;
        let size = style.font_size(&ui.theme);
        let dim = [label::width(ui, size, self.text), size as f64];
        let color = style.color(&ui.theme);
        let form = text(Text::from_string(self.text.to_string())
                            .color(color)
                            .height(size as f64)).shift(xy[0].floor(), xy[1].floor());
        collage(dim[0] as i32, dim[1] as i32, vec![form])
    }
}


impl Style {

    /// Construct the default Style.
    pub fn new() -> Style {
        Style {
            maybe_color: None,
            maybe_font_size: None,
        }
    }

    /// Get the Color for an Element.
    pub fn color(&self, theme: &Theme) -> Color {
        self.maybe_color.unwrap_or(theme.label_color)
    }

    /// Get the label font size for an Element.
    pub fn font_size(&self, theme: &Theme) -> FontSize {
        self.maybe_font_size.unwrap_or(theme.font_size_medium)
    }

}


impl<'a> Colorable for Label<'a> {
    fn color(mut self, color: Color) -> Self {
        self.style.maybe_color = Some(color);
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

