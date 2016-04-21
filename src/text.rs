//! Text layout logic.

/// Logic and types specific to individual character layout.
pub mod char {
    use {CharacterCache, FontSize, GlyphCache, Scalar};

    /// An iterator yielding the widths of each consecutive character in some sequence.
    #[derive(Clone)]
    pub struct Widths<'a, I, C: 'a> {
        chars: I,
        cache: &'a GlyphCache<C>,
        font_size: FontSize,
    }

    /// Some position along the X axis (used within `CharXs`).
    pub type X = Scalar;

    /// The half of the width of some character (used within `CharXs`).
    pub type HalfW = Scalar;

    /// An iterator that converts the given sequnce of `char`s into their consecutive positions along
    /// the x-axis.
    #[derive(Clone)]
    pub struct Xs<'a, I, C: 'a> {
        widths: Widths<'a, I, C>,
        maybe_next: Option<(HalfW, X)>,
    }

    impl<'a, I, C> Xs<'a, I, C> {

        /// Consumes the `Xs` and produces another whose first char's *x* position (that is, the
        /// position of the middle of the character along the *x* axis) is at the given `Scalar`.
        ///
        /// All following *x* positions will follow from the first character's.
        pub fn from_first_x(self, first_x: Scalar) -> Self {
            let Xs { maybe_next, widths } = self;
            Xs {
                maybe_next: maybe_next.map(|(half_w, _)| (half_w, first_x)),
                widths: widths,
            }
        }

        /// Consumes the `Xs` and produces another whose first char's left edge is aligned with the
        /// given `Scalar`.
        ///
        /// All following *x* positions will follow from the first character's.
        pub fn from_left_x(self, left_x: Scalar) -> Self {
            let Xs { maybe_next, widths } = self;
            Xs {
                maybe_next: maybe_next.map(|(half_w, _)| (half_w, left_x + half_w)),
                widths: widths,
            }
        }

    }

    /// Converts the given sequence of `char`s into their Scalar widths.
    pub fn widths<I, C>(chars: I,
                        cache: &GlyphCache<C>,
                        font_size: FontSize) -> Widths<I::IntoIter, C>
        where I: IntoIterator<Item=char>,
              C: CharacterCache,
    {
        Widths {
            chars: chars.into_iter(),
            cache: cache,
            font_size: font_size,
        }
    }

    /// Converts the given sequnce of `char`s into their consecutive positions along the x-axis.
    ///
    /// The first character's *x* position will be `0.0`, and all following characters will follow
    /// sequentially.
    ///
    /// To offset the first character's *x* position, consider chaining this with either
    /// `Xs::from_first_x` or `Xs::from_left_x`.
    pub fn xs<I, C>(chars: I,
                    cache: &GlyphCache<C>,
                    font_size: FontSize) -> Xs<I::IntoIter, C>
        where I: IntoIterator<Item=char>,
              C: CharacterCache,
    {
        let mut widths = widths(chars, cache, font_size);
        let maybe_first = widths.next().map(|w| (w / 2.0, 0.0));
        Xs {
            maybe_next: maybe_first,
            widths: widths,
        }
    }

    impl<'a, I, C> Iterator for Widths<'a, I, C>
        where I: Iterator<Item=char>,
              C: CharacterCache,
    {
        type Item = Scalar;
        fn next(&mut self) -> Option<Self::Item> {
            let Widths { font_size, cache, ref mut chars } = *self;
            chars.next().map(|ch| cache.char_width(font_size, ch))
        }
    }

    impl<'a, I, C> Iterator for Xs<'a, I, C>
        where I: Iterator<Item=char>,
              C: CharacterCache,
    {
        type Item = Scalar;
        fn next(&mut self) -> Option<Self::Item> {
            self.maybe_next.take().map(|(half_w, x)| {
                self.maybe_next = self.widths.next().map(|next_w| {
                    let next_half_w = next_w / 2.0;
                    let step = half_w + next_half_w;
                    (next_half_w, x + step)
                });
                x
            })
        }
    }

}

/// Logic and types specific to layout of `str`s.
pub mod str {
    use {CharacterCache, FontSize, GlyphCache, Scalar};
    pub use self::line::Lines;

    /// An iterator yielding the widths of each consecutive `&str` in some sequence.
    #[derive(Clone)]
    pub struct Widths<'a, I, C: 'a> {
        strs: I,
        cache: &'a GlyphCache<C>,
        font_size: FontSize,
    }

    /// Converts the given sequence of `&str`s into their Scalar widths.
    pub fn widths<I, C>(strs: I,
                        cache: &GlyphCache<C>,
                        font_size: FontSize) -> Widths<I::IntoIter, C>
        where for<'a> I: IntoIterator<Item=&'a str>,
                      C: CharacterCache,
    {
        Widths {
            strs: strs.into_iter(),
            cache: cache,
            font_size: font_size,
        }
    }

    impl<'a, I, C> Iterator for Widths<'a, I, C>
        where for<'b> I: Iterator<Item=&'b str>,
              C: CharacterCache,
    {
        type Item = Scalar;
        fn next(&mut self) -> Option<Self::Item> {
            let Widths { font_size, cache, ref mut strs } = *self;
            strs.next().map(|s| cache.width(font_size, s))
        }
    }

