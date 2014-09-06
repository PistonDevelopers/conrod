
/// An enum to diversify and simplify style-related arg passing.
pub enum Styling {
    DefaultStyle,
    StyleFromTheme(Theme),
    Style(Framing, Coloring),
}

