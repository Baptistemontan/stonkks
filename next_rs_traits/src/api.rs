use super::pointers::*;
use super::predule::*;
use super::routes::DynRoutable;

/// For now api routes just respond with a string,
/// this will change just need to figure out the api.
/// Also need a way for ressources,
/// they could be store in a HashMap<TypeId, Box<dyn Any>>,
/// but I will take care of that later.
#[async_trait::async_trait]
pub trait Api: Routable {
    async fn respond<'url>(route: Self::Route<'url>) -> String;
}

#[async_trait::async_trait]
pub trait DynApi: DynRoutable {
    async unsafe fn respond<'url>(&self, route_ptr: RouteUntypedPtr<'url>) -> String;
}

#[async_trait::async_trait]
impl<T: Api> DynApi for T {
    async unsafe fn respond<'url>(&self, route_ptr: RouteUntypedPtr<'url>) -> String {
        let route = route_ptr.cast::<T>();
        let response = <T as Api>::respond(*route).await;
        response
    }
}
