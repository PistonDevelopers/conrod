//! A type for working one-dimensional ranges.


use super::Scalar;


/// Some start and end position along a single axis.
///
/// As an example, a **Rect** is made up of two **Range**s; one along the *x* axis, and one along
/// the *y* axis.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Range {
    /// The start of some `Range` along an axis.
    pub start: Scalar,
    /// The end of some `Range` along an axis.
    pub end: Scalar,
}

/// Represents either the **Start** or **End** **Edge** of a **Range**.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Edge {
    /// The beginning of a **Range**.
    Start,
    /// The end of a **Range**.
    End,
}


impl Range {

    /// Construct a new `Range` from a given range, i.e. `Range::new(start, end)`.
    ///
    /// # Examples
    ///
    /// ```
    /// use conrod_core::Range;
    ///
    /// assert_eq!(Range { start: 0.0, end: 10.0 }, Range::new(0.0, 10.0));
    /// ```
    pub fn new(start: Scalar, end: Scalar) -> Range {
        Range {
            start: start,
            end: end,
        }
    }

    /// Construct a new `Range` from a given length and its centered position.
    ///
    /// # Examples
    ///
    /// ```
    /// use conrod_core::Range;
    ///
    /// assert_eq!(Range::new(0.0, 10.0), Range::from_pos_and_len(5.0, 10.0));
    /// assert_eq!(Range::new(-5.0, 1.0), Range::from_pos_and_len(-2.0, 6.0));
    /// assert_eq!(Range::new(-100.0, 200.0), Range::from_pos_and_len(50.0, 300.0));
    /// ```
    pub fn from_pos_and_len(pos: Scalar, len: Scalar) -> Range {
        let half_len = len / 2.0;
        let start = pos - half_len;
        let end = pos + half_len;
        Range::new(start, end)
    }

    /// The `start` value subtracted from the `end` value.
    ///
    /// # Examples
    ///
    /// ```
    /// use conrod_core::Range;
    ///
    /// assert_eq!(Range::new(-5.0, 5.0).magnitude(), 10.0);
    /// assert_eq!(Range::new(5.0, -5.0).magnitude(), -10.0);
    /// assert_eq!(Range::new(15.0, 10.0).magnitude(), -5.0);
    /// ```
    pub fn magnitude(&self) -> Scalar {
        self.end - self.start
    }

    /// The absolute length of the Range aka the absolute magnitude.
    ///
    /// # Examples
    ///
    /// ```
    /// use conrod_core::Range;
    ///
    /// assert_eq!(Range::new(-5.0, 5.0).len(), 10.0);
    /// assert_eq!(Range::new(5.0, -5.0).len(), 10.0);
    /// assert_eq!(Range::new(15.0, 10.0).len(), 5.0);
    /// ```
    pub fn len(&self) -> Scalar {
        self.magnitude().abs()
    }

    /// Return the value directly between the start and end values.
    ///
    /// # Examples
    ///
    /// ```
    /// use conrod_core::Range;
    ///
    /// assert_eq!(Range::new(-5.0, 5.0).middle(), 0.0);
    /// assert_eq!(Range::new(5.0, -5.0).middle(), 0.0);
    /// assert_eq!(Range::new(10.0, 15.0).middle(), 12.5);
    /// assert_eq!(Range::new(20.0, 40.0).middle(), 30.0);
    /// assert_eq!(Range::new(20.0, -40.0).middle(), -10.0);
    /// ```
    pub fn middle(&self) -> Scalar {
        (self.end + self.start) / 2.0
    }

    /// The current range with its start and end values swapped.
    ///
    /// # Examples
    ///
    /// ```
    /// use conrod_core::Range;
    ///
    /// assert_eq!(Range::new(-5.0, 5.0).invert(), Range::new(5.0, -5.0));
    /// assert_eq!(Range::new(-10.0, 10.0).invert(), Range::new(10.0, -10.0));
    /// assert_eq!(Range::new(0.0, 7.25).invert(), Range::new(7.25, 0.0));
    /// assert_eq!(Range::new(5.0, 1.0).invert(), Range::new(1.0, 5.0));
    /// ```
    pub fn invert(self) -> Range {
        Range { start: self.end, end: self.start }
    }

