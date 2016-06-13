//! This module provides the `texture::Id` and `texture::Map` types.
//!
//! `Image` widgets are instantiated with a `texture::Id`, which can be later used to retrieve the
//! necessary `Texture` from the `texture::Map`.
//!
//! Conrod requires no trait bounds on the `texture::Map`'s `T` type in order to not constrain the
//! kinds of textures that the user can use in any way. `texture::Map` is solely a helper struct
//! to simplify the creation and mapping of `texture::Id`s to their respective texture `T`.

use std;

/// A unique identifier that represents some texture.
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Id(usize);

/// Mappings from `texture::Id`s to some texture type `T`.
pub struct Map<T> {
    next_index: usize,
    map: std::collections::HashMap<Id, T>,
}


impl Id {
    /// Returns the inner `usize` from the `Id`.
    pub fn index(self) -> usize {
        self.0
    }
}

impl<T> Map<T> {

    /// Construct the new, empty `Map`.
    pub fn new() -> Self {
        Map {
            next_index: 0,
            map: std::collections::HashMap::new(),
        }
    }

    /// Borrow the `rusttype::Font` associated with the given `font::Id`.
    pub fn get(&self, id: Id) -> Option<&T> {
        self.map.get(&id)
    }

    /// Borrow the `rusttype::Font` associated with the given `font::Id`.
    pub fn get_mut(&mut self, id: Id) -> Option<&mut T> {
        self.map.get_mut(&id)
    }

    /// Adds the given `rusttype::Font` to the `Map` and returns a unique `Id` for it.
    pub fn insert(&mut self, texture: T) -> Id {
        let index = self.next_index;
        self.next_index = index.wrapping_add(1);
        let id = Id(index);
        self.map.insert(id, texture);
        id
    }

}
