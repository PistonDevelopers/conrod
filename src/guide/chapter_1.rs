/*!

**What is Conrod?**


## A Brief Summary

Conrod is a portable, 2D, [GUI][1] library for the [Rust programming language][2].

It provides an immediate-mode API that wraps a retained-mode widget state graph. This allows us to
expose a simple, robust and reactive interface while approaching the performance of traditional,
retained GUI frameworks.

We provide a set of primitive and commonly useful widgets, along with a [Widget] API that allows
for designing new, custom widgets composed of unique logic and other widgets.

Widgets are instantiated using the "Builder" aka "Method Chaining" pattern within an update loop.
In this sense, we can think of a Widget and its builder methods as a function with a highly
flexible and readable set of arguments.


## Screenshots and Videos

If you have an awesome looking GUI made with conrod, please send us a screenshot and we'll add it here :)

Here's [a youtube demo](https://www.youtube.com/watch?v=n2UrjogA0j0) of the [all_widgets.rs example](https://github.com/PistonDevelopers/conrod/blob/master/examples/all_widgets.rs).

Here's [another youtube demo](https://www.youtube.com/watch?v=_ZXLCVibI8c) of conrod being used to create a basic synth editor.

**A multimedia timeline widget.**

<img src="http://imgur.com/34NEw90.png" alt="timeline" style="width:700px;">

**The [all_widgets.rs example](https://github.com/PistonDevelopers/conrod/blob/master/examples/all_widgets.rs).**

<img src="http://i.imgur.com/xKXISdc.png" alt="all_widgets.rs" style="width:700px;">

**The [canvas.rs example](https://github.com/PistonDevelopers/conrod/blob/master/examples/canvas.rs).**

<img src="http://i.imgur.com/YtjjEJX.png" alt="canvas.rs" style="width:700px;">


## Feature Overview

- **Reactive** and **Immediate** API. Produce the UI from your application state in one place, rather than as a separate entity spread across callbacks, instantiation, updates and drawing.
- A **Widget trait** to simplify the composition and design of widgets.
- **Scrolling** that is opt-in, generalised and easy-to-use. Call `.scroll_kids(true)` on a widget and it will become a scrollable container for its children.
- **Powerful layout and positioning**:
    - Placement - `.middle()`, `.top_left_of(CANVAS)`, `.mid_right_with_margin(20.0)`, etc.
    - Alignment - `.align_left()`, `.align_top_of(LABEL)`, etc.
    - Relative - `.down(20.0)`, `.right_from(BUTTON, 40.0)`, `.x_y_relative(20.0, 42.0)`, etc.
    - Absolute - `.x_y(6.0, 7.0)`
    - Draggable pop-up / floating canvasses - `.floating(true)`.
    - **WidgetMatrix** and **PositionMatrix** for instantiating a grid of widgets. 
- **Theme**s for unique style and layout defaults.
- **widget_ids!** macro for easily and safely generating unique widget identifiers.
- Generic over events and graphics backends - compatible with:
    - [glutin][7], [sdl2][8] and [glfw][9] [**Window**][10] backends.
    - [gfx][11], [glium][12] and raw [opengl][13] [**Graphics**][14] backends.
- Provides a collection of commonly useful widgets.


## Available Widgets

The widgets that are provided by conrod can be broken into two sets: [**primitive**][3] and [**common
use**][4].


### Primitive Widgets

Primitive widgets are the fundamental graphical components in any conrod user interface. They are a
special set of widgets from which all other widgets may be composed. For example, a `Button` widget
may be composed of one `Rectangle` primitive widget and one `Text` primitive widget. A
`DropDownList` may be composed of *n* `Button` widgets, and in turn *n* `Rectangle` primitive
widgets and *n* `Text` primitive widgets.

When drawing the widget graph to the screen, conrod *only* draws the primitive widgets. This allows
widget designers to create custom widgets by simply instantiating other widgets, without having to
directly write any graphics code themselves. This also greatly simplifies the caching of common
widget state like text, positioning and styling as the primitive widgets take care of all of this
automatically.

The following are the primitive widgets provided by conrod:

- **[Line]**
- **[PointPath]**
- Shapes:
    - **[Circle]**
    - **[Rectangle]**
    - **[Oval]**
    - **[Polygon]**
    - **[FramedRectangle]**
- **[Text]** - automatic line-wrapping, line spacing, etc.

If conrod is lacking a primitive widget that you require, please let us know! There are currently
plans to allow for a user to design their own primitive widgets, however these will likely remain
low priority until requested.


### Common Use Widgets

Common use widgets are simply widgets that are commonly required by GUIs. There is nothing special
about these widgets in the sense that they could just as easily be implemented in an external
crate by third-party users. However, their usage is common enough that we choose to distribute them
with conrod itself.

The following are the common use widgets provided by conrod:

- **[Button]**
- **[Canvas]** - a container-like widget
- **[DropDownList]**
- **[EnvelopeEditor]**
- **[NumberDialer]**
- **[Slider]**
- **[Tabs]** - for easily switching between multiple [Canvas]ses with ease
- **[TextBox]**
- **[TitleBar]**
- **[Toggle]**
- **[WidgetMatrix]**
- **[XYPad]**

The following are planned, but not yet implemented:

- [Image (primitive widget)](https://github.com/PistonDevelopers/conrod/issues/647)
- [Menu Bar / Tool Bar](https://github.com/PistonDevelopers/conrod/issues/417)
- [Right-click Context Menu](https://github.com/PistonDevelopers/conrod/issues/394)
- [Multi-line Text Editor](https://github.com/PistonDevelopers/conrod/issues/62)
- [Graph / Chart](https://github.com/PistonDevelopers/conrod/issues/84)
- [File/Directory Navigator](https://github.com/PistonDevelopers/conrod/issues/381)
- [Advanced graph visualisation and control](https://github.com/PistonDevelopers/mush)

If you notice that we're missing important widgets, or if you have a widget that you feel would
make a nice addition to conrod, please let us know by leaving an [issue][5] or [pull request][6]!
Be sure to check [issues with the widget
label](https://github.com/PistonDevelopers/conrod/issues?q=is%3Aopen+is%3Aissue+label%3Awidget)
first, as your desired widget may have already been requested.


## "Immediate Mode"

### What is it?

The term "Immediate Mode" describes a style of user interface API.

In an immediate mode GUI, widgets are instantiated using functions in an *update* or *draw* loop.
This is quite different to the more traditional "retained mode", where widget *types* are
constructed during the *setup* stage.

Immediate mode encourages a less stateful, more data-driven design as the user interface is
instantiated as a *result* of the application state every frame or update. On the other hand,
retained user interfaces tend to work the other way around, driving the application by sending
events or triggering callback functions upon user interaction.

### Why use it?

Immediate mode tends to be the easier style to use for highly dynamic interfaces and those that
require frequent synchronisation with application state. This is because immediate widgets handle
changes in application state at the same place in which they are instantiated (during the update
or draw stage). While retained interface logic is often divided up between instantiation, updating,
drawing and event handling, immediate interfaces aim to consolidate all UI related logic within one
place.

Historically, due to their statelessness and frequent re-instantiation, immediate mode interfaces
have been known to pay the price of performance for their convenience. Although retained interfaces
can be more tedious to maintain and synchronise with application state, they are often less CPU
intensive as the majority of their state remains untouched following the application's setup stage.

### Is Conrod Immediate or Retained?

Conrod aims to adopt the best of both worlds by providing an immediate mode API over a hidden,
retained widget state graph. From the user's perspective conrod widgets tend to feel stateless, as
though they only exist for the lifetime of the scope in which they are instantiated. In actuality,
each widget's state is cached (or "retained") within the `Ui`'s internal widget graph. This allows
for efficient re-use of allocations and the ability to easily defer the drawing of widgets to a
stage that is more suited to the user's application. As a result, Conrod should be able to provide
the convenience of an immediate mode API alongside performance that approaches that of traditional,
retained GUIs.



[1]:  https://en.wikipedia.org/wiki/Graphical_user_interface        "Wikipedia - Graphical User Interface"
[2]:  https://www.rust-lang.org/                                    "The Rust Programming Language"
[3]:  ./index.html#primitive-widgets                                "Primitive Widgets"
[4]:  ./index.html#common-use-widgets                               "Common Use Widgets"
[5]:  https://github.com/PistonDevelopers/conrod/issues             "Conrod Github Issues"
[6]:  https://github.com/PistonDevelopers/conrod/pulls              "Conrod Github Pull Requests"
[7]:  https://github.com/PistonDevelopers/glutin_window             "glutin_window crate"
[8]:  https://github.com/PistonDevelopers/sdl2_window               "sdl2_window crate"
[9]:  https://github.com/PistonDevelopers/glfw_window               "glfw_window crate"
[10]: http://docs.piston.rs/piston/piston/window/trait.Window.html  "piston Window trait"
[11]: https://github.com/PistonDevelopers/gfx_graphics              "gfx_graphics crate"
[12]: https://github.com/PistonDevelopers/glium_graphics            "glium_graphics crate"
[13]: https://github.com/PistonDevelopers/opengl_graphics           "opengl_graphics crate"
[14]: http://docs.piston.rs/graphics/graphics/trait.Graphics.html   "piston Graphics trait"

[Widget]:       ../../trait.Widget.html         "Widget trait"
[Positionable]: ../../trait.Positionable.html   "Positionable trait"
[Theme]:        ../../struct.Theme.html         "Theme struct"
[widget_ids!]:  ../../macro.widget_ids!.html    "widget_ids! macro"

[Line]:             ../../struct.Line.html              "Line Widget"
[PointPath]:        ../../struct.PointPath.html         "PointPath Widget"
[Circle]:           ../../struct.Circle.html            "Circle Widget"
[Rectangle]:        ../../struct.Rectangle.html         "Rectangle Widget"
[Oval]:             ../../struct.Oval.html              "Oval Widget"
[Polygon]:          ../../struct.Polygon.html           "Polygon Widget"
[FramedRectangle]:  ../../struct.FramedRectangle.html   "FramedRectangle Widget"
[Text]:             ../../struct.Text.html              "Text Widget"

[Button]:           ../../struct.Button.html            "Button Widget"
[Canvas]:           ../../struct.Canvas.html            "Canvas Widget"
[DropDownList]:     ../../struct.DropDownList.html      "DropDownList Widget"
[EnvelopeEditor]:   ../../struct.EnvelopeEditor.html    "EnvelopeEditor Widget"
[NumberDialer]:     ../../struct.NumberDialer.html      "NumberDialer Widget"
[Slider]:           ../../struct.Slider.html            "Slider Widget"
[Tabs]:             ../../struct.Tabs.html              "Tabs Widget"
[TextBox]:          ../../struct.TextBox.html           "TextBox Widget"
[TitleBar]:         ../../struct.TitleBar.html          "TitleBar Widget"
[Toggle]:           ../../struct.Toggle.html            "Toggle Widget"
[WidgetMatrix]:     ../../struct.WidgetMatrix.html      "WidgetMatrix Widget"
[XYPad]:            ../../struct.XYPad.html             "XYPad Widget"

*/
