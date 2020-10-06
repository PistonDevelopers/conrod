use rendy::{
    hal::format::Format,
    mesh::{AsAttribute, AsVertex, VertexFormat},
};

/// The position of the vertex within vector space.
///
/// [-1.0, 1.0] is the leftmost, bottom position of the display.
/// [1.0, -1.0] is the rightmost, top position of the display.
#[repr(transparent)]
#[derive(Debug, Default, Clone, Copy, PartialEq, PartialOrd)]
pub struct Position(pub [f32; 2]);

/// The coordinates of the texture used by this `Vertex`.
///
/// [0.0, 0.0] is the leftmost, top position of the texture.
/// [1.0, 1.0] is the rightmost, bottom position of the texture.
#[repr(transparent)]
#[derive(Debug, Default, Clone, Copy, PartialEq, PartialOrd)]
pub struct TexCoords(pub [f32; 2]);

/// A color associated with the `Vertex`.
///
/// The way that the color is used depends on the `mode`.
#[repr(transparent)]
#[derive(Debug, Default, Clone, Copy, PartialEq, PartialOrd)]
pub struct Color(pub [f32; 4]);

/// The mode with which the `Vertex` will be drawn within the fragment shader.
///
/// `0` for rendering text.
/// `1` for rendering an image.
/// `2` for rendering non-textured 2D geometry.
///
/// If any other value is given, the fragment shader will not output any color.
#[repr(transparent)]
#[derive(Debug, Default, Clone, Copy, PartialEq, PartialOrd)]
pub struct Mode(pub u32);

/// The `Vertex` type passed to the vertex shader.
#[derive(Debug, Default, Clone, Copy, PartialEq, PartialOrd)]
#[repr(C)]
pub struct Vertex {
    pub pos: Position,
    pub uv: TexCoords,
    pub color: Color,
    pub mode: Mode,
}

impl From<[f32; 2]> for Position {
    fn from(v: [f32; 2]) -> Self {
        Position(v)
    }
}

impl From<[f32; 2]> for TexCoords {
    fn from(v: [f32; 2]) -> Self {
        TexCoords(v)
    }
}

impl From<[f32; 4]> for Color {
    fn from(v: [f32; 4]) -> Self {
        Color(v)
    }
}

impl From<u32> for Mode {
    fn from(v: u32) -> Self {
        Mode(v)
    }
}

impl AsAttribute for Position {
    const NAME: &'static str = "pos";
    const FORMAT: Format = Format::Rg32Sfloat;
}

impl AsAttribute for TexCoords {
    const NAME: &'static str = "uv";
    const FORMAT: Format = Format::Rg32Sfloat;
}

impl AsAttribute for Color {
    const NAME: &'static str = "color";
    const FORMAT: Format = Format::Rgba32Sfloat;
}

impl AsAttribute for Mode {
    const NAME: &'static str = "mode";
    const FORMAT: Format = Format::R32Uint;
}

impl AsVertex for Vertex {
    fn vertex() -> VertexFormat {
        VertexFormat::new((
            Position::vertex(),
            TexCoords::vertex(),
            Color::vertex(),
            Mode::vertex(),
        ))
    }
}
