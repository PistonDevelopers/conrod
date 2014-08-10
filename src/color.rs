
use std::default::Default;
use utils::clampf32;
use std::rand::random;
use std::num::abs;

/// A basic color struct for general color use
/// made of red, green, blue and alpha elements.
#[deriving(Show, Clone, Encodable, Decodable)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {

    /// Basic constructor for a Color struct.
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Color {
        Color { r: r, g: g, b: b, a: a }
    }

    /// Basic constructor for a Black Color struct.
    pub fn black() -> Color {
        Color { r: 0f32, g: 0f32, b: 0f32, a: 1f32 }
    }

    /// Basic constructor for a White Color struct.
    pub fn white() -> Color {
        Color { r: 1f32, g: 1f32, b: 1f32, a: 1f32 }
    }

    /// Return color as a vector.
    pub fn as_vec(&self) -> Vec<f32> {
        vec![self.r, self.g, self.b, self.a]
    }

    /// Return color as a tuple.
    pub fn as_tuple(&self) -> (f32, f32, f32, f32) {
        (self.r, self.g, self.b, self.a)
    }

    /// Clamp the Color's values between 0f32 and 1f32.
    fn clamp(c: Color) -> Color {
        Color::new(clampf32(c.r), clampf32(c.g), clampf32(c.b), clampf32(c.a))
    }

    /// Return a highlighted version of the current Color.
    pub fn highlighted(&self) -> Color {
        let r = clampf32((1f32 - self.r) * 0.5f32 * self.r + self.r);
        let g = clampf32((1f32 - self.g) * 0.1f32 * self.g + self.g);
        let b = clampf32((1f32 - self.b) * 0.1f32 * self.b + self.b);
        let a = clampf32((1f32 - self.a) * 0.5f32 + self.a);
        Color::new(r, g, b, a)
    }

    /// Return a clicked version of the current Color.
    pub fn clicked(&self) -> Color {
        let r = clampf32((1f32 - self.r) * 0.75f32 + self.r);
        let g = clampf32((1f32 - self.g) * 0.25f32 + self.g);
        let b = clampf32((1f32 - self.b) * 0.25f32 + self.b);
        let a = clampf32((1f32 - self.a) * 0.75f32 + self.a);
        Color::new(r, g, b, a)
    }

    /// Return a random color.
    pub fn random() -> Color {
        let r = random::<f32>();
        let g = random::<f32>();
        let b = random::<f32>();
        let a = 1f32;
        Color::new(r, g, b, a)
    }

    /// Return an inverted version of the color.
    pub fn invert(&self) -> Color {
        let r = abs(1f32 - self.r);
        let g = abs(1f32 - self.g);
        let b = abs(1f32 - self.b);
        Color::new(r, g, b, self.a)
    }

}

impl Default for Color {
    /// Default constructor for a Color struct.
    fn default() -> Color {
        Color { r: 0.5, g: 0.8, b: 0.6, a: 0.5 }
    }
}

impl Add<Color, Color> for Color {
    fn add(&self, rhs: &Color) -> Color {
        Color::clamp(Color::new(self.r + rhs.r, self.g + rhs.g, self.b + rhs.b, self.a + rhs.a))
    }
}

impl Sub<Color, Color> for Color {
    fn sub(&self, rhs: &Color) -> Color {
        Color::clamp(Color::new(self.r - rhs.r, self.g - rhs.g, self.b - rhs.b, self.a - rhs.a))
    }
}

