
use color::{Color, Colorable};
use elmesque::Element;
use frame::Frameable;
use graphics::character::CharacterCache;
use graphics::math::Scalar;
use label::FontSize;
use mouse::Mouse;
use num::Float;
use input::keyboard::Key::{Backspace, Left, Right, Return, A, E, LCtrl, RCtrl};
use position::{self, Dimensions, Point};
use theme::Theme;
use ui::GlyphCache;
use vecmath::vec2_sub;
use widget::{self, Widget};


pub type Idx = usize;
pub type CursorX = f64;

const TEXT_PADDING: Scalar = 5.0;

/// A widget for displaying and mutating a given one-line text `String`. It's reaction is
/// triggered upon pressing of the `Enter`/`Return` key.
pub struct TextBox<'a, F> {
    common: widget::CommonBuilder,
    text: &'a mut String,
    maybe_react: Option<F>,
    style: Style,
    enabled: bool,
}

/// Styling for the TextBox, necessary for constructing its renderable Element.
#[allow(missing_docs, missing_copy_implementations)]
#[derive(Clone, Debug, PartialEq, RustcEncodable, RustcDecodable)]
pub struct Style {
    pub maybe_color: Option<Color>,
    pub maybe_frame: Option<Scalar>,
    pub maybe_frame_color: Option<Color>,
    pub maybe_font_size: Option<u32>,
}

/// The State of the TextBox widget that will be cached within the Ui.
#[derive(Clone, Debug, PartialEq)]
pub struct State {
    interaction: Interaction,
    text: String,
    control_pressed: bool
}

/// Represents the state of the text_box widget.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Interaction {
    Captured(View),
    Uncaptured(Uncaptured),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct View {
    cursor: Cursor,
    offset: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Cursor {
    anchor: Anchor,
    start: Idx,
    end: Idx,
}

/// The beginning of the cursor's highlighted range.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Anchor {
    None,
    Start,
    End,
}

/// The TextBox's state if it is uncaptured.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Uncaptured {
    Highlighted,
    Normal,
}

/// Represents an element of the TextBox widget.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Elem {
    Char(Idx),
    Nill,
    Rect,
}


impl Interaction {
    /// Return the color associated with the current state.
    fn color(&self, color: Color) -> Color {
        match *self {
            Interaction::Captured(_) => color,
            Interaction::Uncaptured(state) => match state {
                Uncaptured::Highlighted => color.highlighted(),
                Uncaptured::Normal => color,
            }
        }
    }
}


impl Cursor {

    /// Construct a Cursor from a String index.
    fn from_index(idx: Idx) -> Cursor {
        Cursor { anchor: Anchor::Start, start: idx, end: idx }
    }

    /// Construct a Cursor from an index range.
    fn from_range(start: Idx, end: Idx) -> Cursor {
        if start < end {
            Cursor { anchor: Anchor::Start, start: start, end: end }
        } else {
            Cursor { anchor: Anchor::End, start: end, end: start }
        }
    }

    /// Ensure the cursor does not go past end_limit. Returns true if the cursor was
    /// previously too large.
    fn limit_end_to(&mut self, end_limit: usize) -> bool {
        if self.start > end_limit {
            self.start = end_limit;
            self.end = end_limit;
            return true;
        }
        else if self.end > end_limit {
            self.end = end_limit;
            return true
        }
        false
    }

    /// Shift the curosr by the given number of indices.
    fn shift(&mut self, by: i32) {
        if by == 0 {
             return;
        }
        let mut new_idx = self.start as i32 + by;
        if new_idx < 0 {
            new_idx = 0;
        }
        self.start = new_idx as Idx;
        self.end = new_idx as Idx;
    }

    fn is_cursor(&self) -> bool {
        self.start == self.end
    }
}

// widget_fns!(TextBox, State, Kind::TextBox(State::Uncaptured(Uncaptured::Normal)));

/// Find the position of a character in a text box.
fn cursor_position<C: CharacterCache>(glyph_cache: &GlyphCache<C>,
                                      idx: usize,
                                      mut text_start_x: f64,
                                      font_size: FontSize,
                                      text: &str) -> CursorX {
    assert!(idx <= text.chars().count());

    if idx == 0 {
         return text_start_x;
    }

    for (i, ch) in text.chars().enumerate() {
        if i >= idx { break; }
        text_start_x += glyph_cache.char_width(font_size, ch);
    }
    text_start_x
}

