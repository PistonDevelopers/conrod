//! A type used to manage a user's image data and map them to `Image` widgets:
//!
//! - [Map](./struct.Map.html)

use std;
use fnv;

/// Unique image identifier.
///
/// Throughout conrod, images are referred to via their unique `Id`. By referring to images via
/// `Id`s, conrod can remain agnostic of the actual image or texture types used to represent each
/// image.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Id(u32);

/// A type used to map the `widget::Id` of `Image` widgets to their associated `Img` data.
///
/// The `image::Map` type is usually instantiated and loaded during the "setup" stage of the
/// application before the main loop begins. A macro is provided to simplify the construction of
/// maps with multiple images.
///
/// ```ignore
/// let image_map = image_map! {
///     (RUST_LOGO, image::open("rust-logo.png")?),
///     (CAT_PIC, image::open("floof.jpeg")?),
/// };
/// ```
pub struct Map<Img> {
    next_index: u32,
    map: HashMap<Img>,
    /// Whether or not the `image::Map` will trigger a redraw the next time `Ui::draw` is called.
    ///
    /// This is automatically set to `true` when any method that takes `&mut self` is called.
    pub trigger_redraw: std::cell::Cell<bool>,
}

/// The type of `std::collections::HashMap` with `fnv::FnvHasher` used within the `image::Map`.
pub type HashMap<Img> = fnv::FnvHashMap<Id, Img>;

/// An iterator yielding an `Id` for each new `Img` inserted into the `Map` via the `extend`
/// method.
pub struct NewIds {
    index_range: std::ops::Range<u32>,
}


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
            next_index: 0,
            map: HashMap::<Img>::default(),
            trigger_redraw: std::cell::Cell::new(true),
        }
    }

    // Calling any of the following methods will trigger a redraw when using `Ui::draw_if_changed`.

    /// Uniquely borrow the `Img` associated with the given widget.
    ///
    /// Note: Calling this will trigger a redraw the next time `Ui::draw_if_changed` is called.
    pub fn get_mut(&mut self, id: Id) -> Option<&mut Img> {
        self.trigger_redraw.set(true);
        self.map.get_mut(&id)
    }

    /// Inserts the given image into the map, returning its associated `image::Id`. The user *must*
    /// store the returned `image::Id` in order to use, modify or remove the inserted image.
    ///
    /// Note: Calling this will trigger a redraw the next time `Ui::draw_if_changed` is called.
    pub fn insert(&mut self, img: Img) -> Id {
        self.trigger_redraw.set(true);
        let index = self.next_index;
        self.next_index = index.wrapping_add(1);
        let id = Id(index);
        self.map.insert(id, img);
        id
    }

    /// Replaces the given image in the map if it exists. Returns the image or None.
    ///
    /// Note: Calling this will trigger a redraw the next time `Ui::draw_if_changed` is called.
    pub fn replace(&mut self, id: Id, img: Img) -> Option<Img> {
        self.trigger_redraw.set(true);
        self.map.insert(id, img)
    }

    /// Removes the given image from the map if it exists. Returns the image or None.
    ///
    /// Any future use of the given `image::Id` will be invalid.
    ///
    /// Note: Calling this will trigger a redraw the next time `Ui::draw_if_changed` is called.
    pub fn remove(&mut self, id: Id) -> Option<Img> {
        self.trigger_redraw.set(true);
        self.map.remove(&id)
    }

    /// Insert each of the images yielded by the given iterator and produce an iterator yielding
    /// their generated `Ids` in the same order.
    ///
    /// Note: Calling this will trigger a redraw the next time `Ui::draw_if_changed` is called.
    pub fn extend<I>(&mut self, images: I) -> NewIds
        where I: IntoIterator<Item=Img>,
    {
        self.trigger_redraw.set(true);
        let start_index = self.next_index;
        let mut end_index = start_index;
        for image in images {
            self.map.insert(Id(end_index), image);
            end_index += 1;
        }
        NewIds { index_range: start_index..end_index }
    }

}

impl Iterator for NewIds {
    type Item = Id;
    fn next(&mut self) -> Option<Self::Item> {
        self.index_range.next().map(|i| Id(i))
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(self.len()))
    }
}

impl ExactSizeIterator for NewIds {
    fn len(&self) -> usize {
        self.index_range.len()
    }
}
