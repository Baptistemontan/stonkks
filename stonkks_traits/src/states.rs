use std::{
    any::{Any, TypeId},
    collections::HashMap,
    ops::Deref,
};

#[derive(Default)]
pub struct StatesMap(HashMap<TypeId, Box<dyn Any + Send + Sync>>);

impl StatesMap {
    pub fn extract<'a, T: ExtractState<'a>>(&'a self) -> Result<T, &'static str> {
        T::extract(self)
    }

    pub fn add_state<T: AnyState>(&mut self, state: T) -> Option<Box<dyn Any + Send + Sync>> {
        let type_id = TypeId::of::<T>();
        self.0.insert(type_id, Box::new(state))
    }

    fn get<T: AnyState>(&self) -> Option<&T> {
        let type_id = TypeId::of::<T>();
        let possible_state = self.0.get(&type_id)?;
        possible_state.downcast_ref::<T>()
    }

    pub fn get_state<T: AnyState>(&self) -> Result<&T, &'static str> {
        self.get::<T>().ok_or_else(|| std::any::type_name::<T>())
    }
}

pub trait ExtractState<'r>: Sized + Send {
    fn extract(states: &'r StatesMap) -> Result<Self, &'static str>;
}

pub struct State<T>(pub T);

pub trait AnyState: Any + Send + Sync {}

impl<T: Send + Sync + Any> AnyState for T {}

impl<T> Deref for State<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'r, T: ExtractState<'r>> ExtractState<'r> for State<T> {
    fn extract(states: &'r StatesMap) -> Result<Self, &'static str> {
        T::extract(states).map(State)
    }
}

impl<'r> ExtractState<'r> for () {
    fn extract(_states: &'r StatesMap) -> Result<Self, &'static str> {
        Ok(())
    }
}

impl<'r, T: AnyState> ExtractState<'r> for &'r T {
    fn extract<'a>(states: &'r StatesMap) -> Result<Self, &'static str> {
        states.get_state::<T>()
    }
}

mod impl_macro {
    use super::*;

    // macro shamefully stolen from the std,
    // originaly meant from implementing PartialEq and other trait for tuples
    // just modified to implement this.
    macro_rules! tuple_impls {
        // Stopping criteria (1-ary tuple)
        ($T:ident) => {
            tuple_impls!(@impl $T);
        };
        // Running criteria (n-ary tuple, with n >= 2)
        ($T:ident $( $U:ident )+) => {
            tuple_impls!($( $U )+);
            tuple_impls!(@impl $T $( $U )+);
        };
        // "Private" internal implementation
        (@impl $( $T:ident )+) => {
            impl<'r, $($T:AnyState),+> ExtractState<'r> for ($(&'r $T,)+) {
                fn extract(states: &'r StatesMap) -> Result<Self, &'static str> {
                    Ok(($(states.get_state::<$T>()?,)+))
                }
            }
        }
    }

    tuple_impls!(A B C D E F G H I J K L M O P Q); // 16 Max
}
