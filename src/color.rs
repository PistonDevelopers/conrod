//! 
//! Inspiration taken from [elm-lang's color module]
//! (https://github.com/elm-lang/core/blob/62b22218c42fb8ccc996c86bea450a14991ab815/src/Color.elm)
//!
//!
//! Module for working with colors. Includes [RGB](https://en.wikipedia.org/wiki/RGB_color_model)
//! and [HSL](http://en.wikipedia.org/wiki/HSL_and_HSV) creation, gradients and built-in names.
//!

use num::Float;
use rustc_serialize::hex::ToHex;
use std::ascii::AsciiExt;
use std::f32::consts::PI;
use utils::{clampf32, degrees, fmod, min, max, turns};


/// Color supporting RGB and HSL variants.
#[derive(Copy, Clone, RustcEncodable, RustcDecodable)]
pub enum Color {
    Rgba(f32, f32, f32, f32),
    Hsla(f32, f32, f32, f32),
}
/// Regional spelling alias.
pub type Colour = Color;


/// Create RGB colors with an alpha component for transparency.
/// The alpha component is specified with numbers between 0 and 1.
#[inline]
pub fn rgba(r: f32, g: f32, b: f32, a: f32) -> Color {
    Color::Rgba(r, g, b, a)
}


/// Create RGB colors from numbers between 0.0 and 1.0.
#[inline]
pub fn rgb(r: f32, g: f32, b: f32) -> Color {
    Color::Rgba(r, g, b, 1.0)
}


/// Create RGB colors from numbers between 0 and 255 inclusive.
/// The alpha component is specified with numbers between 0 and 1.
#[inline]
pub fn rgba_bytes(r: u8, g: u8, b: u8, a: f32) -> Color {
    Color::Rgba(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, a)
}


/// Create RGB colors from numbers between 0 and 255 inclusive.
#[inline]
pub fn rgb_bytes(r: u8, g: u8, b: u8) -> Color {
    rgba_bytes(r, g, b, 1.0)
}


/// Create [HSL colors](http://en.wikipedia.org/wiki/HSL_and_HSV) with an alpha component for
/// transparency.
#[inline]
pub fn hsla(hue: f32, saturation: f32, lightness: f32, alpha: f32) -> Color {
    Color::Hsla(hue - turns((hue / (2.0 * PI)).floor()), saturation, lightness, alpha)
}


/// Create [HSL colors](http://en.wikipedia.org/wiki/HSL_and_HSV). This gives you access to colors
/// more like a color wheel, where all hues are arranged in a circle that you specify with radians.
/// 
///   red        = hsl(degrees(0.0)   , 1.0 , 0.5)
///   green      = hsl(degrees(120.0) , 1.0 , 0.5)
///   blue       = hsl(degrees(240.0) , 1.0 , 0.5)
///   pastel_red = hsl(degrees(0.0)   , 0.7 , 0.7)
///
/// To cycle through all colors, just cycle through degrees. The saturation level is how vibrant
/// the color is, like a dial between grey and bright colors. The lightness level is a dial between
/// white and black.
#[inline]
pub fn hsl(hue: f32, saturation: f32, lightness: f32) -> Color {
    hsla(hue, saturation, lightness, 1.0)
}


/// Produce a gray based on the input. 0.0 is white, 1.0 is black.
pub fn grayscale(p: f32) -> Color {
    Color::Hsla(0.0, 0.0, 1.0-p, 1.0)
}
/// Produce a gray based on the input. 0.0 is white, 1.0 is black.
pub fn greyscale(p: f32) -> Color {
    Color::Hsla(0.0, 0.0, 1.0-p, 1.0)
}


/// Construct a random color.
pub fn random() -> Color {
    rgb(::rand::random(), ::rand::random(), ::rand::random())
}


impl Color {

    /// Produce a complementary color. The two colors will accent each other. This is the same as
    /// rotating the hue by 180 degrees.
    pub fn complement(self) -> Color {
        match self {
            Color::Hsla(h, s, l, a) => hsla(h + degrees(180.0), s, l, a),
            Color::Rgba(r, g, b, a) => {
                let (h, s, l) = rgb_to_hsl(r, g, b);
                hsla(h + degrees(180.0), s, l, a)
            },
        }
    }

