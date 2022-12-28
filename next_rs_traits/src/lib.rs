pub mod pages;
pub mod routes;

pub mod predule {
    pub use super::pages::{BasePage, DynPage};
    pub use super::routes::{Route, UrlInfos};
}
