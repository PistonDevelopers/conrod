
use {
    CharacterCache,
    Color,
    Colorable,
    Dimension,
    FontSize,
    Horizontal,
    LineBreak,
    Range,
    Rect,
    Scalar,
    Theme,
    Ui,
    Widget,
};
use widget;


/// Displays some given text centred within a rectangular area.
///
/// By default, the rectangular dimensions are fit to the area occuppied by the text.
///
/// If some horizontal dimension is given, the text will automatically wrap to the width and align
/// in accordance with the produced **Horizontal**.
pub struct Text<'a> {
    /// Data necessary and common for all widget builder types.
    pub common: widget::CommonBuilder,
    /// The text to be drawn by the **Text**.
    pub text: &'a str,
    /// Unique styling for the **Text**.
    pub style: Style,
}


/// The styling for a **Text**'s graphics.
#[derive(Copy, Clone, Debug, PartialEq, RustcEncodable, RustcDecodable)]
pub struct Style {
    /// The font size for the **Text**.
    pub maybe_font_size: Option<FontSize>,
    /// The color of the **Text**.
    pub maybe_color: Option<Color>,
    /// Whether or not the text should wrap around the width.
    pub maybe_wrap: Option<Option<Wrap>>,
    /// The spacing between consecutive lines.
    pub maybe_line_spacing: Option<Scalar>,
    // /// The typeface with which the Text is rendered.
    // pub maybe_typeface: Option<Path>,
    // /// The line styling for the text.
    // pub maybe_line: Option<Option<Line>>,
}

/// The unique kind for the widget.
pub const KIND: widget::Kind = "Text";

/// The way in which text should wrap around the width.
#[derive(Copy, Clone, Debug, PartialEq, RustcEncodable, RustcDecodable)]
pub enum Wrap {
    /// Wrap at the first character that exceeds the width.
    Character,
    /// Wrap at the first word that exceeds the width.
    Whitespace,
}

// /// Line styling for the **Text**.
// pub enum Line {
//     /// Underline the text.
//     Under,
//     /// Overline the text.
//     Over,
//     /// Strikethrough the text.
//     Through,
// }

/// The state to be stored between updates for the **Text**.
#[derive(Clone, Debug, PartialEq)]
pub struct State {
    /// An owned version of the string.
    string: String,
    /// An index range for each line in the string.
    line_breaks: Vec<(usize, Option<LineBreak>)>,
}


impl<'a> Text<'a> {

    /// Build a new **Text** widget.
    pub fn new(text: &'a str) -> Self {
        Text {
            common: widget::CommonBuilder::new(),
            text: text,
            style: Style::new(),
        }
    }

    /// Build the **Text** with the given font size.
    pub fn font_size(mut self, size: FontSize) -> Self {
        self.style.maybe_font_size = Some(size);
        self
    }

    /// Specify that the **Text** should not wrap lines around the width.
    pub fn no_line_wrap(mut self) -> Self {
        self.style.maybe_wrap = Some(None);
        self
    }

    /// Line wrap the **Text** at the beginning of the first word that exceeds the width.
    pub fn wrap_by_word(mut self) -> Self {
        self.style.maybe_wrap = Some(Some(Wrap::Whitespace));
        self
    }

    /// Line wrap the **Text** at the beginning of the first character that exceeds the width.
    pub fn wrap_by_character(mut self) -> Self {
        self.style.maybe_wrap = Some(Some(Wrap::Character));
        self
    }

    /// Build the **Text** with the given **Style**.
    pub fn with_style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

}

impl Style {

    /// Construct the default Style.
    pub fn new() -> Style {
        Style {
            maybe_color: None,
            maybe_font_size: None,
            maybe_wrap: None,
            maybe_line_spacing: None,
        }
    }

    /// Get the Color for an Element.
    pub fn color(&self, theme: &Theme) -> Color {
        self.maybe_color.unwrap_or(theme.label_color)
    }

    /// Get the text font size for an Element.
    pub fn font_size(&self, theme: &Theme) -> FontSize {
        self.maybe_font_size.unwrap_or(theme.font_size_medium)
    }

    /// Get whether or not the text should wrap around if the length exceeds the width.
    pub fn wrap(&self, theme: &Theme) -> Option<Wrap> {
        const DEFAULT_WRAP: Option<Wrap> = Some(Wrap::Whitespace);
        self.maybe_wrap.or(theme.widget_style::<Self>(KIND).map(|default| {
            default.style.maybe_wrap.unwrap_or(DEFAULT_WRAP)
        })).unwrap_or(DEFAULT_WRAP)
    }