    /// Calculate and return the luminance of the Color.
    pub fn luminance(&self) -> f32 {
        match *self {
            Color::Rgba(r, g, b, _) => (r + g + b) / 3.0,
            Color::Hsla(_, _, l, _) => l,
        }
    }

    /// Return either black or white, depending which contrasts the Color the most. This will be
    /// useful for determining a readable color for text on any given background Color.
    pub fn plain_contrast(self) -> Color {
        if self.luminance() > 0.5 { black() } else { white() }
    }

    /// Extract the components of a color in the HSL format.
    pub fn to_hsl(self) -> Hsla {
        match self {
            Color::Hsla(h, s, l, a) => Hsla(h, s, l, a),
            Color::Rgba(r, g, b, a) => {
                let (h, s, l) = rgb_to_hsl(r, g, b);
                Hsla(h, s, l, a)
            },
        }
    }

    /// Extract the components of a color in the RGB format.
    pub fn to_rgb(self) -> Rgba {
        match self {
            Color::Rgba(r, g, b, a) => Rgba(r, g, b, a),
            Color::Hsla(h, s, l, a) => {
                let (r, g, b) = hsl_to_rgb(h, s, l);
                Rgba(r, g, b, a)
            },
        }
    }

    /// Extract the components of a color in the RGB format within a fixed-size array.
    pub fn to_fsa(self) -> [f32; 4] {
        let Rgba(r, g, b, a) = self.to_rgb();
        [r, g, b, a]
    }

    /// Same as `to_fsa`, except r, g, b and a are represented in byte form.
    pub fn to_byte_fsa(self) -> [u8; 4] {
        let Rgba(r, g, b, a) = self.to_rgb();
        [f32_to_byte(r), f32_to_byte(g), f32_to_byte(b), f32_to_byte(a)]
    }

    /// Return the hex representation of this color in the format #RRGGBBAA
    /// e.g. `Color(1.0, 0.0, 5.0, 1.0) == "#FF0080FF"`
    pub fn to_hex(self) -> String {
        let vals = self.to_byte_fsa();
        let hex = vals.to_hex().to_ascii_uppercase();
        format!("#{}", &hex)
    }

    /// Return the same color but with the given luminance.
    pub fn with_luminance(self, l: f32) -> Color {
        let Hsla(h, s, _, a) = self.to_hsl();
        Color::Hsla(h, s, l, a)
    }

    /// Return the same color but with the alpha multiplied by the given alpha.
    pub fn alpha(self, alpha: f32) -> Color {
        match self {
            Color::Rgba(r, g, b, a) => Color::Rgba(r, g, b, a * alpha),
            Color::Hsla(h, s, l, a) => Color::Hsla(h, s, l, a * alpha),
        }
    }

    /// Return the same color but with the given alpha.
    pub fn with_alpha(self, a: f32) -> Color {
        match self {
            Color::Rgba(r, g, b, _) => Color::Rgba(r, g, b, a),
            Color::Hsla(h, s, l, _) => Color::Hsla(h, s, l, a),
        }
    }

    /// Return a highlighted version of the current Color.
    pub fn highlighted(self) -> Color {
        let luminance = self.luminance();
        let Rgba(r, g, b, a) = self.to_rgb();
        let (r, g, b) = {
            if      luminance > 0.8 { (r - 0.2, g - 0.2, b - 0.2) }
            else if luminance < 0.2 { (r + 0.2, g + 0.2, b + 0.2) }
            else {
                (clampf32((1.0 - r) * 0.5 * r + r),
                 clampf32((1.0 - g) * 0.1 * g + g),
                 clampf32((1.0 - b) * 0.1 * b + b))
            }
        };
        let a = clampf32((1.0 - a) * 0.5 + a);
        rgba(r, g, b, a)
    }

    /// Return a clicked version of the current Color.
    pub fn clicked(&self) -> Color {
        let luminance = self.luminance();
        let Rgba(r, g, b, a) = self.to_rgb();
        let (r, g, b) = {
            if      luminance > 0.8 { (r      , g - 0.2, b - 0.2) }
            else if luminance < 0.2 { (r + 0.4, g + 0.2, b + 0.2) }
            else {
                (clampf32((1.0 - r) * 0.75 + r),
                 clampf32((1.0 - g) * 0.25 + g),
                 clampf32((1.0 - b) * 0.25 + b))
            }
        };
        let a = clampf32((1.0 - a) * 0.75 + a);
        rgba(r, g, b, a)
    }

