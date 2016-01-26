# Conrod [![Build Status](https://travis-ci.org/PistonDevelopers/conrod.svg?branch=master)](https://travis-ci.org/PistonDevelopers/conrod) [![Crates.io](https://img.shields.io/crates/v/conrod.svg)](https://crates.io/crates/conrod) [![Crates.io](https://img.shields.io/crates/l/conrod.svg)](https://github.com/PistonDevelopers/conrod/blob/master/LICENSE)

An easy-to-use, 2D GUI library written entirely in Rust.

**[The API Documentation]**.

**[The Guide]**:

1. [**What is Conrod?**][1]
    - [A Brief Summary][1.1]
    - [Screenshots and Videos][1.2]
    - [Feature Overview][1.3]
    - [Available Widgets][1.4]
        - [Primitive Widgets][1.4.1]
        - [Common Use Widgets][1.4.2]
    - [Immediate Mode][1.5]
        - [What is it?][1.5.1]
        - [Why use it?][1.5.2]
        - [Is Conrod Immediate or Retained?][1.5.3]


Current State
-------------

We're just starting to reach a stable-ish API pattern! There will still be some large changes however these are more likely to be new features than API overhauls.

To get a clearer idea of where we're at see the [issues] and in particular, the [1.0.0 milestone].


Getting Started
---------------

[Get freetype][freetype download] - at the moment, Conrod uses [freetype-rs] for its font rendering, which means you'll need to have the freetype library installed on your system. You can [download and install the freetype library here][freetype download].


Build the conrod lib like this:

```
git clone https://github.com/PistonDevelopers/conrod.git
cd conrod
cargo build
```

And then build and run the examples like this:

```
cargo run --release --example all_widgets
cargo run --release --example canvas
```

You can add it to your project by adding this to your Cargo.toml:

```toml
[dependencies]
conrod = "X.Y.Z"
```


Dependency Graph
----------------

![dependencies](./Cargo.png)


Contributing
------------

Want to help out? See [Piston's how to contribute guide][Contributing].


License
-------

[MIT].

[Example assets].


[The API Documentation]: http://docs.piston.rs/conrod/conrod/
[The Guide]: http://docs.piston.rs/conrod/conrod/guide/index.html

[1]:        http://docs.piston.rs/conrod/conrod/guide/chapter_1/index.html
[1.1]:      http://docs.piston.rs/conrod/conrod/guide/chapter_1/index.html#a-brief-history
[1.2]:      http://docs.piston.rs/conrod/conrod/guide/chapter_1/index.html#screenshots-and-videos
[1.3]:      http://docs.piston.rs/conrod/conrod/guide/chapter_1/index.html#feature-overview
[1.4]:      http://docs.piston.rs/conrod/conrod/guide/chapter_1/index.html#available-widgets
[1.4.1]:    http://docs.piston.rs/conrod/conrod/guide/chapter_1/index.html#primitive-widgets
[1.4.2]:    http://docs.piston.rs/conrod/conrod/guide/chapter_1/index.html#common-use-widgets
[1.5]:      http://docs.piston.rs/conrod/conrod/guide/chapter_1/index.html#immediate-mode
[1.5.1]:    http://docs.piston.rs/conrod/conrod/guide/chapter_1/index.html#what-is-it
[1.5.2]:    http://docs.piston.rs/conrod/conrod/guide/chapter_1/index.html#why-use-it
[1.5.3]:    http://docs.piston.rs/conrod/conrod/guide/chapter_1/index.html#is-conrod-immediate-or-retained

[issues]: https://github.com/PistonDevelopers/conrod/issues
[1.0.0 milestone]: https://github.com/PistonDevelopers/conrod/milestones/1.0.0

[freetype download]: http://www.freetype.org/download.html
[freetype-rs]: https://github.com/PistonDevelopers/freetype-rs

[Contributing]: https://github.com/PistonDevelopers/piston/blob/master/CONTRIBUTING.md

[MIT]: https://github.com/PistonDevelopers/conrod/blob/master/LICENSE
[Example assets]: https://github.com/PistonDevelopers/conrod/issues/319
