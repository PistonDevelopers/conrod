
/// Clamp a value between a given min and max.
pub fn clamp<T: Num + Primitive>(n: T, min: T, max: T) -> T {
    if n < min { min } else if n > max { max } else { n }
}

/// Clamp a f32 between 0f32 and 1f32.
pub fn clampf32(f: f32) -> f32 { if f < 0f32 { 0f32 } else if f > 1f32 { 1f32 } else { f } }