/// Check if cursor is over the pad and if so, which
fn over_elem<C: CharacterCache>(glyph_cache: &GlyphCache<C>,
                                mouse_xy: Point,
                                dim: Dimensions,
                                pad_dim: Dimensions,
                                text_start_x: f64,
                                text_w: f64,
                                font_size: FontSize,
                                text: &str) -> Elem {
    use position::is_over_rect;
    if is_over_rect([0.0, 0.0], dim, mouse_xy) {
        if is_over_rect([0.0, 0.0], pad_dim, mouse_xy) {
            let (idx, _) = closest_idx(glyph_cache, mouse_xy, text_start_x, text_w, font_size, text);
            Elem::Char(idx)
        } else {
            Elem::Rect
        }
    } else {
        Elem::Nill
    }
}

/// Check which character is closest to the mouse cursor.
fn closest_idx<C: CharacterCache>(glyph_cache: &GlyphCache<C>,
                                  mouse_xy: Point,
                                  text_start_x: f64,
                                  text_w: f64,
                                  font_size: FontSize,
                                  text: &str) -> (Idx, f64) {
    if mouse_xy[0] <= text_start_x { return (0, text_start_x) }
    let mut left_x = text_start_x;
    let mut x = left_x;
    let mut prev_x = x;
    for (i, ch) in text.chars().enumerate() {
        let char_w = glyph_cache.char_width(font_size, ch);
        x += char_w;
        let right_x = prev_x + char_w / 2.0;
        if mouse_xy[0] > left_x && mouse_xy[0] <= right_x { return (i, prev_x) }
        prev_x = x;
        left_x = right_x;
    }
    (text.chars().count(), text_start_x + text_w)
}

/// Check and return the current state of the TextBox.
fn get_new_interaction(over_elem: Elem, prev_interaction: Interaction, mouse: Mouse) -> Interaction {
    use mouse::ButtonPosition::{Down, Up};
    use self::Interaction::{Captured, Uncaptured};
    use self::Uncaptured::{Normal, Highlighted};

    match prev_interaction {
        Interaction::Captured(mut prev) => match mouse.left.position {
            Down => match over_elem {
                Elem::Nill => if prev.cursor.anchor == Anchor::None {
                    Uncaptured(Normal)
                } else {
                    prev_interaction
                },
                Elem::Rect =>  if prev.cursor.anchor == Anchor::None {
                    prev.cursor = Cursor::from_index(0);
                    Captured(prev)
                } else {
                    prev_interaction
                },
                Elem::Char(idx) => {
                    match prev.cursor.anchor {
                        Anchor::None  => prev.cursor = Cursor::from_index(idx),
                        Anchor::Start => prev.cursor = Cursor::from_range(prev.cursor.start, idx),
                        Anchor::End   => prev.cursor = Cursor::from_range(prev.cursor.end, idx),
                    }
                    Captured(prev)
                },
            },
            Up => {
                prev.cursor.anchor = Anchor::None;
                Captured(prev)
            },
        },

        Interaction::Uncaptured(prev) => match mouse.left.position {
            Down => match over_elem {
                Elem::Nill => Uncaptured(Normal),
                Elem::Rect => match prev {
                    Normal => prev_interaction,
                    Highlighted => Captured(View {
                         cursor: Cursor::from_index(0),
                         offset: 0.0,
                    })
                },
                Elem::Char(idx) =>  match prev {
                    Normal => prev_interaction,
                    Highlighted => Captured(View {
                        cursor: Cursor::from_index(idx),
                        offset: 0.0,
                    })
                },
            },
            Up => match over_elem {
                Elem::Nill => Uncaptured(Normal),
                Elem::Char(_) | Elem::Rect => match prev {
                    Normal => Uncaptured(Highlighted),
                    Highlighted => prev_interaction,
                },
            },
        },
    }
}

impl<'a, F> TextBox<'a, F> {

    /// Construct a TextBox widget.
    pub fn new(text: &'a mut String) -> TextBox<'a, F> {
        TextBox {
            common: widget::CommonBuilder::new(),
            text: text,
            maybe_react: None,
            style: Style::new(),
            enabled: true,
        }
    }

    /// Set the font size of the text.
    pub fn font_size(mut self, font_size: FontSize) -> TextBox<'a, F> {
        self.style.maybe_font_size = Some(font_size);
        self
    }

