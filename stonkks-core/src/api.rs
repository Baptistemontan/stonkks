use crate::pointers::*;
use crate::predule::*;
use crate::response::IntoResponse;
use crate::response::Response;
use crate::routes::DynRoutable;
use crate::states::ExtractState;
use crate::states::StatesMap;
/// For now api routes just respond with a string,
/// this will change just need to figure out the api.
/// TODO:
///  - support for method type (GET, POST, ect..) (only support get rn)
///  - better return type
///  - ??
use std::fmt::Debug;

/// Trait use to create an API route.
#[async_trait::async_trait]
pub trait Api: Routable {
    /// Error returned by the `respond` function.
    /// Must implement `Debug`.
    type Err<'url>: Debug;
    /// Extractor used to access states of the server.
    type State<'r>: ExtractState<'r>;
    type Output<'url>: IntoResponse;

    /// Function executed when the supplied route match the targeted URL.
    async fn respond<'url, 'r>(
        route: Self::Route<'url>,
        state: Self::State<'r>,
    ) -> Result<Self::Output<'url>, Self::Err<'url>>;
}

/// Internal trait used to implement the `Api` trait in a dynamic dispatch way.
/// This trait is NOT meant to be implemented by hand,
/// it is automaticaly implemented for all types implementing the `Api` trait.
#[async_trait::async_trait]
pub unsafe trait DynApi: DynRoutable {
    /// Wrapper for the `Api::respond` function, marked as unsafe because of the used of the `RouteUntypedPtr`,
    /// the implementation internally trust the `route_ptr` to be of the valid type.
    /// It is therefore to the caller to make sure the data backed by the pointer is of the correct type.
    /// (lifetime should'nt be a problem, the borrow checker should track it normally)
    async unsafe fn respond<'url, 'r>(
        &self,
        route_ptr: RouteUntypedPtr<'url>,
        state: &'r StatesMap,
    ) -> Result<Response, String>;
}

#[async_trait::async_trait]
unsafe impl<T: Api> DynApi for T {
    async unsafe fn respond<'url, 'r>(
        &self,
        route_ptr: RouteUntypedPtr<'url>,
        state: &'r StatesMap,
    ) -> Result<Response, String> {
        // trust the caller to pass down a route_ptr of the valid type.
        let route = route_ptr.downcast::<T>();
        // extract requested states.
        let state = state
            .extract::<T::State<'r>>()
            // extract return the name of the missing ressource
            .map_err(|err| format!("Missing state {}.", err))?;
        // execute original respond function.
        <T as Api>::respond(*route, state)
            .await
            // if failed return the erreur in a Debug formatted string.
            .map_err(|err| format!("{:?}", err))?
            // turn it into a response
            .into_response()
            // return the error in a debug formatted way
            .map_err(|err| format!("{:?}", err))
    }
}
