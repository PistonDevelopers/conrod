
use piston::{
    Game,
    GameEvent,
    RenderArgs,
    UpdateArgs,
    KeyPressArgs,
    KeyReleaseArgs,
    MousePressArgs,
    MouseReleaseArgs,
    MouseMoveArgs,
    MouseRelativeMoveArgs,
    MouseScrollArgs,
    Render,
    Update,
    KeyPress,
    KeyRelease,
    MousePress,
    MouseRelease,
    MouseMove,
    MouseRelativeMove,
    MouseScroll,
};
use piston::mouse;
use opengl_graphics::Gl;
use point::Point;
use std::default::Default;

/// State of the widget.
#[deriving(Show, Clone)]
pub enum DrawState {
    Normal,
    Highlighted,
    Clicked,
}

/// Location at which the widget will be added
/// to it's parent after the previous child. The
/// `uint` in the four directional variants specifies the
/// padding between this widget and the previous.
/// The `Relative` variant specifies a location relative
/// to the location of the previous widget.
/// The `Specific` variant specifies an exact location
/// for the Widget if this is preferred.
#[deriving(Show, Clone)]
pub enum RelativePosition {
    Up(uint),
    Down(uint),
    Left(uint),
    Right(uint),
    Relative(Point<int>),
    Specific(Point<int>),
}

impl RelativePosition {
    /// Returns an absolute position as a `Point` determined by using
    /// the RelativePosition in association with a previous `Point`.
    fn as_point(&self, prev: Point<int>) -> Point<int> {
        match self {
            &Up(pad) => prev + Point::new(0, -(pad as int), 0),
            &Down(pad) => prev + Point::new(0, pad as int, 0),
            &Left(pad) => prev + Point::new(-(pad as int), 0, 0),
            &Right(pad) => prev + Point::new(pad as int, 0, 0),
            &Relative(p) => prev + p,
            &Specific(p) => p,
        }
    }
}

/// Essential data for widget types. One of these must
/// be implemented for each type that implements
/// `Widget` and a reference to the Data must be returned
/// via the `get_widget_data` methods. A macro has been
/// provided to simplify implementation. I.e.:
///
/// pub struct MyWidget {
///     widget_data: widget::Data,
/// }
///
/// impl Widget for MyWidget {
///     impl_get_widget_data!(widget_data)
/// }
///
/// - `rel_pos` (Relative Position) indicates where the widget
/// will be positioned compared to it's previous sibling.
/// - `abs_pos` (Absolute Position) indicates the exact
/// location at which the widget exists. This is normally
/// determined and set by the parent widget using the `rel_pos`.
/// - `draw_state` describes the current visible state of the
/// widget.
#[deriving(Show, Clone)]
pub struct Data {
    rel_pos: RelativePosition,
    pub abs_pos: Point<int>,
    draw_state: DrawState,
}

impl Data {
    /// Basic constructor for Widget Data.
    pub fn new(pos: RelativePosition) -> Data {
        Data {
            rel_pos: pos,
            abs_pos: Default::default(),
            draw_state: Normal,
        }
    }
}

impl Default for Data {
    /// Default constructor for Widget Data.
    fn default() -> Data {
        Data {
            rel_pos: Down(0u),
            abs_pos: Default::default(),
            draw_state: Normal,
        }
    }
}

/// The base trait for all UI Widget types. The Widget system
/// currently takes the form of a hierarchical node structure,
/// where each Widget may be constructed of a series of 'children'
/// Widgets.
pub trait Widget {

    /// Return a reference to the widget data.
    fn get_widget_data(&self) -> &Data;

    /// Return a reference to the widget data.
    fn get_widget_data_mut(&mut self) -> &mut Data;

    /// Return the dimensions as a tuple holding width and height.
    fn get_dimensions(&self) -> (uint, uint) { (0u, 0u) }

