Conrod
======

A graph based, immediate mode, user interface with Boolean predicate search and flexible panel system written in Rust.

Here's an example of the c++ implementation of the nodal UI.

http://www.youtube.com/watch?v=BK0f3CicNR4

The plan is not to port all of it directly, but to closesly draw inspiration while implementing a much more powerful, searchable graph/node system.

Current State
-------------

Currently in the middle of designing the widget framework and basic widgets. An example project will be kept up-to-date featuring demonstrations of all working widgets. Conrod is still in very early stages and may change a lot!

Dependencies
------------

At the moment, Conrod uses [freetype-rs](https://github.com/PistonDevelopers/freetype-rs) for its font rendering, which means you'll need to have the freetype library installed on your system. You can [download and install the freetype library here] (http://www.freetype.org/download.html).


![NodalUI Demo](https://raw.githubusercontent.com/PistonDevelopers/conrod/master/nodalUIdemo.png)

# Goals

* Graph based navigation
* Boolean predicate search
* Simple immediate-mode user implementation
