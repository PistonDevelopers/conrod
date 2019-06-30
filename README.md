# Conrod [![Build Status](https://travis-ci.org/PistonDevelopers/conrod.svg?branch=master)](https://travis-ci.org/PistonDevelopers/conrod) [![Crates.io](https://img.shields.io/crates/l/conrod_core.svg)](https://github.com/PistonDevelopers/conrod/blob/master/LICENSE)
[![Build Status](https://dev.azure.com/K1720055F/conrod/_apis/build/status/alanpoon.conrod%20(1)?branchName=master)](https://dev.azure.com/K1720055F/conrod/_build/latest?definitionId=3&branchName=master)
An easy-to-use, 2D GUI library written entirely in Rust.

Guide
-----

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
    - [The Builder Pattern][1.6]
2. [**Getting Started**][2]
    - [Installing Rust and Cargo][2.1]
    - [Running the Conrod Examples][2.2]
3. **Let's Create a GUI**
    - Setup a Basic Window (using piston_window)
    - Conrod Setup
    - Instantiating Widgets
    - Widget Positioning and Layout
4. **Using and Customising Themes**
    - What is a `Theme`?
    - Custom Themes
    - Serializing Themes
5. **Designing Custom Widgets (using the Widget trait)**
    - The `Widget` trait
    - The `widget_style!` macro
    - The `builder_methods!` macro
    - Making a `Button` widget
6. **Custom Graphics and Window Backends**
    - Demonstration of Backend Implementation (using glium and glutin)
7. **Internals**
    - The `Ui`'s Widget `Graph`
    - `Ui::set_widgets` - How does it work?
8. **FAQ**

*The Guide is a work-in-progress. If a section is not linked, it is likely not yet implemented.*


Crates
------

| Crate | Badges | Description |
| --- | --- | --- |
| **`conrod_core`** | [![Crates.io](https://img.shields.io/crates/v/conrod_core.svg)](https://crates.io/crates/conrod_core) [![docs.rs](https://docs.rs/conrod_core/badge.svg)](https://docs.rs/conrod_core/) | The fundamentals for any conrod project. |
| **`conrod_derive`** | [![Crates.io](https://img.shields.io/crates/v/conrod_derive.svg)](https://crates.io/crates/conrod_derive) [![docs.rs](https://docs.rs/conrod_derive/badge.svg)](https://docs.rs/conrod_derive/) | Provides the `WidgetCommon` and `WidgetStyle` derive macros. |
| **`conrod_winit`** | [![Crates.io](https://img.shields.io/crates/v/conrod_winit.svg)](https://crates.io/crates/conrod_winit) [![docs.rs](https://docs.rs/conrod_winit/badge.svg)](https://docs.rs/conrod_winit/) | Simplifies using `conrod_core` with `winit` |
| **`conrod_gfx`** | [![Crates.io](https://img.shields.io/crates/v/conrod_gfx.svg)](https://crates.io/crates/conrod_gfx) [![docs.rs](https://docs.rs/conrod_gfx/badge.svg)](https://docs.rs/conrod_gfx/) | Simplifies using `conrod_core` with the gfx ecosystem |
| **`conrod_glium`** | [![Crates.io](https://img.shields.io/crates/v/conrod_glium.svg)](https://crates.io/crates/conrod_glium) [![docs.rs](https://docs.rs/conrod_glium/badge.svg)](https://docs.rs/conrod_glium/) | Simplifies using `conrod_core` with `glium` |
| **`conrod_piston`** | [![Crates.io](https://img.shields.io/crates/v/conrod_piston.svg)](https://crates.io/crates/conrod_piston) [![docs.rs](https://docs.rs/conrod_piston/badge.svg)](https://docs.rs/conrod_piston/) | Simplifies using `conrod_core` with `piston` |
| **`conrod_vulkano`** | [![Crates.io](https://img.shields.io/crates/v/conrod_vulkano.svg)](https://crates.io/crates/conrod_vulkano) [![docs.rs](https://docs.rs/conrod_vulkano/badge.svg)](https://docs.rs/conrod_vulkano/) | Simplifies using `conrod_core` with `vulkano` |


Current State
-------------

We're just starting to reach a stable-ish API pattern! There will still be some
large changes, however these are more likely to be new features than API
overhauls.

To get a clearer idea of where we're at see the [issues] and in particular, the
[1.0.0 milestone].


Contributing
------------

Want to help out? See [Piston's how to contribute guide][Contributing].


License
-------

Licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.


**Contributions**

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.

**Example Assets**

- [Google Noto](https://www.google.com/get/noto/) (Apache2)


[The API Documentation]: https://docs.rs/conrod/
[The Guide]: https://docs.rs/conrod/latest/conrod/guide/index.html

[1]:        https://docs.rs/conrod/latest/conrod/guide/chapter_1/index.html
[1.1]:      https://docs.rs/conrod/latest/conrod/guide/chapter_1/index.html#a-brief-history
[1.2]:      https://docs.rs/conrod/latest/conrod/guide/chapter_1/index.html#screenshots-and-videos
[1.3]:      https://docs.rs/conrod/latest/conrod/guide/chapter_1/index.html#feature-overview
[1.4]:      https://docs.rs/conrod/latest/conrod/guide/chapter_1/index.html#available-widgets
[1.4.1]:    https://docs.rs/conrod/latest/conrod/guide/chapter_1/index.html#primitive-widgets
[1.4.2]:    https://docs.rs/conrod/latest/conrod/guide/chapter_1/index.html#common-use-widgets
[1.5]:      https://docs.rs/conrod/latest/conrod/guide/chapter_1/index.html#immediate-mode
[1.5.1]:    https://docs.rs/conrod/latest/conrod/guide/chapter_1/index.html#what-is-it
[1.5.2]:    https://docs.rs/conrod/latest/conrod/guide/chapter_1/index.html#why-use-it
[1.5.3]:    https://docs.rs/conrod/latest/conrod/guide/chapter_1/index.html#is-conrod-immediate-or-retained
[1.6]:      https://docs.rs/conrod/latest/conrod/guide/chapter_1/index.html#the-builder-pattern
[2]:        https://docs.rs/conrod/latest/conrod/guide/chapter_2/index.html
[2.1]:      https://docs.rs/conrod/latest/conrod/guide/chapter_2/index.html#installing-rust-and-cargo
[2.2]:      https://docs.rs/conrod/latest/conrod/guide/chapter_2/index.html#running-the-conrod-examples

[issues]: https://github.com/PistonDevelopers/conrod/issues
[1.0.0 milestone]: https://github.com/PistonDevelopers/conrod/milestones/1.0.0

[Contributing]: https://github.com/PistonDevelopers/piston/blob/master/CONTRIBUTING.md
