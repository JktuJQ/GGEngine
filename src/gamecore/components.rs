//! `gamecore::components` submodule defines [`Component`] trait
//! that allows binding game logic that is represented in form of Rust types
//! to exact [`GameObject`](super::gameobjects::GameObject),
//! and implements several basic components used in games.
//!

use std::{
    any::{type_name, Any},
    collections::LinkedList,
    fmt,
};

/// `as_any` hidden module is needed to hide trait upcasting workaround.
///
/// Currently, trait upcasting is not stable,
/// but it is needed to properly use runtime reflection
/// (convert `&dyn Component` to `&dyn Any`).
/// Workaround is implemented in `AsAny` trait.
///
/// # Note
/// Although `as_any` method could belong to the [`Component`] trait itself,
/// it would not fit ECS component meaning (even if it was hidden in docs)
/// and also would require for users to manually implement it (even if it is trivial),
/// so it is instead moved to the hidden trait.
/// This approach also helps mantaining SemVer compatibility. When trait upcasting
/// will be implemented, there will be no use for `as_any` method and it removing it
/// would break compatibility (if `as_any` belonged to [`Component`] trait), but
/// current implementation would allow to just delete `AsAny` trait completely
/// without breaking compatibility.
///
pub(in crate::gamecore) mod as_any {
    use std::any::Any;

    /// [`AsAny`] trait is a workaround for trait upcasting.
    ///
    /// [`AsAny`] trait blanked implementation for all types that implement `Any`
    /// allows for `&self` to coerce to `&dyn Any`.
    ///
    pub trait AsAny {
        /// Method that coerces `&self` to `&dyn Any`.
        ///
        fn as_any(&self) -> &dyn Any;
    }
    impl<T: Any> AsAny for T {
        fn as_any(&self) -> &dyn Any {
            self
        }
    }
}

/// [`Component`] trait defines objects that are components by ECS terminology.
///
/// In ECS, components define objects, almost like in Rust
/// traits define structs. So basically, components just are parts of `GameObject`
/// that are responsible for its functionality.
/// ECS pattern encourages clean, decoupled design that
/// splits up your app data and logic into its core components.
///
/// # Implementation
/// [`Component`] trait requires `'static` trait bound, because `Any`
/// is a supertrait of [`Component`] trait and it requires `'static` trait bound.
///
/// There is also `AsAny` supertrait which may seem a 'seal' trait
/// that would forbid any external implementations - but it is not!
/// `AsAny` trait has blanket implementation for every type that has `Any` implemented
/// and so it is not a constraint at all.
/// That why implementing [`Component`] trait is so easy:
///
/// ```rust
/// use ggengine::gamecore::components::Component;
/// struct T;
/// impl Component for T {}
/// ```
///
/// # Examples
/// Any Rust type that fits [`Component`]'s constraints can be a [`Component`].
/// They are usually structs, but can also be enums or zero sized types.
/// The following example shows how one might define components for RPG:
///
/// ```rust
/// # use ggengine::gamecore::components::Component;
///
/// struct Player;
/// impl Component for Player {}
///
/// struct Name(String);
/// impl Component for Name {}
///
/// struct Position {
///     x: f32,
///     y: f32,
/// }
/// impl Component for Position {}
///
/// enum Race {
///     Elf,
///     Dwarf,
///     Human,
///     Orc,
/// }
/// impl Component for Race {}
///
/// enum Weapon {
///     Sword,
///     Spear,
///     Bow {
///         arrows: u32,
///     },
/// }
/// impl Component for Weapon {}
/// ```
///
pub trait Component: Any + as_any::AsAny {}
impl fmt::Debug for dyn Component {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "dyn Component ({:?})", type_name::<Self>())
    }
}