    /// Return all children widgets.
    fn get_children(&self) -> Vec<&Widget> { Vec::new() }

    /// Return all children widgets.
    fn get_children_mut(&mut self) -> Vec<&mut Widget> { Vec::new() }

    /// Return the current draw state of the widget.
    fn get_draw_state(&self) -> DrawState { self.get_widget_data().draw_state }

    /// Set the current draw state for the widget.
    fn set_draw_state(&mut self, state: DrawState) {
        self.get_widget_data_mut().draw_state = state;
    }

    /// Get relative position of the widget.
    fn get_rel_pos(&self) -> RelativePosition {
        self.get_widget_data().rel_pos
    }

    /// Set relative position of the widget.
    fn set_rel_pos(&mut self, pos: RelativePosition) {
        self.get_widget_data_mut().rel_pos = pos;
    }

    /// Return the absolute position.
    fn get_abs_pos(&self) -> Point<int> { self.get_widget_data().abs_pos }

    /// Set the absolute position. Once set for this widget, the `abs_pos` will
    /// then be refreshed for all children widgets too.
    fn set_abs_pos(&mut self, pos: Point<int>) {
        self.get_widget_data_mut().abs_pos = pos;
        self.set_abs_pos_children(pos);
    }

    /// Set the absolute position for each child using their relative positions.
    fn set_abs_pos_children(&mut self, pos: Point<int>) {
        let mut prev = pos;
        self.get_children_mut().mut_iter().all(|child| {
            let next = child.get_rel_pos().as_point(prev);
            child.set_abs_pos(next);
            prev = next;
            true
        });
    }

    /// Get the number of children widgets.
    fn get_num_children(&self) -> uint { self.get_children().len() }

    /// Return whether or not the widget has been hit by a mouse_press.
    fn is_over(&self, _mouse_pos: Point<int>) -> bool { false }

    /// Return whether or not the widget has been clicked.
    fn is_clicked(&self, args: &MousePressArgs) -> bool {
        match (args.button, self.get_draw_state()) {
            (mouse::Left, Highlighted) => true,
            _ => false,
        }
    }

    /// Handle a Piston Game event. This allows for easy
    /// integration with the GameIterator. The `draw` method
    /// has been excluded in case the user would prefer to
    /// retain control over the order of rendering. `draw` also
    /// currently requires an `&mut Gl` arg, which is not
    /// integrated into the `Render` event.
    fn event(&mut self, event: &mut GameEvent) {
        match *event {
            Render(_) => (), // Call 'draw' manually.
            Update(ref mut args) => self.update(args),
            KeyPress(ref args) => self.key_press(args),
            KeyRelease(ref args) => self.key_release(args),
            MousePress(ref args) => self.mouse_press(args),
            MouseRelease(ref args) => self.mouse_release(args),
            MouseMove(ref args) => self.mouse_move(args),
            MouseRelativeMove(ref args) => self.mouse_relative_move(args),
            MouseScroll(ref args) => self.mouse_scroll(args),
        }
    }

    /// Setup the widget.
    fn load(&mut self) {
        self.load_children();
    }

    /// Setup the widget's children.
    fn load_children(&mut self) {
        self.get_children_mut().mut_iter().all(|child| { child.load(); true });
    }

    /// Draw the widget at the given location.
    fn draw(&mut self, args: &RenderArgs, gl: &mut Gl) {
        self.draw_children(args, gl); 
    }

    /// Draw the widget and all of it's children to screen.
    fn draw_children(&mut self, args: &RenderArgs, gl: &mut Gl) {
        self.get_children_mut().mut_iter().all(|child| { child.draw(args, gl); true });
    }

    /// Update the widget.
    fn update(&mut self, args: &UpdateArgs) {
        self.update_children(args);
    }

    /// Update all of the widget's children.
    fn update_children(&mut self, args: &UpdateArgs) {
        self.get_children_mut().mut_iter().all(|child| { child.update(args); true });
    }

