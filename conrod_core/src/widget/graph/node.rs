//! A default container widget to use for nodes that exist within a `Graph` widget.

use {widget, color, Color, Point, Positionable, Scalar, Sizeable, Widget, Ui};
use graph;
use position::{Axis, Direction, Range, Rect};
use std::iter::once;
use std::ops::{Deref, DerefMut};

/// A widget that acts as a convenience container for some `Node`'s unique widgets.
#[derive(Clone, Debug, WidgetCommon_)]
pub struct Node<W> {
    /// Data necessary and common for all widget builder types.
    #[conrod(common_builder)]
    pub common: widget::CommonBuilder,
    /// Unique styling for the **Node**.
    pub style: Style,
    /// The widget wrapped by this node container.
    pub widget: W,
    /// The number of input sockets on the node.
    pub inputs: usize,
    /// The number of output sockets on the node.
    pub outputs: usize,
}

#[allow(missing_docs)]
pub const DEFAULT_BORDER_THICKNESS: Scalar = 6.0;
#[allow(missing_docs)]
pub const DEFAULT_SOCKET_LENGTH: Scalar = DEFAULT_BORDER_THICKNESS;

/// Unique styling for the **BorderedRectangle** widget.
#[derive(Copy, Clone, Debug, Default, PartialEq, WidgetStyle_)]
pub struct Style {
    /// Shape color for the inner rectangle.
    #[conrod(default = "color::TRANSPARENT")]
    pub color: Option<Color>,
    /// The length of each rectangle along its `SocketSide`.
    #[conrod(default = "6.0")]
    pub socket_length: Option<Scalar>,
    /// The widget of the border around the widget.
    ///
    /// this should always be a positive value in order for sockets to remain visible.
    #[conrod(default = "6.0")]
    pub border: Option<Scalar>,
    /// The radius of the rounded corners of the border.
    ///
    /// This value will be clamped to the `border` thickness.
    ///
    /// A value of `0.0` gives square corners.
    #[conrod(default = "6.0")]
    pub border_radius: Option<Scalar>,
    /// Color of the border.
    #[conrod(default = "color::DARK_CHARCOAL")]
    pub border_color: Option<Color>,
    /// Color of the sockets.
    #[conrod(default = "color::DARK_GREY")]
    pub socket_color: Option<Color>,
    /// Default layout for input sockets.
    #[conrod(default = "SocketLayout { side: SocketSide::Left, direction: Direction::Backwards }")]
    pub input_socket_layout: Option<SocketLayout>,
    /// Default layout for node output sockets.
    #[conrod(default = "SocketLayout { side: SocketSide::Right, direction: Direction::Backwards }")]
    pub output_socket_layout: Option<SocketLayout>,
}

/// Describes the layout of either input or output sockets.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct SocketLayout {
    /// Represents the side of a node widget's bounding rectangle.
    pub side: SocketSide,
    /// The direction in which sockets will be laid out over the side.
    pub direction: Direction,
}

/// Represents the side of a node widget's bounding rectangle.
///
/// This is used to describe default node socket layout.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[allow(missing_docs)]
pub enum SocketSide {
    Left,
    Right,
    Top,
    Bottom,
}

widget_ids! {
    struct Ids {
        // Use triangles to describe graphics for the entire widget.
        //
        // The `Node` widget will be used a lot, so the less `widget::Id`s required the better.
        //
        // Triangulation order is as follows:
        //
        // 1. Inner rectangle surface (two triangles).
        // 2. Border (eight triangles).
        // 3. Sockets (two triangles per socket).
        triangles,
        // The unique identifier for the wrapped widget.
        widget,
    }
}

/// Unique state for the `Node`.
pub struct State {
    ids: Ids,
    // Tracks whether or not a socket is currently captured under the mouse.
    capturing_socket: Option<(SocketType, usize)>,
    // The number of input sockets.
    inputs: usize,
    // The number of output sockets.
    outputs: usize,
}

