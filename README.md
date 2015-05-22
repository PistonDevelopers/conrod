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
- Drop Down List
- Envelope Editor
- Label
- Number Dialer
- Slider
- TextBox
- Toggle
- XYPad
- Custom: Conrod also provides a `Widget` trait for designing and implementing custom widgets. All [internal widgets](https://github.com/PistonDevelopers/conrod/blob/master/src/widget) also use this same trait so they should make for decent examples. If you feel like your widget is useful enough to be included within the internal widget library, feel free add them in a pull request :)

**To-do:**
- [Menu Bar / Tool Bar](https://github.com/PistonDevelopers/conrod/issues/417)
- [Right-click Context Menu](https://github.com/PistonDevelopers/conrod/issues/394)
- [Text Area](https://github.com/PistonDevelopers/conrod/issues/62)
- [Graph / Chart](https://github.com/PistonDevelopers/conrod/issues/84)
- [File/Directory Navigator](https://github.com/PistonDevelopers/conrod/issues/381)
- [Advanced graph visualisation and control](https://github.com/PistonDevelopers/mush)

If conrod is missing anything you really wish it had, let us know with an issue describing the widget's style, behaviour and functionality - or even better, submit a pull request :D

Dependencies
------------

- [freetype](http://www.freetype.org/download.html) - at the moment, Conrod uses [freetype-rs](https://github.com/PistonDevelopers/freetype-rs) for its font rendering, which means you'll need to have the freetype library installed on your system. You can [download and install the freetype library here](http://www.freetype.org/download.html).


Getting Started
---------------

Build the conrod lib like this:

    git clone https://github.com/PistonDevelopers/conrod.git
    cd conrod
    cargo build

And then build and run the examples like this:

    cargo run --example all_widgets
    cargo run --example canvas

## Dependencies

![dependencies](./assets/images/Cargo.png)
