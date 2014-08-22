
use std::num::pow;

/// Clamp a value between a given min and max.
pub fn clamp<T: Num + PartialOrd>(n: T, min: T, max: T) -> T {
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
pub fn percentage<T: Num + Copy + FromPrimitive + ToPrimitive>
    (value: T, min: T, max: T) -> f32 {
    let v = value.to_f32().unwrap();
    let mn = min.to_f32().unwrap();
    let mx = max.to_f32().unwrap();
    (v - mn) / (mx - mn)
}

/// Adjust the value to the given percentage.
pub fn value_from_perc<T: Num + Copy + FromPrimitive + ToPrimitive>
    (perc: f32, min: T, max: T) -> T {
    min + FromPrimitive::from_f32((max - min).to_f32().unwrap() * perc).unwrap()
}

/// Map a value from a given range to a new given range.
pub fn map_range<X: Num + Copy + FromPrimitive + ToPrimitive,
                 Y: Num + Copy + FromPrimitive + ToPrimitive>
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
                (val: T, max: T, val_rng: T, pixel_range: uint) -> String {
    let mut s = val.to_string();
    let decimal = s.as_slice().chars().position(|ch| ch == '.');
    match decimal {
        None => s,
        Some(idx) => {
            // Find the minimum string length by determing
            // what power of ten both the max and range are.
            let val_rng_f = val_rng.to_f64().unwrap();
            let max_f = max.to_f64().unwrap();
            let mut n = 0f64;
            let mut pow_ten = 0f64;
            while pow_ten < val_rng_f || pow_ten < max_f {
                pow_ten = (10f64).powf(n);
                n += 1.0
            }
            let min_string_len = n as uint;

            // Find out how many pixels there are to actually use
            // and judge a reasonable precision from this.
            let mut n = 0u;
            while pow(10u, n) < pixel_range { n += 1u }
            let precision = n - 1u;

            // Truncate the length to the pixel precision as
            // long as this doesn't cause it to be smaller
            // than the necessary decimal place.
            let truncate_len = {
                if precision >= min_string_len { precision }
                else { min_string_len }
            };
            if truncate_len - 1u == idx { s.truncate(truncate_len + 1u) }
            else { s.truncate(truncate_len) }
            s
        }
    }
}