    /// Set the reaction for the TextBox. It will be triggered upon pressing of the
    /// `Enter`/`Return` key.
    pub fn react(mut self, reaction: F) -> TextBox<'a, F> {
        self.maybe_react = Some(reaction);
        self
    }

    /// If true, will allow user inputs.  If false, will disallow user inputs.
    pub fn enabled(mut self, flag: bool) -> Self {
        self.enabled = flag;
        self
    }

}

impl<'a, F> Widget for TextBox<'a, F> where F: FnMut(&mut String) {
    type State = State;
    type Style = Style;
    fn common(&self) -> &widget::CommonBuilder { &self.common }
    fn common_mut(&mut self) -> &mut widget::CommonBuilder { &mut self.common }
    fn unique_kind(&self) -> &'static str { "TextBox" }
    fn init_state(&self) -> State {
        State {
            interaction: Interaction::Uncaptured(Uncaptured::Normal),
            text: String::new(),
            control_pressed: false,
        }
    }
    fn style(&self) -> Style { self.style.clone() }

    fn default_width<C: CharacterCache>(&self, theme: &Theme, _: &GlyphCache<C>) -> Scalar {
        const DEFAULT_WIDTH: Scalar = 192.0;
        theme.maybe_text_box.as_ref().map(|default| {
            default.common.maybe_width.unwrap_or(DEFAULT_WIDTH)
        }).unwrap_or(DEFAULT_WIDTH)
    }

    fn default_height(&self, theme: &Theme) -> Scalar {
        const DEFAULT_HEIGHT: Scalar = 48.0;
        theme.maybe_text_box.as_ref().map(|default| {
            default.common.maybe_height.unwrap_or(DEFAULT_HEIGHT)
        }).unwrap_or(DEFAULT_HEIGHT)
    }

