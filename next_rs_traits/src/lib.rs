pub mod layout;
pub mod pages;
pub mod pointers;
pub mod routes;
pub mod props;

pub mod predule {
    use super::*;
    pub use props::{Props, ReactiveProps, IntoProps};
    pub use layout::Layout;
    pub use pages::{Component, DynPage, NotFoundPage, NotFoundPageProps, Page, ComponentReactiveProps};
    pub use routes::{Route, UrlInfos};
}
