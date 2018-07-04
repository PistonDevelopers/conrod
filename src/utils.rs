//! 
//! Various utility functions used throughout Conrod.
//!


use num::{Float, NumCast, PrimInt, ToPrimitive};
use position::{Point, Range, Rect};
use std::borrow::Cow;
use std::iter::{Chain, once, Once};
use std;


/// Compare to PartialOrd values and return the min.
pub fn partial_min<T: PartialOrd>(a: T, b: T) -> T {
    if a <= b { a } else { b }
}

/// Compare to PartialOrd values and return the max.
pub fn partial_max<T: PartialOrd>(a: T, b: T) -> T {
    if a >= b { a } else { b }
}

/// Clamp a value between some range.
pub fn clamp<T: PartialOrd>(n: T, start: T, end: T) -> T {
    if start <= end {
        if n < start { start } else if n > end { end } else { n }
    } else {
        if n < end { end } else if n > start { start } else { n }
    }
}

/// Convert degrees to radians.
pub fn degrees<F: Float + NumCast>(d: F) -> F {
    use std::f32::consts::PI;
    d * NumCast::from(PI / 180.0).unwrap()
}

/// Modulo float.
pub fn fmod(f: f32, n: i32) -> f32 {
    let i = f.floor() as i32;
    modulo(i, n) as f32 + f - i as f32
}

/// The modulo function.
#[inline]
pub fn modulo<I: PrimInt>(a: I, b: I) -> I {
    match a % b {
        r if (r > I::zero() && b < I::zero())
          || (r < I::zero() && b > I::zero()) => r + b,
        r                                     => r,
    }
}

/// Map a value from a given range to a new given range.
pub fn map_range<X, Y>(val: X, in_min: X, in_max: X, out_min: Y, out_max: Y) -> Y
    where X: NumCast,
          Y: NumCast,
{
    let val_f: f64 = NumCast::from(val).unwrap();
    let in_min_f: f64 = NumCast::from(in_min).unwrap();
    let in_max_f: f64 = NumCast::from(in_max).unwrap();
    let out_min_f: f64 = NumCast::from(out_min).unwrap();
    let out_max_f: f64 = NumCast::from(out_max).unwrap();
    NumCast::from(
        (val_f - in_min_f) / (in_max_f - in_min_f) * (out_max_f - out_min_f) + out_min_f
    ).unwrap()
}

/// Get value percentage between max and min.
pub fn percentage<T: Float + NumCast>(value: T, min: T, max: T) -> f32 {
    let v: f32 = NumCast::from(value).unwrap();
    let mn: f32 = NumCast::from(min).unwrap();
    let mx: f32 = NumCast::from(max).unwrap();
    (v - mn) / (mx - mn)
}

/// Convert turns to radians.
pub fn turns<F: Float + NumCast>(t: F) -> F {
    use std::f32::consts::PI;
    let f: F = NumCast::from(2.0 * PI).unwrap();
    f * t
}

/// Adjust the value to the given percentage.
pub fn value_from_perc<T: Float + NumCast + ToPrimitive>(perc: f32, min: T, max: T) -> T {
    let f: f32 = (max - min).to_f32().unwrap() * perc;
    min + NumCast::from(f).unwrap()
}

