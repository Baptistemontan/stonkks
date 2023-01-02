use std::{
    any::{Any, TypeId},
    collections::HashMap, ops::Deref,
};

#[derive(Default)]
pub struct RessourceMap(HashMap<TypeId, Box<dyn Any + Send + Sync>>);

impl RessourceMap {
    pub fn extract<'a, T: ExtractRessources<'a>>(&'a self) -> Result<T, &'static str> {
        T::extract(self)
    }

    pub fn add_ressource<T: AnyRessource>(
        &mut self,
        ressource: T,
    ) -> Option<Box<dyn Any + Send + Sync>> {
        let type_id = TypeId::of::<T>();
        self.0.insert(type_id, Box::new(ressource))
    }

    fn get<T: AnyRessource>(&self) -> Option<&T> {
        let type_id = TypeId::of::<T>();
        let possible_ressource = self.0.get(&type_id)?;
        possible_ressource.downcast_ref::<T>()
    }

    pub fn get_ressource<T: AnyRessource>(&self) -> Result<&T, &'static str> {
        self.get::<T>().ok_or_else(|| std::any::type_name::<T>())
    }
}

pub trait ExtractRessources<'r>: Sized + Send {
    fn extract(ressources: &'r RessourceMap) -> Result<Self, &'static str>;
}

pub struct Ressource<T>(pub T);

pub trait AnyRessource: Any + Send + Sync {}

impl<T: Send + Sync + Any> AnyRessource for T {}

impl<T> Deref for Ressource<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'r, T: ExtractRessources<'r>> ExtractRessources<'r> for Ressource<T> {

    fn extract(ressources: &'r RessourceMap) -> Result<Self, &'static str> {
        T::extract(ressources).map(Ressource)
    }
}

impl<'r> ExtractRessources<'r> for () {
    fn extract(_ressources: &'r RessourceMap) -> Result<Self, &'static str> {
        Ok(())
    }
}

impl<'r, T: AnyRessource> ExtractRessources<'r> for &'r T {
    fn extract<'a>(ressources: &'r RessourceMap) -> Result<Self, &'static str> {
        ressources.get_ressource::<T>()
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
            impl<'r, $($T:AnyRessource),+> ExtractRessources<'r> for ($(&'r $T,)+) {
                fn extract(ressources: &'r RessourceMap) -> Result<Self, &'static str> {
                    Ok(($(ressources.get_ressource::<$T>()?,)+))
                }
            }
        }
    }

    tuple_impls!(A B C D E F G H I J K L M O P Q); // 16 Max
}
