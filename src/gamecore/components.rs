//! `gamecore::components` submodule defines [`Component`] trait
//! that allows binding game logic that is represented in form of Rust types
//! to exact [`Entity`](super::entities::Entity),
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
/// (convert `&dyn Component`/`&dyn Resource` to `&dyn Any`).
/// Workaround is implemented in `AsAny` trait.
///
/// # Note
/// Although `as_any_ref` method could belong to the [`Component`]/[`Resource`] trait itself,
/// it would not fit ECS component meaning (even if it was hidden in docs)
/// and also would require for users to manually implement it (even if it is trivial),
/// so it is instead moved to the hidden trait.
/// This approach also helps to maintain SemVer compatibility. When trait upcasting
/// will be implemented, there will be no use for `as_any` method and it removing it
/// would break compatibility (if `as_any` belonged to [`Component`]/[`Resource`] trait), but
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
        fn as_any_ref(&self) -> &dyn Any;
        /// Method that coerces `&mut self` to `&mut dyn Any`.
        ///
        fn as_any_mut(&mut self) -> &mut dyn Any;
        /// Method that coerces `Box<dyn T>` to `Box<dyn Any>`.
        ///
        fn as_any_box(self: Box<Self>) -> Box<dyn Any>;
    }
    impl<T: Any> AsAny for T {
        fn as_any_ref(&self) -> &dyn Any {
            self
        }
        fn as_any_mut(&mut self) -> &mut dyn Any {
            self
        }
        fn as_any_box(self: Box<Self>) -> Box<dyn Any> {
            self
        }
    }
}

/// [`Component`] trait defines objects that are components by ECS terminology.
///
/// In ECS, components define objects, almost like in Rust
/// traits define structs. So basically, components just are parts of `Entity`
/// that are responsible for its functionality.
/// ECS pattern encourages clean, decoupled design that
/// splits up your app data and logic into its core components.
///
/// `ggengine` supports having only one component of each type binded to `Entity`.
/// Trying to add two components of one type to `Entity` could lead to unexpected
/// behaviour, as `Entity` will only use the latest component.
///
/// # Implementation
/// [`Component`] trait requires `'static` trait bound, because `Any`
/// is a supertrait of [`Component`] trait, and it requires `'static` trait bound.
///
/// There is also `AsAny` supertrait which may seem a 'seal' trait
/// that would forbid any external implementations - but it is not.
/// `AsAny` trait has blanket implementation for every type that has `Any` implemented
/// and so it is not a constraint at all.
///
/// That is why implementing [`Component`] trait is so easy:
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
/// Internally `ggengine` has to operate with trait objects, and
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
/// by some logic, and it just makes sense to use those together.
///
/// Bundles are only a convenient way to group components in a set, and they should
/// not be used as units of behaviour. That is because multiple bundles could contain
/// the same [`Component`] type, and adding both of them to one
/// [`Entity`](super::entities::Entity) would lead to unexpected behaviour
/// (see [`Component`] trait docs).
/// For this reason it is impossible to use [`Bundle`] for querying. Instead, you should
/// operate on [`Component`]s which define your game logic, querying those you need to use.
///
/// # Examples
/// Every [`Component`] is a [`Bundle`], because component is basically a set (bundle) of one component.
/// Additionally, tuples of bundles are also [`Bundle`] (with up to 12 items,
/// but those tuples can be nested, which practically removes that bound).
/// This allows you to combine the necessary components into a [`Bundle`],
/// for example defining a PlayerBundle containing components that describe the player
/// can be written as follows:
///
/// ```rust
/// # use ggengine::gamecore::components::Component;
/// #
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
/// However, tuples do not support the struct update syntax
/// and for complex cases, their initialization is inconvenient.
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
///
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
/// You may even try to leverage provided implementations to construct your own:
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
/// }
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
/// You may notice that return type of `Bundle::components` is `LinkedList` struct.
/// That is because `ggengine` recursively flattens all nested
/// bundles to one, and the nature of `LinkedList` allows
/// easy list concatenation without any amortization.
///
/// Since `LinkedList` struct is not so different from its node,
/// creation of a list for every component in a bundle that is
/// already flattened and then concatenation of those lists
/// is not as inefficient as `Vec` implementation would be.
///
/// The choice between `LinkedList` and `Vec` boils down to the following 2 points:
/// 1. Recursive flattening of bundles requires creation of many single-item
/// lists and multiple concatenations of those. `LinkedList` is perfectly
/// suited for it, because `LinkedList` struct itself is just a node and
/// concatenation of `LinkedList`s is very fast.
/// One of the advantages of `Vec` is that it performs one big allocation
/// (which is better than doing multiple small allocations)
/// and then works with allocated memory.
/// That does not work when we are concatenating multiple single-item lists, and it may even overallocate
/// (we need space for only one component for single-item lists, but we need more when we concatenate).
/// 2. Cache locality in `Vec` and absence of it in `LinkedList` is an important aspect
/// to consider when it comes to performance. `Vec` is faster than `LinkedList`,
/// but the only thing that happens to collection that represents unpacked bundle
/// is that it is traversed once to assign all components to `Entity`.
/// Considering that bundles are usually not very big, there is no significant performance gain
/// of `Vec`.
///
/// In summary, although `LinkedList` is inferior to `Vec` in most of the cases,
/// its usage is justified for [`Bundle`]:
/// nature of `LinkedList` is suited for creation of multiple single-item lists (nodes)
/// and for concatenating them, and performance gains of `Vec` are not significant in
/// most of the use cases.
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