/// Describes whether a socket is associated with a node's inputs or outputs.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[allow(missing_docs)]
pub enum SocketType { Input, Output }

/// The event produced by the `Node` widget.
#[derive(Clone, Debug)]
pub struct Event<W> {
    /// The event produced by the inner widget `W`.
    pub widget_event: W,
}

impl<W> Node<W> {
    /// Begin building a new `Node` widget.
    pub fn new(widget: W) -> Self {
        Node {
            common: widget::CommonBuilder::default(),
            style: Style::default(),
            widget,
            inputs: 0,
            outputs: 0,
        }
    }

    /// Specify the number of input sockets for the node.
    pub fn inputs(mut self, inputs: usize) -> Self {
        self.inputs = inputs;
        self
    }

    /// Specify the number of output sockets for the node.
    pub fn outputs(mut self, outputs: usize) -> Self {
        self.outputs = outputs;
        self
    }

    /// Specify the color for the node's inner rectangle.
    pub fn color(mut self, color: Color) -> Self {
        self.style.color = Some(color);
        self
    }

    /// The thickness of the border around the inner widget.
    ///
    /// This must always be a positive value in order for sockets to remain visible.
    pub fn border_thickness(mut self, thickness: Scalar) -> Self {
        assert!(thickness > 0.0);
        self.style.border = Some(thickness);
        self
    }

    /// Specify the color for the node's border.
    pub fn border_color(mut self, color: Color) -> Self {
        self.style.border_color = Some(color);
        self
    }

    /// Specify the radius for the node's border.
    pub fn border_radius(mut self, radius: Scalar) -> Self {
        self.style.border_radius = Some(radius);
        self
    }

    /// Specify the color for the node's sockets.
    pub fn socket_color(mut self, color: Color) -> Self {
        self.style.socket_color = Some(color);
        self
    }

    /// Specify the layout of the input sockets.
    pub fn input_socket_layout(mut self, layout: SocketLayout) -> Self {
        self.style.input_socket_layout = Some(layout);
        self
    }

    /// Specify the layout of the input sockets.
    pub fn output_socket_layout(mut self, layout: SocketLayout) -> Self {
        self.style.output_socket_layout = Some(layout);
        self
    }
}

impl<W> Deref for Event<W> {
    type Target = W;
    fn deref(&self) -> &Self::Target {
        &self.widget_event
    }
}

impl<W> DerefMut for Event<W> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.widget_event
    }
}


// A multiplier for the scalar direction.
fn direction_scalar(direction: Direction) -> Scalar {
    match direction {
        Direction::Forwards => 1.0,
        Direction::Backwards => -1.0,
    }
}

// Axis from a given side and the scalar offset from the centre of the rect.
fn side_axis_and_scalar(rect: Rect, side: SocketSide, border: Scalar) -> (Axis, Scalar) {
    match side {
        SocketSide::Left => (Axis::Y, rect.left() + border / 2.0),
        SocketSide::Right => (Axis::Y, rect.right() - border / 2.0),
        SocketSide::Bottom => (Axis::X, rect.bottom() + border / 2.0),
        SocketSide::Top => (Axis::X, rect.top() - border / 2.0),
    }
}

// The position of the socket at the given index.
fn socket_position(index: usize, start_pos: Point, step: [Scalar; 2]) -> Point {
    let x = start_pos[0] + step[0] * index as Scalar;
    let y = start_pos[1] + step[1] * index as Scalar;
    [x, y]
}

// Dimensions for a socket rectangle given some axis.
fn socket_rect_dim(axis: Axis, border: Scalar, socket_length: Scalar) -> [Scalar; 2] {
    match axis {
        Axis::Y => [border, socket_length],
        Axis::X => [socket_length, border],
    }
}

// Returns the range along the rect for the given axis.
fn rect_range(axis: Axis, rect: Rect) -> Range {
    match axis {
        Axis::X => rect.x,
        Axis::Y => rect.y,
    }
}

