/*!

**The Conrod Guide**

## Table of Contents

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
3. [**Hello World**][3]
    - [Creating a new project][3.1]
    - [Conrod Setup][3.2]
        -[Back ends][3.2.1]
    - [Creating a Window][3.3]
    - [Event Handling][3.4]
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


[1]:        ./chapter_1/index.html
[1.1]:      ./chapter_1/index.html#a-brief-history
[1.2]:      ./chapter_1/index.html#screenshots-and-videos
[1.3]:      ./chapter_1/index.html#feature-overview
[1.4]:      ./chapter_1/index.html#available-widgets
[1.4.1]:    ./chapter_1/index.html#primitive-widgets
[1.4.2]:    ./chapter_1/index.html#common-use-widgets
[1.5]:      ./chapter_1/index.html#immediate-mode
[1.5.1]:    ./chapter_1/index.html#what-is-it
[1.5.2]:    ./chapter_1/index.html#why-use-it
[1.5.3]:    ./chapter_1/index.html#is-conrod-immediate-or-retained
[1.6]:      ./chapter_1/index.html#the-builder-pattern
[2]:        ./chapter_2/index.html
[2.1]:      ./chapter_2/index.html#installing-rust-and-cargo
[2.2]:      ./chapter_2/index.html#running-the-conrod-examples
[3]:        ./chapter_3/index.html
[3.1]:      ./chapter_3/index.html#creating-a-new-project
[3.2]:      ./chapter_3/index.html#setting-up-conrod
[3.2.1]:    ./chapter_3/index.html#backends
[3.3]:      ./chapter_3/index.html#creating-a-window
[3.4]:      ./chapter_3/index.html#handling-events


*/

pub mod chapter_1;
pub mod chapter_2;
pub mod chapter_3;
