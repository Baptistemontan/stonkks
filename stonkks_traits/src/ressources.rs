use std::{
    any::{Any, TypeId},
    collections::HashMap,
};

#[derive(Default)]
pub struct RessourceMap(HashMap<TypeId, Box<dyn Any + Send + Sync>>);

impl RessourceMap {
    pub fn extract<T: ExtractRessources>(&self) -> Result<T::Output<'_>, &'static str> {
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

pub trait ExtractRessources {
    type Output<'a>: Send + Sync;
    fn extract<'a>(ressources: &'a RessourceMap) -> Result<Self::Output<'a>, &'static str>;
}

impl ExtractRessources for () {
    type Output<'a> = ();

    fn extract<'a>(_ressources: &'a RessourceMap) -> Result<Self::Output<'a>, &'static str> {
        Ok(())
    }
}

pub struct RessourceExtractor<T: AnyRessource>(pub T);

pub trait AnyRessource: Any + Send + Sync {}

impl<T: Send + Sync + Any> AnyRessource for T {}

impl<T: AnyRessource> ExtractRessources for RessourceExtractor<T> {
    type Output<'a> = &'a T;
    fn extract<'a>(ressources: &'a RessourceMap) -> Result<Self::Output<'a>, &'static str> {
        ressources.get_ressource::<T>()
    }
}

pub struct MultiRessourcesExtractor<T>(pub T);

impl<T: AnyRessource, U: AnyRessource> ExtractRessources for MultiRessourcesExtractor<(T, U)> {
    type Output<'a> = (&'a T, &'a U);

    fn extract<'a>(ressources: &'a RessourceMap) -> Result<Self::Output<'a>, &'static str> {
        let a = ressources.get_ressource::<T>()?;
        let b = ressources.get_ressource::<U>()?;
        Ok((a, b))
    }
}