/// [`Resource`] trait defines unique global data that is bounded to the `Scene`.
///
/// [`Resource`]s are very similar to [`Component`]s, with the only difference is that
/// [`Component`]s are bounded to `Entity`s and [`Resource`]s are bound to the `Scene`.
///
/// Applications often have some global data that they share, it could be time, score, asset collection, etc.
/// Although global resources could be implemented as components that belong to some 'global' Entity,
/// that would be confusing and would not convey intention logic.
/// [`Resource`] trait supports this pattern, enforcing it through type system and allowing
/// for data to be shared easily.
///
/// # Implementation
/// [`Resource`] trait requires `'static` trait bound, because `Any`
/// is a supertrait of [`Resource`] trait, and it requires `'static` trait bound.
///
/// There is also `AsAny` supertrait which may seem a 'seal' trait
/// that would forbid any external implementations - but it is not.
/// `AsAny` trait has blanket implementation for every type that has `Any` implemented
/// and so it is not a constraint at all.
///
/// That is why implementing [`Resource`] trait is so easy:
///
/// ```rust
/// use ggengine::gamecore::components::Resource;
/// struct T;
/// impl Resource for T {}
/// ```
///
/// Considering that [`Resource`] is basically a [`Component`], almost everything
/// that goes for [`Component`]s is also true for [`Resource`]s.
/// To further understand relations between those traits, it is encouraged to read docs for submodule items.
///
pub trait Resource: Any + as_any::AsAny {}
impl fmt::Debug for dyn Resource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "dyn Resource ({:?})", type_name::<Self>())
    }
}
/// Type alias for `Box<dyn Resource>`.
///
/// This type alias will be frequently used in situations in which
/// ownership of resource is needed.
///
/// Internally `ggengine` has to operate with trait objects, and
/// they need to be boxed, because ownership is required.
/// Passing single resource is trivial: `ggengine` will just ask for
/// `T: Resource` and construct [`BoxedResource`] by itself.
/// On the other hand, passing multiple resources
/// is not as easy, because caller will need to construct [`BoxedResource`]s manually.
/// That may be inconvenient, but `ggengine` does its best to abstract that away.
///
pub type BoxedResource = Box<dyn Resource>;