    /// Return the Color's invert.
    pub fn invert(self) -> Color {
        let Rgba(r, g, b, a) = self.to_rgb();
        rgba((r - 1.0).abs(), (g - 1.0).abs(), (b - 1.0).abs(), a)
    }

    /// Return the red value.
    pub fn red(&self) -> f32 {
        let Rgba(r, _, _, _) = self.to_rgb();
        r
    }

    /// Return the green value.
    pub fn green(&self) -> f32 {
        let Rgba(_, g, _, _) = self.to_rgb();
        g
    }

    /// Return the blue value.
    pub fn blue(&self) -> f32 {
        let Rgba(_, _, b, _) = self.to_rgb();
        b
    }

    /// Set the red value.
    pub fn set_red(&mut self, r: f32) {
        let Rgba(_, g, b, a) = self.to_rgb();
        *self = rgba(r, g, b, a);
    }

    /// Set the green value.
    pub fn set_green(&mut self, g: f32) {
        let Rgba(r, _, b, a) = self.to_rgb();
        *self = rgba(r, g, b, a);
    }

    /// Set the blue value.
    pub fn set_blue(&mut self, b: f32) {
        let Rgba(r, g, _, a) = self.to_rgb();
        *self = rgba(r, g, b, a);
    }

}


/// The parts of HSL along with an alpha for transparency.
#[derive(Copy, Clone, Debug)]
pub struct Hsla(pub f32, pub f32, pub f32, pub f32);


/// The parts of RGB along with an alpha for transparency.
#[derive(Copy, Clone, Debug)]
pub struct Rgba(pub f32, pub f32, pub f32, pub f32);


/// Convert an f32 color to a byte.
#[inline]
pub fn f32_to_byte(c: f32) -> u8 { (c * 255.0) as u8 }


/// Pure function for converting rgb to hsl.
pub fn rgb_to_hsl(r: f32, g: f32, b: f32) -> (f32, f32, f32) {
    let c_max = max(max(r, g), b);
    let c_min = min(min(r, g), b);
    let c = c_max - c_min;
    let hue = degrees(60.0) * if      c_max == r { fmod(((g - b) / c), 6) }
                              else if c_max == g { ((b - r) / c) + 2.0 }
                              else               { ((r - g) / c) + 4.0 };
    let lightness = (c_max + c_min) / 2.0;
    let saturation = if lightness == 0.0 { 0.0 }
                     else { c / (1.0 - (2.0 * lightness - 1.0).abs()) };
    (hue, saturation, lightness)
}


/// Pure function for converting hsl to rgb.
pub fn hsl_to_rgb(hue: f32, saturation: f32, lightness: f32) -> (f32, f32, f32) {
    let chroma = (1.0 - (2.0 * lightness - 1.0).abs()) * saturation;
    let hue = hue / degrees(60.0);
    let x = chroma * (1.0 - (fmod(hue, 2) - 1.0).abs());
    let (r, g, b) = match hue {
        hue if hue < 0.0 => (0.0, 0.0, 0.0),
        hue if hue < 1.0 => (chroma, x, 0.0),
        hue if hue < 2.0 => (x, chroma, 0.0),
        hue if hue < 3.0 => (0.0, chroma, x),
        hue if hue < 4.0 => (0.0, x, chroma),
        hue if hue < 5.0 => (x, 0.0, chroma),
        hue if hue < 6.0 => (chroma, 0.0, x),
        _ => (0.0, 0.0, 0.0),
    };
    let m = lightness - chroma / 2.0;
    (r + m, g + m, b + m)
}


/// Linear or Radial Gradient.
#[derive(Clone, Debug)]
pub enum Gradient {
    Linear((f64, f64), (f64, f64), Vec<(f64, Color)>),
    Radial((f64, f64), f64, (f64, f64), f64, Vec<(f64, Color)>),
}


/// Create a linear gradient. Takes a start and end point and then a series of color stops that
/// indicate how to interpolate between the start and end points.
pub fn linear(start: (f64, f64), end: (f64, f64), colors: Vec<(f64, Color)>) -> Gradient {
    Gradient::Linear(start, end, colors)
}


/// Create a radial gradient. First takes a start point and inner radius. Then takes an end point
/// and outer radius. It then takes a series of color stops that indicate how to interpolate
/// between the inner and outer circles.
pub fn radial(start: (f64, f64), start_r: f64,
              end: (f64, f64), end_r: f64,
              colors: Vec<(f64, Color)>) -> Gradient {
    Gradient::Radial(start, start_r, end, end_r, colors)
}