    /// Map the given Scalar from `Self` to some other given `Range`.
    ///
    /// # Examples
    ///
    /// ```
    /// use conrod_core::Range;
    ///
    /// let a = Range::new(0.0, 5.0);
    ///
    /// let b = Range::new(0.0, 10.0);
    /// assert_eq!(a.map_value_to(2.5, &b), 5.0);
    /// assert_eq!(a.map_value_to(0.0, &b), 0.0);
    /// assert_eq!(a.map_value_to(5.0, &b), 10.0);
    /// assert_eq!(a.map_value_to(-5.0, &b), -10.0);
    /// assert_eq!(a.map_value_to(10.0, &b), 20.0);
    ///
    /// let c = Range::new(10.0, -10.0);
    /// assert_eq!(a.map_value_to(2.5, &c), 0.0);
    /// assert_eq!(a.map_value_to(0.0, &c), 10.0);
    /// assert_eq!(a.map_value_to(5.0, &c), -10.0);
    /// assert_eq!(a.map_value_to(-5.0, &c), 30.0);
    /// assert_eq!(a.map_value_to(10.0, &c), -30.0);
    /// ```
    pub fn map_value_to(&self, value: Scalar, other: &Range) -> Scalar {
        ::utils::map_range(value, self.start, self.end, other.start, other.end)
    }

    /// Shift the `Range` start and end points by a given `Scalar`.
    ///
    /// # Examples
    ///
    /// ```
    /// use conrod_core::Range;
    ///
    /// assert_eq!(Range::new(0.0, 5.0).shift(5.0), Range::new(5.0, 10.0));
    /// assert_eq!(Range::new(0.0, 5.0).shift(-5.0), Range::new(-5.0, 0.0));
    /// assert_eq!(Range::new(5.0, -5.0).shift(-5.0), Range::new(0.0, -10.0));
    /// ```
    pub fn shift(self, amount: Scalar) -> Range {
        Range { start: self.start + amount, end: self.end + amount }
    }

    /// The direction of the Range represented as a normalised scalar.
    ///
    /// # Examples
    ///
    /// ```
    /// use conrod_core::Range;
    ///
    /// assert_eq!(Range::new(0.0, 5.0).direction(), 1.0);
    /// assert_eq!(Range::new(0.0, 0.0).direction(), 0.0);
    /// assert_eq!(Range::new(0.0, -5.0).direction(), -1.0);
    /// ```
    pub fn direction(&self) -> Scalar {
        if      self.start < self.end { 1.0 }
        else if self.start > self.end { -1.0 }
        else                          { 0.0 }
    }

    /// Converts the Range to an undirected Range. By ensuring that `start` <= `end`.
    ///
    /// If `start` > `end`, then the start and end points will be swapped.
    ///
    /// # Examples
    ///
    /// ```
    /// use conrod_core::Range;
    ///
    /// assert_eq!(Range::new(0.0, 5.0).undirected(), Range::new(0.0, 5.0));
    /// assert_eq!(Range::new(5.0, 1.0).undirected(), Range::new(1.0, 5.0));
    /// assert_eq!(Range::new(10.0, -10.0).undirected(), Range::new(-10.0, 10.0));
    /// ```
    pub fn undirected(self) -> Range {
        if self.start > self.end { self.invert() } else { self }
    }

