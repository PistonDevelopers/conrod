# Conrod [![Build Status](https://travis-ci.org/PistonDevelopers/conrod.svg?branch=master)](https://travis-ci.org/PistonDevelopers/conrod)

An easy-to-use, 2D GUI library written entirely in Rust.

Here's a demo!

[https://www.youtube.com/watch?v=n2UrjogA0j0](https://www.youtube.com/watch?v=n2UrjogA0j0)

You can find the example project that was used for that video [here](https://github.com/PistonDevelopers/conrod/blob/master/examples/all_widgets.rs).

[Documentation](http://docs.piston.rs/conrod/conrod/)

[How to contribute](https://github.com/PistonDevelopers/piston/blob/master/CONTRIBUTING.md)

[Licenses of example assets](https://github.com/PistonDevelopers/conrod/issues/319)

Current State
-------------

We're just starting to reach a stable-ish API pattern! There will still be large changes however these are more likely to be new features than API overhauls. Next on the todo list is `Theme`s!

An example project will be kept up-to-date featuring demonstrations of all working widgets. Conrod is still in very early stages however definitely usable.

Available Widgets
-----------------

- Button
- Canvas (Can be positioned manually or by using Splits or Tabs)
- Drop Down List
- Envelope Editor
- Label
- Number Dialer
- Slider
- TextBox
- Toggle
- XYPad
- Custom: Conrod also provides a `Widget` trait for designing and implementing custom widgets. You can find an annotated demonstration of designing a custom widget implementation [here](https://github.com/PistonDevelopers/conrod/blob/master/examples/custom_widget.rs). All [internal widgets](https://github.com/PistonDevelopers/conrod/blob/master/src/widget) also use this same trait so they should make for decent examples. If you feel like your widget is useful enough to be included within the internal widget library, feel free to add them in a pull request :)

**To-do:**
- [Menu Bar / Tool Bar](https://github.com/PistonDevelopers/conrod/issues/417)
- [Right-click Context Menu](https://github.com/PistonDevelopers/conrod/issues/394)
- [Text Area](https://github.com/PistonDevelopers/conrod/issues/62)
- [Graph / Chart](https://github.com/PistonDevelopers/conrod/issues/84)
- [File/Directory Navigator](https://github.com/PistonDevelopers/conrod/issues/381)
- [Advanced graph visualisation and control](https://github.com/PistonDevelopers/mush)

If conrod is missing anything you really wish it had, let us know with an issue describing the widget's style, behaviour and functionality - or even better, submit a pull request :D

Make sure you check the [`widget` label](https://github.com/PistonDevelopers/conrod/labels/widget) for your desired widget first as it may have already been requested.

Dependencies
------------

- [freetype](http://www.freetype.org/download.html) - at the moment, Conrod uses [freetype-rs](https://github.com/PistonDevelopers/freetype-rs) for its font rendering, which means you'll need to have the freetype library installed on your system. You can [download and install the freetype library here](http://www.freetype.org/download.html).


Getting Started
---------------

Build the conrod lib like this:

```
git clone https://github.com/PistonDevelopers/conrod.git
cd conrod
cargo build
```

And then build and run the examples like this:

```
cargo run --example all_widgets
cargo run --example canvas
```

You can add it to your project by adding this to your Cargo.toml:

```toml
[dependencies]
conrod = "*"
```

## Dependencies

![dependencies](./Cargo.png)

## Conrod uses Elmesque

Conrod uses [Elmesque](https://github.com/mitchmindtree/elmesque) under the hood for its 2D
graphics and layout. You don't need to know about Elmesque to use Conrod. But if you want to
combine Conrod with your own custom Elmesque drawing, see [the example](https://github.com/PistonDevelopers/conrod/blob/master/examples/elmesque.rs).
