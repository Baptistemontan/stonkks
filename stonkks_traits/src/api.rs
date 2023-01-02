use std::fmt::Debug;

use crate::states::ExtractState;
use crate::states::StatesMap;

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
    type State<'r>: ExtractState<'r>;
    async fn respond<'url, 'r>(
        route: Self::Route<'url>,
        states: Self::State<'r>,
    ) -> Result<String, Self::Err<'url>>;
}

#[async_trait::async_trait]
pub trait DynApi: DynRoutable {
    async unsafe fn respond<'url, 'r>(
        &self,
        route_ptr: RouteUntypedPtr<'url>,
        states: &'r StatesMap,
    ) -> Result<String, String>;
}

#[async_trait::async_trait]
impl<T: Api> DynApi for T {
    async unsafe fn respond<'url, 'r>(
        &self,
        route_ptr: RouteUntypedPtr<'url>,
        states: &'r StatesMap,
    ) -> Result<String, String> {
        let route = route_ptr.cast::<T>();
        let state = states
            .extract::<T::State<'r>>()
            .map_err(|err| format!("Missing state {}.", err))?;
        <T as Api>::respond(*route, state)
            .await
            .map_err(|err| format!("{:?}", err))
    }
}