    /// The Range that encompasses both self and the given Range.
    ///
    /// The returned Range's `start` will always be <= its `end`.
    ///
    /// # Examples
    ///
    /// ```
    /// use conrod_core::Range;
    ///
    /// let a = Range::new(0.0, 3.0);
    /// let b = Range::new(7.0, 10.0);
    /// assert_eq!(a.max(b), Range::new(0.0, 10.0));
    ///
    /// let c = Range::new(-20.0, -30.0);
    /// let d = Range::new(5.0, -7.5);
    /// assert_eq!(c.max(d), Range::new(-30.0, 5.0));
    /// ```
    pub fn max(self, other: Self) -> Range {
        let start = self.start.min(self.end).min(other.start).min(other.end);
        let end = self.start.max(self.end).max(other.start).max(other.end);
        Range::new(start, end)
    }

    /// The Range that represents the range of the overlap between two Ranges if there is some.
    ///
    /// Note that If one end of `self` aligns exactly with the opposite end of `other`, `Some`
    /// `Range` will be returned with a magnitude of `0.0`. This is useful for algorithms that
    /// involve calculating the visibility of widgets, as it allows for including widgets whose
    /// bounding box may be a one dimensional straight line.
    ///
    /// The returned `Range`'s `start` will always be <= its `end`.
    ///
    /// # Examples
    ///
    /// ```
    /// use conrod_core::Range;
    ///
    /// let a = Range::new(0.0, 6.0);
    /// let b = Range::new(4.0, 10.0);
    /// assert_eq!(a.overlap(b), Some(Range::new(4.0, 6.0)));
    ///
    /// let c = Range::new(10.0, -30.0);
    /// let d = Range::new(-5.0, 20.0);
    /// assert_eq!(c.overlap(d), Some(Range::new(-5.0, 10.0)));
    /// 
    /// let e = Range::new(0.0, 2.5);
    /// let f = Range::new(50.0, 100.0);
    /// assert_eq!(e.overlap(f), None);
    /// ```
    pub fn overlap(mut self, mut other: Self) -> Option<Range> {
        self = self.undirected();
        other = other.undirected();
        let start = ::utils::partial_max(self.start, other.start);
        let end = ::utils::partial_min(self.end, other.end);
        let magnitude = end - start;
        if magnitude >= 0.0 {
            Some(Range::new(start, end))
        } else {
            None
        }
    }

    /// The Range that encompasses both self and the given Range.
    ///
    /// The same as [**Range::max**](./struct.Range#method.max) but retains `self`'s original
    /// direction.
    ///
    /// # Examples
    ///
    /// ```
    /// use conrod_core::Range;
    ///
    /// let a = Range::new(0.0, 3.0);
    /// let b = Range::new(7.0, 10.0);
    /// assert_eq!(a.max_directed(b), Range::new(0.0, 10.0));
    ///
    /// let c = Range::new(-20.0, -30.0);
    /// let d = Range::new(5.0, -7.5);
    /// assert_eq!(c.max_directed(d), Range::new(5.0, -30.0));
    /// ```
    pub fn max_directed(self, other: Self) -> Range {
        if self.start <= self.end { self.max(other) }
        else                      { self.max(other).invert() }
    }

    /// Is the given scalar within our range.
    ///
    /// # Examples
    ///
    /// ```
    /// use conrod_core::Range;
    ///
    /// let range = Range::new(0.0, 10.0);
    /// assert!(range.is_over(5.0));
    /// assert!(!range.is_over(12.0));
    /// assert!(!range.is_over(-1.0));
    /// assert!(range.is_over(0.0));
    /// assert!(range.is_over(10.0));
    /// ```
    pub fn is_over(&self, pos: Scalar) -> bool {
        let Range { start, end } = self.undirected();
        pos >= start && pos <= end
    }

    /// Round the values at both ends of the Range and return the result.
    ///
    /// # Examples
    ///
    /// ```
    /// use conrod_core::Range;
    ///
    /// assert_eq!(Range::new(0.25, 9.5).round(), Range::new(0.0, 10.0));
    /// assert_eq!(Range::new(4.95, -5.3).round(), Range::new(5.0, -5.0));
    /// ```
    pub fn round(self) -> Range {
        Range::new(self.start.round(), self.end.round())
    }