    /// Update the state of the TextBox.
    fn update<C: CharacterCache>(mut self, args: widget::UpdateArgs<Self, C>) {
        let widget::UpdateArgs { state, rect, style, mut ui, .. } = args;

        let (xy, dim) = rect.xy_dim();
        let maybe_mouse = ui.input().maybe_mouse.map(|mouse| mouse.relative_to(xy));
        let frame = style.frame(ui.theme());
        let font_size = style.font_size(ui.theme());
        let pad_dim = vec2_sub(dim, [frame * 2.0; 2]);
        let text_w = ui.glyph_cache().width(font_size, &self.text);
        let text_x = position::align_left_of(pad_dim[0], text_w) + TEXT_PADDING;
        let text_start_x = text_x - text_w / 2.0;
        let mut new_control_pressed = state.view().control_pressed;
        let mut new_interaction = match (self.enabled, maybe_mouse) {
            (false, _) | (true, None) => Interaction::Uncaptured(Uncaptured::Normal),
            (true, Some(mouse)) => {
                let over_elem = over_elem(ui.glyph_cache(), mouse.xy, dim, pad_dim, text_start_x,
                                          text_w, font_size, &self.text);
                get_new_interaction(over_elem, state.view().interaction, mouse)
            },
        };

        // Check cursor validity (and update new_interaction if necessary).
        if let Interaction::Captured(view) = new_interaction {
            let mut cursor = view.cursor;
            let mut v_offset = view.offset;

            // Ensure the cursor is still valid.
            cursor.limit_end_to(self.text.chars().count());

            // This matters if the text is scrolled with the mouse.
            let cursor_idx = match cursor.anchor {
                Anchor::End => cursor.start,
                Anchor::Start | Anchor::None => cursor.end,
            };
            let cursor_x = cursor_position(ui.glyph_cache(), cursor_idx, text_start_x, font_size,
                                           &self.text);

            if cursor.is_cursor() || cursor.anchor != Anchor::None {
                let cursor_x_view = cursor_x - v_offset;
                let text_right = dim[0] - TEXT_PADDING - frame;

                if cursor_x_view < text_x {
                    v_offset += cursor_x_view - text_x;
                } else if cursor_x_view > text_right {
                    v_offset += cursor_x_view - text_right;
                }
            }

            // Set the updated state.
            new_interaction = Interaction::Captured(View { cursor: cursor, offset: v_offset });
        }

        // If TextBox is captured, check for recent input and update the text accordingly.
        if let Interaction::Captured(captured) = new_interaction {
            let mut cursor = captured.cursor;
            let input = ui.input();

            // Check for entered text.
            for text in input.entered_text {
                if new_control_pressed { break; }
                if text.chars().count() == 0 { continue; }

                let max_w = pad_dim[0] - TEXT_PADDING * 2.0;
                if text_w + ui.glyph_cache().width(font_size, &text) > max_w { continue; }

                let end: String = self.text.chars().skip(cursor.end).collect();
                let start: String = self.text.chars().take(cursor.start).collect();
                self.text.clear();
                self.text.push_str(&start);
                self.text.push_str(&text);
                self.text.push_str(&end);
                cursor.shift(text.chars().count() as i32);
            }

            // Check for control keys.
            for key in input.pressed_keys.iter() {
                match *key {
                    Backspace => if cursor.is_cursor() {
                        if cursor.start > 0 {
                            let start: String = self.text.chars().take(cursor.start-1).collect();
                            let end: String = self.text.chars().skip(cursor.end).collect();
                            self.text.clear();
                            self.text.push_str(&start);
                            self.text.push_str(&end);
                            cursor.shift(-1);
                        }
                    } else {
                        let start: String = self.text.chars().take(cursor.start).collect();
                        let end: String = self.text.chars().skip(cursor.end).collect();
                        self.text.clear();
                        self.text.push_str(&start);
                        self.text.push_str(&end);
                        cursor.end = cursor.start;
                    },
                    Left => if cursor.is_cursor() {
                        cursor.shift(-1);
                    },
                    Right => if cursor.is_cursor() && self.text.chars().count() > cursor.end {
                        cursor.shift(1);
                    },
                    Return => if self.text.chars().count() > 0 {
                        let TextBox { ref mut maybe_react, ref mut text, .. } = self;
                        if let Some(ref mut react) = *maybe_react {
                            react(*text);
                        }
                    },
                    LCtrl | RCtrl if !new_control_pressed => {
                        new_control_pressed = true;
                    },
                    A if new_control_pressed => {
                        if cursor.is_cursor() {
                            cursor.start = 0;
                            cursor.end = self.text.chars().count();
                        }
                    },
                    E if new_control_pressed => {
                        if cursor.is_cursor() {
                            cursor.start = self.text.chars().count();
                            cursor.end = self.text.chars().count();
                        }
                    },
                    _ => (),
                }
            }

            for key in input.released_keys.iter() {
                match *key {
                    LCtrl | RCtrl if new_control_pressed => {
                        new_control_pressed = false;
                    },
                    _ => (),
                }
            }

            // In case the string text was mutated with the `react` function, we need to make sure
            // the cursor is limited to the current number of chars.
            cursor.limit_end_to(self.text.chars().count());

            new_interaction = Interaction::Captured(View { cursor: cursor, .. captured });
        }

        // Check the interactions to determine whether we need to capture or uncapture the keyboard.
        match (state.view().interaction, new_interaction) {
            (Interaction::Uncaptured(_), Interaction::Captured(_)) => { ui.capture_keyboard(); },
            (Interaction::Captured(_), Interaction::Uncaptured(_)) => { ui.uncapture_keyboard(); },
            _ => (),
        }

        if state.view().interaction != new_interaction {
            state.update(|state| state.interaction = new_interaction);
        }

        if &state.view().text[..] != &self.text[..] {
            state.update(|state| state.text = self.text.clone());
        }

        if state.view().control_pressed != new_control_pressed {
            state.update(|state| state.control_pressed = new_control_pressed);
        }
    }

