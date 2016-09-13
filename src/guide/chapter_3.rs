/*!

**Let's Create a GUI**

## Setting Up Our Project

The first thing we need to do is get a project set up for our `conrod` application.
You will need to have `rustc` and `cargo` installed, please refer to the previous
chapter if you do not have them.

First, create a new project with `cargo`, like this:

    $ cargo new conrod-app --bin

This will setup a project named `conrod-app`, that is configured to create an
executable by default. If we didn't pass `--bin`, it would create a library instead.

If you `cd` into `conrod-app`, you will see we have a `Cargo.toml` file, and a `src`
directory. The `Cargo.toml` is the way that we configure how `cargo` behaves for our
application. It is the place we can specify build artifacts, dependencies, etc. Let's
open it up and take a look. Yours should be similar to the following:

```toml
[package]
name = "conrod-app"
version = "0.1.0"
authors = ["Paul Woolcock <paul@woolcock.us>"]

[dependencies]
```

Of course, your `authors` line will be different :).

Now, let's add a couple dependencies so that we can actually start building our application!
For now, all we need is the `conrod` and `piston_window` libraries, so let's tell `cargo`
about them. Your `[dependencies]` section should look like this:

```toml
[dependencies]
conrod = "*"
piston_window = "*"
```

In the future we will want to specify actually version numbers instead of the `*` you see here,
but we can do that later.

Now, save and close the `Cargo.toml` file, and let's run our application!

In your `conrod-app` directory, run the following command:

```bash
$ cargo run
```

It will probably take a while the very first time you do this, but don't worry, it won't take
that long every time.

When it finishes compiling, you should see output like this:

```bash
$ cargo run
   Compiling conrod-app v0.1.0 (file:///home/paul/code/conrod-app)
    Finished debug [unoptimized + debuginfo] target(s) in 0.58 secs
     Running `target/debug/conrod-app`
Hello, world!
```

Congratulations, you just ran your first Rust application! It doesn't do much yet, but we
will take care of that in the next section.

## Setup a Basic Window (using piston_window)

Now that we have our project set up and going, let's get an actual UI running. `conrod` itself
does not actually do the window management & drawing, it leaves that up to a back end that the
programmer can specify. We are going to be using the [piston_window](https://github.com/PistonDevelopers/piston_window)
backend.

First of all, we add the necessary lines to make sure that we can use the functionality in the
`piston_window` library. This means adding a couple lines to our `src/main.rs`:

```rust
// src/main.rs
extern crate piston_window;
use piston_window::*;
```

Great! Now we are linking the `piston_window` library into our application, and importing symbols
from its prelude so that we can use them.

Next, we'll construct a window:

```rust
// ...

fn main() {
    let mut window: PistonWindow = WindowSettings::new()
        .exit_on_esc(true)
        .build()
        .unwrap_or_else(|e| panic!("failed to build window: {0}", e));
}
```

Here, you see we are using a few of the symbols that have been imported from our
`use piston_window::*` declaration, namely `PistonWindow` and `WindowSettings`. However,
this isn't going to do much except create a window and immediately exit the program.

So, how do we actually get this to do something? `piston_window` has a concept of events
that come from the `PistonWindow`, and we can set up a loop to listen for, and respond to,
those events. Let's do that:

```rust
// ...

fn main() {
    // ...

    while let Some(event) = window.next() {
        window.draw_2d(&event, |_c, g| {
            clear([0.5, 1.0, 0.5, 1.0], g);
        });
    }
}
```

*Now*, if you compile & run the program, you should see a window come up & display a pleasant
green-ish color. Congrats, you now have a UI that we can start to draw things onto!

## Conrod Setup

## Instantiating Widgets

## Widget Positioning and Layout

*/
