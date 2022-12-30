use std::fmt::Debug;

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
    async fn respond<'url>(route: Self::Route<'url>) -> Result<String, Self::Err<'url>>;
}

#[async_trait::async_trait]
pub trait DynApi: DynRoutable {
    async unsafe fn respond<'url>(&self, route_ptr: RouteUntypedPtr<'url>) -> Result<String, String>;
}

#[async_trait::async_trait]
impl<T: Api> DynApi for T {
    async unsafe fn respond<'url>(&self, route_ptr: RouteUntypedPtr<'url>) -> Result<String, String> {
        let route = route_ptr.cast::<T>();
        <T as Api>::respond(*route).await.map_err(|err| format!("{:?}", err))
    }
}
