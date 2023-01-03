pub mod api;
pub mod layout;
pub mod pages;
pub mod pointers;
pub mod props;
pub mod response;
pub mod routes;
pub mod states;

pub mod predule {
    use super::*;
    pub use api::Api;
    pub use layout::Layout;
    pub use pages::{
        Component, ComponentReactiveProps, DynPage, NotFoundPage, NotFoundPageProps, Page,
        StaticPage,
    };
    pub use props::{IntoProps, Props, ReactiveProps};
    pub use response::{ContentType, IntoResponse, Json, Response};
    pub use routes::{OwnedUrlInfos, Routable, Route, UrlInfos};
    pub use states::State;
}
