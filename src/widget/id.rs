//! The widget identifier type used throughout conrod, along with helper types and macros to
//! simplify the process of generating them.

use daggy;
use std;
use UiCell;

/// Unique widget identifier.
///
/// Each widget instance must use its own, uniquely generated `widget::Id` so that it's state can
/// be cached within the `Ui` type.
///
/// Indices are generated consecutively from `0`. This allows us to avoid the need for hashing
/// identifiers in favour of indexing directly into the `Graph`'s underlying node array.
///
/// `widget::Id`s may be generated via the `widget_ids!` macro.
pub type Id = daggy::NodeIndex<u32>;

/// A single lazily generated `widget::Id`.
pub struct Single(std::cell::Cell<Option<Id>>);
/// A list of lazily generated `widget::Id`s.
pub struct List(Vec<Id>);
/// An iterator-like type for producing indices from a `List`.
#[allow(missing_copy_implementations)]
pub struct ListWalk { i: usize }


impl Single {

    /// Construct a cache for a single index.
    pub fn new() -> Self {
        Single(std::cell::Cell::new(None))
    }

    /// Retrieve the `widget::Id`, or generate a new one if one hasn't yet been retrieved.
    pub fn get(&self, ui: &mut UiCell) -> Id {
        match self.0.get() {
            Some(id) => id,
            None => {
                let id = ui.new_unique_widget_id();
                self.0.set(Some(id));
                id
            },
        }
    }

}

impl List {

    /// Construct a cache for multiple indices.
    pub fn new() -> Self {
        List(Vec::new())
    }

    /// Produce a walker for producing the `List`'s indices.
    pub fn walk(&self) -> ListWalk {
        ListWalk { i: 0 }
    }

    /// Resizes the `List`'s inner `Vec` to the given target length, using the given `UiCell` to
    /// generate new unique `widget::Id`s if necessary.
    pub fn resize(&mut self, target_len: usize, ui: &mut UiCell) {
        while self.len() < target_len {
            let new_id = ui.new_unique_widget_id();
            self.0.push(new_id);
        }
        while self.len() > target_len {
            self.0.pop();
        }
    }

}

