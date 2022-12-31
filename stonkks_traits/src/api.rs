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
    type Ressource: ExtractRessources;
    async fn respond<'url, 'r>(
        route: Self::Route<'url>,
        ressources: ApiExtractedRessource<'r, Self>,
    ) -> Result<String, Self::Err<'url>>;
}

pub type ApiExtractedRessource<'a, T> = <<T as Api>::Ressource as ExtractRessources>::Output<'a>;

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
        let ressource = ressources
            .extract::<T::Ressource>()
            .map_err(|err| format!("Missing ressource {}.", err))?;
        <T as Api>::respond(*route, ressource)
            .await
            .map_err(|err| format!("{:?}", err))
    }
}
