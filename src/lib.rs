mod app;
// #[cfg(target = "wasm32")]
mod client;
mod default;
mod pages;
mod server;

pub mod prelude {
    use super::*;
    pub use app::App;
    #[cfg(target = "wasm32")]
    pub use client::Client;
    pub use next_rs_traits::predule::*;
    pub use server::Server;
}

// use sycamore::prelude::*;

/// #[route("/index/<greeting>")] // impl Route
/// struct IndexRoute {
///     pub greeting: String
/// }
///
/// #[props(alias = "IndexPagePropsRx")] // impl Serialize, Deserialize and create proxy reactive struct
/// struct IndexPageProps {
///     pub greeting: String
/// }
///
/// #[component]
/// fn index<'a, G: Html>(cx: Scope<'a>, props: IndexPagePropsRx<'a>) -> View<G> {
///     view! { cx,
///         p { "Index " (props.greeting.get())}
///     }
/// }
///
/// async fn get_server_side_props(route: IndexRoute) -> IndexPageProps {
///     IndexPageProps {
///         greeting: route.greeting
///     }
/// }
///
/// pub fn get_dyn_page<G: Html>() -> impl DynPage<G> {
///     dyn_page! {
///         Route = IndexRoute,
///         Component = index,
///         ServerSideProps = get_server_side_props
///     }
/// }
#[allow(unused)]
struct Nothing;
