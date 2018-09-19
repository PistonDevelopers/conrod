//! An adaptation of the **Oval** type where the width and height are equal.

use {Color, Dimensions, Scalar};
use super::oval::{Full, Oval};
use super::Style as Style;
use widget;

/// A tiny wrapper around the **Oval** widget type.
#[derive(Copy, Clone, Debug)]
pub struct Circle;


fn rad_to_dim(radius: Scalar) -> Dimensions {
    let side = radius * 2.0;
    [side, side]
}


impl Circle {
    /// Build a circular **Oval** with the given dimensions and style.
    pub fn styled(radius: Scalar, style: Style) -> Oval<Full> {
        Oval::styled(rad_to_dim(radius), style)
    }

    /// Build a new **Fill**ed circular **Oval**.
    pub fn fill(radius: Scalar) -> Oval<Full> {
        Oval::fill(rad_to_dim(radius))
    }

    /// Build a new circular **Oval** **Fill**ed with the given color.
    pub fn fill_with(radius: Scalar, color: Color) -> Oval<Full> {
        Oval::fill_with(rad_to_dim(radius), color)
    }

    /// Build a new circular **Outline**d **Oval** widget.
    pub fn outline(radius: Scalar) -> Oval<Full> {
        Oval::outline(rad_to_dim(radius))
    }

    /// Build a new circular **Oval** **Outline**d with the given style.
    pub fn outline_styled(radius: Scalar, line_style: widget::line::Style) -> Oval<Full> {
        Oval::outline_styled(rad_to_dim(radius), line_style)
    }
}

