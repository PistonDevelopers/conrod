# Conrod [![Build Status](https://travis-ci.org/PistonDevelopers/conrod.svg?branch=master)](https://travis-ci.org/PistonDevelopers/conrod)

Graph based, immediate mode, user interface with fancy search capabilities written in Rust.

Here's a demo!

[https://www.youtube.com/watch?v=n2UrjogA0j0](https://www.youtube.com/watch?v=n2UrjogA0j0)

Here's an example of the "Nodal" style we're going for. This particular vid is of an old C++ implementation.

[http://www.youtube.com/watch?v=BK0f3CicNR4](http://www.youtube.com/watch?v=BK0f3CicNR4)

The plan is not to port all of it directly, but to closesly draw inspiration while implementing a much more powerful, searchable graph/node system.

![NodalUI Demo](https://raw.githubusercontent.com/PistonDevelopers/conrod/master/nodalUIdemo.png)


Current State
-------------

Currently in the middle of designing the widget framework and basic widgets. An example project will be kept up-to-date featuring demonstrations of all working widgets. Conrod is still in very early stages and may change a lot!

Available Widgets
-----------------

- Buttons
- Toggles
- Envelope Editor
- Number Dialers
- Sliders
- TextBox
- XYPad

Dependencies
------------

- [rustc](http://www.rust-lang.org/) - we try to keep up to date with the latest nightly build.
- [cargo](https://github.com/rust-lang/cargo) - for handling other rust dependencies and building the project.
- [freetype](http://www.freetype.org/download.html) - at the moment, Conrod uses [freetype-rs](https://github.com/PistonDevelopers/freetype-rs) for its font rendering, which means you'll need to have the freetype library installed on your system. You can [download and install the freetype library here](http://www.freetype.org/download.html).


Usage
-----

Build the conrod lib like this:

    git clone https://github.com/PistonDevelopers/conrod.git
    cd conrod
    cargo build

And then build and run the examples like this:

    cd examples/all_widgets
    cargo build
    ./target/all_widgets


Goals
-----

* Graph based navigation
* Boolean predicate search
* Simple immediate-mode user implementation

