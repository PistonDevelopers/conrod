
#![feature(slicing_syntax)]
#![deny(missing_copy_implementations)]
#![allow(unstable)]

#[macro_use] extern crate bitflags;
extern crate input;
extern crate event;
extern crate freetype;
extern crate graphics;
extern crate opengl_graphics;
extern crate "rustc-serialize" as rustc_serialize;
extern crate clock_ticks;
extern crate vecmath;
extern crate quack;

pub use background::Background;
pub use button::Button;
pub use drop_down_list::DropDownList;
pub use envelope_editor::EnvelopeEditorBuilder as EnvelopeEditor;
pub use envelope_editor::EnvelopePoint;
pub use label::Label;
pub use number_dialer::NumberDialer;
pub use slider::SliderBuilder as Slider;
pub use text_box::TextBoxBuilder as TextBox;
pub use toggle::ToggleBuilder as Toggle;
pub use widget_matrix::WidgetMatrixBuilder as WidgetMatrix;
pub use xy_pad::XYPadBuilder as XYPad;

pub use callback::Callable;
pub use color::{Color, Colorable};
pub use draw::Drawable;
pub use frame::{Framing, Frameable};
pub use label::Labelable;
pub use point::Point;
pub use position::Positionable;
pub use shape::Shapeable;
pub use theme::Theme;
pub use ui_context::UiContext;
pub use widget::Widget;

use quack::{ GetFrom, Get };

#[macro_use]
pub mod macros;

pub mod internal;
pub mod background;
pub mod button;
pub mod callback;
pub mod color;
pub mod draw;
pub mod drop_down_list;
pub mod envelope_editor;
pub mod frame;
pub mod label;
pub mod mouse;
pub mod number_dialer;
pub mod point;
pub mod position;
pub mod rectangle;
pub mod shape;
pub mod slider;
pub mod text_box;
pub mod theme;
pub mod toggle;
pub mod ui_context;
pub mod utils;
pub mod widget;
pub mod widget_matrix;
pub mod xy_pad;

/// Font size property.
#[derive(Copy, Show)]
pub enum FontSize {
    Value(internal::FontSize),
    Small,
    Medium,
    Large
}

impl FontSize {
    pub fn size(&self, theme: &Theme) -> internal::FontSize {
        match self {
            &FontSize::Value(x) => x,
            &FontSize::Small => theme.font_size_small,
            &FontSize::Medium => theme.font_size_medium,
            &FontSize::Large => theme.font_size_large,
        }
    }
}

/// Point property.
#[derive(Copy)]
pub struct Position(pub internal::Point);

impl Position {
    /// Creates position to the right of another widget.
    pub fn right_from<T>(widget: &T, offset: internal::Scalar) -> Self
        where
            (Position, T): GetFrom<Property = Position, Object = T>,
            (Dimensions, T): GetFrom<Property = Dimensions, Object = T>,
    {
        let Position(pos) = widget.get();
        let Dimensions(dim) = widget.get();
        Position([pos[0] + dim[0] + offset, pos[1]])
    }

    /// Creates a position below another widget.
    pub fn down<T>(widget: &T, offset: internal::Scalar) -> Self
        where
            (Position, T): GetFrom<Property = Position, Object = T>,
            (Dimensions, T): GetFrom<Property = Dimensions, Object = T>,
    {
        let Position(pos) = widget.get();
        let Dimensions(dim) = widget.get();
        Position([pos[0], pos[1] + dim[1] + offset])
    }
}

/// Point property.
#[derive(Copy)]
pub struct Dimensions(pub internal::Dimensions);

/// Text property.
#[derive(Copy)]
pub struct Text<'a>(pub &'a str);

/// Frame property.
#[derive(Copy)]
pub struct Frame(pub internal::Frame);

/// Frame color property.
#[derive(Copy)]
pub struct FrameColor(pub internal::Color);

#[derive(Copy)]
pub struct MaybeColor(pub Option<internal::Color>);