    /// Floor the values at both ends of the Range and return the result.
    ///
    /// # Examples
    ///
    /// ```
    /// use conrod_core::Range;
    ///
    /// assert_eq!(Range::new(0.25, 9.5).floor(), Range::new(0.0, 9.0));
    /// assert_eq!(Range::new(4.95, -5.3).floor(), Range::new(4.0, -6.0));
    /// ```
    pub fn floor(self) -> Range {
        Range::new(self.start.floor(), self.end.floor())
    }

    /// The Range with some padding given to the `start` value.
    ///
    /// # Examples
    ///
    /// ```
    /// use conrod_core::Range;
    ///
    /// assert_eq!(Range::new(0.0, 10.0).pad_start(2.0), Range::new(2.0, 10.0));
    /// assert_eq!(Range::new(10.0, 0.0).pad_start(2.0), Range::new(8.0, 0.0));
    /// ```
    pub fn pad_start(mut self, pad: Scalar) -> Range {
        self.start += if self.start <= self.end { pad } else { -pad };
        self
    }

    /// The Range with some padding given to the `end` value.
    ///
    /// # Examples
    ///
    /// ```
    /// use conrod_core::Range;
    ///
    /// assert_eq!(Range::new(0.0, 10.0).pad_end(2.0), Range::new(0.0, 8.0));
    /// assert_eq!(Range::new(10.0, 0.0).pad_end(2.0), Range::new(10.0, 2.0));
    /// ```
    pub fn pad_end(mut self, pad: Scalar) -> Range {
        self.end += if self.start <= self.end { -pad } else { pad };
        self
    }

    /// The Range with some given padding to be applied to each end.
    ///
    /// # Examples
    ///
    /// ```
    /// use conrod_core::Range;
    ///
    /// assert_eq!(Range::new(0.0, 10.0).pad(2.0), Range::new(2.0, 8.0));
    /// assert_eq!(Range::new(10.0, 0.0).pad(2.0), Range::new(8.0, 2.0));
    /// ```
    pub fn pad(self, pad: Scalar) -> Range {
        self.pad_start(pad).pad_end(pad)
    }

    /// The Range with some padding given for each end.
    ///
    /// # Examples
    ///
    /// ```
    /// use conrod_core::Range;
    ///
    /// assert_eq!(Range::new(0.0, 10.0).pad_ends(1.0, 2.0), Range::new(1.0, 8.0));
    /// assert_eq!(Range::new(10.0, 0.0).pad_ends(4.0, 3.0), Range::new(6.0, 3.0));
    /// ```
    pub fn pad_ends(self, start: Scalar, end: Scalar) -> Range {
        self.pad_start(start).pad_end(end)
    }

    /// Clamp the given value to the range.
    ///
    /// # Examples
    ///
    /// ```
    /// use conrod_core::Range;
    ///
    /// assert_eq!(Range::new(0.0, 5.0).clamp_value(7.0), 5.0);
    /// assert_eq!(Range::new(5.0, -2.5).clamp_value(-3.0), -2.5);
    /// assert_eq!(Range::new(5.0, 10.0).clamp_value(0.0), 5.0);
    /// ```
    pub fn clamp_value(&self, value: Scalar) -> Scalar {
        ::utils::clamp(value, self.start, self.end)
    }

