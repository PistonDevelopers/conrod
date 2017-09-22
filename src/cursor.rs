/// This enum specifies cursor types used by prebuilt widgets. For custom widgets using custom
/// cursor types, you can still use this enum by specifying a numbered custom variant.
#[derive(Copy, Clone, Debug)]
pub enum MouseCursor {
    Arrow,
    Text,
    VerticalText,
    Hand,
    Grab,
    Grabbing,
    ResizeVertical,
    ResizeHorizontal,
    ResizeTopLeftDownRight,
    ResizeTopRightDownLeft,
    Custom(u8),
}