// The gap between each socket and the position of the first socket.
fn socket_step_and_start(
    n_sockets: usize,
    axis: Axis,
    direction: Direction,
    inner_rect: Rect,
    socket_length: Scalar,
    side_scalar: Scalar,
) -> ([Scalar; 2], Point)
{
    let direction_scalar = direction_scalar(direction);
    let socket_range = rect_range(axis, inner_rect);
    let socket_position_range = socket_range.pad(socket_length / 2.0);
    let socket_start_scalar = match direction {
        Direction::Forwards => socket_position_range.start,
        Direction::Backwards => socket_position_range.end,
    };
    let step = if n_sockets > 1 {
        socket_position_range.len() * direction_scalar / (n_sockets - 1) as Scalar
    } else {
        0.0
    };
    let (step, socket_start_position) = match axis {
        Axis::X => {
            let step = [step, 0.0];
            let x = socket_start_scalar;
            let y = side_scalar;
            (step, [x, y])
        },
        Axis::Y => {
            let step = [0.0, step];
            let x = side_scalar;
            let y = socket_start_scalar;
            (step, [x, y])
        },
    };
    (step, socket_start_position)
}

// Produce the `Rect` for a socket from the raw params required.
fn socket_rectangle(
    index: usize,
    n_sockets: usize,
    node_rect: Rect,
    border: Scalar,
    layout: SocketLayout,
    socket_length: Scalar,
) -> Rect {
    let SocketLayout { side, direction } = layout;
    let (axis, side_scalar) = side_axis_and_scalar(node_rect, side, border);
    let inner_rect = node_rect.pad(border);
    let (step, start_pos) = socket_step_and_start(n_sockets, axis, direction, inner_rect,
                                                  socket_length, side_scalar);
    let xy = socket_position(index, start_pos, step);
    let socket_dim = socket_rect_dim(axis, border, socket_length);
    let rect = Rect::from_xy_dim(xy, socket_dim);
    rect
}


/// Retrieve the `Rect` for the given socket on the given node.
///
/// Returns `None` if there is no node for the given `Id` or if the `socket_index` is out of range.
pub fn socket_rect(
    node_id: widget::Id,
    socket_type: SocketType,
    socket_index: usize,
    ui: &Ui,
) -> Option<Rect> {
    ui.widget_graph()
        .widget(node_id)
        .and_then(|container| {
            let unique = container.state_and_style::<State, Style>();
            let &graph::UniqueWidgetState { ref state, ref style } = match unique {
                None => return None,
                Some(unique) => unique,
            };
            let rect = container.rect;
            let border = style.border(&ui.theme);
            let socket_length = style.socket_length(&ui.theme);

            let (n_sockets, layout) = match socket_type {
                SocketType::Input => (state.inputs, style.input_socket_layout(&ui.theme)),
                SocketType::Output => (state.outputs, style.output_socket_layout(&ui.theme)),
            };

            let rect = socket_rectangle(socket_index, n_sockets, rect, border, layout,
                                        socket_length);
            Some(rect)
        })
}

/// Returns a `Rect` for an edge's start and end nodes.
pub fn edge_socket_rects<NI>(edge: &super::Edge<NI>, ui: &Ui) -> (Rect, Rect)
where
    NI: super::NodeId,
{
    let (start_id, end_id) = super::edge_node_widget_ids(edge, ui);
    let start = edge.start();
    let end = edge.end();
    let start_rect = socket_rect(start_id, SocketType::Output, start.socket_index, ui)
        .expect("no node widget found for the edge's `start_id`");
    let end_rect = socket_rect(end_id, SocketType::Input, end.socket_index, ui)
        .expect("no node widget found for the edge's `end_id`");
    (start_rect, end_rect)
}