impl std::ops::Deref for List {
    type Target = [Id];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl ListWalk {

    /// Yield the next index, generating one if it does not yet exist.
    pub fn next(&mut self, &mut List(ref mut ids): &mut List, ui: &mut UiCell) -> Id {
        while self.i >= ids.len() {
            ids.push(ui.new_unique_widget_id());
        }
        let ix = ids[self.i];
        self.i += 1;
        ix
    }

}


/// From
///
/// ```ignore
/// widget_ids_define_struct! {
///     Ids {
///         button,
///         toggles[],
///     }
/// }
/// ```
///
/// this macro generates
///
/// ```ignore
/// struct Ids {
///     button: conrod::widget::id::Single,
///     toggles: conrod::widget::id::List,
/// }
/// ```
#[doc(hidden)]
#[macro_export]
macro_rules! widget_ids_define_struct {

    // Converts `foo[]` tokens to `foo: conrod::widget::id::List`.
    ($Ids:ident { { $($id_field:ident: $T:path,)* } $id:ident[], $($rest:tt)* }) => {
        widget_ids_define_struct! {
            $Ids {
                {
                    $($id_field: $T,)*
                    $id: $crate::widget::id::List,
                }
                $($rest)*
            }
        }
    };

    // Converts `foo` tokens to `foo: conrod::widget::id::Single`.
    ($Ids:ident { { $($id_field:ident: $T:path,)* } $id:ident, $($rest:tt)* }) => {
        widget_ids_define_struct! {
            $Ids {
                {
                    $($id_field: $T,)*
                    $id: $crate::widget::id::Single,
                }
                $($rest)*
            }
        }
    };

    // Same as above but without the trailing comma.
    ($Ids:ident { { $($id_field:ident: $T:path,)* } $id:ident[] }) => {
        widget_ids_define_struct! { $Ids { { $($id_field: $T,)* } $id[], } }
    };
    ($Ids:ident { { $($id_field:ident: $T:path,)* } $id:ident }) => {
        widget_ids_define_struct! { $Ids { { $($id_field: $T,)* } $id, } }
    };

    // Generates the struct using all the `ident: path` combinations generated above.
    ($Ids:ident { { $($id:ident: $T:path,)* } }) => {
        struct $Ids {
            $(
                $id: $T,
            )*
        }
    };

}


/// From
///
/// ```ignore
/// widget_ids_constructor! {
///     Ids {
///         button,
///         toggles[],
///     }
/// }
/// ```
///
/// this macro generates
///
/// ```ignore
/// struct Ids {
///     button: conrod::widget::id::Single::new(),
///     toggles: conrod::widget::id::List::new(),
/// }
/// ```
///
#[doc(hidden)]
#[macro_export]
macro_rules! widget_ids_constructor {

    // Converts `foo[]` to `foo: conrod::widget::id::List::new()`.
    ($Ids:ident { { $($id_field:ident: $new:expr,)* } $id:ident[], $($rest:tt)* }) => {
        widget_ids_constructor! {
            $Ids {
                {
                    $($id_field: $new,)*
                    $id: $crate::widget::id::List::new(),
                }
                $($rest)*
            }
        }
    };

    // Converts `foo` to `foo: conrod::widget::id::Single::new()`.
    ($Ids:ident { { $($id_field:ident: $new:expr,)* } $id:ident, $($rest:tt)* }) => {
        widget_ids_constructor! {
            $Ids {
                {
                    $($id_field: $new,)*
                    $id: $crate::widget::id::Single::new(),
                }
                $($rest)*
            }
        }
    };

    // Same as above but without the trailing comma.
    ($Ids:ident { { $($id_field:ident: $new:expr,)* } $id:ident[] }) => {
        widget_ids_constructor! { $Ids { { $($id_field: $new,)* } $id[], } }
    };
    ($Ids:ident { { $($id_field:ident: $new:expr,)* } $id:ident }) => {
        widget_ids_constructor! { $Ids { { $($id_field: $new,)* } $id, } }
    };

    // Generatees the `$Ids` constructor using the `field: expr`s generated above.
    ($Ids:ident { { $($id:ident: $new:expr,)* } }) => {
        $Ids {
            $(
                $id: $new,
            )*
        }
    };

}


/// A macro used to generate a struct with a field for each unique identifier given.
/// Each field can then be used to generate unique `widget::Id`s.
///
/// For example, given the following invocation:
///
/// ```ignore
/// widget_ids! {
///     Ids {
///         button,
///         toggles[],
///     }
/// }
/// ```
///
/// The following will be produced:
///
/// ```ignore
/// struct Ids {
///     button: conrod::widget::id::Single,
///     toggles: conrod::widget::id::List,
/// }
///
/// impl Ids {
///     pub fn new() -> Self {
///         button: conrod::widget::id::Single::new(),
///         toggles: conrod::widget::id::List::new(),
///     }
/// }
/// ```
///
/// The in the example above, the generated `Ids` type can be used as follows.
///
/// ```ignore
/// widget::Button::new().set(ids.button.get(ui), ui);
/// 
/// ids.toggles.resize(5, ui);
/// for &id in &ids.toggles {
///     widget::Toggle::new(true).set(id, ui);
/// }
/// ```
#[macro_export]
macro_rules! widget_ids {

    ($Ids:ident { $($id:tt)* }) => {
        widget_ids_define_struct! {
            $Ids { {} $($id)* }
        }

        impl $Ids {

            /// Construct a new, empty `widget::Id` cache.
            pub fn new() -> Self {
                widget_ids_constructor! {
                    $Ids { {} $($id)* }
                }
            }

        }
    };

}


#[test]
fn test() {
    use ui::UiBuilder;
    use widget::{self, Widget};

    widget_ids!(Ids { button, toggles[] });

    let ui = &mut UiBuilder::new().build();
    let ids = &mut Ids::new();

    for _ in 0..10 {
        let ref mut ui = ui.set_widgets();

        // Single button index.
        widget::Button::new().set(ids.button.get(ui), ui);

        // Lazily generated toggle indices.
        ids.toggles.resize(5, ui);
        for &id in ids.toggles.iter() {
            widget::Toggle::new(true).set(id, ui);
        }
    }
}