    /// Logic and types related to handling and creating line breaks and line wrapping.
    pub mod line {
        use {CharacterCache, FontSize, GlyphCache, Scalar};

        /// The two types of **Break** indices returned by the **WrapIndicesBy** iterators.
        #[derive(Copy, Clone, Debug, PartialEq)]
        pub enum Break {
            /// The first `usize` is an index at which the string should wrap due to exceeding a
            /// maximum width.
            ///
            /// The second `usize` is the byte length which should be skipped in order to reach the
            /// first non-whitespace character to use as the beginning of the next line.
            Wrap(usize, usize),
            /// An index at which the string breaks due to a newline character, along with the
            /// width of the "newline" token in bytes.
            Newline(usize, usize),
        }

        /// An iterator that yields the indices at which some text should wrap in accordance with
        /// the given wrap function.
        #[derive(Clone)]
        pub struct BreaksBy<'a, C: 'a, F> {
            font_size: FontSize,
            cache: &'a GlyphCache<C>,
            text: &'a str,
            max_width: Scalar,
            break_fn: F,
            start: usize,
        }

        /// A function that returns the first index at which the text should wrap for the given max
        /// width.
        pub type NextBreakFn<C> = fn(&GlyphCache<C>, FontSize, &str, Scalar) -> Option<Break>;

        /// An iterator that yields the indices at which some text should wrap via a character.
        pub type BreaksByCharacter<'a, C> = BreaksBy<'a, C, NextBreakFn<C>>;

        /// An iterator that yields the indices at which some text should wrap via whitespace.
        pub type BreaksByWhitespace<'a, C> = BreaksBy<'a, C, NextBreakFn<C>>;

        /// A wrapper over an iterator yielding **Break**s that yields each line divided by the
        /// breaks.
        pub struct Lines<'a, I> {
            text: &'a str,
            breaks: I,
        }

        /// An iterator yielding lines for text wrapped with the given function.
        pub type WrappedBy<'a, C, F> = Lines<'a, BreaksBy<'a, C, F>>;

        /// An iterator yielding lines for text wrapped via the first character exceeding a max
        /// width.
        pub type WrappedByCharacter<'a, C> = WrappedBy<'a, C, NextBreakFn<C>>;

        /// An iterator yielding lines for text wrapped via the first character exceeding a max
        /// width.
        pub type WrappedByWhitespace<'a, C> = WrappedBy<'a, C, NextBreakFn<C>>;


        impl Break {

            /// Extracts the index at which the break occurs within the text (i.e. the index
            /// following the last byte of the line).
            pub fn index(self) -> usize {
                match self {
                    Break::Wrap(idx, _) | Break::Newline(idx, _) => idx,
                }
            }

            /// Returns the next index at which the text will break by either:
            /// - A newline character.
            /// - A line wrap at the beginning of the first character exceeding the `max_width`.
            pub fn next_by_character<C>(cache: &GlyphCache<C>,
                                        font_size: FontSize,
                                        text: &str,
                                        max_width: Scalar) -> Option<Self>
                where C: CharacterCache,
            {
                let mut width = 0.0;
                let mut char_indices = text.char_indices().peekable();
                while let Some((i, ch)) = char_indices.next() {

                    // Check for a newline.
                    if ch == '\r' {
                        if let Some(&(_, '\n')) = char_indices.peek() {
                            return Some(Break::Newline(i, 2))
                        }
                    } else if ch == '\n' {
                        return Some(Break::Newline(i, 1));
                    }

                    // Update the width.
                    width += cache.char_width(font_size, ch);

                    // Check for a line wrap.
                    if width > max_width {
                        return Some(Break::Wrap(i, 0));
                    }
                }
                None
            }

            /// Returns the next index at which the text will break by either:
            /// - A newline character.
            /// - A line wrap at the beginning of the whitespace that preceeds the first word
            /// exceeding the `max_width`.
            pub fn next_by_whitespace<C>(cache: &GlyphCache<C>,
                                         font_size: FontSize,
                                         text: &str,
                                         max_width: Scalar) -> Option<Self>
                where C: CharacterCache,
            {
                let mut width = 0.0;
                let mut last_whitespace_start = 0;
                let mut char_indices = text.char_indices().peekable();
                while let Some((i, ch)) = char_indices.next() {

                    // Check for a newline.
                    if ch == '\r' {
                        if let Some(&(_, '\n')) = char_indices.peek() {
                            return Some(Break::Newline(i, 2))
                        }
                    } else if ch == '\n' {
                        return Some(Break::Newline(i, 1));
                    }

                    // Check for a new whitespace.
                    else if ch.is_whitespace() {
                        last_whitespace_start = i;
                    }

                    // Update the width.
                    width += cache.char_width(font_size, ch);

                    // Check for a line wrap.
                    if width > max_width {
                        return Some(Break::Wrap(last_whitespace_start, 1));
                    }
                }
                None
            }

        }

