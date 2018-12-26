//! The widget identifier type used throughout conrod, along with helper types and macros to
//! simplify the process of generating them.

use daggy;
use graph::Graph;
use std;

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

/// Used for generating new unique `widget::Id`s.
///
/// `Generator` is used by the `widget_ids!` macro and the types and fields that it generates in
/// order to generate unique `widget::Id`s for each of the generated fields.
pub struct Generator<'a> { widget_graph: &'a mut Graph }

/// A list of lazily generated `widget::Id`s.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct List(Vec<Id>);
/// An iterator-like type for producing indices from a `List`.
#[allow(missing_copy_implementations)]
pub struct ListWalk { i: usize }


impl<'a> Generator<'a> {

    /// Constructor for a new `widget::Id` generator.
    pub fn new(widget_graph: &'a mut Graph) -> Self {
        Generator {
            widget_graph: widget_graph,
        }
    }

    /// Generate a new, unique `widget::Id` into a Placeholder node within the widget graph. This
    /// should only be called once for each unique widget needed to avoid unnecessary bloat within
    /// the `Ui`'s widget graph.
    ///
    /// When using this method, be sure to store the returned `widget::Id` somewhere so that it can
    /// be re-used on next update.
    ///
    /// **Panics** if adding another node would exceed the maximum capacity for node indices.
    pub fn next(&mut self) -> Id {
        self.widget_graph.add_placeholder()
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
    pub fn resize(&mut self, target_len: usize, id_generator: &mut Generator) {
        if self.len() < target_len {
            self.0.reserve(target_len);
            while self.len() < target_len {
                self.0.push(id_generator.next());
            }
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
    pub fn next(&mut self, &mut List(ref mut ids): &mut List, id_gen: &mut Generator) -> Id {
        while self.i >= ids.len() {
            ids.push(id_gen.next());
        }
        let ix = ids[self.i];
        self.i += 1;
        ix
    }

}


/// A macro used to generate a struct with a field for each unique identifier given.
/// Each field can then be used to generate unique `widget::Id`s.
///
/// For example, given the following invocation:
///
/// ```
/// #[macro_use]
/// extern crate conrod_core;
///
/// widget_ids! {
///     struct Ids {
///         button,
///         toggles[],
///     }
/// }
/// # fn main() {}
/// ```
///
/// The following will be produced:
///
/// ```
/// # extern crate conrod_core;
/// # fn main() {}
/// struct Ids {
///     button: conrod_core::widget::Id,
///     toggles: conrod_core::widget::id::List,
/// }
///
/// impl Ids {
///     pub fn new(mut generator: conrod_core::widget::id::Generator) -> Self {
///         Ids {
///             button: generator.next(),
///             toggles: conrod_core::widget::id::List::new(),
///         }
///     }
/// }
/// ```
///
/// In the example above, the generated `Ids` type can be used as follows.
///
/// ```ignore
/// widget::Button::new().set(ids.button, ui);
/// 
/// ids.toggles.resize(5, &mut ui.widget_id_generator());
/// for &id in &ids.toggles {
///     widget::Toggle::new(true).set(id, ui);
/// }
/// ```
#[macro_export]
macro_rules! widget_ids {


    ///////////////////////
    ///// widget_ids! /////
    ///////////////////////

    // The top-level branch, to be invoked by the user.
    ($(#[$attr:meta])*     struct $Ids:ident { $($id:tt)* }) => {
        widget_ids! { implement $(#[$attr])* []    $Ids { $($id)* } }
    };
    // The same as above, but makes the generated `$Ids` type `pub`.
    ($(#[$attr:meta])* pub struct $Ids:ident { $($id:tt)* }) => {
        widget_ids! { implement $(#[$attr])* [pub] $Ids { $($id)* } }
    };

    // Calls the `define_struct` and `constructor` branches, implementing the struct and
    // constructor method respectively.
    (implement $(#[$attr:meta])* [$($public:tt)*] $Ids:ident { $($id:tt)* }) => {
        widget_ids! {
            define_struct $(#[$attr])* [$($public)*] $Ids { {} $($id)* }
        }

        impl $Ids {

            /// Construct a new `widget::Id` container.
            #[allow(unused_mut, unused_variables)]
            pub fn new(mut generator: $crate::widget::id::Generator) -> Self {
                widget_ids! {
                    constructor $Ids, generator { {} $($id)* }
                }
            }

        }
    };


    /////////////////////////
    ///// define_struct /////
    /////////////////////////
    //
    // From
    //
    // ```ignore
    // widget_ids! {
    //     define_struct Ids {
    //         button,
    //         toggles[],
    //     }
    // }
    // ```
    //
    // these branches generate
    //
    // ```ignore
    // struct Ids {
    //     button: conrod_core::widget::Id,
    //     toggles: conrod_core::widget::id::List,
    // }
    // ```

    // Converts `foo[]` tokens to `foo: conrod_core::widget::id::List`.
    (define_struct $(#[$attr:meta])* [$($public:tt)*] $Ids:ident {
        { $($id_field:ident: $T:path,)* } $id:ident[], $($rest:tt)*
    }) => {
        widget_ids! {
            define_struct $(#[$attr])* [$($public)*] $Ids {
                {
                    $($id_field: $T,)*
                    $id: $crate::widget::id::List,
                }
                $($rest)*
            }
        }
    };

    // Converts `foo` tokens to `foo: conrod_core::widget::Id`.
    (define_struct $(#[$attr:meta])* [$($public:tt)*] $Ids:ident {
        { $($id_field:ident: $T:path,)* } $id:ident, $($rest:tt)*
    }) => {
        widget_ids! {
            define_struct $(#[$attr])* [$($public)*] $Ids {
                {
                    $($id_field: $T,)*
                    $id: $crate::widget::Id,
                }
                $($rest)*
            }
        }
    };

    // Same as above but without the trailing comma.
    (define_struct $(#[$attr:meta])* [$($public:tt)*] $Ids:ident {
        { $($id_field:ident: $T:path,)* } $id:ident[]
    }) => {
        widget_ids! {
            define_struct $(#[$attr])* [$($public)*] $Ids { { $($id_field: $T,)* } $id[], }
        }
    };
    (define_struct $(#[$attr:meta])* [$($public:tt)*] $Ids:ident {
        { $($id_field:ident: $T:path,)* } $id:ident
    }) => {
        widget_ids! {
            define_struct $(#[$attr])* [$($public)*] $Ids { { $($id_field: $T,)* } $id, }
        }
    };

    // Generates the struct using all the `ident: path` combinations generated above.
    (define_struct $(#[$attr:meta])* [pub] $Ids:ident { { $($id:ident: $T:path,)* } }) => {
        $(#[$attr])*
        pub struct $Ids {
            $(pub $id: $T,
            )*
        }
    };

    // The same as the previous branch, but private.
    (define_struct $(#[$attr:meta])* [] $Ids:ident { { $($id:ident: $T:path,)* } }) => {
        $(#[$attr])*
        struct $Ids {
            $(pub $id: $T,
            )*
        }
    };


    ///////////////////////
    ///// constructor /////
    ///////////////////////
    //
    // From
    //
    // ```ignore
    // widget_ids! {
    //     constructor Ids, generator {
    //         button,
    //         toggles[],
    //     }
    // }
    // ```
    //
    // these branches generate
    //
    // ```ignore
    // Ids {
    //     button: generator.next(),
    //     toggles: conrod_core::widget::id::List::new(),
    // }
    // ```

    // Converts `foo[]` to `foo: conrod_core::widget::id::List::new()`.
    (constructor $Ids:ident, $generator:ident {
        { $($id_field:ident: $new:expr,)* } $id:ident[], $($rest:tt)*
    }) => {
        widget_ids! {
            constructor $Ids, $generator {
                {
                    $($id_field: $new,)*
                    $id: $crate::widget::id::List::new(),
                }
                $($rest)*
            }
        }
    };

    // Converts `foo` to `foo: generator.next()`.
    (constructor $Ids:ident, $generator:ident {
        { $($id_field:ident: $new:expr,)* } $id:ident, $($rest:tt)*
    }) => {
        widget_ids! {
            constructor $Ids, $generator {
                {
                    $($id_field: $new,)*
                    $id: $generator.next(),
                }
                $($rest)*
            }
        }
    };

    // Same as above but without the trailing comma.
    (constructor $Ids:ident, $generator:ident {
        { $($id_field:ident: $new:expr,)* } $id:ident[]
    }) => {
        widget_ids! { constructor $Ids, $generator { { $($id_field: $new,)* } $id[], } }
    };
    (constructor $Ids:ident, $generator:ident {
        { $($id_field:ident: $new:expr,)* } $id:ident
    }) => {
        widget_ids! { constructor $Ids, $generator { { $($id_field: $new,)* } $id, } }
    };

    // Generates the `$Ids` constructor using the `field: expr`s generated above.
    (constructor $Ids:ident, $generator:ident { { $($id:ident: $new:expr,)* } }) => {
        $Ids {
            $(
                $id: $new,
            )*
        }
    };

}


#[test]
fn test() {
    use ui::UiBuilder;
    use widget::{self, Widget};

    widget_ids! {
        /// Testing generated Ids doc comments.
        #[derive(Clone)]
        pub struct Ids {
            button,
            toggles[],
        }
    }

    let mut ui = UiBuilder::new([800.0, 600.0]).build();
    let mut ids = Ids::new(ui.widget_id_generator());

    for _ in 0..10 {
        let ref mut ui = ui.set_widgets();

        // Single button index.
        widget::Button::new().set(ids.button, ui);

        // Lazily generated toggle indices.
        ids.toggles.resize(5, &mut ui.widget_id_generator());
        for &id in ids.toggles.iter() {
            widget::Toggle::new(true).set(id, ui);
        }
    }
}

#[test]
#[allow(unused_variables)]
fn test_invocation_variations() {
    use ui::UiBuilder;

    widget_ids! { struct A { foo, bar } }
    widget_ids! { pub struct B { foo, bar } }
    widget_ids! { struct C { foo, bar, } }
    widget_ids! { pub struct D { foo, bar, } }
    widget_ids! { struct E { foo[], bar } }
    widget_ids! { pub struct F { foo, bar[] } }
    widget_ids! { struct G { foo[], bar, } }
    widget_ids! { pub struct H { foo, bar[], } }

    let mut ui = UiBuilder::new([800.0, 600.0]).build();
    let mut ui = ui.set_widgets();
    let a = A::new(ui.widget_id_generator());
    let b = B::new(ui.widget_id_generator());
    let c = C::new(ui.widget_id_generator());
    let d = D::new(ui.widget_id_generator());
    let e = E::new(ui.widget_id_generator());
    let f = F::new(ui.widget_id_generator());
    let g = G::new(ui.widget_id_generator());
    let h = H::new(ui.widget_id_generator());
}
