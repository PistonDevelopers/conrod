/*!

**Getting Started**


In this chapter we'll make sure that your environment is setup correctly for using Conrod.


## Installing Rust and Cargo

Conrod is a Rust library (aka crate), so you'll need Rust! Conrod tracks the stable branch, so you
can be assured that we'll always be compatible with the latest stable version of rustc.

We also rely on the Rust package manager [Cargo](https://crates.io/) for managing dependencies
and hosting the latest version of conrod.

The easiest way to acquire both of these is by downloading the Rust installer from [the Rust
homepage][rust-lang]. This installer will install the latest stable version of both rustc and
cargo.

Once installed, you can test that rustc and cargo are working by entering `rustc --version` and
`cargo --version` into your command line.

If you're brand new to Rust, we recommend first checking out [The Official Rust Book], or at least
keeping it on hand as a reference. It also contains a [Getting Started][rust getting started] guide
with more details on installing Rust, which may be useful in the case that you run into any issues
with the above steps.


## Running the Conrod Examples

We can test that everything is working by cloning the github repository and running the examples.
First, open up the command line on your system and follow these steps:

1. Clone the repo

  ```txt
  git clone https://github.com/PistonDevelopers/conrod.git
  ```

2. Change to the `conrod` directory that we just cloned

  ```txt
  cd conrod
  ```

3. Test that conrod builds without problems

  ```txt
  cargo build --release
  ```

4. Change to the `conrod_glium` backend directory (it has the most examples)

  ```txt
  cd backends/conrod_glium
  ```

5. Build and run the examples (with --release optimisations turned on)!

  ```txt
  cargo run --release --example all_winit_glium
  cargo run --release --example canvas
  cargo run --release --example primitives
  cargo run --release --example text
  ```

If you ran into any issues with these steps, please let us know by filing an issue at the Conrod
[issue tracker]. Be sure to search for your issue first, as another user may have already
encountered your problem.

Otherwise, you're now ready to use conrod!

[rust-lang]:                https://www.rust-lang.org/                          "The Rust Homepage"
[The Official Rust Book]:   https://doc.rust-lang.org/book/                     "The Official Rust Book"
[rust getting started]:     https://doc.rust-lang.org/book/getting-started.html "Getting Started with Rust"
[issue tracker]:            https://github.com/PistonDevelopers/conrod/issues   "Conrod issue tracker"

*/
