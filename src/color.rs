
use std::default::Default;
use utils::clampf32;
use std::rand::random;
use std::num::abs;

/// A basic color struct for general color use
/// made of red, green, blue and alpha elements.
pub struct Color(pub [f32, ..4]);

impl Color {

    /// Basic constructor for a Color struct.
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Color {
        Color([r, g, b, a])
    }

    /// Returns the red component of the color.
    #[inline(always)]
    pub fn r(&self) -> f32 {
        let &Color(c) = self;
        c[0]
    }

    /// Returns the green component of the color.
    #[inline(always)]
    pub fn g(&self) -> f32 {
        let &Color(c) = self;
        c[1]
    }

    /// Returns the blue component of the color.
    #[inline(always)]
    pub fn b(&self) -> f32 {
        let &Color(c) = self;
        c[2]
    }

    /// Sets the red component of the color.
    #[inline(always)]
    pub fn set_r(&mut self, r: f32) {
        let &Color(ref mut c) = self;
        c[0] = r;
    }

    /// Sets the green component of the color.
    #[inline(always)]
    pub fn set_g(&mut self, g: f32) {
        let &Color(ref mut c) = self;
        c[1] = g;
    }

    /// Sets the blue component of the color.
    #[inline(always)]
    pub fn set_b(&mut self, b: f32) {
        let &Color(ref mut c) = self;
        c[2] = b;
    }

    /// Sets the alpha component of the color.
    #[inline(always)]
    pub fn set_a(&mut self, a: f32) {
        let &Color(ref mut c) = self;
        c[3] = a;
    }

    /// Returns the alpha component of the color.
    #[inline(always)]
    pub fn a(&self) -> f32 {
        let &Color(c) = self;
        c[3]
    }

    /// Basic constructor for a Black Color struct.
    pub fn black() -> Color {
        Color([0f32, 0f32, 0f32, 1f32])
    }

    /// Basic constructor for a White Color struct.
    pub fn white() -> Color {
        Color([1f32, 1f32, 1f32, 1f32])
    }

    /// Return color as a vector.
    pub fn as_vec(&self) -> Vec<f32> {
        vec![self.r(), self.g(), self.b(), self.a()]
    }

    /// Return color as a tuple.
    pub fn as_tuple(&self) -> (f32, f32, f32, f32) {
        let &Color(c) = self;
        (c[0], c[1], c[2], c[3])
    }

    /// Clamp the Color's values between 0f32 and 1f32.
    fn clamp(c: Color) -> Color {
        Color([
            clampf32(c.r()), 
            clampf32(c.g()), 
            clampf32(c.b()), 
            clampf32(c.a())
        ])
    }

    /// Return a highlighted version of the current Color.
    pub fn highlighted(&self) -> Color {
        let r = clampf32((1f32 - self.r()) * 0.5f32 * self.r() + self.r());
        let g = clampf32((1f32 - self.g()) * 0.1f32 * self.g() + self.g());
        let b = clampf32((1f32 - self.b()) * 0.1f32 * self.b() + self.b());
        let a = clampf32((1f32 - self.a()) * 0.5f32 + self.a());
        Color::new(r, g, b, a)
    }

    /// Return a clicked version of the current Color.
    pub fn clicked(&self) -> Color {
        let r = clampf32((1f32 - self.r()) * 0.75f32 + self.r());
        let g = clampf32((1f32 - self.g()) * 0.25f32 + self.g());
        let b = clampf32((1f32 - self.b()) * 0.25f32 + self.b());
        let a = clampf32((1f32 - self.a()) * 0.75f32 + self.a());
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
        let r = abs(1f32 - self.r());
        let g = abs(1f32 - self.g());
        let b = abs(1f32 - self.b());
        Color::new(r, g, b, self.a())
    }

    /// Return either black or white, depending which contrasts
    /// the Color the most. This will be useful for determining
    /// a readable color for text on any given background Color.
    pub fn plain_contrast(&self) -> Color {
        if (self.r() + self.g() + self.b()) > 1.5f32 { Color::black() }
        else { Color::white() }
    }

}

impl Clone for Color {
    fn clone(&self) -> Color {
        *self
    }
}

impl Default for Color {
    /// Default constructor for a Color struct.
    fn default() -> Color {
        Color([0.5, 0.8, 0.6, 1.0])
    }
}

impl Add<Color, Color> for Color {
    fn add(&self, rhs: &Color) -> Color {
        Color::clamp(
            Color([
                self.r() + rhs.r(), 
                self.g() + rhs.g(), 
                self.b() + rhs.b(), 
                self.a() + rhs.a()
            ])
        )
    }
}

impl Sub<Color, Color> for Color {
    fn sub(&self, rhs: &Color) -> Color {
        Color::clamp(
            Color([
                self.r() - rhs.r(), 
                self.g() - rhs.g(), 
                self.b() - rhs.b(), 
                self.a() - rhs.a()
            ])
        )
    }
}