    /// Get the length of the separating space between consecutive lines of text.
    pub fn line_spacing(&self, theme: &Theme) -> Scalar {
        const DEFAULT_LINE_SPACING: Scalar = 1.0;
        self.maybe_line_spacing.or(theme.widget_style::<Self>(KIND).map(|default| {
            default.style.maybe_line_spacing.unwrap_or(DEFAULT_LINE_SPACING)
        })).unwrap_or(DEFAULT_LINE_SPACING)
    }

}


impl<'a> Widget for Text<'a> {
    type State = State;
    type Style = Style;

    fn common(&self) -> &widget::CommonBuilder {
        &self.common
    }

    fn common_mut(&mut self) -> &mut widget::CommonBuilder {
        &mut self.common
    }

    fn unique_kind(&self) -> &'static str {
        KIND
    }

    fn init_state(&self) -> State {
        State {
            string: String::new(),
            line_breaks: Vec::new(),
        }
    }

    fn style(&self) -> Style {
        self.style.clone()
    }

    /// If no specific width was given, we'll use the width of the widest line as a default.
    fn default_x_dimension<C: CharacterCache>(&self, ui: &Ui<C>) -> Dimension {
        let font_size = self.style.font_size(&ui.theme);
        let mut max_width = 0.0;
        for line in self.text.lines() {
            let width = ui.glyph_cache.width(font_size, line);
            max_width = ::utils::partial_max(max_width, width);
        }
        Dimension::Absolute(max_width)
    }

    /// If no specific height was given, we'll use the total height of the text as a default.
    fn default_y_dimension<C: CharacterCache>(&self, ui: &Ui<C>) -> Dimension {
        use position::Sizeable;
        let text = &self.text;
        let font_size = self.style.font_size(&ui.theme);
        let num_lines = match self.style.wrap(&ui.theme) {
            None => text.lines().count(),
            Some(wrap) => match self.get_width(ui) {
                None => text.lines().count(),
                Some(max_w) => match wrap {
                    Wrap::Character =>
                        ui.glyph_cache.line_breaks_by_character(font_size, text, max_w).count(),
                    Wrap::Whitespace =>
                        ui.glyph_cache.line_breaks_by_whitespace(font_size, text, max_w).count(),
                },
            },
        };
        let line_spacing = self.style.line_spacing(&ui.theme);
        let height = total_height(num_lines, font_size, line_spacing);
        Dimension::Absolute(height)
    }

    /// Update the state of the Text.
    fn update<C: CharacterCache>(self, args: widget::UpdateArgs<Self, C>) {
        let widget::UpdateArgs { rect, state, style, ui, .. } = args;
        let Text { text, .. } = self;

        let maybe_wrap = style.wrap(ui.theme());
        let font_size = style.font_size(ui.theme());

        // Produces an iterator yielding the line breaks for the `text`.
        let new_line_breaks = || match maybe_wrap {
            None =>
                // This branch could be faster if we just used `.lines()` somehow.
                ui.glyph_cache().line_breaks_by_character(font_size, text, ::std::f64::MAX),
            Some(Wrap::Character) =>
                ui.glyph_cache().line_breaks_by_character(font_size, text, rect.w()),
            Some(Wrap::Whitespace) =>
                ui.glyph_cache().line_breaks_by_whitespace(font_size, text, rect.w()),
        };

        // If the string is different, we must update both the string and the line breaks.
        if &state.view().string[..] != text {
            state.update(|state| {
                state.string = text.to_owned();
                state.line_breaks = new_line_breaks().collect();
            });

        // Otherwise, we'll check to see if we have to update the line breaks.
        } else {
            use utils::write_if_different;
            use std::borrow::Cow;

            // Compare the line_breaks and only collect the new ones if they are different.
            let maybe_new_line_breaks = {
                let line_breaks = &state.view().line_breaks[..];
                match write_if_different(line_breaks, new_line_breaks()) {
                    Cow::Owned(new) => Some(new),
                    _ => None,
                }
            };

            if let Some(new_line_breaks) = maybe_new_line_breaks {
                state.update(|state| state.line_breaks = new_line_breaks);
            }
        }
    }

}


impl<'a> Colorable for Text<'a> {
    fn color(mut self, color: Color) -> Self {
        self.style.maybe_color = Some(color);
        self
    }
}


impl State {