    /// Stretch the end that is closest to the given value only if it lies outside the Range.
    ///
    /// The resulting Range will retain the direction of the original range.
    ///
    /// # Examples
    ///
    /// ```
    /// use conrod_core::Range;
    ///
    /// let a = Range::new(2.5, 5.0);
    /// assert_eq!(a.stretch_to_value(10.0), Range::new(2.5, 10.0));
    /// assert_eq!(a.stretch_to_value(0.0), Range::new(0.0, 5.0));
    ///
    /// let b = Range::new(0.0, -5.0);
    /// assert_eq!(b.stretch_to_value(10.0), Range::new(10.0, -5.0));
    /// assert_eq!(b.stretch_to_value(-10.0), Range::new(0.0, -10.0));
    /// ```
    pub fn stretch_to_value(self, value: Scalar) -> Range {
        let Range { start, end } = self;
        if start <= end {
            if value < start {
                Range { start: value, end: end }
            } else if value > end {
                Range { start: start, end: value }
            } else {
                self
            }
        } else {
            if value < end {
                Range { start: start, end: value }
            } else if value > start {
                Range { start: value, end: end }
            } else {
                self
            }
        }
    }

    /// Does `self` have the same direction as `other`.
    ///
    /// # Examples
    ///
    /// ```
    /// use conrod_core::Range;
    ///
    /// assert!(Range::new(0.0, 1.0).has_same_direction(Range::new(100.0, 200.0)));
    /// assert!(Range::new(0.0, -5.0).has_same_direction(Range::new(-2.5, -6.0)));
    /// assert!(!Range::new(0.0, 5.0).has_same_direction(Range::new(2.5, -2.5)));
    /// ```
    pub fn has_same_direction(self, other: Self) -> bool {
        let self_direction = self.start <= self.end;
        let other_direction = other.start <= other.end;
        self_direction == other_direction
    }

    /// Align the `start` of `self` to the `start` of the `other` **Range**.
    ///
    /// If the direction of `other` is different to `self`, `self`'s `end` will be aligned to the
    /// `start` of `other` instead.
    ///
    /// # Examples
    ///
    /// ```
    /// use conrod_core::Range;
    ///
    /// let a = Range::new(2.5, 7.5);
    /// let b = Range::new(0.0, 10.0);
    /// assert_eq!(a.align_start_of(b), Range::new(0.0, 5.0));
    /// assert_eq!(b.align_start_of(a), Range::new(2.5, 12.5));
    ///
    /// let c = Range::new(2.5, -2.5);
    /// let d = Range::new(-5.0, 5.0);
    /// assert_eq!(c.align_start_of(d), Range::new(0.0, -5.0));
    /// assert_eq!(d.align_start_of(c), Range::new(-7.5, 2.5));
    /// ```
    pub fn align_start_of(self, other: Self) -> Self {
        let diff = if self.has_same_direction(other) {
            other.start - self.start
        } else {
            other.start - self.end
        };
        self.shift(diff)
    }

    /// Align the `end` of `self` to the `end` of the `other` **Range**.
    ///
    /// If the direction of `other` is different to `self`, `self`'s `start` will be aligned to the
    /// `end` of `other` instead.
    ///
    /// # Examples
    ///
    /// ```
    /// use conrod_core::Range;
    ///
    /// let a = Range::new(2.5, 7.5);
    /// let b = Range::new(0.0, 10.0);
    /// assert_eq!(a.align_end_of(b), Range::new(5.0, 10.0));
    /// assert_eq!(b.align_end_of(a), Range::new(-2.5, 7.5));
    ///
    /// let c = Range::new(2.5, -2.5);
    /// let d = Range::new(-5.0, 5.0);
    /// assert_eq!(c.align_end_of(d), Range::new(5.0, 0.0));
    /// assert_eq!(d.align_end_of(c), Range::new(-2.5, 7.5));
    /// ```
    pub fn align_end_of(self, other: Self) -> Self {
        let diff = if self.has_same_direction(other) {
            other.end - self.end
        } else {
            other.end - self.start
        };
        self.shift(diff)
    }

