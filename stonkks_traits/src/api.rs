use std::fmt::Debug;

use crate::ressources::ExtractRessources;
use crate::ressources::RessourceMap;

use super::pointers::*;
use super::predule::*;
use super::routes::DynRoutable;

/// For now api routes just respond with a string,
/// this will change just need to figure out the api.
/// TODO:
///  - support for method type (GET, POST, ect..) (only support get rn)
///  - better return type
///  - ??
#[async_trait::async_trait]
pub trait Api: Routable {
    type Err<'url>: Debug;
    type Ressource<'r>: ExtractRessources<'r>;
    async fn respond<'url, 'r>(
        route: Self::Route<'url>,
        ressources: Self::Ressource<'r>,
    ) -> Result<String, Self::Err<'url>>;
}

#[async_trait::async_trait]
pub trait DynApi: DynRoutable {
    async unsafe fn respond<'url, 'r>(
        &self,
        route_ptr: RouteUntypedPtr<'url>,
        ressources: &'r RessourceMap,
    ) -> Result<String, String>;
}

#[async_trait::async_trait]
impl<T: Api> DynApi for T {
    async unsafe fn respond<'url, 'r>(
        &self,
        route_ptr: RouteUntypedPtr<'url>,
        ressources: &'r RessourceMap,
    ) -> Result<String, String> {
        let route = route_ptr.cast::<T>();
        let ressource = ressources
            .extract::<T::Ressource<'r>>()
            .map_err(|err| format!("Missing ressource {}.", err))?;
        <T as Api>::respond(*route, ressource)
            .await
            .map_err(|err| format!("{:?}", err))
    }
}
