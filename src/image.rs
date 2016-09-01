//! A type used to manage a user's image data and map them to `Image` widgets:
//!
//! - [Map](./struct.Map.html)

use std;
use widget;

/// A type used to map the `widget::Id` of `Image` widgets to their associated `Img` data.
///
/// The `image::Map` type is usually instantiated and loaded during the "setup" stage of the
/// application before the main loop begins. A macro is provided to simplify the construction of
/// maps with multiple images.
///
/// ```ignore
/// let image_map = image_map! {
///     (RUST_LOGO, try!(image::open("rust-logo.png"))),
///     (CAT_PIC, try!(image::open("floof.jpeg"))),
/// };
/// ```
pub struct Map<Img> {
    map: HashMap<Img>,
    /// Whether or not the `image::Map` will trigger a redraw the next time `Ui::draw` is called.
    ///
    /// This is automatically set to `true` when any method that takes `&mut self` is called.
    pub trigger_redraw: std::cell::Cell<bool>,
}

/// The type of `std::collections::HashMap` used within the `image::Map`.
pub type HashMap<Img> = std::collections::HashMap<widget::Id, Img>;


impl<Img> std::ops::Deref for Map<Img> {
    type Target = HashMap<Img>;
    fn deref(&self) -> &Self::Target {
        &self.map
    }
}


impl<Img> Map<Img> {

    /// Construct a new, empty `image::Map`.
    pub fn new() -> Self {
        Map {
            map: std::collections::HashMap::new(),
            trigger_redraw: std::cell::Cell::new(true),
        }
    }


    // Calling any of the following methods will trigger a redraw when using `Ui::draw_if_changed`.


    /// Uniquely borrow the `Img` associated with the given widget.
    ///
    /// Note: Calling this will trigger a redraw the next time `Ui::draw_if_changed` is called.
    pub fn get_mut(&mut self, id: widget::Id) -> Option<&mut Img> {
        self.trigger_redraw.set(true);
        self.map.get_mut(&id)
    }

    /// Inserts the given widget-image pair into the map.
    ///
    /// If the map did not already have an image associated with this widget, `None` is returned.
    ///
    /// If the map did already have an image associated with this widget, the old value is removed
    /// from the map and returned.
    ///
    /// Note: Calling this will trigger a redraw the next time `Ui::draw_if_changed` is called.
    pub fn insert(&mut self, id: widget::Id, img: Img) -> Option<Img> {
        self.trigger_redraw.set(true);
        self.map.insert(id, img)
    }

}

impl<Img> std::iter::Extend<(widget::Id, Img)> for Map<Img> {
    fn extend<I>(&mut self, mappings: I)
        where I: IntoIterator<Item=(widget::Id, Img)>,
    {
        self.trigger_redraw.set(true);
        self.map.extend(mappings.into_iter().map(|(id, img)| (id, img)));
    }
}


/// A macro for simplifying the instantiation of an `image::Map`.
///
/// See the [**Map**](./image/struct.Map.html) documentation for an example.
#[macro_export]
macro_rules! image_map {
    ($(($idx:expr, $img:expr)),* $(,)*) => {{
        let mut map = $crate::image::Map::new();
        $(
            map.insert($idx, $img);
        )*
        map
    }};
}
