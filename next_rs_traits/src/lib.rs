pub mod layout;
pub mod pages;
pub mod routes;

pub mod predule {
    use super::*;
    pub use layout::Layout;
    pub use pages::{Component, DynPage, Page};
    pub use routes::{Route, UrlInfos};
}
