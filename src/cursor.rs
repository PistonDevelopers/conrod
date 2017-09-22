//! Contains an extendable enum of supported mouse cursor types.
//!
//! Use this module to map from the conrod's mouse cursor types to the types known to the window
//! backend you are using. A lot of these are already implemented in `conrod::backend`. Unless you
//! are using custom mouse cursor types not provided here, then using one of the implementations in
//! `conrod::backend` should be sufficient.

/// This enum specifies cursor types used by internal widgets. For custom widgets using custom
/// cursor types, you can still use this enum by specifying a numbered custom variant.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum MouseCursor {
    /// Default mouse cursor.
    Arrow,
    /// Text input curosr.
    Text,
    /// Text input for vertical text.
    VerticalText,
    /// Open hand with index finger pointing up.
    Hand,
    /// Open hand.
    Grab,
    /// Closed hand.
    Grabbing,
    /// Vertical resize cursor.
    ResizeVertical,
    /// Horizontal resize cursor.
    ResizeHorizontal,
    /// Diagonal resize cursor pointing to top left and bottom right corners.
    ResizeTopLeftBottomRight,
    /// Diagonal resize cursor pointing to top right to bottom left corners.
    ResizeTopRightBottomLeft,
    /// Custom cursor variant. Encode your favourite cursor with a u8.
    Custom(u8),
}