/// Produces an iterator yielding a `Rect` for each socket for both inputs and outputs
/// respectively.
///
/// Returns `None` if no node is found for the given `widget::Id`.
pub fn socket_rects(node_id: widget::Id, ui: &Ui) -> Option<(SocketRects, SocketRects)> {
    ui.widget_graph()
        .widget(node_id)
        .and_then(|container| {
            let unique = container.state_and_style::<State, Style>();
            let &graph::UniqueWidgetState { ref state, ref style } = match unique {
                None => return None,
                Some(unique) => unique,
            };
            let rect = container.rect;
            let border = style.border(&ui.theme);
            let socket_length = style.socket_length(&ui.theme);
            let input_socket_rects = SocketRects {
                index: 0,
                n_sockets: state.inputs,
                node_rect: rect,
                border,
                layout: style.input_socket_layout(&ui.theme),
                socket_length,
            };
            let output_socket_rects = SocketRects {
                index: 0,
                n_sockets: state.outputs,
                node_rect: rect,
                border,
                layout: style.output_socket_layout(&ui.theme),
                socket_length,
            };
            Some((input_socket_rects, output_socket_rects))
        })
}

/// The rectangle for each socket (either inputs or outputs only).
#[derive(Clone)]
#[allow(missing_copy_implementations)]
pub struct SocketRects {
    // Current socket index.
    index: usize,
    // Total number of sockets.
    n_sockets: usize,
    node_rect: Rect,
    border: Scalar,
    layout: SocketLayout,
    // The length of the socket rectangle along the axis along which it is placed.
    socket_length: Scalar,
}

impl Iterator for SocketRects {
    type Item = Rect;
    fn next(&mut self) -> Option<Self::Item> {
        let SocketRects {
            ref mut index,
            n_sockets,
            node_rect,
            border,
            layout,
            socket_length,
        } = *self;

        // If the index is equal to or greater than the number of sockets, we're done.
        if *index >= n_sockets {
            return None;
        }

        let rect = socket_rectangle(*index, n_sockets, node_rect, border, layout, socket_length);
        *index += 1;
        Some(rect)
    }
}

