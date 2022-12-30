pub mod api;
pub mod layout;
pub mod pages;
pub mod pointers;
pub mod props;
pub mod ressources;
pub mod routes;

pub mod predule {
    use super::*;
    pub use api::Api;
    pub use layout::Layout;
    pub use pages::{
        Component, ComponentReactiveProps, DynPage, NotFoundPage, NotFoundPageProps, Page,
    };
    pub use props::{IntoProps, Props, ReactiveProps};
    pub use ressources::{MultiRessourcesExtractor, RessourceExtractor};
    pub use routes::{Routable, Route, UrlInfos};
}