    /// Iterator that yields a new line at both "newline"s (i.e. `\n`) and `line_wrap_indices`.
    pub fn lines(&self) -> Lines {
        ::glyph_cache::Lines::new(&self.string, self.line_breaks.iter().cloned())
    }

    /// The total height from the top of the first line to the bottom of the last line.
    pub fn total_height(&self, font_size: FontSize, line_spacing: Scalar) -> Scalar {
        let num_lines = self.lines().count();
        total_height(num_lines, font_size, line_spacing)
    }

    /// An iterator yielding a **Rect** (representing the absolute position and dimensions) for
    /// every **line** in the `string`
    pub fn line_rects<'a>(&'a self,
                          container: Rect,
                          h_align: Horizontal,
                          font_size: FontSize,
                          line_spacing: Scalar) -> LineRects<'a, Lines<'a>>
    {
        let lines = self.lines();
        LineRects::new(lines, container, h_align, font_size, line_spacing)
    }

    // /// An iterator yielding a **Rect** (representing the absolute position and dimensions) for
    // /// every **character** in the `string`
    // pub fn char_rects<'a, C>(&'a self,
    //                          cache: &'a GlyphCache<C>,
    //                          container: Rect,
    //                          h_align: Horizontal,
    //                          font_size: FontSize,
    //                          line_spacing: Scalar) -> CharRects<'a, C>
    //     where C: CharacterCache,
    // {
    //     let lines = self.lines();
    //     let line_rects = self.line_rects(cache, container, h_align, font_size, line_spacing);
    //     let mut lines_with_rects = lines.zip(line_rects);
    //     let maybe_first_line = lines_with_rects.next().and_then(|(line, line_rect)| {
    //         char_widths_and_xs_for_line(cache, font_size, line, line_rect)
    //             .map(|char_widths_and_xs| (char_widths_and_xs, line_rect.y()))
    //     });
    //     let char_height = font_size as Scalar;
    //     CharRects {
    //         cache: cache,
    //         font_size: font_size,
    //         lines_with_rects: lines_with_rects,
    //         maybe_current_line: maybe_first_line,
    //     }
    // }

}


/// Calculate the total height of the text from the given number of lines, font_size and
/// line_spacing.
pub fn total_height(num_lines: usize, font_size: FontSize, line_spacing: Scalar) -> Scalar {
    font_size as Scalar * num_lines as Scalar + line_spacing * (num_lines - 1) as Scalar
}


/// Shorthand for the **Lines** iterator yielded by **State::lines**.
pub type Lines<'a> =
    ::glyph_cache::Lines<'a, ::std::iter::Cloned<::std::slice::Iter<'a, (usize, Option<LineBreak>)>>>;


/// An walker yielding a **Rect** (representing the absolute position and dimensions) for
/// every **line** in its given line iterator.
pub struct LineRects<'a, I: 'a> {
    lines: I,
    font_size: FontSize,
    y_step: Scalar,
    h_align: Horizontal,
    container_x: Range,
    y: Scalar,
    strs: ::std::marker::PhantomData<&'a ()>,
}


impl<'a, I> LineRects<'a, I> {

    /// Construct a new **LineRects**.
    pub fn new(lines: I,
               container: Rect,
               h_align: Horizontal,
               font_size: FontSize,
               line_spacing: Scalar) -> LineRects<'a, I>
        where I: Iterator<Item=&'a str>,
    {
        let height = font_size as Scalar;
        LineRects {
            lines: lines,
            font_size: font_size,
            y_step: -(line_spacing + height),
            h_align: h_align,
            container_x: container.x,
            y: Range::new(0.0, height).align_end_of(container.y).middle(),
            strs: ::std::marker::PhantomData,
        }
    }

    /// Returns the next line **Rect**.
    ///
    /// Returns **None** if there are no more lines in the **Lines** iter.
    ///
    /// This can be used similarly to an **Iterator**, i.e:
    ///
    /// `while let Some(rect) = line_rects.next() { ... }`
    ///
    /// The difference being that this method does not require borrowing the **CharacterCache**.
    pub fn next<C>(&mut self, cache: &mut C) -> Option<Rect>
        where I: Iterator<Item=&'a str>,
              C: CharacterCache,
    {
        self.next_with_line(cache).map(|(rect, _)| rect)
    }

