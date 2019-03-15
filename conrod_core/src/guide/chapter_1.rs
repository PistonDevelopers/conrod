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

Here's [a youtube demo](https://www.youtube.com/watch?v=n2UrjogA0j0) of the all widgets example.

Here's [another youtube demo](https://www.youtube.com/watch?v=_ZXLCVibI8c) of conrod being used to create a basic synth editor.

**A multimedia timeline widget.**

<img src="http://imgur.com/34NEw90.png" alt="timeline" style="width:700px;">

**The [all_piston_window.rs example](https://github.com/PistonDevelopers/conrod/blob/master/backends/conrod_piston/examples/all_piston_window.rs).**

<img src="http://i.imgur.com/xKXISdc.png" alt="all_widgets.rs" style="width:700px;">

**The [canvas.rs example](https://github.com/PistonDevelopers/conrod/blob/master/backends/conrod_glium/examples/canvas.rs).**

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
- **[Theme]**s for unique style and layout defaults.
- **[widget_ids!]** macro for easily and safely generating unique widget identifiers.
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
- **[Text]** - automatic line-wrapping, line spacing, etc.
- **[Image]**

If conrod is lacking a primitive widget that you require, please let us know! There are currently
plans to allow for a user to design their own primitive widgets, however these will likely remain
low priority until requested.


### Common Use Widgets

Common use widgets are simply widgets that are commonly required by GUIs. There is nothing special
about these widgets in the sense that they could just as easily be implemented in an external
crate by third-party users. However, their usage is common enough that we choose to distribute them
with conrod itself.

The following are the common use widgets provided by conrod:

- **[BorderedRectangle]**
- **[Button]**
- **[Canvas]** - a container-like widget
- **[DropDownList]**
- **[EnvelopeEditor]**
- **[Matrix]**
- **[NumberDialer]**
- **[PlotPath]**
- **[Scrollbar]** - emits scroll events to some target scroll widget
- **[Slider]**
- **[Tabs]** - for easily switching between multiple [Canvas]ses with ease
- **[TextBox]** - an editable one-line text field
- **[TextEdit]** - an editable block of text
- **[TitleBar]**
- **[Toggle]**
- **[XYPad]**

The following are planned, but not yet implemented:

- [Menu Bar / Tool Bar](https://github.com/PistonDevelopers/conrod/issues/417)
- [Right-click Context Menu](https://github.com/PistonDevelopers/conrod/issues/394)
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


## The Builder Pattern

The "Builder Pattern" describes the process of "building" a type by chaining together method calls.
Conrod uses this pattern within its API for widget instantiation. This pattern is perhaps easiest
to grasp with an example.

Let's say we have a `Button` widget. We want to make sure that a user may instantiate it with
specific dimensions, position, color, label, label font size and label color. A naiive constructor
might look like this:

```ignore
Button::new(dimensions,
            position,
            color::RED,
            "DONT PRESS",
            label_font_size,
            label_color,
            || do_stuff(),
            BUTTON_ID,
            &mut ui);
```

The problem is that most of the time users will just want the `Button` to choose defaults straight
from the `Ui`'s `Theme`, rather than specifying every parameter for every instance of a `Button`
themselves. We want parameters to be *optional*. Perhaps then, we should make optional parameters
take an `Option`?

```ignore
Button::new(Some(dimensions),
            Some(position),
            None,
            "DONT PRESS",
            None,
            None,
            Some(|| do_stuff()),
            BUTTON_ID,
            &mut ui);
```

Although the user no longer has to think about the values for parameters they don't care about,
this is still extremely verbose and the `None` parameters seem quite ambiguous.

In some languages, this problem might be solved using a combination of features called "named
parameters" and "default parameters". If rust had these features, the above might look like this:

```ignore
Button::new(dimensions=dimensions,
            position=position,
            label="DONT PRESS",
            react=|| do_stuff(),
            id=BUTTON_ID,
            ui=&mut ui);
```

Certainly an improvement in conciseness and readability. Although these features are not available
to us in Rust, there is a way in which we can emulate their behaviour - This is where the builder
pattern comes in.

By moving each parameter out of the `new` function args and into their own methods, we can treat
methods almost as though they are named parameters as in the above example. Here is what the above
might look like using the builder pattern:

```ignore
Button::new()
    .dimensions(dimensions)
    .position(position)
    .label("DONT PRESS")
    .react(|| do_stuff())
    .id(BUTTON_ID)
    .ui(&mut ui);
```


*This is in fact the method of widget instantiation used by conrod, though with some slight
differences in method naming (best to check the examples directory for a proper demo).*

Although this certainly seems like the nicest solution from an API perspective, the
attentive rustacean may notice that this requires extra work for the widget designer compared to
the previous examples. Previously, all work for widget instantiation was done within the `new`
function. The builder pattern implementation introduces a few differences worth noticing:

1. A method must be implemented on `Button` for every optional parameter.
2. The `Button::new` function must return some type that can be treated as a set of `Option`
   arguments which can be set as it is passed from builder method to builder method.
3. There must be some method that indicates the end of building and the instantiation of the widget.

In Conrod, we are of the opinion that, in the common case, the extra work on behalf of the widget
designers is worth the benefits gained by the users. However, we also make a significant effort to
address each of the above points best we can in order to reduce the workload of widget developers:

1. We provide a [`builder_methods!`][builder_methods!] macro which reduces the boilerplate of builder method
   implementation.
2. In Conrod, widgets are treated as a set of arguments to their state, rather than containing
   their own state themselves. This suits the immediate mode style of conrod, where widget state is
   hidden and stored within the `Ui`.
3. All Conrod widgets must take a unique identifier and a mutable reference to the `Ui`. We
   consolidate this into a single `.set(ID, &mut ui)` method, which we also use as the indication
   method to stop building and instantiate the widget within the `Ui`.

All of these points will be covered later in more detail within the widget implementation section
of the guide.



[1]:  https://en.wikipedia.org/wiki/Graphical_user_interface        "Wikipedia - Graphical User Interface"
[2]:  https://www.rust-lang.org/                                    "The Rust Programming Language"
[3]:  ./index.html#primitive-widgets                                "Primitive Widgets"
[4]:  ./index.html#common-use-widgets                               "Common Use Widgets"
[5]:  https://github.com/PistonDevelopers/conrod/issues             "Conrod Github Issues"
[6]:  https://github.com/PistonDevelopers/conrod/pulls              "Conrod Github Pull Requests"
[7]:  https://github.com/PistonDevelopers/glutin_window             "glutin_window crate"
[8]:  https://github.com/PistonDevelopers/sdl2_window               "sdl2_window crate"
[9]:  https://github.com/PistonDevelopers/glfw_window               "glfw_window crate"
[10]: http://docs.piston.rs/piston/window/trait.Window.html         "src Window trait"
[11]: https://github.com/PistonDevelopers/gfx_graphics              "gfx_graphics crate"
[12]: https://github.com/PistonDevelopers/glium_graphics            "glium_graphics crate"
[13]: https://github.com/PistonDevelopers/opengl_graphics           "opengl_graphics crate"
[14]: http://docs.piston.rs/graphics/graphics/trait.Graphics.html   "src Graphics trait"

[Widget]:               ../../widget/trait.Widget.html      "Widget trait"
[Theme]:                ../../theme/struct.Theme.html       "Theme struct"
[builder_methods!]:     ../../macro.builder_methods!.html   "builder_methods! macro"
[widget_ids!]:          ../../macro.widget_ids!.html        "widget_ids! macro"

[Line]:      ../../widget/primitive/line/struct.Line.html                 "Line Widget"
[PointPath]: ../../widget/primitive/point_path/struct.PointPath.html      "PointPath Widget"
[Circle]:    ../../widget/primitive/shape/circle/struct.Circle.html       "Circle Widget"
[Rectangle]: ../../widget/primitive/shape/rectangle/struct.Rectangle.html "Rectangle Widget"
[Oval]:      ../../widget/primitive/shape/oval/struct.Oval.html           "Oval Widget"
[Polygon]:   ../../widget/primitive/shape/polygon/struct.Polygon.html     "Polygon Widget"
[Text]:      ../../widget/primitive/text/struct.Text.html                 "Text Widget"
[Image]:     ../../widget/primitive/image/struct.Image.html               "Image Widget"

[BorderedRectangle]: ../../widget/bordered_rectangle/struct.BorderedRectangle.html "BorderedRectangle Widget"
[Button]:            ../../widget/button/struct.Button.html                        "Button Widget"
[Canvas]:            ../../widget/canvas/struct.Canvas.html                        "Canvas Widget"
[DropDownList]:      ../../widget/drop_down_list/struct.DropDownList.html          "DropDownList Widget"
[EnvelopeEditor]:    ../../widget/envelope_editor/struct.EnvelopeEditor.html       "EnvelopeEditor Widget"
[NumberDialer]:      ../../widget/number_dialer/struct.NumberDialer.html           "NumberDialer Widget"
[PlotPath]:          ../../widget/plot_path/struct.PlotPath.html                   "PlotPath Widget"
[Scrollbar]:         ../../widget/scrollbar/struct.Scrollbar.html                  "Scrollbar Widget"
[Slider]:            ../../widget/slider/struct.Slider.html                        "Slider Widget"
[Tabs]:              ../../widget/tabs/struct.Tabs.html                            "Tabs Widget"
[TextBox]:           ../../widget/text_box/struct.TextBox.html                     "TextBox Widget"
[TextEdit]:          ../../widget/text_edit/struct.TextEdit.html                   "TextBox Widget"
[TitleBar]:          ../../widget/title_bar/struct.TitleBar.html                   "TitleBar Widget"
[Toggle]:            ../../widget/toggle/struct.Toggle.html                        "Toggle Widget"
[Matrix]:            ../../widget/matrix/struct.Matrix.html                        "Matrix Widget"
[XYPad]:             ../../widget/xy_pad/struct.XYPad.html                         "XYPad Widget"

*/
