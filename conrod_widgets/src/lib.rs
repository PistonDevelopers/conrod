#![feature(custom_attribute)]

use conrod_core::widget as widget;
use conrod_core::theme::Theme as Theme;

pub use crate::{
    canvas::Canvas,
    collapsible_area::CollapsibleArea,
    drop_down_list::DropDownList,
    envelope_editor::EnvelopeEditor,
    list::List,
    list_select::ListSelect,
    grid::Grid,
    matrix::Matrix,
    number_dialer::NumberDialer,
    plot_path::PlotPath,
    range_slider::RangeSlider,
    scrollbar::Scrollbar,
    slider::Slider,
    tabs::Tab,
    text_box::TextBox,
    text_edit::TextEdit,
    title_bar::TitleBar,
    toggle::Toggle,
    xy_pad::XYPad,
};

mod canvas;
mod collapsible_area;
mod drop_down_list;
mod envelope_editor;
mod grid;
mod list;
mod list_select;
mod matrix;
mod number_dialer;
mod plot_path;
mod range_slider;
mod scrollbar;
mod slider;
mod tabs;
mod text_box;
mod text_edit;
mod title_bar;
mod toggle;
mod xy_pad;
