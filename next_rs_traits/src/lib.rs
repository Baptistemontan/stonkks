pub mod pages;
pub mod routes;

pub mod predule {
    pub use super::pages::{Page, Component, DynPage};
    pub use super::routes::{Route, UrlInfos};
}
