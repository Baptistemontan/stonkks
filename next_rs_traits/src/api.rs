use std::fmt::Debug;

use crate::ressources::ExtractRessources;
use crate::ressources::RessourceMap;

use super::pointers::*;
use super::predule::*;
use super::routes::DynRoutable;

/// For now api routes just respond with a string,
/// this will change just need to figure out the api.
/// Also need a way for ressources,
/// they could be store in a HashMap<TypeId, Box<dyn Any>>,
/// but I will take care of that later.
/// TODO:
///  - support for method type (GET, POST, ect..) (only support get rn)
///  - ressources
///  - better return type
///  - ??
#[async_trait::async_trait]
pub trait Api: Routable {
    type Err<'url>: Debug;
    type Ressource: ExtractRessources;
    async fn respond<'url, 'r>(
        route: Self::Route<'url>,
        ressources: ExtractedRessource<'r, Self>,
    ) -> Result<String, Self::Err<'url>>;
}

pub type ExtractedRessource<'a, T> = <<T as Api>::Ressource as ExtractRessources>::Output<'a>;

#[async_trait::async_trait]
pub trait DynApi: DynRoutable {
    async unsafe fn respond<'url>(
        &self,
        route_ptr: RouteUntypedPtr<'url>,
        ressources: &RessourceMap,
    ) -> Result<String, String>;
}

#[async_trait::async_trait]
impl<T: Api> DynApi for T {
    async unsafe fn respond<'url>(
        &self,
        route_ptr: RouteUntypedPtr<'url>,
        ressources: &RessourceMap,
    ) -> Result<String, String> {
        let route = route_ptr.cast::<T>();
        let ressource = ressources.extract::<T::Ressource>();
        let ressource = match ressource {
            Ok(ressource) => ressource,
            Err(missing_ressource_name) => panic!("Missing ressource {}.", missing_ressource_name),
        };
        <T as Api>::respond(*route, ressource)
            .await
            .map_err(|err| format!("{:?}", err))
    }
}