/// Type alias for `Box<dyn Component>`.
///
/// This type alias will be frequently used in situations in which
/// ownership of components is needed.
///
/// Internally `ggengine` has to operate with trait objects and
/// they need to be boxed, because ownership is required.
/// Passing single component is trivial: `ggengine` will just ask for
/// `T: Component` and construct [`BoxedComponent`] by itself.
/// On the other hand, passing multiple components (for example in form of a bundle)
/// is not as easy, because caller will need to construct [`BoxedComponent`]s manually.
/// That may be inconvenient, but `ggengine` does its best to abstract that away.
/// Docs on [`Bundle`] show some examples on that topic.
///
pub type BoxedComponent = Box<dyn Component>;

/// [`Bundle`] trait defines a set of [`Component`]s.
///
/// In ECS, components define objects and systems operate on combinations of components.
/// [`Bundle`] trait provides a way to create a set of [`Component`]s that are coupled
/// by some logic and it just makes sense to use them together.
///
/// # Examples
/// Every [`Component`] is a [`Bundle`], because [`Component`] is basically a set (bundle) of one component.
/// Additionally, tuples of [`Bundle`]s are also [`Bundle`] (with up to 12 items,
/// but those tuples can be nested, which practically removes that bound).
/// That allows coupling necessary components in a [`Bundle`], for example to
/// define a `PlayerBundle` that contains components that describe a player.
///
/// ```rust
/// # use ggengine::gamecore::components::Component;
/// 
/// #[derive(Default)]
/// struct Player;
/// impl Component for Player {}
///
/// #[derive(Default)]
/// struct Name(String);
/// impl Component for Name {}
///
/// #[derive(Default)]
/// struct Position {
///     x: f32,
///     y: f32,
/// }
/// impl Component for Position {}
///
/// type PlayerBundle = (Player, Name, Position);
/// ```
///
/// You might want to customize initialization of your [`Bundle`].
/// You can, of course, use `Default::default()`:
/// ```rust
/// # use ggengine::gamecore::components::Component;
/// #
/// # #[derive(Default)]
/// # struct Player;
/// # impl Component for Player {}
/// #
/// # #[derive(Default)]
/// # struct Name(String);
/// # impl Component for Name {}
/// #
/// # #[derive(Default)]
/// # struct Position {
/// #     x: f32,
/// #     y: f32,
/// # }
/// # impl Component for Position {}
/// # 
/// type PlayerBundle = (Player, Name, Position);
///
/// let player: PlayerBundle = Default::default();
/// ```
/// However, tuples do no support the struct update syntax
/// and for the most cases, their initialization is inconvenient.
///
/// That is where you have two options.
/// 1. You can use extension trait pattern to define constructors
/// for tuples:
/// ```rust
/// # use ggengine::gamecore::components::Component;
/// #
/// # #[derive(Default)]
/// # struct Player;
/// # impl Component for Player {}
/// #
/// # #[derive(Default)]
/// # struct Name(String);
/// # impl Component for Name {}
/// #
/// # #[derive(Default)]
/// # struct Position {
/// #     x: f32,
/// #     y: f32,
/// # }
/// # impl Component for Position {}
/// # 
/// type PlayerBundle = (Player, Name, Position);
/// trait WithName {
///     fn with_name(name: String) -> Self;
/// }
/// impl WithName for PlayerBundle {
///     fn with_name(name: String) -> PlayerBundle {
///         let mut result: PlayerBundle = PlayerBundle::default();
///         result.1 = Name(name);
///         result
///     }
/// }
///
/// let player: PlayerBundle = PlayerBundle::with_name("Player".to_string());
/// ```
/// 2. You can implement [`Bundle`] trait manually.
/// You may try to use provided implementations to construct your own:
/// ```rust
/// # use ggengine::gamecore::components::{Bundle, Component, BoxedComponent};
/// # use std::collections::LinkedList;
/// #
/// # #[derive(Default)]
/// # struct Player;
/// # impl Component for Player {}
/// #
/// # #[derive(Default)]
/// # struct Name(String);
/// # impl Component for Name {}
/// #
/// # #[derive(Default)]
/// # struct Position {
/// #     x: f32,
/// #     y: f32,
/// # }
/// # impl Component for Position {}
/// # 
/// #[derive(Default)]
/// struct PlayerBundle {
///     player: Player,
///     name: Name,
///     position: Position,
/// };
/// impl Bundle for PlayerBundle {
///     fn components(self) -> LinkedList<BoxedComponent> {
///         (self.player, self.name, self.position).components()
///     }
/// }
///
/// let player: PlayerBundle = PlayerBundle {
///     name: Name("Player".to_string()),
///     ..Default::default()
/// };
/// ```
/// That approach allows to free yourself from all restrictions,
/// and just 'pack a bundle' at the very end.
///
/// # Note
/// You may noticed that return type of `Bundle::components` is `LinkedList` struct.
/// That is because `ggengine` recursively flattens all nested
/// bundles to one, and the nature of `LinkedList` allows
/// easy list concatenation without any amortization.
///
/// Since `LinkedList` struct is not so different from its node,
/// creation of a list for every component in a bundle that is
/// already flattened and then concatenation of those lists
/// is not as inefficient as `Vec` implementation would be.
///
/// The choice between `LinkedList` and `Vec` boils down to following 2 points:
/// 1.
/// Recursive flattening of bundles requires creation of many single-item
/// lists and multiple concatenations of those. `LinkedList` is perfectly
/// suited for it, because `LinkedList` struct itself is just a node and
/// concatenation of `LinkedList`s is very fast.
/// One of the advantages of `Vec` is that it performs one big allocation
/// (which is better than doing multiple small allocations)
/// and then works with allocated memory.
/// That does not work when we are concatenating multiple single-item lists,
/// and it may even overallocate (we need space for only one component for single-item lists).
/// 2.
/// Cache locality in `Vec` and absence of it in `LinkedList` is a important aspect
/// to consider when it comes to perfomance. `Vec` is faster than `LinkedList`,
/// but the only thing that happens to collection that represents unpacked bundle
/// is that it is traversed once to assign all [`Component`]s to `GameObject`.
/// Considering that bundles are usually not very big, there is no significant perfomance gain
/// of `Vec`.
///
/// In summary, although `LinkedList` is inferior to `Vec` in most of the cases,
/// its usage is justified for [`Bundle`]:
/// nature of `LinkedList` is suited for creation of multiple single-item lists (nodes)
/// and for concatenating them, and perfomance gains of `Vec` are not significant in
/// most of use cases.
/// Ergonomics of `LinkedList` fit [`Bundle`] use case, and that is why
/// it has been chosen.
///
pub trait Bundle {
    /// Consumes itself and returns list of [`BoxedComponent`]s.
    ///
    fn components(self) -> LinkedList<BoxedComponent>;
}
impl<T: Component> Bundle for T {
    fn components(self) -> LinkedList<BoxedComponent> {
        let boxed_self: Box<dyn Component> = Box::new(self);
        LinkedList::from([boxed_self])
    }
}
/// [`impl_bundle`] macro implements [`Bundle`] trait for tuples of arity 12 or less.
///
macro_rules! impl_bundle {
    () => {
        impl Bundle for ()
        {
            fn components(self) -> LinkedList<BoxedComponent> {
                LinkedList::new()
            }
        }
    };
    (($t_start:ident, $index_start:tt), $(($t:ident, $index:tt),)*) => {
        impl<$t_start: Bundle, $($t: Bundle),*> Bundle for ($t_start, $($t,)*)
        {
            fn components(self) -> LinkedList<BoxedComponent> {
                let mut result: LinkedList<BoxedComponent> = LinkedList::new();
                result.append(&mut self.$index_start.components());
                $(
                    result.append(&mut self.$index.components());
                )*
                result
            }
        }
        impl_bundle!($(($t, $index),)*);
    };
}
impl_bundle!(
    (T00, 11),
    (T01, 10),
    (T02, 9),
    (T03, 8),
    (T04, 7),
    (T05, 6),
    (T06, 5),
    (T07, 4),
    (T08, 3),
    (T09, 2),
    (T10, 1),
    (T11, 0),
);
