use std::num::Float;
use std::default::Default;
use std::fmt::{Show, Formatter, Error};
use std::ops::{Add, Sub, Mul, Div};
use std::rand::random;
use std::ascii::AsciiExt;
use rustc_serialize::hex::ToHex;
use rustc_serialize::{
    Decodable, Encodable,
    Decoder, Encoder,
    DecoderHelpers, EncoderHelpers
};
use utils::clampf32;
use internal;

/// A basic color struct for general color use
/// made of red, green, blue and alpha elements.
#[derive(Copy)]
pub struct Color(pub internal::Color);

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
        let &mut Color(ref mut c) = self;
        c[0] = r;
    }

    /// Sets the green component of the color.
    #[inline(always)]
    pub fn set_g(&mut self, g: f32) {
        let &mut Color(ref mut c) = self;
        c[1] = g;
    }

    /// Sets the blue component of the color.
    #[inline(always)]
    pub fn set_b(&mut self, b: f32) {
        let &mut Color(ref mut c) = self;
        c[2] = b;
    }

    /// Sets the alpha component of the color.
    #[inline(always)]
    pub fn set_a(&mut self, a: f32) {
        let &mut Color(ref mut c) = self;
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
        let luminance = self.luminance();
        let (r, g, b) = {
            if luminance > 0.8 {
                (self.r() - 0.2,
                 self.g() - 0.2,
                 self.b() - 0.2)
            }
            else if luminance < 0.2 {
                (self.r() + 0.2,
                 self.g() + 0.2,
                 self.b() + 0.2)
            }
            else {
                (clampf32((1f32 - self.r()) * 0.5f32 * self.r() + self.r()),
                 clampf32((1f32 - self.g()) * 0.1f32 * self.g() + self.g()),
                 clampf32((1f32 - self.b()) * 0.1f32 * self.b() + self.b()))
            }
        };
        let a = clampf32((1f32 - self.a()) * 0.5f32 + self.a());
        Color::new(r, g, b, a)
    }

    /// Return a clicked version of the current Color.
    pub fn clicked(&self) -> Color {
        let luminance = self.luminance();
        let (r, g, b) = {
            if luminance > 0.8 {
                (self.r(),
                 self.g() - 0.2,
                 self.b() - 0.2)
            }
            else if luminance < 0.2 {
                (self.r() + 0.4,
                 self.g() + 0.2,
                 self.b() + 0.2)
            }
            else {
                (clampf32((1f32 - self.r()) * 0.75f32 + self.r()),
                 clampf32((1f32 - self.g()) * 0.25f32 + self.g()),
                 clampf32((1f32 - self.b()) * 0.25f32 + self.b()))
            }
        };
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
        let r = (1f32 - self.r()).abs();
        let g = (1f32 - self.g()).abs();
        let b = (1f32 - self.b()).abs();
        Color::new(r, g, b, self.a())
    }

    /// Return either black or white, depending which contrasts
    /// the Color the most. This will be useful for determining
    /// a readable color for text on any given background Color.
    pub fn plain_contrast(&self) -> Color {
        if self.luminance() > 0.5f32 { Color::black() }
        else { Color::white() }
    }

    /// Return the luminance of the color.
    pub fn luminance(&self) -> f32 {
        (self.r() + self.g() + self.b()) / 3f32
    }

    /// Return an array of the channels in this color
    /// clamped to [0..255]
    pub fn to_32_bit(&self) -> [u8; 4] {
        [
            to_8_bit(self.r()),
            to_8_bit(self.g()),
            to_8_bit(self.b()),
            to_8_bit(self.a()),
        ]
    }

    /// Return the hex representation of this color
    /// in the format #RRGGBBAA
    /// e.g. `Color(1.0, 0.0, 5.0, 1.0) == "#FF0080FF"`
    pub fn to_hex(&self) -> String {
        let vals = self.to_32_bit();
        // Hex colors are always uppercased
        let hex = vals.as_slice().to_hex().to_ascii_uppercase();
        format!("#{}", hex.as_slice())
    }
}

fn to_8_bit(chan: f32) -> u8 {
    let chan = clampf32(chan);
    (chan * 255.0) as u8
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

impl Add<Color> for Color {
    type Output = Color;
    fn add(self, rhs: Color) -> Color {
        Color::clamp(
            Color([
                self.r() + rhs.r(),
                self.g() + rhs.g(),
                self.b() + rhs.b(),
                self.a()
            ])
        )
    }
}

impl Sub<Color> for Color {
    type Output = Color;
    fn sub(self, rhs: Color) -> Color {
        Color::clamp(
            Color([
                self.r() - rhs.r(),
                self.g() - rhs.g(),
                self.b() - rhs.b(),
                self.a()
            ])
        )
    }
}

impl Div<Color> for Color {
    type Output = Color;
    fn div(self, rhs: Color) -> Color {
        Color::clamp(
            Color([
                self.r() / rhs.r(),
                self.g() / rhs.g(),
                self.b() / rhs.b(),
                self.a() / rhs.a(),
            ])
        )
    }
}

impl Mul<Color> for Color {
    type Output = Color;
    fn mul(self, rhs: Color) -> Color {
        Color::clamp(
            Color([
                self.r() * rhs.r(),
                self.g() * rhs.g(),
                self.b() * rhs.b(),
                self.a() * rhs.a(),
            ])
        )
    }
}

impl Show for Color {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        let hex = self.to_hex();
        fmt.pad(hex.as_slice())
    }
}

impl Decodable for Color {
    fn decode<D: Decoder>(dec: &mut D) -> Result<Color, D::Error> {
        let vec = try!(dec.read_to_vec(|le_dec| le_dec.read_f32()));

        if vec.len() != 4 {
            return Err(dec.error(format!(
                "Expected a 4 element vector when decoding Color.
                Found a vector of length {}.", vec.len()).as_slice()));
        }

        Ok(Color([vec[0], vec[1], vec[2], vec[3]]))
    }
}

impl Encodable for Color {
    fn encode<S: Encoder>(&self, enc: &mut S) -> Result<(), S::Error> {
        let Color(ref vec) = *self;

        enc.emit_from_vec(vec, |le_enc, elt| le_enc.emit_f32(*elt))
    }
}

/// A trait used for "colorable" widget context types.
pub trait Colorable {
    fn color(self, color: Color) -> Self;
    /// A method used for passing color as rgba.
    fn rgba(self, r: f32, g: f32, b: f32, a: f32) -> Self;
}