    /// Call key_press for the widget.
    fn key_press(&mut self, args: &KeyPressArgs) {
        self.key_press_children(args);
    }

    /// Call key_press for the widget's children.
    fn key_press_children(&mut self, args: &KeyPressArgs) {
        self.get_children_mut().mut_iter().all(|child| { child.key_press(args); true });
    }

    /// Call key_release for the widget.
    fn key_release(&mut self, args: &KeyReleaseArgs) {
        self.key_release_children(args);
    }

    /// Call key_release for the widget's children.
    fn key_release_children(&mut self, args: &KeyReleaseArgs) {
        self.get_children_mut().mut_iter().all(|child| { child.key_release(args); true });
    }

    /// Call mouse_press for the widget.
    fn mouse_press(&mut self, args: &MousePressArgs) {
        self.mouse_press_update_draw_state(args);
        self.mouse_press_children(args);
    }

    /// Call mouse_press for the widget's children.
    fn mouse_press_children(&mut self, args: &MousePressArgs) {
        self.get_children_mut().mut_iter().all(|child| { child.mouse_press(args); true });
    }

    /// Call mouse_release for the widget.
    fn mouse_release(&mut self, args: &MouseReleaseArgs) {
        self.mouse_release_update_draw_state(args);
        self.mouse_release_children(args);
    }

    /// Call mouse_release for the widget's children.
    fn mouse_release_children(&mut self, args: &MouseReleaseArgs) {
        self.get_children_mut().mut_iter().all(|child| { child.mouse_release(args); true });
    }

    /// Call mouse move for the widget.
    fn mouse_move(&mut self, args: &MouseMoveArgs) {
        self.mouse_move_update_draw_state(args);
        self.mouse_move_children(args);
    }

    /// Call mouse_move for the widget's children.
    fn mouse_move_children(&mut self, args: &MouseMoveArgs) {
        self.get_children_mut().mut_iter().all(|child| { child.mouse_move(args); true });
    }

    /// Call mouse_relative_move for the widget.
    fn mouse_relative_move(&mut self, args: &MouseRelativeMoveArgs) {
        self.mouse_relative_move_children(args);
    }

    /// Call mouse_relative_move for the widget's children.
    fn mouse_relative_move_children(&mut self, args: &MouseRelativeMoveArgs) {
        self.get_children_mut().mut_iter().all(|child| { child.mouse_relative_move(args); true });
    }

    /// Call mouse_scroll for widget.
    fn mouse_scroll(&mut self, args: &MouseScrollArgs) {
        self.mouse_scroll_children(args);
    }

    /// Call mouse_scroll for the widget's children.
    fn mouse_scroll_children(&mut self, args: &MouseScrollArgs) {
        self.get_children_mut().mut_iter().all(|child| { child.mouse_scroll(args); true });
    }

    /// Check the draw state upon mouse_press and update if necessary.
    fn mouse_press_update_draw_state(&mut self, args: &MousePressArgs) {
       if self.is_clicked(args) {
           self.set_draw_state(Clicked);
       }
    }

    /// Check the draw state upon mouse_release and update if necessary.
    fn mouse_release_update_draw_state(&mut self, args: &MouseReleaseArgs) {
        match (args.button, self.get_draw_state()) {
            (mouse::Left, Clicked) => {
                self.set_draw_state(Highlighted);
            },
            _ => (),
        }
    }

    /// Check the draw state upon mouse_move and update if necessary.
    fn mouse_move_update_draw_state(&mut self, args: &MouseMoveArgs) {
        let p = Point::new(args.x as int, args.y as int, 0);
        match (self.is_over(p), self.get_draw_state()) {
            (true, Normal) | (true, Highlighted) => self.set_draw_state(Highlighted),
            (true, Clicked) => self.set_draw_state(Clicked),
            _ => self.set_draw_state(Normal),
        }
    }

}

