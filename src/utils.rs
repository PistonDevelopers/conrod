use std::cmp::Ordering::{self, Less, Equal, Greater};
use std::num::Float;
use std::num::Int;
use std::num::ToPrimitive;
use std::num::FromPrimitive;

/// Clamp a value between a given min and max.
pub fn clamp<T: Float + PartialOrd>(n: T, min: T, max: T) -> T {
    if n < min { min } else if n > max { max } else { n }
}

/// Clamp a f32 between 0f32 and 1f32.
pub fn clampf32(f: f32) -> f32 {
    if f < 0f32 { 0f32 } else if f > 1f32 { 1f32 } else { f }
}

/// Compare two f64s and return an Ordering.
pub fn compare_f64s(a: f64, b: f64) -> Ordering {
    if a > b { Greater }
    else if a < b { Less }
    else { Equal }
}

/// Get value percentage between max and min.
pub fn percentage<T: Float + Copy + FromPrimitive + ToPrimitive>
    (value: T, min: T, max: T) -> f32 {
    let v = value.to_f32().unwrap();
    let mn = min.to_f32().unwrap();
    let mx = max.to_f32().unwrap();
    (v - mn) / (mx - mn)
}

/// Adjust the value to the given percentage.
pub fn value_from_perc<T: Float + Copy + FromPrimitive + ToPrimitive>
    (perc: f32, min: T, max: T) -> T {
    min + FromPrimitive::from_f32((max - min).to_f32().unwrap() * perc).unwrap()
}

/// Map a value from a given range to a new given range.
pub fn map_range<X: Float + Copy + FromPrimitive + ToPrimitive,
                 Y: Float + Copy + FromPrimitive + ToPrimitive>
(val: X, in_min: X, in_max: X, out_min: Y, out_max: Y) -> Y {
    let (val_f, in_min_f, in_max_f, out_min_f, out_max_f) = (
        val.to_f64().unwrap(),
        in_min.to_f64().unwrap(),
        in_max.to_f64().unwrap(),
        out_min.to_f64().unwrap(),
        out_max.to_f64().unwrap(),
    );
    FromPrimitive::from_f64(
        (val_f - in_min_f) / (in_max_f - in_min_f) * (out_max_f - out_min_f) + out_min_f
    ).unwrap()
}

/// Get a suitable string from the value, its max and the pixel range.
pub fn val_to_string<T: ToString + ToPrimitive>
(val: T, max: T, val_rng: T, pixel_range: usize) -> String {
    let mut s = val.to_string();
    let decimal = s.as_slice().chars().position(|ch| ch == '.');
    match decimal {
        None => s,
        Some(idx) => {
            // Find the minimum string length by determing
            // what power of ten both the max and range are.
            let val_rng_f = val_rng.to_f64().unwrap();
            let max_f = max.to_f64().unwrap();
            let mut n: f64 = 0.0;
            let mut pow_ten = 0.0;
            while pow_ten < val_rng_f || pow_ten < max_f {
                pow_ten = (10.0).powf(n);
                n += 1.0
            }
            let min_string_len = n as usize + 1;

            // Find out how many pixels there are to actually use
            // and judge a reasonable precision from this.
            let mut n: usize = 1;
            while 10.pow(n) < pixel_range { n += 1 }
            let precision = n;

            // Truncate the length to the pixel precision as
            // long as this doesn't cause it to be smaller
            // than the necessary decimal place.
            let mut truncate_len = min_string_len + (precision - 1);
            if idx + precision < truncate_len { truncate_len = idx + precision }
            if s.len() > truncate_len { s.truncate(truncate_len) }
            s
        }
    }
}