/// Get a suitable string from the value, its max and the pixel range.
pub fn val_to_string<T: ToString + NumCast>
(val: T, max: T, val_rng: T, pixel_range: usize) -> String {
    let mut s = val.to_string();
    let decimal = s.chars().position(|ch| ch == '.');
    match decimal {
        None => s,
        Some(idx) => {
            // Find the minimum string length by determing
            // what power of ten both the max and range are.
            let val_rng_f: f64 = NumCast::from(val_rng).unwrap();
            let max_f: f64 = NumCast::from(max).unwrap();
            let mut n: f64 = 0.0;
            let mut pow_ten = 0.0;
            while pow_ten < val_rng_f || pow_ten < max_f {
                pow_ten = (10.0).powf(n);
                n += 1.0
            }
            let min_string_len = n as usize + 1;

            // Find out how many pixels there are to actually use
            // and judge a reasonable precision from this.
            let mut n = 1;
            while 10.pow(n) < pixel_range { n += 1 }
            let precision = n as usize;

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

/// Add `a` and `b`.
pub fn vec2_add<T>(a: [T; 2], b: [T; 2]) -> [T; 2]
    where T: std::ops::Add<Output=T> + Copy,
{
    [a[0] + b[0], a[1] + b[1]]
}

/// Subtract `b` from `a`.
pub fn vec2_sub<T>(a: [T; 2], b: [T; 2]) -> [T; 2]
    where T: std::ops::Sub<Output=T> + Copy,
{
    [a[0] - b[0], a[1] - b[1]]
}


/// Find the bounding rect for the given series of points.
pub fn bounding_box_for_points<I>(mut points: I) -> Rect
    where I: Iterator<Item=Point>,
{
    points.next().map(|first| {
        let start_rect = Rect {
            x: Range { start: first[0], end: first[0] },
            y: Range { start: first[1], end: first[1] },
        };
        points.fold(start_rect, Rect::stretch_to_point)
    }).unwrap_or_else(|| Rect::from_xy_dim([0.0, 0.0], [0.0, 0.0]))
}

/// A type returned by the `iter_diff` function.
///
/// Represents way in which the elements (of type `E`) yielded by the iterator `I` differ to some
/// other iterator yielding borrowed elements of the same type.
///
/// `I` is some `Iterator` yielding elements of type `E`.
pub enum IterDiff<E, I> {
    /// The index of the first non-matching element along with the iterator's remaining elements
    /// starting with the first mis-matched element.
    FirstMismatch(usize, Chain<Once<E>, I>),
    /// The remaining elements of the iterator.
    Longer(Chain<Once<E>, I>),
    /// The total number of elements that were in the iterator.
    Shorter(usize),
}

/// Compares every element yielded by both elems and new_elems in lock-step.
///
/// If the number of elements yielded by `b` is less than the number of elements yielded by `a`,
/// the number of `b` elements yielded will be returned as `IterDiff::Shorter`.
///
/// If the two elements of a step differ, the index of those elements along with the remaining
/// elements are returned as `IterDiff::FirstMismatch`.
///
/// If `a` becomes exhausted before `b` becomes exhausted, the remaining `b` elements will be
/// returned as `IterDiff::Longer`.
///
/// This function is useful when comparing a non-`Clone` `Iterator` of elements to some existing
/// collection. If there is any difference between the elements yielded by the iterator and those
/// of the collection, a suitable `IterDiff` is returned so that the existing collection may be
/// updated with the difference using elements from the very same iterator.
pub fn iter_diff<'a, A, B>(a: A, b: B) -> Option<IterDiff<B::Item, B::IntoIter>>
    where A: IntoIterator<Item=&'a B::Item>,
          B: IntoIterator,
          B::Item: PartialEq + 'a
{
    let mut b = b.into_iter();
    for (i, a_elem) in a.into_iter().enumerate() {
        match b.next() {
            None => return Some(IterDiff::Shorter(i)),
            Some(b_elem) => if *a_elem != b_elem {
                return Some(IterDiff::FirstMismatch(i, once(b_elem).chain(b)));
            },
        }
    }
    b.next().map(|elem| IterDiff::Longer(once(elem).chain(b)))
}

/// Returns `Borrowed` `elems` if `elems` contains the same elements as yielded by `new_elems`.
///
/// Allocates a new `Vec<T>` and returns `Owned` if either the number of elements or the elements
/// themselves differ.
pub fn write_if_different<T, I>(elems: &[T], new_elems: I) -> Cow<[T]>
    where T: PartialEq + Clone,
          I: IntoIterator<Item=T>,
{
    match iter_diff(elems.iter(), new_elems.into_iter()) {
        Some(IterDiff::FirstMismatch(i, mismatch)) =>
            Cow::Owned(elems[0..i].iter().cloned().chain(mismatch).collect()),
        Some(IterDiff::Longer(remaining)) =>
            Cow::Owned(elems.iter().cloned().chain(remaining).collect()),
        Some(IterDiff::Shorter(num_new_elems)) =>
            Cow::Owned(elems.iter().cloned().take(num_new_elems).collect()),
        None => Cow::Borrowed(elems),
    }
}

/// Compares two iterators to see if they yield the same thing.
pub fn iter_eq<A, B>(mut a: A, mut b: B) -> bool
    where A: Iterator,
          B: Iterator<Item=A::Item>,
          A::Item: PartialEq,
{
    loop {
        match (a.next(), b.next()) {
            (None, None) => return true,
            (maybe_a, maybe_b) => if maybe_a != maybe_b {
                return false
            },
        }
    }
}



#[test]
fn test_map_range() {

    // Normal positive to normal positive.
    assert_eq!(map_range(0.0, 0.0, 5.0, 0.0, 10.0), 0.0);
    assert_eq!(map_range(2.5, 0.0, 5.0, 0.0, 10.0), 5.0);
    assert_eq!(map_range(5.0, 0.0, 5.0, 0.0, 10.0), 10.0);

    // Normal positive to normal negative.
    assert_eq!(map_range(0.0, 0.0, 5.0, -10.0, 0.0), -10.0);
    assert_eq!(map_range(2.5, 0.0, 5.0, -10.0, 0.0), -5.0);
    assert_eq!(map_range(5.0, 0.0, 5.0, -10.0, 0.0), 0.0);

    // Normal positive to inverse positive.
    assert_eq!(map_range(0.0, 0.0, 5.0, 10.0, 0.0), 10.0);
    assert_eq!(map_range(2.5, 0.0, 5.0, 10.0, 0.0), 5.0);
    assert_eq!(map_range(5.0, 0.0, 5.0, 10.0, 0.0), 0.0);

    // Normal positive to inverse negative.
    assert_eq!(map_range(0.0, 0.0, 5.0, 0.0, -10.0), 0.0);
    assert_eq!(map_range(2.5, 0.0, 5.0, 0.0, -10.0), -5.0);
    assert_eq!(map_range(5.0, 0.0, 5.0, 0.0, -10.0), -10.0);


    // Normal negative to normal positive.
    assert_eq!(map_range(-5.0, -5.0, 0.0, 0.0, 10.0), 0.0);
    assert_eq!(map_range(-2.5, -5.0, 0.0, 0.0, 10.0), 5.0);
    assert_eq!(map_range(0.0, -5.0, 0.0, 0.0, 10.0), 10.0);

    // Normal negative to normal negative.
    assert_eq!(map_range(-5.0, -5.0, 0.0, -10.0, 0.0), -10.0);
    assert_eq!(map_range(-2.5, -5.0, 0.0, -10.0, 0.0), -5.0);
    assert_eq!(map_range(0.0, -5.0, 0.0, -10.0, 0.0), 0.0);


    // Inverse positive to normal positive.
    assert_eq!(map_range(5.0, 5.0, 0.0, 0.0, 10.0), 0.0);
    assert_eq!(map_range(2.5, 5.0, 0.0, 0.0, 10.0), 5.0);
    assert_eq!(map_range(0.0, 5.0, 0.0, 0.0, 10.0), 10.0);

}