    /// The same as [**LineRects::next**](./struct.LineRects@method.next) but also yields the
    /// line's `&'a str` alongside the **Rect**.
    pub fn next_with_line<C>(&mut self, cache: &mut C) -> Option<(Rect, &'a str)>
        where I: Iterator<Item=&'a str>,
              C: CharacterCache,
    {
        let LineRects { ref mut lines, font_size, y_step, h_align, container_x, ref mut y, .. } = *self;
        lines.next().map(|line| {
            let w = cache.width(font_size, line);
            let h = font_size as Scalar;
            let w_range = Range::new(0.0, w);
            let x = match h_align {
                Horizontal::Left => w_range.align_start_of(container_x),
                Horizontal::Middle => w_range.align_middle_of(container_x),
                Horizontal::Right => w_range.align_end_of(container_x),
            }.middle();
            *y += y_step;
            let xy = [x, *y];
            let wh = [w, h];
            let rect = Rect::from_xy_dim(xy, wh);
            (rect, line)
        })
    }

}


// /// Shorthand for the `CharWidths` iterator used within the `CharRects` iterator.
// pub type CharWidths<'a, C> = ::glyph_cache::CharWidths<'a, C, ::std::str::Chars<'a>>;
// 
// /// Shorthand for the `CharXs` iterator used within the `CharRects` iterator.
// pub type CharXs<'a, C> = ::glyph_cache::CharXs<'a, C, ::std::str::Chars<'a>>;
// 
// /// Shorthand for the `Zip`ped `CharWidths` and `CharXs` iterators used within the `CharRects`
// /// iterator.
// pub type CharWidthsAndXs<'a, C> = ::std::iter::Zip<CharWidths<'a, C>, CharXs<'a, C>>;
// 
// /// Shorthand for the `Zip`ped `Lines` and `LineRects` iterators used within the `CharRects`
// /// iterator.
// pub type LinesWithRects<'a, C> = ::std::iter::Zip<Lines<'a>, LineRects<'a, C>>;
// 
// type Y = Scalar;
// type CurrentLine<'a, C> = (CharWidthsAndXs<'a, C>, Y);


// /// An iterator yielding a **Rect** (representing the absolute position and dimensions) for
// /// every **character** in the `string`
// pub struct CharRects<'a, C: 'a> {
//     font_size: FontSize,
//     cache: &'a GlyphCache<C>,
//     lines_with_rects: LinesWithRects<'a, C>,
//     maybe_current_line: Option<CurrentLine<'a, C>>,
// }
// 
// 
// impl<'a, C> Iterator for CharRects<'a, C>
//     where C: CharacterCache,
// {
//     type Item = Rect;
//     fn next(&mut self) -> Option<Self::Item> {
//         let CharRects {
//             font_size,
//             cache,
//             ref mut lines_with_rects,
//             ref mut maybe_current_line,
//         } = *self;
// 
//         // Continue trying each line until we find one with characters.
//         loop {
//             match *maybe_current_line {
//                 // If the current line has some characters, return the next `Rect`.
//                 Some((ref mut char_widths_and_xs, y)) => match char_widths_and_xs.next() {
//                     Some((w, x)) => {
//                         let xy = [x, y];
//                         let dim = [w, font_size as Scalar];
//                         return Some(Rect::from_xy_dim(xy, dim));
//                     },
//                     None => (),
//                 },
//                 // If we have no more lines, we're done.
//                 None => return None,
//             }
// 
//             // If our line had no more characters, make the next line the current line.
//             *maybe_current_line = lines_with_rects.next().and_then(|(line, line_rect)| {
//                 char_widths_and_xs_for_line(cache, font_size, line, line_rect)
//                     .map(|char_widths_and_xs| (char_widths_and_xs, line_rect.y()))
//             });
//         }
//     }
// }
// 
// 
// /// Returns a `CharWidthsAndXs` iterator for the given `line`.
// ///
// /// Rturns `None` if there are no characters within the `line`.
// fn char_widths_and_xs_for_line<'a, C>(cache: &'a GlyphCache<C>,
//                                       font_size: FontSize,
//                                       line: &'a str,
//                                       line_rect: Rect) -> Option<CharWidthsAndXs<'a, C>>
//     where C: CharacterCache,
// {
//     line.chars().next().map(|ch| {
//         let ch_w = cache.char_width(font_size, ch);
//         let ch_w_range = Range::new(0.0..ch_w);
//         let start_x = ch_w_range.align_start_of(line_rect.x).middle();
//         let char_widths = cache.char_widths(font_size, line.chars());
//         let char_xs = cache.char_xs(font_size, start_x, line.chars());
//         char_widths.zip(char_xs)
//     })
// }