impl<W> Widget for Node<W>
where
    W: Widget,
{
    type State = State;
    type Style = Style;
    type Event = Event<W::Event>;

    fn init_state(&self, id_gen: widget::id::Generator) -> Self::State {
        State {
            ids: Ids::new(id_gen),
            capturing_socket: None,
            inputs: self.inputs,
            outputs: self.outputs,
        }
    }

    fn style(&self) -> Self::Style {
        self.style.clone()
    }

    fn update(self, args: widget::UpdateArgs<Self>) -> Self::Event {
        let widget::UpdateArgs { id, state, style, rect, ui, .. } = args;
        let Node { widget, inputs, outputs, .. } = self;
        let socket_length = style.socket_length(&ui.theme);
        let border = style.border(&ui.theme);

        if state.inputs != inputs {
            state.update(|state| state.inputs = inputs);
        }

        if state.outputs != outputs {
            state.update(|state| state.outputs = outputs);
        }

        let input_socket_layout = style.input_socket_layout(&ui.theme);
        let output_socket_layout = style.output_socket_layout(&ui.theme);

        // A function for producing the rectangles of sockets along some axis.
        let socket_rectangles = |n_sockets, layout| {
            SocketRects {
                index: 0,
                n_sockets,
                layout,
                node_rect: rect,
                border,
                socket_length,
            }
        };

        // Whether or not the given point is over a socket.
        let over_socket = |abs_point: Point| -> Option<(SocketType, usize)> {
            for (i, rect) in socket_rectangles(inputs, input_socket_layout).enumerate() {
                if rect.is_over(abs_point) {
                    return Some((SocketType::Input, i));
                }
            }
            for (i, rect) in socket_rectangles(outputs, output_socket_layout).enumerate() {
                if rect.is_over(abs_point) {
                    return Some((SocketType::Output, i));
                }
            }
            None
        };

        #[derive(Copy, Clone)]
        enum Interaction { Hover, Press }
        let maybe_socket_interaction = ui.widget_input(id)
            .mouse()
            .and_then(|m| match m.buttons.left().is_down() {
                // If the mouse isn't down, we must be hovering over the widget.
                false => over_socket(m.abs_xy()).map(|(ty, ix)| (ty, ix, Interaction::Hover)),
                // Otherwise we're currently pressing some part of the widget.
                true => {
                    // If the left mouse button was just pressed, check to see if over a socket.
                    if ui.widget_input(id).presses().mouse().left().next().is_some() {
                        let maybe_socket = over_socket(m.abs_xy());
                        if maybe_socket.is_some() {
                            state.update(|state| state.capturing_socket = maybe_socket);
                        }
                    }
                    // If some socket is captured by the mouse, it's pressed.
                    state.capturing_socket.map(|(ty, ix)| (ty, ix, Interaction::Press))
                },
            });

        // The triangles for the inner rectangle surface first.
        let inner_rect = rect.pad(border);
        let (inner_tri_a, inner_tri_b) = widget::primitive::shape::rectangle::triangles(inner_rect);
        let inner_color = style.color(&ui.theme).into();
        let inner_triangles = once(inner_tri_a)
            .chain(once(inner_tri_b))
            .map(|tri| tri.color_all(inner_color));

        // Triangles for the border.
        //
        // Color the border based on interaction.
        let border_color = style.border_color(&ui.theme);
        let border_color = match maybe_socket_interaction.is_some() {
            true => border_color,
            false => {
                ui.widget_input(id)
                    .mouse()
                    .map(|m| match inner_rect.is_over(m.abs_xy()) {
                        true => border_color,
                        false => match m.buttons.left().is_down() {
                            true => border_color.clicked(),
                            false => border_color.highlighted(),
                        },
                    })
                    .unwrap_or(border_color)
            },
        };

        let border_radius = style.border_radius(&ui.theme).min(border);
        let border_triangles = widget::bordered_rectangle::rounded_border_triangles(
            rect,
            border,
            border_radius,
            widget::oval::DEFAULT_RESOLUTION,
        );
        let border_rgba = border_color.into();
        let border_triangles = border_triangles.map(|tri| tri.color_all(border_rgba));

        // A function for producing the triangles for sockets along some axis.
        let socket_color = style.socket_color(&ui.theme);
        let socket_triangles = |socket_type, n_sockets, layout| {
            socket_rectangles(n_sockets, layout)
                .enumerate()
                .flat_map(move |(i, rect)| {
                    let (tri_a, tri_b) = widget::primitive::shape::rectangle::triangles(rect);
                    let color = match maybe_socket_interaction {
                        Some((ty, ix, action)) if ty == socket_type && ix == i => match action {
                            Interaction::Hover => socket_color.highlighted(),
                            Interaction::Press => socket_color.clicked(),
                        },
                        _ => socket_color,
                    };
                    let rgba = color.into();
                    let a = tri_a.color_all(rgba);
                    let b = tri_b.color_all(rgba);
                    once(a).chain(once(b))
                })
        };

        // Triangles for sockets.
        let input_socket_triangles = socket_triangles(SocketType::Input, inputs, input_socket_layout);
        let output_socket_triangles = socket_triangles(SocketType::Output, outputs, output_socket_layout);

        // Submit the triangles for the graphical elements of the widget.
        let triangles = inner_triangles
            .chain(border_triangles)
            .chain(input_socket_triangles)
            .chain(output_socket_triangles);
        widget::Triangles::multi_color(triangles)
            .with_bounding_rect(rect)
            .graphics_for(id)
            .parent(id)
            .set(state.ids.triangles, ui);

        // Instantiate the widget.
        let widget_event = widget
            .wh(inner_rect.dim())
            .xy(inner_rect.xy())
            .parent(id)
            .set(state.ids.widget, ui);

        Event { widget_event }
    }
}