/// Built-in colors.
///
/// These colors come from the
/// [Tango palette](http://tango.freedesktop.org/Tango_Icon_Theme_Guidelines) which provides
/// aesthetically reasonable defaults for colors. Each color also comes with a light and dark
/// version.

pub fn light_red()      -> Color { rgb_bytes(239 , 41  , 41 ) }
pub fn red()            -> Color { rgb_bytes(204 , 0   , 0  ) }
pub fn dark_red()       -> Color { rgb_bytes(164 , 0   , 0  ) }

pub fn light_orange()   -> Color { rgb_bytes(252 , 175 , 62 ) }
pub fn orange()         -> Color { rgb_bytes(245 , 121 , 0  ) }
pub fn dark_orange()    -> Color { rgb_bytes(206 , 92  , 0  ) }

pub fn light_yellow()   -> Color { rgb_bytes(255 , 233 , 79 ) }
pub fn yellow()         -> Color { rgb_bytes(237 , 212 , 0  ) }
pub fn dark_yellow()    -> Color { rgb_bytes(196 , 160 , 0  ) }

pub fn light_green()    -> Color { rgb_bytes(138 , 226 , 52 ) }
pub fn green()          -> Color { rgb_bytes(115 , 210 , 22 ) }
pub fn dark_green()     -> Color { rgb_bytes(78  , 154 , 6  ) }

pub fn light_blue()     -> Color { rgb_bytes(114 , 159 , 207) }
pub fn blue()           -> Color { rgb_bytes(52  , 101 , 164) }
pub fn dark_blue()      -> Color { rgb_bytes(32  , 74  , 135) }

pub fn light_purple()   -> Color { rgb_bytes(173 , 127 , 168) }
pub fn purple()         -> Color { rgb_bytes(117 , 80  , 123) }
pub fn dark_purple()    -> Color { rgb_bytes(92  , 53  , 102) }

pub fn light_brown()    -> Color { rgb_bytes(233 , 185 , 110) }
pub fn brown()          -> Color { rgb_bytes(193 , 125 , 17 ) }
pub fn dark_brown()     -> Color { rgb_bytes(143 , 89  , 2  ) }

pub fn black()          -> Color { rgb_bytes(0   , 0   , 0  ) }
pub fn white()          -> Color { rgb_bytes(255 , 255 , 255) }

pub fn light_gray()     -> Color { rgb_bytes(238 , 238 , 236) }
pub fn gray()           -> Color { rgb_bytes(211 , 215 , 207) }
pub fn dark_gray()      -> Color { rgb_bytes(186 , 189 , 182) }

pub fn light_grey()     -> Color { rgb_bytes(238 , 238 , 236) }
pub fn grey()           -> Color { rgb_bytes(211 , 215 , 207) }
pub fn dark_grey()      -> Color { rgb_bytes(186 , 189 , 182) }

pub fn light_charcoal() -> Color { rgb_bytes(136 , 138 , 133) }
pub fn charcoal()       -> Color { rgb_bytes(85  , 87  , 83 ) }
pub fn dark_charcoal()  -> Color { rgb_bytes(46  , 52  , 54 ) }



impl ::std::fmt::Debug for Color {
    fn fmt(&self, fmt: &mut ::std::fmt::Formatter) -> Result<(), ::std::fmt::Error> {
        let hex = self.clone().to_hex();
        fmt.pad(&hex)
    }
}


/// A trait used for "colorable" widget context types.
pub trait Colorable: Sized {
    fn color(self, color: Color) -> Self;
    /// A method used for passing color as rgba.
    fn rgba(self, r: f32, g: f32, b: f32, a: f32) -> Self {
        self.color(rgba(r, g, b, a))
    }
    /// A method used for passing color as rgb.
    fn rgb(self, r: f32, g: f32, b: f32) -> Self {
        self.color(rgb(r, g, b))
    }
    /// A method used for passing color as hsla.
    fn hsla(self, h: f32, s: f32, l: f32, a: f32) -> Self {
        self.color(hsla(h, s, l, a))
    }
    /// A method used for passing color as hsl.
    fn hsl(self, h: f32, s: f32, l: f32) -> Self {
        self.color(hsl(h, s, l))
    }
}