        impl<'a, I> Lines<'a, I> {
            /// Construct a new **Lines** iterator from the given text and breaks.
            ///
            /// **Note:** This function assumes that the given `breaks` correctly represent all
            /// line breaks within the given `text`, starting from the `0`th byte index.
            pub fn new(text: &'a str, breaks: I) -> Self {
                Lines {
                    text: text,
                    breaks: breaks,
                }
            }
        }


        /// An iterator that yields the indices at which some text should wrap in accordance with
        /// the given wrap function.
        pub fn breaks_by<'a, C, F>(text: &'a str,
                                   cache: &'a GlyphCache<C>,
                                   font_size: FontSize,
                                   max_width: Scalar,
                                   break_fn: F) -> BreaksBy<'a, C, F>
        {
            BreaksBy {
                text: text,
                cache: cache,
                font_size: font_size,
                start: 0,
                max_width: max_width,
                break_fn: break_fn,
            }
        }

        /// An iterator that yields the indices at which some text should wrap via a character.
        pub fn breaks_by_character<'a, C>(text: &'a str,
                                          cache: &'a GlyphCache<C>,
                                          font_size: FontSize,
                                          max_width: Scalar) -> BreaksByCharacter<'a, C>
            where C: CharacterCache,
        {
            breaks_by(text, cache, font_size, max_width, Break::next_by_character)
        }

        /// An iterator that yields the indices at which some text should wrap via whitespace.
        pub fn breaks_by_whitespace<'a, C>(text: &'a str,
                                           cache: &'a GlyphCache<C>,
                                           font_size: FontSize,
                                           max_width: Scalar) -> BreaksByWhitespace<'a, C>
            where C: CharacterCache,
        {
            breaks_by(text, cache, font_size, max_width, Break::next_by_whitespace)
        }

        /// An iterator that behaves the same as `text.lines()` but inserts a break before the
        /// first character that would cause the line to exceed the given `max_width`.
        pub fn wrapped_by<'a, C, F>(text: &'a str,
                                    cache: &'a GlyphCache<C>,
                                    font_size: FontSize,
                                    max_width: Scalar,
                                    wrap_fn: F) -> WrappedBy<'a, C, F>
        {
            let breaks = breaks_by(text, cache, font_size, max_width, wrap_fn);
            Lines::new(text, breaks)
        }

        /// An iterator that behaves the same as `text.lines()` but inserts a break before the
        /// first character that would cause the line to exceed the given `max_width`.
        pub fn wrapped_by_character<'a, C>(text: &'a str,
                                           cache: &'a GlyphCache<C>,
                                           font_size: FontSize,
                                           max_width: Scalar) -> WrappedByCharacter<'a, C>
            where C: CharacterCache,
        {
            let breaks = breaks_by_character(text, cache, font_size, max_width);
            Lines::new(text, breaks)
        }

        /// An iterator that behaves the same as `text.lines()` but inserts a break before the
        /// first character that would cause the line to exceed the given `max_width`.
        pub fn wrapped_by_whitespace<'a, C>(text: &'a str,
                                            cache: &'a GlyphCache<C>,
                                            font_size: FontSize,
                                            max_width: Scalar) -> WrappedByWhitespace<'a, C>
            where C: CharacterCache,
        {
            let breaks = breaks_by_character(text, cache, font_size, max_width);
            Lines::new(text, breaks)
        }


        impl<'a, C, F> Iterator for BreaksBy<'a, C, F>
            where C: CharacterCache,
                  for<'b> F: FnMut(&'b GlyphCache<C>, FontSize, &'b str, Scalar) -> Option<Break>,
        {
            /// The index into the start of the line along with the line's break if it has one.
            type Item = (usize, Option<Break>);
            fn next(&mut self) -> Option<Self::Item> {
                let BreaksBy {
                    cache,
                    font_size,
                    ref text,
                    max_width,
                    ref mut break_fn,
                    ref mut start,
                } = *self;

                match break_fn(cache, font_size, &text[*start..], max_width) {
                    Some(next) => {
                        let next = match next {
                            Break::Newline(idx, width) => Break::Newline(*start + idx, width),
                            Break::Wrap(idx, width) => Break::Wrap(*start + idx, width),
                        };
                        let range = (*start, Some(next));
                        *start = match next {
                            Break::Newline(idx, width) => idx + width,
                            Break::Wrap(idx, width) => idx + width,
                        };
                        Some(range)
                    },
                    None => if *start < text.len() {
                        let last = Some((*start, None));
                        *start = text.len();
                        last
                    } else {
                        None
                    },
                }
            }
        }

        impl<'a, I> Iterator for Lines<'a, I>
            where I: Iterator<Item=(usize, Option<Break>)>,
        {
            type Item = &'a str;
            fn next(&mut self) -> Option<Self::Item> {
                let Lines {
                    ref text,
                    ref mut breaks,
                } = *self;
                breaks.next().map(|(start, maybe_break)| match maybe_break {
                    Some(line_break) => &text[start..line_break.index()],
                    None => &text[start..],
                })
            }
        }

    }

}
