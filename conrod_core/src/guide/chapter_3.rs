/*!
**Setting up a basic window using glium and winit**

In this chapter we'll get a "hello world" app up and running. "Hello World" is
the traditional greeting from the first program in a new language or
environment. In this case, it'll just do exactly that: open a window that says
"Hello World", but it'll introduce most of the key concepts behind Conrod.

## Creating a New Project

Use the command line or terminal to go to a directory that suits you, for
example on a mac: /Users/bob/Projects, or perhaps D:\Projects on a PC, and use
cargo to create a new project. I've called the project conrod_hw for hello
world, you can use any name. Type at the command line:

```txt
cargo new conrod_hw --bin
```
`--bin` tells cargo to build an executable project.

 If you now go into the project and run it with:

```txt
cd conrod_hw cargo run
```

you'll probably see a fairly long and slow list of rust files being downloaded
and compiled, then something like:

```txt
   Compiling conrod_hw v0.1.0
   (file:///Users/declan/Projects/rust/conrod_hw)
   Finished dev [unoptimized + debuginfo] target(s) in 0.34 secs
   Running `target/debug/conrod_hw`
   Hello, world!
```
If you don't see these, you've got a problem with your Rust installation.
Return here once it is sorted out!

Cargo builds a default "Hello World" command line app when you ask it to
create a new --bin project. In future, most of the crates that you've just
compiled won't need recompilation, and things will happen more quickly.

## Setting up Cargo for Conrod

To use Conrod in your application, you need to tell your program to use it,
and you need to tell Cargo to make it available to your programme. The program
itself is in the `src` folder of your project, in the file called `main.rs`,
and should look like this right now:

```ignore
fn main() {
    println!("Hello, world!");
}
```

To allow your programme to use conrod, you need to add a reference to the
conrod crate, with:

```ignore
#[macro_use] extern crate conrod;

fn main() {
    println!("Hello, world!");
}
```

If you try to run the above code, you'll get an error like:

```txt
error[E0463]: can't find crate for "conrod"

 --> src/main.rs:1:1

error: aborting due to previous error(s)
```

Rust doesn't know how to deal with the crate that your programme has called.
It needs to be added to the manifest for the project, in the  `cargo.toml`
file, which is in the root of the project. The file currently contains
something like:

```toml
[package]
name = "conrod_hw"
version = "0.1.0"
authors = ["bobgates"]

[dependencies]
```

Add `conrod = { version = "0.55.0", features = ["glium", "winit"] }` to the
`[dependencies]` section.

This incantation needs a little explanation, because it specifies the
backends.

### Backends

Conrod is designed as an IMGUI that can use a number of backends. It was
initially developed to support PistonWindow, but then had backends added for
the winit event crate, and the glium and gfx graphics backend crates. Lately,
support for the SDL2 crate has also started to be added. The backends are
handled by conditional compilation in the conrod code. For simplicity, I'm
specifying the backend in `cargo.toml` because we then won't have to think
about it in the code itself. Alternatively, the application can be run using
cargo "features" as described in the previous chapter.

```txt
cargo run --release --features "winit glium" --example all_winit_glium
```

At the time of writing (September 2017), the winit and glium backend
combination seems to be the best supported, and its the one that I am familiar
with.

When the program compiles, we're ready to build a UI (there will be  a
warning about macro use, but we'll be using macros soon).

## Creating a Window

There is quite a lot to do for even an empty window, so let's get to it.

I'm using the glium backend. [Glium](https://github.com/glium/glium) is a
cross-platform, safe wrapper for OpenGL, written in Rust. It is listed as not-
maintained on Github, but limited maintenance is being undertaken by the
developer of Conrod. Glium has a very good, gentle
[introduction](http://glium.github.io/glium/book/) to OpenGL - enough to
convince me that I really want Conrod to do the work for me!

Conrod makes Glium available through an export. Put the following line into
the code, near the top, to include it:

```ignore
use conrod::backend::glium::glium::{self, Surface};
```

`Surface` is a trait required for glium, specifically for the call to
`target.clear_color` which is coming later.

The first chunk of boilerplate creates an event loop, which will handle
interaction with the UI, then a window, then a context, then finally links the
event loop, window and context together into a display. The display is the
home for the UI, and is an OpenGL context provided by glium.

```ignore
const WIDTH: u32 = 400;
const HEIGHT: u32 = 200;

let mut events_loop = glium::glutin::EventsLoop::new();
let window = glium::glutin::WindowBuilder::new()
                .with_title("Hello Conrod")
                .with_dimensions(WIDTH, HEIGHT);
let context = glium::glutin::ContextBuilder::new()
                .with_vsync(true)
                .with_multisampling(4);
let display = glium::Display::new(window, context, &events_loop).unwrap();
```

Now create the UI itself. Conrod has a builder that contains and looks after
the UI for the user.

```ignore
let mut ui = conrod::UiBuilder::new([WIDTH as f64, HEIGHT as f64]).build();
```


Conrod can use graphics. It stores these in a map. The system needs the map,
even though it doesn't contain anything at this time, so create it:

```ignore
let image_map = conrod::image::Map::<glium::texture::Texture2d>::new();
```

Finally, Conrod needs to render its UI. It uses a renderer to do this, so
create one:

```ignore
let mut renderer = conrod::backend::glium::Renderer::new(&display).unwrap();
```

As an Immediate Mode GUI, Conrod sits in the main loop of the program, drawing
the UI every time round the loop. Here's the main loop:

```ignore
'main: loop {
    // Render the `Ui` and then display it on the screen.
    if let Some(primitives) = ui.draw_if_changed() {
        renderer.fill(&display, primitives, &image_map);
        let mut target = display.draw();
        target.clear_color(0.0, 1.0, 0.0, 1.0);
        renderer.draw(&display, &mut target, &image_map).unwrap();
        target.finish().unwrap();
    }
}
```

Right now, the program really doesn't do much. This is the minimum set of bits
and pieces to get a window up on the screen. The UI has nothing in it, so the
call to `ui.draw_if_changed()` has nothing to do. The UI remembers whether
there is a need to draw itself, so it does not draw on every cycle of the main
loop, only if things have changed.

`target = display.draw()` starts drawing on the backbuffer. It returns a
Frame, which the renderer can draw everything to. The first command to
`target` is to `clear_color`. This takes fractional red, green and blue values
between 0.0 and 1.0, and an alpha value, also between 0.0 and 1.0 and fills
the frame, in this case with bright green. The renderer is then asked to draw
the rest of the UI (ie nothing) to the frame, and finally, `target`, the
frame, is told to finish. It stops the drawing, swaps the OpenGL buffers
(putting the just drawn frame on the screen) and consumes the frame. The frame
docs are part of glium
[here](https://docs.rs/glium/0.17.1/glium/struct.Frame.html).


If you've run this program, you'll have seen the single least useful output
ever - a green block with a white "menu bar" that has nothing in it. Let's add
at least something to see.

After
```ignore
const WIDTH: i32 = 400;
const HEIGHT: i32 = 640;
let mut ui = conrod::UiBuilder::new([WIDTH as f64, HEIGHT as f64]).build();
```

add the code that will initiate and build the UI:

```ignore
widget_ids!(struct Ids { text });
let ids = Ids::new(ui.widget_id_generator());
```

`widget_ids!` is, as its name suggests, a macro provided by conrod to create
widget ids. Conrod's primary data structure contains all the widgets, and uses
their ids to keep track of them. The `widget_ids!` macro just provides a very
quick and easy way of defining widget names (in this case just `text`) and
giving them ids.

After defining the widget_ids, `Ids::new` creates the the widget structure.

To instantiate the widgets, insert the following code in the main loop, before
the rendering statement.

```ignore
let ui = &mut ui.set_widgets();

// "Hello World!" in the middle of the screen.
widget::Text::new("Hello World!")
    .middle_of(ui.window)
    .color(conrod::color::WHITE)
    .font_size(32)
    .set(ids.text, ui);
```

In the definition of the Ids struct above, only one widget was named: `text`.
Here, first we instantiate all the widgets, using the `ui.set_widgets()` call.
Then we write code to create each widget of the application. There is only one
widget in this application - it's a text widget, showing the text "Hello
World!". It is defined to be positioned in the middle of the `ui.window`, with
text color of white, and font size of 32. Finally, the widget is bound to the
UI, using `set(ids.text, ui)`

The UI code will now ensure that events are passed back from the UI, linked
through the id, `ids.text`. A text widget doesn't respond to events, so
there's nothing more to do right now.

Except that if you compile the code right now, it fails with

```txt
error[E0599]: no method named 'middle_of' found for type
conrod::widget::Text<'_>' in the current scope

  --> src/main.rs:89:18`

   = help: items from traits can only be used if the trait is in scope`

   = note: the following trait is implemented but not in scope, perhaps add a 'use' for it:`
           candidate #1: 'use conrod::Positionable;'
```


`middle_of` is a Conrod trait, of the type `Positionable` and color is also a
trait of type 'Colorable'. To ensure that the program can find the trait, we
need to add to the `use conrod` line, as suggested in the error message. We
also need to define `widget' and the type of Widget, so add the following near
the top of the programme:
```ignore
use conrod::{widget, Positionable, Colorable, Widget};
```

There's one more thing we need to do. Conrod needs to load fonts before it can
use them. Some boilerplate in the early part of the programme can do this:

```ignore
let assets = find_folder::Search::KidsThenParents(3, 5)
    .for_folder("assets")
    .unwrap();
let font_path = assets.join("fonts/NotoSans/NotoSans-Regular.ttf");
ui.fonts.insert_from_file(font_path).unwrap();
```

The code uses the find_folder crate. Modify the Cargo.toml file for this
project by adding find_folder to the dependencies. Its current version at time
of writing is 0.3.0:

```toml
[dependencies]
conrod = {
    version = "0.55.0",
    features = ["glium", "winit"]
}
find_folder="0.3.0"
```

and reference the crate at the beginning of the file:
```ignore
extern crate find_folder;
```

`find_folder` and some Rust path handling generates the path to the font. It
is in the folder `assets/fonts/NotoSans` and is called `NotoSans-Regular.ttf`.
NotoSans is as font designed by
[Google](https://fonts.google.com/specimen/Noto+Sans), and has an Apache
License. It is available [here](https://noto-
website.storage.googleapis.com/pkgs/NotoSans-unhinted.zip), so download and
put it in the appropriate folder. Make sure that folder and file names agree
on your computer and in your code.

On my mac, I now see this (it has an empty white header bar that is not
visible on this white web page):

<img src="https://i.imgur.com/ECzjqdp.png">

Not quite what we want, yet.

## Event Handling

The window is not getting all its furniture because its events are not being
handled. At its most simple, events can be handled by fetching them from the
`event_loop` and dispatching them accordingly. Here's the code in the main
loop:

```ignore
let mut events = Vec::new();
events_loop.poll_events(|event| events.push(event));

for event in events{
    match event {
        glium::glutin::Event::WindowEvent { event, ..} => match event {
            glium::glutin::WindowEvent::Closed |
            glium::glutin::WindowEvent::KeyboardInput {
                input: glium::glutin::KeyboardInput {
                    virtual_keycode: Some(glium::glutin::VirtualKeyCode::Escape),
                    ..
                },
                ..
            } => break 'main,
            _ => (),
        },
        _ => (),
    }
}
```

The `glutin::EventsLoop` defined in the first line of code in `main` stores
and returns events in the glutin format. We simply create a queue using a
vector, and push all events onto the queue each time the main loop executes.
Then, a `for` loop runs through the queued events, and executes a break from
the main loop if the event is a WindowEvent::Closed or a
WindowEvent::KeyboardInput of Escape.

But that's enough. Our "Hello World" is finished for now.

<img src="https://i.imgur.com/JgjvuSH.png">

Actually, it isn't. If you open up your performance monitoring app (Activity
Monitor on the Mac), you'll see that this simple app is running, in my case,
at 100% of one CPU. The events loop is braindead - it simply checks to see if
there's an event, acts immediately then tries again. We can be a lot more
subtle, but it is going to take some code. Also, this app doesn't have to
respond to any of its own events, so they're not taken into account.

First, let's add event handling for Conrod. Putting the following at the
beginning of the event for loop will take care of UI events:

```ignore
if let Some(event) = conrod::backend::winit::convert_event(
    event.clone(),
    &display
) {
    ui.handle_event(event);
}
```

`ui.handle_event()` is the business end of Conrod - it takes events off the
queue, works out which widget they apply to and looks after dispatch, etc. The
rest happens elsewhere, as we'll see in the next Chapter.


Then there's some straightforward event boilerplate that most Conrod apps have
somewhere convenient. In this case, I'm going to put it into the same file as
`fn main()'.

```ignore
pub struct EventLoop {
    ui_needs_update: bool,
    last_update: std::time::Instant,
}

impl EventLoop {
    pub fn new() -> Self {
        EventLoop { last_update: std::time::Instant::now(),
                    ui_needs_update: true,
                  }
    }

    /// Produce an iterator yielding all available events.
    pub fn next(&mut self, events_loop: &mut glium::glutin::EventsLoop) ->
                Vec<glium::glutin::Event> {

        // We don't want to loop any faster than 60 FPS, so wait until it has been at least 16ms
        // since the last yield.
        let last_update = self.last_update;
        let sixteen_ms = std::time::Duration::from_millis(16);
        let duration_since_last_update = std::time::Instant::now().duration_since(last_update);
        if duration_since_last_update < sixteen_ms {
            std::thread::sleep(sixteen_ms - duration_since_last_update);
        }

        // Collect all pending events.
        let mut events = Vec::new();
        events_loop.poll_events(|event| events.push(event));

        // If there are no events and the UI does not need updating, wait
        // for the next event.
        if events.is_empty() && !self.ui_needs_update {
            events_loop.run_forever(|event| { events.push(event);
                                    glium::glutin::ControlFlow::Break });
        }

        self.ui_needs_update = false;
        self.last_update = std::time::Instant::now();

        events
    }

    /// Notifies the event loop that the `Ui` requires another update whether
    /// or not there are any pending events.
    ///
    /// This is primarily used on the occasion that some part of the UI is
    /// still animating and requires further updates to do so.
    pub fn needs_update(&mut self) {
        self.ui_needs_update = true;
    }
```

'EventLoop' is a simple structure that stores the time of the last update, and
a boolean that tells it whether it needs to update itself. It is implemented
as an iterator that fetches items from the glutin::EventsLoop and returns them
to the calling code, then goes to sleep for 16 ms, so as to produce about 60
fps. There's also another function that tells the events loop that it needs an
update.

The main events loop can now be changed. Insert a constructor for the event
loop just before the 'main loop

```ignore
let mut event_loop = EventLoop::new();
```

Remove the quick-and-dirty event queue introduced above

```ignore
let mut events = Vec::new();
events_loop.poll_events(|event|events.push(event));
```

and change the event `for` loop to read from the `event_loop` that was
constructed above.

```ignore
for event in event_loop.next(&mut events_loop){
```

`conrod_hw` is now taking 0% of available CPU. Much better!


This chapter has introduced the basic elements of a Conrod application: the
graphics framework, the event loop and the UI, but it hasn't gone beyond that.
It's time to take on something a little more complicated to show how to create
the UI on the fly, and respond to events in our own widgets.

The code produced in this chapter is available on the Conrod Github site
within the examples.


*/