    /// Construct an Element from the given TextBox State.
    fn draw<C: CharacterCache>(args: widget::DrawArgs<Self, C>) -> Element {
        use elmesque::form::{self, collage, line, solid, text};
        use elmesque::text::Text;

        let widget::DrawArgs { rect, state, style, theme, glyph_cache, .. } = args;

        // Construct the frame and inner rectangle Forms.
        let (xy, dim) = rect.xy_dim();
        let frame = style.frame(theme);
        let pad_dim = vec2_sub(dim, [frame * 2.0; 2]);
        let color = state.interaction.color(style.color(theme));
        let frame_color = style.frame_color(theme);
        let frame_form = form::rect(dim[0], dim[1]).filled(frame_color);
        let inner_form = form::rect(pad_dim[0], pad_dim[1]).filled(color);
        let font_size = style.font_size(theme);
        let text_w = glyph_cache.width(font_size, &state.text[..]);
        let text_x = position::align_left_of(pad_dim[0], text_w) + TEXT_PADDING;
        let text_start_x = text_x - text_w / 2.0;

        let (maybe_cursor_form, text_form) = if let Interaction::Captured(view) = state.interaction {
            // Construct the Cursor's Form.
            let cursor = view.cursor;

            // This matters if the text is scrolled with the mouse.
            let cursor_idx = match cursor.anchor {
                Anchor::End => cursor.start,
                Anchor::Start | Anchor::None => cursor.end,
            };

            let cursor_x = cursor_position(glyph_cache, cursor_idx, text_start_x, font_size, &state.text);

            let cursor_form = if cursor.is_cursor() {
                let half_pad_h = pad_dim[1] / 2.0;
                line(solid(color.plain_contrast()), 0.0, half_pad_h, 0.0, -half_pad_h)
                    .alpha(0.75)
                    .shift_x(cursor_x)
            } else {
                let (block_xy, dim) = {
                    let (start, end) = (cursor.start, cursor.end);
                    let cursor_x = cursor_position(glyph_cache, start, text_start_x, font_size, &state.text);
                    let htext: String = state.text.chars().skip(start).take(end - start).collect();
                    let htext_w = glyph_cache.width(font_size, &htext);
                    ([cursor_x + htext_w / 2.0, 0.0], [htext_w, dim[1]])
                };
                form::rect(dim[0], dim[1] - frame * 2.0).filled(color.highlighted())
                    .shift(block_xy[0], block_xy[1])
            };

            // Construct the text's Form.
            let text_form = text(Text::from_string(state.text.clone())
                                     .color(color.plain_contrast())
                                     .height(font_size as f64)).shift_x(text_x.floor());

            (Some(cursor_form), text_form)
        } else {

            // Construct the text's Form.
            let text_form = text(Text::from_string(state.text.clone())
                                     .color(color.plain_contrast())
                                     .height(font_size as f64)).shift_x(text_x.floor());
            (None, text_form)
        };

        // Chain the Forms and shift them into position.
        let form_chain = Some(frame_form).into_iter()
            .chain(Some(inner_form))
            .chain(maybe_cursor_form)
            .chain(Some(text_form))
            .map(|form| form.shift(xy[0], xy[1]));

        // Collect the Forms into a renderable `Element`.
        collage(dim[0] as i32, dim[1] as i32, form_chain.collect())
    }

}

impl Style {

    /// Construct the default Style.
    pub fn new() -> Style {
        Style {
            maybe_color: None,
            maybe_frame: None,
            maybe_frame_color: None,
            maybe_font_size: None,
        }
    }

    /// Get the Color for an Element.
    pub fn color(&self, theme: &Theme) -> Color {
        self.maybe_color.or(theme.maybe_text_box.as_ref().map(|default| {
            default.style.maybe_color.unwrap_or(theme.shape_color)
        })).unwrap_or(theme.shape_color)
    }

    /// Get the frame for an Element.
    pub fn frame(&self, theme: &Theme) -> f64 {
        self.maybe_frame.or(theme.maybe_text_box.as_ref().map(|default| {
            default.style.maybe_frame.unwrap_or(theme.frame_width)
        })).unwrap_or(theme.frame_width)
    }

    /// Get the frame Color for an Element.
    pub fn frame_color(&self, theme: &Theme) -> Color {
        self.maybe_frame_color.or(theme.maybe_text_box.as_ref().map(|default| {
            default.style.maybe_frame_color.unwrap_or(theme.frame_color)
        })).unwrap_or(theme.frame_color)
    }

    /// Get the label font size for an Element.
    pub fn font_size(&self, theme: &Theme) -> FontSize {
        const DEFAULT_FONT_SIZE: u32 = 24;
        self.maybe_font_size.or(theme.maybe_text_box.as_ref().map(|default| {
            default.style.maybe_font_size.unwrap_or(DEFAULT_FONT_SIZE)
        })).unwrap_or(DEFAULT_FONT_SIZE)
    }

}

impl<'a, F> Colorable for TextBox<'a, F> {
    fn color(mut self, color: Color) -> Self {
        self.style.maybe_color = Some(color);
        self
    }
}

impl<'a, F> Frameable for TextBox<'a, F> {
    fn frame(mut self, width: f64) -> Self {
        self.style.maybe_frame = Some(width);
        self
    }
    fn frame_color(mut self, color: Color) -> Self {
        self.style.maybe_frame_color = Some(color);
        self
    }
}