    /// Align the middle of `self` to the middle of the `other` **Range**.
    ///
    /// # Examples
    ///
    /// ```
    /// use conrod_core::Range;
    ///
    /// let a = Range::new(0.0, 5.0);
    /// let b = Range::new(0.0, 10.0);
    /// assert_eq!(a.align_middle_of(b), Range::new(2.5, 7.5));
    /// assert_eq!(b.align_middle_of(a), Range::new(-2.5, 7.5));
    ///
    /// let c = Range::new(2.5, -2.5);
    /// let d = Range::new(-10.0, 0.0);
    /// assert_eq!(c.align_middle_of(d), Range::new(-2.5, -7.5));
    /// assert_eq!(d.align_middle_of(c), Range::new(-5.0, 5.0));
    /// ```
    pub fn align_middle_of(self, other: Self) -> Self {
        let diff = other.middle() - self.middle();
        self.shift(diff)
    }

    /// Aligns the `start` of `self` with the `end` of `other`.
    ///
    /// If the directions are opposite, aligns the `end` of self with the `end` of `other`.
    ///
    /// # Examples
    ///
    /// ```
    /// use conrod_core::Range;
    ///
    /// let a = Range::new(2.5, 7.5);
    /// let b = Range::new(0.0, 10.0);
    /// assert_eq!(a.align_after(b), Range::new(10.0, 15.0));
    /// assert_eq!(b.align_after(a), Range::new(7.5, 17.5));
    ///
    /// let c = Range::new(2.5, -2.5);
    /// let d = Range::new(-5.0, 5.0);
    /// assert_eq!(c.align_after(d), Range::new(10.0, 5.0));
    /// assert_eq!(d.align_after(c), Range::new(-12.5, -2.5));
    /// ```
    pub fn align_after(self, other: Self) -> Self {
        let diff = if self.has_same_direction(other) {
            other.end - self.start
        } else {
            other.end - self.end
        };
        self.shift(diff)
    }

    /// Aligns the `end` of `self` with the `start` of `other`.
    ///
    /// If the directions are opposite, aligns the `start` of self with the `start` of `other`.
    ///
    /// # Examples
    ///
    /// ```
    /// use conrod_core::Range;
    ///
    /// let a = Range::new(2.5, 7.5);
    /// let b = Range::new(0.0, 10.0);
    /// assert_eq!(a.align_before(b), Range::new(-5.0, 0.0));
    /// assert_eq!(b.align_before(a), Range::new(-7.5, 2.5));
    ///
    /// let c = Range::new(2.5, -2.5);
    /// let d = Range::new(-5.0, 5.0);
    /// assert_eq!(c.align_before(d), Range::new(-5.0, -10.0));
    /// assert_eq!(d.align_before(c), Range::new(2.5, 12.5));
    /// ```
    pub fn align_before(self, other: Self) -> Self {
        let diff = if self.has_same_direction(other) {
            other.start - self.end
        } else {
            other.start - self.start
        };
        self.shift(diff)
    }

    /// Align `self` to `other` along the *x* axis in accordance with the given `Align` variant.
    pub fn align_to(self, align: super::Align, other: Self) -> Self {
        match align {
            super::Align::Start => self.align_start_of(other),
            super::Align::Middle => self.align_middle_of(other),
            super::Align::End => self.align_end_of(other),
        }
    }

    /// The closest **Edge** of `self` to the given `scalar`.
    ///
    /// Returns **Start** if the distance between both **Edge**s is equal.
    ///
    /// # Examples
    ///
    /// ```
    /// use conrod_core::position::{Edge, Range};
    ///
    /// assert_eq!(Range::new(0.0, 10.0).closest_edge(4.0), Edge::Start);
    /// assert_eq!(Range::new(0.0, 10.0).closest_edge(7.0), Edge::End);
    /// assert_eq!(Range::new(0.0, 10.0).closest_edge(5.0), Edge::Start);
    /// ```
    pub fn closest_edge(&self, scalar: Scalar) -> Edge {
        let Range { start, end } = *self;
        let start_diff = if scalar < start { start - scalar } else { scalar - start };
        let end_diff = if scalar < end { end - scalar } else { scalar - end };
        if start_diff <= end_diff { Edge::Start } else { Edge::End }
    }

}
