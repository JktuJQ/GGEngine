//! `gamecore::systems` submodule provides systems - functions that operate on
//! entities and their components and implement whole behaviour of an application.
//!

use crate::gamecore::scenes::Scene;

pub trait UnpreparedSystem {
    fn prepare_system(self) -> System;
}
impl<F> UnpreparedSystem for F
where
    F: FnMut(&mut Scene) -> (),
{
    fn prepare_system(self) -> System {
        System(self)
    }
}

#[derive(Debug)]
pub struct System(fn(&mut Scene) -> ());
