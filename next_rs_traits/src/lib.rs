pub mod layout;
pub mod pages;
pub mod pointers;
pub mod routes;

pub mod predule {
    use super::*;
    pub use layout::Layout;
    pub use pages::{Component, DynPage, Page, NotFoundPage, NotFoundPageProps};
    pub use routes::{Route, UrlInfos};
}
