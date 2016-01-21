use input::{Input, MouseButton};
use input::keyboard::{ModifierKey};
use position::{Point, Scalar};


#[derive(Clone, PartialEq, Debug)]
#[allow(missing_docs)]
pub enum ConrodEvent {
    Raw(Input),
    MouseClick(MouseClick),
    MouseDrag(MouseDrag),
}

#[derive(Copy, Clone, PartialEq, Debug)]
#[allow(missing_docs)]
pub struct MouseDrag {
    pub button: MouseButton,
    pub start: Point,
    pub end: Point,
    pub modifier: ModifierKey,
    pub in_progress: bool,
}

#[derive(Copy, Clone, PartialEq, Debug)]
#[allow(missing_docs)]
pub struct MouseClick {
    pub button: MouseButton,
    pub location: Point,
    pub modifier: ModifierKey,
}
