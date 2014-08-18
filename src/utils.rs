
/// Clamp a value between a given min and max.
pub fn clamp<T: Num + Primitive>(n: T, min: T, max: T) -> T {
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
pub fn map<T: Num + Copy + FromPrimitive + ToPrimitive>
    (val: T, in_min: T, in_max: T, out_min: T, out_max: T) -> T {
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

